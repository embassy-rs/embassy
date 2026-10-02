//! Enhanced Universal Synchronous/Asynchronous Receiver/Transmitter (EUSART), as a UART.
//!
//! [`Uart`], [`UartTx`] and [`UartRx`] run in blocking mode ([`Uart::new_blocking`]) or async mode
//! ([`Uart::new`]). Async mode needs the `EUSARTn_RX` and `EUSARTn_TX` interrupts bound to
//! [`RxInterruptHandler`] and [`TxInterruptHandler`]. [`Uart::split`] and [`Uart::split_ref`] give
//! the transmitter and receiver halves.
//!
//! # Examples
//!
//! ```rust,ignore
//! use embassy_silabs::{bind_interrupts, eusart, peripherals};
//!
//! bind_interrupts!(struct Irqs {
//!     EUSART0_RX => eusart::RxInterruptHandler<peripherals::EUSART0>;
//!     EUSART0_TX => eusart::TxInterruptHandler<peripherals::EUSART0>;
//! });
//!
//! // TX on PA04, RX on PA05.
//! let mut uart =
//!     eusart::Uart::new(p.EUSART0, p.PA04, p.PA05, Irqs, eusart::Config::default()).unwrap();
//! uart.write(b"hello\r\n").await.unwrap();
//! ```

use core::future::poll_fn;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use core::task::Poll;

use embassy_hal_internal::{Peri, PeripheralType};
use embassy_sync::waitqueue::AtomicWaker;
use eusart_mod::vals;

use crate::gpio::AnyPin;
use crate::interrupt;
use crate::interrupt::typelevel::Interrupt;
use crate::mode::{Async, Blocking, Mode};
use crate::pac::eusart as eusart_mod;
use crate::time::Hertz;
pub use crate::uart_common::{Config, ConfigError, DataBits, Error, Parity, StopBits};

/// Asynchronous oversampling. The divider math assumes 16x.
const OVERSAMPLE: u32 = 16;

fn databits(d: DataBits) -> Result<vals::Databits, ConfigError> {
    match d {
        DataBits::DataBits5 | DataBits::DataBits6 => Err(ConfigError::DataBitsNotSupported),
        DataBits::DataBits7 => Ok(vals::Databits::Seven),
        DataBits::DataBits8 => Ok(vals::Databits::Eight),
        DataBits::DataBits9 => Ok(vals::Databits::Nine),
    }
}

fn parity(p: Parity) -> vals::Parity {
    match p {
        Parity::ParityNone => vals::Parity::None,
        Parity::ParityEven => vals::Parity::Even,
        Parity::ParityOdd => vals::Parity::Odd,
    }
}

fn stopbits(s: StopBits) -> vals::Stopbits {
    match s {
        StopBits::STOP0P5 => vals::Stopbits::Half,
        StopBits::STOP1 => vals::Stopbits::One,
        StopBits::STOP1P5 => vals::Stopbits::Oneandahalf,
        StopBits::STOP2 => vals::Stopbits::Two,
    }
}

/// Returns the nearest `CLKDIV.DIV` value for a baud rate.
///
/// The reference manual gives the asynchronous baud rate as
/// `baud = f / (ovs * (1 + CLKDIV / 256))`, where `CLKDIV` is the register value. `DIV` starts at
/// bit 3, so `CLKDIV = DIV * 8` and `DIV = 32 * f / (ovs * baud) - 32`.
fn compute_clkdiv(src: Hertz, baudrate: u32) -> Result<u32, ConfigError> {
    let denom = OVERSAMPLE as u64 * baudrate as u64;
    if denom == 0 {
        return Err(ConfigError::BaudrateTooLow);
    }
    let n = (32 * src.0 as u64 + denom / 2) / denom;
    if n < 32 {
        return Err(ConfigError::BaudrateTooHigh);
    }
    let div = n - 32;
    // `CLKDIV.DIV` is 20 bits wide.
    if div > 0xF_FFFF {
        return Err(ConfigError::BaudrateTooLow);
    }
    Ok(div as u32)
}

/// Per-instance shared state.
pub(crate) struct State {
    rx_waker: AtomicWaker,
    tx_waker: AtomicWaker,
    /// Live `UartTx` and `UartRx` halves that own the peripheral. The last to drop turns it off.
    tx_rx_refcount: AtomicU8,
    /// Whether a frame was written after the transmitter was last seen idle. See [`tx_busy`].
    tx_pending: AtomicBool,
}

impl State {
    pub(crate) const fn new() -> Self {
        Self {
            rx_waker: AtomicWaker::new(),
            tx_waker: AtomicWaker::new(),
            tx_rx_refcount: AtomicU8::new(0),
            tx_pending: AtomicBool::new(false),
        }
    }
}

/// Returns `true` if a written frame is not sent yet.
///
/// `STATUS` crosses from the EUSART core clock to the APB clock (reference manual 21.3.2.5.3), so
/// `STATUS.TXC` and `STATUS.TXIDLE` can read 1 for a short time after a `TXDATA` write.
/// `tx_pending` and the sticky `IF.TXC` cover that gap: [`push_frame`] sets `tx_pending` and clears
/// `IF.TXC` before each write.
fn tx_busy(info: &Info, state: &State) -> bool {
    if !state.tx_pending.load(Ordering::Relaxed) {
        return false;
    }
    let r = info.regs;
    let st = r.status().read();
    if r.if_().read().txc() && st.txidle() && st.txfcnt() == 0 {
        state.tx_pending.store(false, Ordering::Relaxed);
        return false;
    }
    true
}

/// Writes one frame to the TX FIFO in the asynchronous `TXDATA` layout. The FIFO must have space.
fn push_frame(info: &Info, state: &State, data: u16, txbreak: bool) {
    let r = info.regs;
    state.tx_pending.store(true, Ordering::Relaxed);
    r.if_clr().write(|w| w.set_txc(true));
    r.txdata_async().write(|w| {
        w.set_txdata(data);
        w.set_txbreak(txbreak);
    });
}

/// Type-erased instance data.
pub(crate) struct Info {
    pub(crate) regs: eusart_mod::Eusart,
    pub(crate) enable_clock: fn(),
    pub(crate) disable_clock: fn(),
    pub(crate) frequency: fn() -> Hertz,
    /// Disables and unpends this instance's RX and TX interrupts in the NVIC.
    pub(crate) disable_irqs: fn(),
    /// Writes `GPIO.EUSARTn_TXROUTE`: `port` 0..=3, `pin` 0..=15.
    pub(crate) set_tx_route: fn(u8, u8),
    /// Writes `GPIO.EUSARTn_RXROUTE`: `port` 0..=3, `pin` 0..=15.
    pub(crate) set_rx_route: fn(u8, u8),
    /// Sets (`true`) or clears `GPIO.EUSARTn_ROUTEEN.TXPEN`.
    pub(crate) enable_tx_route: fn(bool),
    /// Sets (`true`) or clears `GPIO.EUSARTn_ROUTEEN.RXPEN`.
    pub(crate) enable_rx_route: fn(bool),
}

pub(crate) trait SealedInstance {
    fn info() -> &'static Info;
    fn state() -> &'static State;
}

/// An EUSART peripheral instance.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + crate::cmu::CmuPeripheral + PeripheralType + 'static {
    /// Interrupt for RX events.
    type RxInterrupt: interrupt::typelevel::Interrupt;
    /// Interrupt for TX events.
    type TxInterrupt: interrupt::typelevel::Interrupt;
}

pin_trait!(TxPin, Instance);
pin_trait!(RxPin, Instance);

/// Implements [`Instance`] for one EUSART. `build.rs` invokes it for each EUSART in chip metadata.
macro_rules! impl_eusart {
    ($inst:ident, $rxirq:ident, $txirq:ident, $txroute:ident, $rxroute:ident, $routeen_set:ident, $routeen_clr:ident $(,)?) => {
        impl crate::eusart::SealedInstance for crate::peripherals::$inst {
            fn info() -> &'static crate::eusart::Info {
                static INFO: crate::eusart::Info = crate::eusart::Info {
                    regs: crate::pac::$inst,
                    enable_clock: crate::cmu::enable::<crate::peripherals::$inst>,
                    disable_clock: crate::cmu::disable::<crate::peripherals::$inst>,
                    frequency: crate::cmu::frequency::<crate::peripherals::$inst>,
                    disable_irqs: || {
                        use crate::interrupt::typelevel::Interrupt;
                        crate::interrupt::typelevel::$rxirq::disable();
                        crate::interrupt::typelevel::$txirq::disable();
                        crate::interrupt::typelevel::$rxirq::unpend();
                        crate::interrupt::typelevel::$txirq::unpend();
                    },
                    set_tx_route: |port, pin| {
                        crate::pac::GPIO.$txroute().write(|w| {
                            w.set_port(port);
                            w.set_pin(pin);
                        })
                    },
                    set_rx_route: |port, pin| {
                        crate::pac::GPIO.$rxroute().write(|w| {
                            w.set_port(port);
                            w.set_pin(pin);
                        })
                    },
                    // The SET and CLR aliases change only the written bit, so the TX and RX halves
                    // need no critical section.
                    enable_tx_route: |on| {
                        if on {
                            crate::pac::GPIO.$routeen_set().write(|w| w.set_txpen(true))
                        } else {
                            crate::pac::GPIO.$routeen_clr().write(|w| w.set_txpen(true))
                        }
                    },
                    enable_rx_route: |on| {
                        if on {
                            crate::pac::GPIO.$routeen_set().write(|w| w.set_rxpen(true))
                        } else {
                            crate::pac::GPIO.$routeen_clr().write(|w| w.set_rxpen(true))
                        }
                    },
                };
                &INFO
            }
            fn state() -> &'static crate::eusart::State {
                static STATE: crate::eusart::State = crate::eusart::State::new();
                &STATE
            }
        }
        impl crate::eusart::Instance for crate::peripherals::$inst {
            type RxInterrupt = crate::interrupt::typelevel::$rxirq;
            type TxInterrupt = crate::interrupt::typelevel::$txirq;
        }
    };
}
pub(crate) use impl_eusart;

/// Interrupt handler for the RX vector (`EUSARTn_RX`).
///
/// Both handlers run the same routine, so either vector serves RX and TX.
pub struct RxInterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::RxInterrupt> for RxInterruptHandler<T> {
    unsafe fn on_interrupt() {
        on_interrupt(T::info(), T::state());
    }
}

/// Interrupt handler for the TX vector (`EUSARTn_TX`).
pub struct TxInterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::TxInterrupt> for TxInterruptHandler<T> {
    unsafe fn on_interrupt() {
        on_interrupt(T::info(), T::state());
    }
}

/// Disables the interrupt sources that fired and wakes their futures, which enable them again.
fn on_interrupt(info: &Info, state: &State) {
    let r = info.regs;
    let if_ = r.if_().read();
    let ien = r.ien().read();

    // A frame with a parity or framing error also sets RXFL, and its read reports the error.
    let rx = (if_.rxfl() && ien.rxfl()) || (if_.rxof() && ien.rxof());
    if rx {
        r.ien_clr().write(|w| {
            w.set_rxfl(true);
            w.set_rxof(true);
        });
        state.rx_waker.wake();
    }

    let tx = (if_.txfl() && ien.txfl()) || (if_.txc() && ien.txc());
    if tx {
        r.ien_clr().write(|w| {
            w.set_txfl(true);
            w.set_txc(true);
        });
        state.tx_waker.wake();
    }
}

/// Waits until `SYNCBUSY` shows no pending `CMD` enable or disable.
fn wait_sync_cmd(r: eusart_mod::Eusart) {
    while {
        let s = r.syncbusy().read();
        s.rxen() || s.txen() || s.rxdis() || s.txdis()
    } {}
}

/// Writes `CLKDIV` while `EN = 1`.
///
/// The register is synchronized, so the write waits for `SYNCBUSY.DIV` before and after.
fn write_clkdiv(r: eusart_mod::Eusart, div: u32) {
    while r.syncbusy().read().div() {}
    r.clkdiv().write(|w| w.set_div(div));
    while r.syncbusy().read().div() {}
}

/// Turns on the requested transmitter and receiver. `EN` must be 1.
fn enable_rxtx(r: eusart_mod::Eusart, tx: bool, rx: bool) {
    while r.syncbusy().read().0 != 0 {}
    r.cmd().write(|w| {
        w.set_rxen(rx);
        w.set_txen(tx);
    });
    wait_sync_cmd(r);
    while {
        let st = r.status().read();
        (tx && !st.txens()) || (rx && !st.rxens())
    } {}
}

/// Turns the EUSART off. FIFO contents are lost.
fn disable(r: eusart_mod::Eusart) {
    if !r.en().read().en() {
        return;
    }
    r.cmd().write(|w| {
        w.set_txdis(true);
        w.set_rxdis(true);
    });
    wait_sync_cmd(r);
    while {
        let st = r.status().read();
        st.txens() || st.rxens()
    } {}
    r.en_clr().write(|w| w.set_en(true));
    while r.en().read().disabling() {}
}

/// Writes reset values to the configuration registers that the driver does not set.
///
/// A bootloader can leave them changed. `EN` must be 0.
///
/// The values are the register reset values (EFR32xG24 reference manual 21.5).
fn reset_config(r: eusart_mod::Eusart) {
    use eusart_mod::regs;
    r.cfg2().write_value(regs::Cfg2(0x0000_0020));
    r.dtxdatcfg().write_value(regs::Dtxdatcfg(0));
    // DALIEN = 0 turns DALI mode off, so TXDATA and RXDATA have the asynchronous layout.
    r.dalicfg().write_value(regs::Dalicfg(0));
    r.timingcfg().write_value(regs::Timingcfg(0x0005_0000));
    r.irhfcfg().write_value(regs::Irhfcfg(0));
    r.irlfcfg().write_value(regs::Irlfcfg(0));
    r.startframecfg().write_value(regs::Startframecfg(0));
    r.sigframecfg().write_value(regs::Sigframecfg(0));
    r.trigctrl().write_value(regs::Trigctrl(0));
}

/// Writes the frame registers `CFG1`, `CFG0` and `FRAMECFG`.
///
/// They are writable only while `EN = 0`.
fn write_frame_config(r: eusart_mod::Eusart, config: &Config, databits: vals::Databits) {
    r.cfg1().write(|w| {
        // One frame sets the FIFO level flags, so no byte waits for more.
        w.set_rxfiw(vals::Rxfiw::Oneframe);
        w.set_txfiw(vals::Txfiw::Oneframe);
    });
    r.cfg0().write(|w| {
        w.set_sync(vals::Sync::Async);
        w.set_ovs(vals::Ovs::X16);
        w.set_msbf(false);
        w.set_rxinv(config.invert_rx);
        w.set_txinv(config.invert_tx);
        w.set_loopbk(config.loopback);
    });
    r.framecfg().write(|w| {
        w.set_databits(databits);
        w.set_parity(parity(config.parity));
        w.set_stopbits(stopbits(config.stop_bits));
    });
}

/// Turns the peripheral on and configures it for the halves whose pins are given.
fn init(
    info: &Info,
    state: &State,
    tx: Option<&AnyPin>,
    rx: Option<&AnyPin>,
    config: &Config,
) -> Result<(), ConfigError> {
    // Check the configuration before the clock goes on, so an error leaves the clock off.
    let databits = databits(config.data_bits)?;
    let src = (info.frequency)();
    let div = compute_clkdiv(src, config.baudrate)?;
    debug!(
        "eusart: src_hz={} baud={} ovs={} clkdiv={}",
        src.0, config.baudrate, OVERSAMPLE, div
    );

    (info.enable_clock)();
    let r = info.regs;
    // A bootloader can leave the EUSART on.
    disable(r);
    reset_config(r);
    write_frame_config(r, config, databits);
    r.ien().write(|_| {});
    r.if_clr().write_value(eusart_mod::regs::If(0xFFFF_FFFF));
    state.tx_pending.store(false, Ordering::Relaxed);

    // TX is a push-pull output at the idle level, and RX is an input.
    if let Some(tx) = tx {
        crate::gpio::set_as_alternate_output(tx, !config.invert_tx);
        (info.set_tx_route)(tx.pin_port() >> 4, tx.pin_port() & 0xF);
        (info.enable_tx_route)(true);
    }
    if let Some(rx) = rx {
        crate::gpio::set_as_alternate_input(rx);
        (info.set_rx_route)(rx.pin_port() >> 4, rx.pin_port() & 0xF);
        (info.enable_rx_route)(true);
    }

    r.en_set().write(|w| w.set_en(true));
    write_clkdiv(r, div);
    enable_rxtx(r, tx.is_some(), rx.is_some());
    // Wait until the enabled directions are idle.
    while {
        let st = r.status().read();
        (tx.is_some() && !st.txidle()) || (rx.is_some() && !st.rxidle())
    } {}
    Ok(())
}

fn set_baudrate(info: &Info, baudrate: u32) -> Result<(), ConfigError> {
    let div = compute_clkdiv((info.frequency)(), baudrate)?;
    write_clkdiv(info.regs, div);
    Ok(())
}

/// Rewrites the frame registers with the EUSART off.
///
/// The EUSART turns on again with the transmitter and receiver that were on before.
fn set_config(info: &Info, state: &State, config: &Config) -> Result<(), ConfigError> {
    let databits = databits(config.data_bits)?;
    let div = compute_clkdiv((info.frequency)(), config.baudrate)?;
    let r = info.regs;
    while tx_busy(info, state) {}
    let st = r.status().read();
    let (tx, rx) = (st.txens(), st.rxens());
    disable(r);
    write_frame_config(r, config, databits);
    r.en_set().write(|w| w.set_en(true));
    write_clkdiv(r, div);
    enable_rxtx(r, tx, rx);
    // A pending future polls again and enables its interrupts again.
    state.tx_waker.wake();
    state.rx_waker.wake();
    Ok(())
}

/// UART transmitter.
pub struct UartTx<'d, M: Mode> {
    info: &'static Info,
    state: &'static State,
    pin: Peri<'d, AnyPin>,
    /// This half turns the peripheral off when it drops.
    ///
    /// A half from [`Uart::split_ref`] does not: its `Uart` does.
    owner: bool,
    _phantom: PhantomData<M>,
}

impl<'d> UartTx<'d, Blocking> {
    /// Creates a blocking transmitter on `tx` for instance `peri`.
    pub fn new_blocking<T: Instance>(
        _peri: Peri<'d, T>,
        tx: Peri<'d, impl TxPin<T>>,
        config: Config,
    ) -> Result<Self, ConfigError> {
        let tx: Peri<'d, AnyPin> = tx.into();
        init(T::info(), T::state(), Some(&tx), None, &config)?;
        T::state().tx_rx_refcount.store(1, Ordering::Relaxed);
        Ok(Self::wrap(T::info(), T::state(), tx))
    }
}

impl<'d> UartTx<'d, Async> {
    /// Creates an interrupt-driven transmitter on `tx` for instance `peri`.
    ///
    /// Bind the `EUSARTn_TX` vector to [`TxInterruptHandler<T>`].
    pub fn new<T: Instance>(
        _peri: Peri<'d, T>,
        tx: Peri<'d, impl TxPin<T>>,
        _irq: impl interrupt::typelevel::Binding<T::TxInterrupt, TxInterruptHandler<T>> + 'd,
        config: Config,
    ) -> Result<Self, ConfigError> {
        let tx: Peri<'d, AnyPin> = tx.into();
        init(T::info(), T::state(), Some(&tx), None, &config)?;
        T::state().tx_rx_refcount.store(1, Ordering::Relaxed);
        T::TxInterrupt::unpend();
        unsafe { T::TxInterrupt::enable() };
        Ok(Self::wrap(T::info(), T::state(), tx))
    }

    /// Writes all bytes. Yields while the TX FIFO is full.
    pub async fn write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        let r = self.info.regs;
        for &b in buffer {
            poll_fn(|cx| {
                if r.status().read().txfl() {
                    return Poll::Ready(());
                }
                self.state.tx_waker.register(cx.waker());
                // Clear a stale flag, so it gives no interrupt at once.
                r.if_clr().write(|w| w.set_txfl(true));
                r.ien_set().write(|w| w.set_txfl(true));
                if r.status().read().txfl() {
                    Poll::Ready(())
                } else {
                    Poll::Pending
                }
            })
            .await;
            push_frame(self.info, self.state, b as u16, false);
        }
        Ok(())
    }

    /// Waits until all written bytes are sent.
    pub async fn flush(&mut self) -> Result<(), Error> {
        let r = self.info.regs;
        poll_fn(|cx| {
            if !self.busy() {
                return Poll::Ready(());
            }
            self.state.tx_waker.register(cx.waker());
            // `push_frame` clears `IF.TXC` before each write, so the flag is set only after the
            // last frame. `busy` reads it, so do not clear it here.
            r.ien_set().write(|w| w.set_txc(true));
            if !self.busy() { Poll::Ready(()) } else { Poll::Pending }
        })
        .await;
        Ok(())
    }
}

impl<'d, M: Mode> UartTx<'d, M> {
    fn wrap(info: &'static Info, state: &'static State, pin: Peri<'d, AnyPin>) -> Self {
        Self {
            info,
            state,
            pin,
            owner: true,
            _phantom: PhantomData,
        }
    }

    /// Returns a half for [`Uart::split_ref`] that does not turn the peripheral off when it drops.
    fn borrowed(&mut self) -> UartTx<'_, M> {
        UartTx {
            info: self.info,
            state: self.state,
            pin: self.pin.reborrow(),
            owner: false,
            _phantom: PhantomData,
        }
    }

    /// Writes all bytes. Blocks until each byte is in the TX FIFO.
    pub fn blocking_write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        let r = self.info.regs;
        for &b in buffer {
            while !r.status().read().txfl() {}
            push_frame(self.info, self.state, b as u16, false);
        }
        Ok(())
    }

    /// Blocks until all written bytes are sent.
    pub fn blocking_flush(&mut self) -> Result<(), Error> {
        while self.busy() {}
        Ok(())
    }

    /// Returns `true` if a written byte is not sent yet, also directly after a write.
    pub fn busy(&self) -> bool {
        tx_busy(self.info, self.state)
    }

    /// Returns `true` if the TX FIFO has space for a frame.
    fn write_ready(&self) -> bool {
        self.info.regs.status().read().txfl()
    }

    /// Sends a break: one frame with data 0 and a low stop bit (`TXDATA.TXBREAK`).
    ///
    /// The line is high again before the next frame, so a longer break needs GPIO. Returns when the
    /// break frame is in the TX FIFO. [`busy`](Self::busy) stays `true` until the break ends.
    pub fn send_break(&mut self) {
        while !self.info.regs.status().read().txfl() {}
        push_frame(self.info, self.state, 0, true);
    }

    /// Changes the frame format and baud rate after the transmitter is idle.
    ///
    /// The frame registers change only while the EUSART is off, so this turns it off and on again,
    /// and unread received data is lost. On a split UART, the receiver changes too.
    pub fn set_config(&mut self, config: &Config) -> Result<(), ConfigError> {
        set_config(self.info, self.state, config)
    }

    /// Changes the baud rate while the EUSART stays on. On a split UART, the receiver changes too.
    pub fn set_baudrate(&mut self, baudrate: u32) -> Result<(), ConfigError> {
        set_baudrate(self.info, baudrate)
    }
}

/// Reads `IF` once and returns whether a frame is available.
///
/// # Errors
///
/// On an overrun, clears the RX error flags that are set and returns [`Error::Overrun`]. Parity and
/// framing errors are per frame: [`read_frame`] reports them.
fn rx_poll(r: eusart_mod::Eusart) -> Result<bool, Error> {
    let if_ = r.if_().read();
    if if_.rxof() {
        r.if_clr().write(|w| {
            w.set_rxof(true);
            w.set_perr(if_.perr());
            w.set_ferr(if_.ferr());
        });
        return Err(Error::Overrun);
    }
    Ok(r.status().read().rxfl())
}

/// Reads one frame in the asynchronous `RXDATA` layout.
///
/// A frame with a parity or framing error enters the FIFO like a good frame. `RXDATA.PERR` and
/// `RXDATA.FERR` give its errors, so the read consumes the frame and reports its own error. The
/// read clears both sticky flags in `IF`, and a parity error wins over a framing error.
fn read_frame(r: eusart_mod::Eusart) -> Result<u8, Error> {
    let x = r.rxdata_async().read();
    if x.perr() || x.ferr() {
        r.if_clr().write(|w| {
            w.set_perr(x.perr());
            w.set_ferr(x.ferr());
        });
        return Err(if x.perr() { Error::Parity } else { Error::Framing });
    }
    Ok(x.rxdata() as u8)
}

/// UART receiver.
pub struct UartRx<'d, M: Mode> {
    info: &'static Info,
    state: &'static State,
    pin: Peri<'d, AnyPin>,
    /// This half turns the peripheral off when it drops. See [`UartTx`].
    owner: bool,
    _phantom: PhantomData<M>,
}

impl<'d> UartRx<'d, Blocking> {
    /// Creates a blocking receiver on `rx` for instance `peri`.
    pub fn new_blocking<T: Instance>(
        _peri: Peri<'d, T>,
        rx: Peri<'d, impl RxPin<T>>,
        config: Config,
    ) -> Result<Self, ConfigError> {
        let rx: Peri<'d, AnyPin> = rx.into();
        init(T::info(), T::state(), None, Some(&rx), &config)?;
        T::state().tx_rx_refcount.store(1, Ordering::Relaxed);
        Ok(Self::wrap(T::info(), T::state(), rx))
    }
}

impl<'d> UartRx<'d, Async> {
    /// Creates an interrupt-driven receiver on `rx` for instance `peri`.
    ///
    /// Bind the `EUSARTn_RX` vector to [`RxInterruptHandler<T>`].
    pub fn new<T: Instance>(
        _peri: Peri<'d, T>,
        rx: Peri<'d, impl RxPin<T>>,
        _irq: impl interrupt::typelevel::Binding<T::RxInterrupt, RxInterruptHandler<T>> + 'd,
        config: Config,
    ) -> Result<Self, ConfigError> {
        let rx: Peri<'d, AnyPin> = rx.into();
        init(T::info(), T::state(), None, Some(&rx), &config)?;
        T::state().tx_rx_refcount.store(1, Ordering::Relaxed);
        T::RxInterrupt::unpend();
        unsafe { T::RxInterrupt::enable() };
        Ok(Self::wrap(T::info(), T::state(), rx))
    }

    /// Fills `buffer`. Yields while no data is available.
    pub async fn read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        let r = self.info.regs;
        for slot in buffer {
            self.wait_rx_ready().await?;
            *slot = read_frame(r)?;
        }
        Ok(())
    }

    /// Waits for at least one byte, then reads available bytes up to `buffer.len()`.
    ///
    /// Returns the number of bytes read.
    pub async fn read_buf(&mut self, buffer: &mut [u8]) -> Result<usize, Error> {
        if buffer.is_empty() {
            return Ok(0);
        }
        self.wait_rx_ready().await?;
        let r = self.info.regs;
        let mut n = 0;
        while n < buffer.len() && r.status().read().rxfl() {
            buffer[n] = read_frame(r)?;
            n += 1;
        }
        Ok(n)
    }

    /// Yields until a frame is available. Returns `Err` on an overrun.
    async fn wait_rx_ready(&mut self) -> Result<(), Error> {
        let r = self.info.regs;
        poll_fn(|cx| {
            match rx_poll(r) {
                Ok(true) => return Poll::Ready(Ok(())),
                Ok(false) => {}
                Err(e) => return Poll::Ready(Err(e)),
            }
            self.state.rx_waker.register(cx.waker());
            // Clear a stale level flag, so it gives no interrupt at once.
            r.if_clr().write(|w| w.set_rxfl(true));
            r.ien_set().write(|w| {
                w.set_rxfl(true);
                w.set_rxof(true);
            });
            match rx_poll(r) {
                Ok(true) => Poll::Ready(Ok(())),
                Ok(false) => Poll::Pending,
                Err(e) => Poll::Ready(Err(e)),
            }
        })
        .await
    }
}

impl<'d, M: Mode> UartRx<'d, M> {
    fn wrap(info: &'static Info, state: &'static State, pin: Peri<'d, AnyPin>) -> Self {
        Self {
            info,
            state,
            pin,
            owner: true,
            _phantom: PhantomData,
        }
    }

    /// Returns a half for [`Uart::split_ref`] that does not turn the peripheral off when it drops.
    fn borrowed(&mut self) -> UartRx<'_, M> {
        UartRx {
            info: self.info,
            state: self.state,
            pin: self.pin.reborrow(),
            owner: false,
            _phantom: PhantomData,
        }
    }

    /// Fills `buffer`. Blocks until each byte is received.
    pub fn blocking_read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        let r = self.info.regs;
        for slot in buffer {
            while !rx_poll(r)? {}
            *slot = read_frame(r)?;
        }
        Ok(())
    }

    /// Blocks until a byte is received, then reads available bytes up to `buffer.len()`.
    ///
    /// Returns the number of bytes read.
    pub fn blocking_read_buf(&mut self, buffer: &mut [u8]) -> Result<usize, Error> {
        if buffer.is_empty() {
            return Ok(0);
        }
        let r = self.info.regs;
        while !rx_poll(r)? {}
        let mut n = 0;
        while n < buffer.len() && r.status().read().rxfl() {
            buffer[n] = read_frame(r)?;
            n += 1;
        }
        Ok(n)
    }

    /// Returns `true` if a frame is available, or `Err` on an overrun.
    ///
    /// A parity or framing error stays with its frame, and the frame read returns it.
    fn read_ready(&mut self) -> Result<bool, Error> {
        rx_poll(self.info.regs)
    }

    /// Changes the frame format and baud rate, like [`UartTx::set_config`].
    ///
    /// Unread received data is lost. On a split UART, the transmitter changes too.
    pub fn set_config(&mut self, config: &Config) -> Result<(), ConfigError> {
        set_config(self.info, self.state, config)
    }

    /// Changes the baud rate. On a split UART, the transmitter changes too.
    pub fn set_baudrate(&mut self, baudrate: u32) -> Result<(), ConfigError> {
        set_baudrate(self.info, baudrate)
    }
}

impl<'d> Uart<'d, Blocking> {
    /// Creates a blocking UART on `tx` and `rx` for instance `peri`.
    pub fn new_blocking<T: Instance>(
        _peri: Peri<'d, T>,
        tx: Peri<'d, impl TxPin<T>>,
        rx: Peri<'d, impl RxPin<T>>,
        config: Config,
    ) -> Result<Self, ConfigError> {
        let tx: Peri<'d, AnyPin> = tx.into();
        let rx: Peri<'d, AnyPin> = rx.into();
        init(T::info(), T::state(), Some(&tx), Some(&rx), &config)?;
        Ok(Self::wrap::<T>(tx, rx))
    }
}

impl<'d> Uart<'d, Async> {
    /// Creates an interrupt-driven UART on `tx` and `rx` for instance `peri`.
    ///
    /// Bind the vectors to [`RxInterruptHandler<T>`] and [`TxInterruptHandler<T>`].
    pub fn new<T: Instance>(
        _peri: Peri<'d, T>,
        tx: Peri<'d, impl TxPin<T>>,
        rx: Peri<'d, impl RxPin<T>>,
        _irqs: impl interrupt::typelevel::Binding<T::RxInterrupt, RxInterruptHandler<T>>
        + interrupt::typelevel::Binding<T::TxInterrupt, TxInterruptHandler<T>>
        + 'd,
        config: Config,
    ) -> Result<Self, ConfigError> {
        let tx: Peri<'d, AnyPin> = tx.into();
        let rx: Peri<'d, AnyPin> = rx.into();
        init(T::info(), T::state(), Some(&tx), Some(&rx), &config)?;

        T::RxInterrupt::unpend();
        T::TxInterrupt::unpend();
        unsafe {
            T::RxInterrupt::enable();
            T::TxInterrupt::enable();
        }

        Ok(Self::wrap::<T>(tx, rx))
    }
}

impl<'d, M: Mode> Uart<'d, M> {
    fn wrap<T: Instance>(tx: Peri<'d, AnyPin>, rx: Peri<'d, AnyPin>) -> Self {
        T::state().tx_rx_refcount.store(2, Ordering::Relaxed);
        Self {
            tx: UartTx::wrap(T::info(), T::state(), tx),
            rx: UartRx::wrap(T::info(), T::state(), rx),
        }
    }
}

impl<M: Mode> Drop for UartTx<'_, M> {
    fn drop(&mut self) {
        if !self.owner {
            return;
        }
        while self.busy() {}
        (self.info.enable_tx_route)(false);
        self.info.regs.cmd().write(|w| w.set_txdis(true));
        wait_sync_cmd(self.info.regs);
        crate::gpio::set_as_disabled(&self.pin);
        drop_tx_rx(self.info, self.state);
    }
}

impl<M: Mode> Drop for UartRx<'_, M> {
    fn drop(&mut self) {
        if !self.owner {
            return;
        }
        // Stop the receiver, so it does not sample the disabled pin.
        (self.info.enable_rx_route)(false);
        self.info.regs.cmd().write(|w| w.set_rxdis(true));
        wait_sync_cmd(self.info.regs);
        crate::gpio::set_as_disabled(&self.pin);
        drop_tx_rx(self.info, self.state);
    }
}

/// Turns off the peripheral, its interrupts, GPIO route and clock when the last half drops.
fn drop_tx_rx(info: &Info, state: &State) {
    if state.tx_rx_refcount.fetch_sub(1, Ordering::Relaxed) == 1 {
        let r = info.regs;
        // The handler reads EUSART registers, so it must not run after the clock is off.
        (info.disable_irqs)();
        disable(r);
        r.ien().write(|_| {});
        r.if_clr().write_value(eusart_mod::regs::If(0xFFFF_FFFF));
        (info.enable_tx_route)(false);
        (info.enable_rx_route)(false);
        (info.disable_clock)();
    }
}

crate::uart_common::impl_uart_common!();
