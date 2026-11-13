//! Series 0 (EFM32GG) USART and UART, as a UART.
//!
//! The API matches the Series 2 `eusart` driver: [`Uart`], [`UartTx`] and [`UartRx`] in `Blocking`
//! and `Async` modes. It also drives UART0 and UART1, which are USART blocks in asynchronous mode.
//!
//! # Pin routing
//!
//! A Series 0 peripheral has a few numbered pin locations. One location number, written to
//! `ROUTE.LOCATION`, selects the pins of all signals of the peripheral.
//!
//! [`Uart::new`] and [`Uart::new_blocking`] need a TX pin and an RX pin that share a location
//! ([`UartPins`]), and use the smallest shared location. Other pin pairs do not compile.
//!
//! # Examples
//!
//! ```rust,ignore
//! use embassy_silabs::{bind_interrupts, peripherals, usart};
//!
//! bind_interrupts!(struct Irqs {
//!     USART1_RX => usart::RxInterruptHandler<peripherals::USART1>;
//!     USART1_TX => usart::TxInterruptHandler<peripherals::USART1>;
//! });
//!
//! // USART1 location 1: TX PD0, RX PD1.
//! let mut uart =
//!     usart::Uart::new(p.USART1, p.PD0, p.PD1, Irqs, usart::Config::default()).unwrap();
//! uart.write(b"hello\r\n").await.unwrap();
//! ```

use core::future::poll_fn;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use core::task::Poll;

use embassy_hal_internal::{Peri, PeripheralType};
use embassy_sync::waitqueue::AtomicWaker;

use crate::gpio::AnyPin;
use crate::interrupt;
use crate::interrupt::typelevel::Interrupt;
use crate::mode::{Async, Blocking, Mode};
use crate::pac::usart as usart_mod;
use crate::pac::usart::vals;
use crate::time::Hertz;
pub use crate::uart_common::{Config, ConfigError, DataBits, Error, Parity, StopBits};

/// Asynchronous oversampling. The divider math assumes 16x.
const OVERSAMPLE: u32 = 16;

fn databits(d: DataBits) -> vals::Databits {
    match d {
        DataBits::DataBits5 => vals::Databits::Five,
        DataBits::DataBits6 => vals::Databits::Six,
        DataBits::DataBits7 => vals::Databits::Seven,
        DataBits::DataBits8 => vals::Databits::Eight,
        // The byte API sends the 9th bit as 0 (`CTRL.BIT8DV`).
        DataBits::DataBits9 => vals::Databits::Nine,
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
/// bit 6, so `CLKDIV = DIV * 64` and `DIV = 4 * f / (ovs * baud) - 4`.
fn compute_clkdiv(src: Hertz, baudrate: u32) -> Result<u16, ConfigError> {
    let denom = OVERSAMPLE as u64 * baudrate as u64;
    if denom == 0 {
        return Err(ConfigError::BaudrateTooLow);
    }
    let quarter_div = (4 * src.0 as u64 + denom / 2) / denom;
    if quarter_div < 4 {
        return Err(ConfigError::BaudrateTooHigh);
    }
    let div = quarter_div - 4;
    // `CLKDIV.DIV` is 15 bits wide.
    if div > 0x7FFF {
        return Err(ConfigError::BaudrateTooLow);
    }
    Ok(div as u16)
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

/// Type-erased instance data.
pub(crate) struct Info {
    pub(crate) regs: usart_mod::Usart,
    pub(crate) enable_clock: fn(),
    pub(crate) disable_clock: fn(),
    pub(crate) frequency: fn() -> Hertz,
    /// Disables and unpends this instance's RX and TX interrupts in the NVIC.
    pub(crate) disable_irqs: fn(),
}

pub(crate) trait SealedInstance {
    fn info() -> &'static Info;
    fn state() -> &'static State;
}

/// A USART or UART instance.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + crate::cmu::CmuPeripheral + PeripheralType + 'static {
    /// Interrupt for RX events.
    type RxInterrupt: interrupt::typelevel::Interrupt;
    /// Interrupt for TX events.
    type TxInterrupt: interrupt::typelevel::Interrupt;
}

pin_trait!(TxPin, Instance, locations);
pin_trait!(RxPin, Instance, locations);

/// A TX pin and an RX pin at one ROUTE LOCATION for instance `T`.
///
/// `build.rs` implements it for each such pair in chip metadata. `LOCATION` is the smallest
/// location that the pins share.
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a (TX, RX) pin pair at one ROUTE LOCATION of `{T}`",
    label = "the TX pin and the RX pin share no ROUTE LOCATION",
    note = "a Series 0 USART routes all of its signals through one ROUTE LOCATION: use a TX pin and an RX pin of the same location"
)]
pub trait UartPins<T: Instance> {
    /// The pair's ROUTE LOCATION.
    const LOCATION: u8;
}

/// Implements [`Instance`] for one USART or UART.
///
/// `build.rs` invokes it for each `usart` peripheral in chip metadata.
macro_rules! impl_usart {
    ($inst:ident, $rxirq:ident, $txirq:ident $(,)?) => {
        impl crate::usart::SealedInstance for crate::peripherals::$inst {
            fn info() -> &'static crate::usart::Info {
                static INFO: crate::usart::Info = crate::usart::Info {
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
                };
                &INFO
            }
            fn state() -> &'static crate::usart::State {
                static STATE: crate::usart::State = crate::usart::State::new();
                &STATE
            }
        }
        impl crate::usart::Instance for crate::peripherals::$inst {
            type RxInterrupt = crate::interrupt::typelevel::$rxirq;
            type TxInterrupt = crate::interrupt::typelevel::$txirq;
        }
    };
}
pub(crate) use impl_usart;

/// Interrupt handler for the RX vector (`USARTn_RX`, `UARTn_RX`).
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

/// Interrupt handler for the TX vector (`USARTn_TX`, `UARTn_TX`).
pub struct TxInterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::TxInterrupt> for TxInterruptHandler<T> {
    unsafe fn on_interrupt() {
        on_interrupt(T::info(), T::state());
    }
}

/// Disables the interrupt sources that fired and wakes their futures, which enable them again.
///
/// `RXDATAV` and `TXBL` are levels whose flags clear only when the condition is false, so the
/// handler must disable them. The receiver arms `RXDATAV` and `RXOF` only. A frame with a parity or
/// framing error also sets `RXDATAV`, and its read reports the error.
fn on_interrupt(info: &Info, state: &State) {
    let r = info.regs;
    let if_ = r.if_().read();
    let ien = r.ien().read();

    let rx = (if_.rxdatav() && ien.rxdatav()) || (if_.rxof() && ien.rxof());
    let tx = (if_.txbl() && ien.txbl()) || (if_.txc() && ien.txc());
    critical_section::with(|_| {
        r.ien().modify(|w| {
            if rx {
                w.set_rxdatav(false);
                w.set_rxof(false);
            }
            if tx {
                w.set_txbl(false);
                w.set_txc(false);
            }
        });
    });
    if rx {
        state.rx_waker.wake();
    }
    if tx {
        state.tx_waker.wake();
    }
}

/// Sets interrupt enables in a critical section, because the handler changes `IEN` too.
fn ien_set(r: usart_mod::Usart, f: impl FnOnce(&mut usart_mod::regs::Ien)) {
    critical_section::with(|_| r.ien().modify(f));
}

/// Writes the frame format, inversion and loopback bits, and baud rate.
///
/// Writes nothing when the baud rate is not possible.
fn apply_config(info: &Info, config: &Config) -> Result<(), ConfigError> {
    let div = compute_clkdiv((info.frequency)(), config.baudrate)?;
    let r = info.regs;
    r.ctrl().modify(|w| {
        w.set_txinv(config.invert_tx);
        w.set_rxinv(config.invert_rx);
        w.set_loopbk(config.loopback);
    });
    r.frame().write(|w| {
        w.set_databits(databits(config.data_bits));
        w.set_parity(parity(config.parity));
        w.set_stopbits(stopbits(config.stop_bits));
    });
    r.clkdiv().write(|w| w.set_div(div));
    debug!("usart: baud={} clkdiv={}", config.baudrate, div);
    Ok(())
}

/// Turns the peripheral on and configures it for the given pins at ROUTE location `location`.
fn init(
    info: &Info,
    state: &State,
    tx: Option<&AnyPin>,
    rx: Option<&AnyPin>,
    location: u8,
    config: &Config,
) -> Result<(), ConfigError> {
    (info.enable_clock)();
    let r = info.regs;
    r.cmd().write(|w| {
        w.set_rxdis(true);
        w.set_txdis(true);
        w.set_masterdis(true);
        w.set_clearrx(true);
        w.set_cleartx(true);
    });
    // MSBF stays 0: LSB first.
    r.ctrl().write(|w| {
        w.set_sync(false);
        w.set_ovs(vals::Ovs::X16);
    });
    if let Err(e) = apply_config(info, config) {
        (info.disable_clock)();
        return Err(e);
    }
    r.ien().write(|_| {});
    r.if_clr().write_value(usart_mod::regs::Ifc(0xFFFF_FFFF));
    state.tx_pending.store(false, Ordering::Relaxed);

    // TX is a push-pull output, idle high, and RX is an input.
    if let Some(tx) = tx {
        crate::gpio::set_as_alternate_output(tx, !config.invert_tx);
    }
    if let Some(rx) = rx {
        crate::gpio::set_as_alternate_input(rx);
    }
    // `build.rs` checks that the TX and RX enable fields are `route.txpen` and `route.rxpen`.
    r.route().write(|w| {
        w.set_location(vals::Location::from_bits(location));
        w.set_txpen(tx.is_some());
        w.set_rxpen(rx.is_some());
    });
    debug!("usart: location={}", location);

    r.cmd().write(|w| {
        w.set_rxen(rx.is_some());
        w.set_txen(tx.is_some());
    });
    while {
        let st = r.status().read();
        (rx.is_some() && !st.rxens()) || (tx.is_some() && !st.txens())
    } {}
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
    /// Creates a blocking transmitter on `tx` for instance `peri`, at the smallest `tx` location.
    pub fn new_blocking<T: Instance, X: TxPin<T>>(
        _peri: Peri<'d, T>,
        tx: Peri<'d, X>,
        config: Config,
    ) -> Result<Self, ConfigError> {
        let location = X::LOCATIONS[0];
        let tx: Peri<'d, AnyPin> = tx.into();
        init(T::info(), T::state(), Some(&tx), None, location, &config)?;
        T::state().tx_rx_refcount.store(1, Ordering::Relaxed);
        Ok(Self::wrap(T::info(), T::state(), tx))
    }
}

impl<'d> UartTx<'d, Async> {
    /// Creates an interrupt-driven transmitter on `tx` for `peri`, at the smallest `tx` location.
    ///
    /// Bind the TX vector to [`TxInterruptHandler<T>`].
    pub fn new<T: Instance, X: TxPin<T>>(
        _peri: Peri<'d, T>,
        tx: Peri<'d, X>,
        _irq: impl interrupt::typelevel::Binding<T::TxInterrupt, TxInterruptHandler<T>> + 'd,
        config: Config,
    ) -> Result<Self, ConfigError> {
        let location = X::LOCATIONS[0];
        let tx: Peri<'d, AnyPin> = tx.into();
        init(T::info(), T::state(), Some(&tx), None, location, &config)?;
        T::state().tx_rx_refcount.store(1, Ordering::Relaxed);
        T::TxInterrupt::unpend();
        unsafe { T::TxInterrupt::enable() };
        Ok(Self::wrap(T::info(), T::state(), tx))
    }

    /// Writes all bytes. Yields while the transmit buffer is full.
    pub async fn write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        let r = self.info.regs;
        for &b in buffer {
            poll_fn(|cx| {
                if r.status().read().txbl() {
                    return Poll::Ready(());
                }
                self.state.tx_waker.register(cx.waker());
                ien_set(r, |w| w.set_txbl(true));
                if r.status().read().txbl() {
                    Poll::Ready(())
                } else {
                    Poll::Pending
                }
            })
            .await;
            // Set before the write, so `busy()` cannot miss this frame.
            self.state.tx_pending.store(true, Ordering::Relaxed);
            r.txdata().write(|w| w.set_txdata(b));
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
            // `IF.TXC` is sticky. A stale flag would fire the interrupt at once, at each poll.
            r.if_clr().write(|w| w.set_txc(true));
            ien_set(r, |w| w.set_txc(true));
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

    /// Returns `true` if the transmit buffer has space for a frame.
    fn write_ready(&self) -> bool {
        self.info.regs.status().read().txbl()
    }

    /// Writes all bytes. Blocks until each byte is in the transmit buffer.
    pub fn blocking_write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        let r = self.info.regs;
        for &b in buffer {
            while !r.status().read().txbl() {}
            self.state.tx_pending.store(true, Ordering::Relaxed);
            r.txdata().write(|w| w.set_txdata(b));
        }
        Ok(())
    }

    /// Blocks until all written bytes are sent.
    pub fn blocking_flush(&mut self) -> Result<(), Error> {
        while self.busy() {}
        Ok(())
    }

    /// Returns `true` if a written byte is not sent yet.
    pub fn busy(&self) -> bool {
        tx_busy(self.info, self.state)
    }

    /// Sends a break: one frame with data 0 and a low stop bit (`TXDATAX.TXBREAK`).
    ///
    /// The line is high again before the next frame, so a longer break needs GPIO. Returns when the
    /// frame is in the transmit buffer, and [`busy`](Self::busy) stays `true` until the break ends.
    pub fn send_break(&mut self) {
        let r = self.info.regs;
        while !r.status().read().txbl() {}
        self.state.tx_pending.store(true, Ordering::Relaxed);
        r.txdatax().write(|w| {
            w.set_txdatax(0);
            w.set_txbreak(true);
        });
    }

    /// Changes the configuration once the transmitter is idle.
    ///
    /// A frame received during the change can be wrong. On a split UART, the receiver changes too.
    pub fn set_config(&mut self, config: &Config) -> Result<(), ConfigError> {
        set_config(self.info, self.state, config)
    }

    /// Changes the baud rate. On a split UART, the receiver changes too.
    pub fn set_baudrate(&mut self, baudrate: u32) -> Result<(), ConfigError> {
        set_baudrate(self.info, baudrate)
    }
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
    /// Creates a blocking receiver on `rx` for instance `peri`, at the smallest `rx` location.
    pub fn new_blocking<T: Instance, R: RxPin<T>>(
        _peri: Peri<'d, T>,
        rx: Peri<'d, R>,
        config: Config,
    ) -> Result<Self, ConfigError> {
        let location = R::LOCATIONS[0];
        let rx: Peri<'d, AnyPin> = rx.into();
        init(T::info(), T::state(), None, Some(&rx), location, &config)?;
        T::state().tx_rx_refcount.store(1, Ordering::Relaxed);
        Ok(Self::wrap(T::info(), T::state(), rx))
    }
}

impl<'d> UartRx<'d, Async> {
    /// Creates an interrupt-driven receiver on `rx` for `peri`, at the smallest `rx` location.
    ///
    /// Bind the RX vector to [`RxInterruptHandler<T>`].
    pub fn new<T: Instance, R: RxPin<T>>(
        _peri: Peri<'d, T>,
        rx: Peri<'d, R>,
        _irq: impl interrupt::typelevel::Binding<T::RxInterrupt, RxInterruptHandler<T>> + 'd,
        config: Config,
    ) -> Result<Self, ConfigError> {
        let location = R::LOCATIONS[0];
        let rx: Peri<'d, AnyPin> = rx.into();
        init(T::info(), T::state(), None, Some(&rx), location, &config)?;
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
            *slot = read_frame(&r)?;
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
        while n < buffer.len() && r.status().read().rxdatav() {
            buffer[n] = read_frame(&r)?;
            n += 1;
        }
        Ok(n)
    }

    /// Yields until a frame is available. Returns `Err` on an overrun.
    async fn wait_rx_ready(&mut self) -> Result<(), Error> {
        let r = self.info.regs;
        poll_fn(|cx| {
            match rx_poll(&r) {
                Ok(true) => return Poll::Ready(Ok(())),
                Ok(false) => {}
                Err(e) => return Poll::Ready(Err(e)),
            }
            self.state.rx_waker.register(cx.waker());
            ien_set(r, |w| {
                w.set_rxdatav(true);
                w.set_rxof(true);
            });
            match rx_poll(&r) {
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

    /// Returns `true` if a frame is available, or `Err` on an overrun.
    fn read_ready(&mut self) -> Result<bool, Error> {
        rx_poll(&self.info.regs)
    }

    /// Fills `buffer`. Blocks until each byte is received.
    pub fn blocking_read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        let r = self.info.regs;
        for slot in buffer {
            while !rx_poll(&r)? {}
            *slot = read_frame(&r)?;
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
        while !rx_poll(&r)? {}
        let mut n = 0;
        while n < buffer.len() && r.status().read().rxdatav() {
            buffer[n] = read_frame(&r)?;
            n += 1;
        }
        Ok(n)
    }

    /// Changes the frame format and baud rate, like [`UartTx::set_config`].
    ///
    /// On a split UART, the transmitter changes too.
    pub fn set_config(&mut self, config: &Config) -> Result<(), ConfigError> {
        set_config(self.info, self.state, config)
    }

    /// Changes the baud rate. On a split UART, the transmitter changes too.
    pub fn set_baudrate(&mut self, baudrate: u32) -> Result<(), ConfigError> {
        set_baudrate(self.info, baudrate)
    }
}

/// Returns `true` if a written frame is not sent yet.
///
/// `STATUS.TXC` is 0 at reset and is set only after a frame is sent, so `tx_pending` tells an
/// unused transmitter from a busy one.
fn tx_busy(info: &Info, state: &State) -> bool {
    if !state.tx_pending.load(Ordering::Relaxed) {
        return false;
    }
    if info.regs.status().read().txc() {
        state.tx_pending.store(false, Ordering::Relaxed);
        return false;
    }
    true
}

/// Waits until the transmitter is idle, then writes the configuration.
fn set_config(info: &Info, state: &State, config: &Config) -> Result<(), ConfigError> {
    // Check the baud rate first, so an error returns at once.
    compute_clkdiv((info.frequency)(), config.baudrate)?;
    while tx_busy(info, state) {}
    apply_config(info, config)
}

fn set_baudrate(info: &Info, baudrate: u32) -> Result<(), ConfigError> {
    let div = compute_clkdiv((info.frequency)(), baudrate)?;
    info.regs.clkdiv().write(|w| w.set_div(div));
    Ok(())
}

/// Reads `IF` once and returns whether a frame is available.
///
/// # Errors
///
/// On an overrun, clears the RX error flags that are set and returns [`Error::Overrun`]. Parity and
/// framing errors are per frame: [`read_frame`] reports them.
fn rx_poll(r: &usart_mod::Usart) -> Result<bool, Error> {
    let if_ = r.if_().read();
    if if_.rxof() {
        r.if_clr().write(|w| {
            w.set_rxof(true);
            w.set_perr(if_.perr());
            w.set_ferr(if_.ferr());
        });
        return Err(Error::Overrun);
    }
    Ok(r.status().read().rxdatav())
}

/// Reads one frame.
///
/// A frame with a parity or framing error enters the receive buffer like a good frame (RM
/// 17.3.2.4.4 and 17.3.2.4.5). `RXDATAX` gives its errors, so the read consumes the frame and
/// reports its own error. The read clears both sticky `IF` flags, and parity wins over framing.
fn read_frame(r: &usart_mod::Usart) -> Result<u8, Error> {
    let x = r.rxdatax().read();
    if x.perr() || x.ferr() {
        r.if_clr().write(|w| {
            w.set_perr(x.perr());
            w.set_ferr(x.ferr());
        });
        return Err(if x.perr() { Error::Parity } else { Error::Framing });
    }
    Ok(x.rxdata() as u8)
}

impl<'d> Uart<'d, Blocking> {
    /// Creates a blocking UART on `tx` and `rx` for instance `peri`.
    ///
    /// The pins must share a ROUTE LOCATION: `(X, R)` must implement [`UartPins`].
    pub fn new_blocking<T: Instance, X: TxPin<T>, R: RxPin<T>>(
        _peri: Peri<'d, T>,
        tx: Peri<'d, X>,
        rx: Peri<'d, R>,
        config: Config,
    ) -> Result<Self, ConfigError>
    where
        (X, R): UartPins<T>,
    {
        let location = <(X, R) as UartPins<T>>::LOCATION;
        let tx: Peri<'d, AnyPin> = tx.into();
        let rx: Peri<'d, AnyPin> = rx.into();
        init(T::info(), T::state(), Some(&tx), Some(&rx), location, &config)?;
        Ok(Self::wrap::<T>(tx, rx))
    }
}

impl<'d> Uart<'d, Async> {
    /// Creates an interrupt-driven UART on `tx` and `rx` for instance `peri`.
    ///
    /// Bind the vectors to [`RxInterruptHandler<T>`] and [`TxInterruptHandler<T>`]. The pins must
    /// share a ROUTE LOCATION: `(X, R)` must implement [`UartPins`].
    pub fn new<T: Instance, X: TxPin<T>, R: RxPin<T>>(
        _peri: Peri<'d, T>,
        tx: Peri<'d, X>,
        rx: Peri<'d, R>,
        _irqs: impl interrupt::typelevel::Binding<T::RxInterrupt, RxInterruptHandler<T>>
        + interrupt::typelevel::Binding<T::TxInterrupt, TxInterruptHandler<T>>
        + 'd,
        config: Config,
    ) -> Result<Self, ConfigError>
    where
        (X, R): UartPins<T>,
    {
        let location = <(X, R) as UartPins<T>>::LOCATION;
        let tx: Peri<'d, AnyPin> = tx.into();
        let rx: Peri<'d, AnyPin> = rx.into();
        init(T::info(), T::state(), Some(&tx), Some(&rx), location, &config)?;

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
        self.info.regs.cmd().write(|w| w.set_txdis(true));
        self.info.regs.route().modify(|w| w.set_txpen(false));
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
        self.info.regs.cmd().write(|w| w.set_rxdis(true));
        self.info.regs.route().modify(|w| w.set_rxpen(false));
        crate::gpio::set_as_disabled(&self.pin);
        drop_tx_rx(self.info, self.state);
    }
}

/// Turns off the peripheral, its route, interrupts and clock when the last half drops.
fn drop_tx_rx(info: &Info, state: &State) {
    if state.tx_rx_refcount.fetch_sub(1, Ordering::Relaxed) == 1 {
        let r = info.regs;
        // The handler reads USART registers, so it must not run after the clock is off.
        (info.disable_irqs)();
        r.ien().write(|_| {});
        r.if_clr().write_value(usart_mod::regs::Ifc(0xFFFF_FFFF));
        r.cmd().write(|w| {
            w.set_rxdis(true);
            w.set_txdis(true);
        });
        r.route().write(|_| {});
        (info.disable_clock)();
    }
}

crate::uart_common::impl_uart_common!();
