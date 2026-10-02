//! Series 0 (EFM32GG) USART and UART, as a UART.
//!
//! The API follows the Series 2 EUSART driver and embassy-stm32: [`Uart`],
//! [`UartTx`] and [`UartRx`] with the [`Blocking`] and [`Async`] modes.
//! UART0 and UART1 are USART blocks in asynchronous mode, so this driver
//! serves them too.
//!
//! # Pin routing
//!
//! A Series 0 peripheral routes its pins through `ROUTE.LOCATION`. One
//! location number selects the pins of all signals of the peripheral. The
//! [`TxPin`] and [`RxPin`] traits give the locations at which a pin carries
//! the signal (`LOCATIONS`). [`Uart::new`] and [`Uart::new_blocking`] use the
//! smallest location that both pins have. A pin that is at no location of
//! the signal does not compile. An RX pin and a TX pin that share no location
//! do not compile either: the constructors check this in a `const` block.
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
//! let mut uart = usart::Uart::new(p.USART1, p.PD1, p.PD0, Irqs, usart::Config::default()).unwrap();
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
pub use crate::mode::{Async, Blocking, Mode};
use crate::pac::usart as usart_mod;
use crate::pac::usart::vals;
use crate::time::Hertz;

/// Asynchronous oversampling. The divider math assumes 16x.
const OVERSAMPLE: u32 = 16;

/// Number of data bits per frame.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum DataBits {
    /// 5 data bits.
    DataBits5,
    /// 6 data bits.
    DataBits6,
    /// 7 data bits.
    DataBits7,
    /// 8 data bits.
    DataBits8,
    /// 9 data bits. The byte API sends the 9th bit as 0 (`CTRL.BIT8DV`), and
    /// drops it on receive.
    DataBits9,
}

impl DataBits {
    fn to_vals(self) -> vals::Databits {
        match self {
            DataBits::DataBits5 => vals::Databits::Five,
            DataBits::DataBits6 => vals::Databits::Six,
            DataBits::DataBits7 => vals::Databits::Seven,
            DataBits::DataBits8 => vals::Databits::Eight,
            DataBits::DataBits9 => vals::Databits::Nine,
        }
    }
}

/// Parity bit.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Parity {
    /// No parity bit.
    ParityNone,
    /// Even parity.
    ParityEven,
    /// Odd parity.
    ParityOdd,
}

impl Parity {
    fn to_vals(self) -> vals::Parity {
        match self {
            Parity::ParityNone => vals::Parity::None,
            Parity::ParityEven => vals::Parity::Even,
            Parity::ParityOdd => vals::Parity::Odd,
        }
    }
}

/// Number of stop bits.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum StopBits {
    /// 0.5 stop bits.
    STOP0P5,
    /// 1 stop bit.
    STOP1,
    /// 1.5 stop bits.
    STOP1P5,
    /// 2 stop bits.
    STOP2,
}

impl StopBits {
    fn to_vals(self) -> vals::Stopbits {
        match self {
            StopBits::STOP0P5 => vals::Stopbits::Half,
            StopBits::STOP1 => vals::Stopbits::One,
            StopBits::STOP1P5 => vals::Stopbits::Oneandahalf,
            StopBits::STOP2 => vals::Stopbits::Two,
        }
    }
}

/// UART configuration.
#[non_exhaustive]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Config {
    /// Baud rate in bits per second.
    pub baudrate: u32,
    /// Number of data bits.
    pub data_bits: DataBits,
    /// Parity.
    pub parity: Parity,
    /// Number of stop bits.
    pub stop_bits: StopBits,
}

impl Default for Config {
    /// 115200 8N1.
    fn default() -> Self {
        Self {
            baudrate: 115_200,
            data_bits: DataBits::DataBits8,
            parity: Parity::ParityNone,
            stop_bits: StopBits::STOP1,
        }
    }
}

/// Runtime UART error.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// Framing error: the stop bit was not high.
    Framing,
    /// The parity bit was wrong.
    Parity,
    /// The receive buffer overflowed and data was lost.
    Overrun,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let s = match self {
            Error::Framing => "framing error",
            Error::Parity => "parity error",
            Error::Overrun => "RX overrun",
        };
        write!(f, "{s}")
    }
}

impl core::error::Error for Error {}

/// Error from a constructor or from `set_config`.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum ConfigError {
    /// The baud rate is too low for HFPERCLK: the divider does not fit in
    /// `CLKDIV.DIV`.
    BaudrateTooLow,
    /// The baud rate is too high for HFPERCLK.
    BaudrateTooHigh,
}

impl core::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let s = match self {
            ConfigError::BaudrateTooLow => "baud rate too low",
            ConfigError::BaudrateTooHigh => "baud rate too high",
        };
        write!(f, "{s}")
    }
}

impl core::error::Error for ConfigError {}

/// The smallest location that is in both lists, at compile time.
const fn common_location(rx: &[u8], tx: &[u8]) -> Option<u8> {
    let mut best: Option<u8> = None;
    let mut i = 0;
    while i < rx.len() {
        let mut j = 0;
        while j < tx.len() {
            if rx[i] == tx[j] {
                best = match best {
                    Some(b) if b <= rx[i] => Some(b),
                    _ => Some(rx[i]),
                };
            }
            j += 1;
        }
        i += 1;
    }
    best
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
fn on_interrupt(info: &Info, state: &State) {
    let r = info.regs;
    let if_ = r.if_().read();
    let ien = r.ien().read();

    let rx = (if_.rxdatav() && ien.rxdatav())
        || (if_.rxof() && ien.rxof())
        || (if_.perr() && ien.perr())
        || (if_.ferr() && ien.ferr());
    let tx = (if_.txbl() && ien.txbl()) || (if_.txc() && ien.txc());
    critical_section::with(|_| {
        r.ien().modify(|w| {
            if rx {
                w.set_rxdatav(false);
                w.set_rxof(false);
                w.set_perr(false);
                w.set_ferr(false);
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

/// Write the frame format and the baud rate. Nothing is written when the
/// baud rate is not possible.
fn apply_config(info: &Info, config: &Config) -> Result<(), ConfigError> {
    let div = compute_clkdiv((info.frequency)(), config.baudrate)?;
    let r = info.regs;
    r.frame().write(|w| {
        w.set_databits(config.data_bits.to_vals());
        w.set_parity(config.parity.to_vals());
        w.set_stopbits(config.stop_bits.to_vals());
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
        crate::gpio::set_as_alternate_output(tx);
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
    /// Create a blocking transmitter on `tx` for instance `peri`, at the
    /// smallest location of `tx`.
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
    /// Create an interrupt-driven transmitter on `tx` for instance `peri`,
    /// at the smallest location of `tx`. Bind the TX vector to
    /// [`TxInterruptHandler<T>`].
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

    /// Write all bytes. Wait for space in the transmit buffer before each
    /// byte.
    pub fn blocking_write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        let r = self.info.regs;
        for &b in buffer {
            while !r.status().read().txbl() {}
            self.state.tx_pending.store(true, Ordering::Relaxed);
            r.txdata().write(|w| w.set_txdata(b));
        }
        Ok(())
    }

    /// Wait until all written bytes are sent.
    pub fn blocking_flush(&mut self) -> Result<(), Error> {
        while self.busy() {}
        Ok(())
    }

    /// The transmitter is busy: a written byte is not sent yet.
    pub fn busy(&self) -> bool {
        if !self.state.tx_pending.load(Ordering::Relaxed) {
            return false;
        }
        if self.info.regs.status().read().txc() {
            self.state.tx_pending.store(false, Ordering::Relaxed);
            return false;
        }
        true
    }

    /// Send a break: one frame with data 0 and a low stop bit
    /// (`TXDATAX.TXBREAK`). The line is high again before the next frame.
    /// A break longer than one frame needs the GPIO.
    pub fn send_break(&mut self) {
        let r = self.info.regs;
        while !r.status().read().txbl() {}
        self.state.tx_pending.store(true, Ordering::Relaxed);
        r.txdatax().write(|w| {
            w.set_txdatax(0);
            w.set_txbreak(true);
        });
    }

    /// Change the frame format and the baud rate. Call it when the
    /// transmitter is idle. On a split driver, the receiver changes too.
    pub fn set_config(&mut self, config: &Config) -> Result<(), ConfigError> {
        apply_config(self.info, config)
    }

    /// Change the baud rate. On a split driver, the receiver changes too.
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
    /// Create a blocking receiver on `rx` for instance `peri`, at the
    /// smallest location of `rx`.
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
    /// Create an interrupt-driven receiver on `rx` for instance `peri`, at
    /// the smallest location of `rx`. Bind the RX vector to
    /// [`RxInterruptHandler<T>`].
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

    /// Fill `buffer`. Wait for each byte.
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

    /// Change the frame format and the baud rate. On a split driver, the
    /// transmitter changes too.
    pub fn set_config(&mut self, config: &Config) -> Result<(), ConfigError> {
        apply_config(self.info, config)
    }

    /// Change the baud rate. On a split driver, the transmitter changes too.
    pub fn set_baudrate(&mut self, baudrate: u32) -> Result<(), ConfigError> {
        set_baudrate(self.info, baudrate)
    }
}

fn set_baudrate(info: &Info, baudrate: u32) -> Result<(), ConfigError> {
    let div = compute_clkdiv((info.frequency)(), baudrate)?;
    info.regs.clkdiv().write(|w| w.set_div(div));
    Ok(())
}

/// Check for an overrun, then return whether a frame is available.
fn rx_poll(r: &usart_mod::Usart) -> Result<bool, Error> {
    if r.if_().read().rxof() {
        r.if_clr().write(|w| w.set_rxof(true));
        return Err(Error::Overrun);
    }
    Ok(r.status().read().rxdatav())
}

/// Read one frame. A frame with a framing or parity error enters the
/// receive buffer like a good frame (RM 17.3.2.4.4 and 17.3.2.4.5).
/// `RXDATAX` gives the error flags of this frame, so the read consumes the
/// frame and reports its own error.
fn read_frame(r: &usart_mod::Usart) -> Result<u8, Error> {
    let x = r.rxdatax().read();
    if x.ferr() {
        r.if_clr().write(|w| w.set_ferr(true));
        return Err(Error::Framing);
    }
    if x.perr() {
        r.if_clr().write(|w| w.set_perr(true));
        return Err(Error::Parity);
    }
    Ok(x.rxdata() as u8)
}

/// Bidirectional UART.
pub struct Uart<'d, M: Mode> {
    tx: UartTx<'d, M>,
    rx: UartRx<'d, M>,
}

impl<'d> Uart<'d, Blocking> {
    /// Create a blocking UART on `rx` and `tx` for instance `peri`.
    ///
    /// The driver uses the smallest location that both pins have. Pins that
    /// share no location fail to compile in `cargo build`. `cargo check` does
    /// not evaluate this check.
    pub fn new_blocking<T: Instance, R: RxPin<T>, X: TxPin<T>>(
        _peri: Peri<'d, T>,
        rx: Peri<'d, R>,
        tx: Peri<'d, X>,
        config: Config,
    ) -> Result<Self, ConfigError> {
        let location = const {
            match common_location(R::LOCATIONS, X::LOCATIONS) {
                Some(l) => l,
                None => ::core::panic!("usart: the RX pin and the TX pin share no ROUTE LOCATION"),
            }
        };
        let tx: Peri<'d, AnyPin> = tx.into();
        let rx: Peri<'d, AnyPin> = rx.into();
        init(T::info(), T::state(), Some(&tx), Some(&rx), location, &config)?;
        Ok(Self::wrap::<T>(tx, rx))
    }
}

impl<'d> Uart<'d, Async> {
    /// Create an interrupt-driven UART on `rx` and `tx` for instance `peri`.
    ///
    /// The driver uses the smallest location that both pins have. Pins that
    /// share no location fail to compile in `cargo build`. `cargo check` does
    /// not evaluate this check.
    pub fn new<T: Instance, R: RxPin<T>, X: TxPin<T>>(
        _peri: Peri<'d, T>,
        rx: Peri<'d, R>,
        tx: Peri<'d, X>,
        _irqs: impl interrupt::typelevel::Binding<T::RxInterrupt, RxInterruptHandler<T>>
        + interrupt::typelevel::Binding<T::TxInterrupt, TxInterruptHandler<T>>
        + 'd,
        config: Config,
    ) -> Result<Self, ConfigError> {
        let location = const {
            match common_location(R::LOCATIONS, X::LOCATIONS) {
                Some(l) => l,
                None => ::core::panic!("usart: the RX pin and the TX pin share no ROUTE LOCATION"),
            }
        };
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

    /// Write all bytes, async.
    pub async fn write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        self.tx.write(buffer).await
    }

    /// Wait until all written bytes are sent.
    pub async fn flush(&mut self) -> Result<(), Error> {
        self.tx.flush().await
    }

    /// Fill `buffer`, async.
    pub async fn read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        self.rx.read(buffer).await
    }

    /// Wait for at least one byte, then read up to `buffer.len()` bytes.
    pub async fn read_buf(&mut self, buffer: &mut [u8]) -> Result<usize, Error> {
        self.rx.read_buf(buffer).await
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

    /// Split into independent TX and RX halves. Each half keeps the
    /// peripheral on until both are dropped.
    pub fn split(self) -> (UartTx<'d, M>, UartRx<'d, M>) {
        (self.tx, self.rx)
    }

    /// Borrow the TX and RX halves. The halves do not turn the peripheral
    /// off when they drop: the `Uart` does.
    pub fn split_ref(&mut self) -> (UartTx<'_, M>, UartRx<'_, M>) {
        (
            UartTx {
                info: self.tx.info,
                state: self.tx.state,
                pin: self.tx.pin.reborrow(),
                owner: false,
                _phantom: PhantomData,
            },
            UartRx {
                info: self.rx.info,
                state: self.rx.state,
                pin: self.rx.pin.reborrow(),
                owner: false,
                _phantom: PhantomData,
            },
        )
    }

    /// Change the frame format and the baud rate. Call it when the
    /// transmitter is idle.
    pub fn set_config(&mut self, config: &Config) -> Result<(), ConfigError> {
        self.tx.set_config(config)
    }

    /// Change the baud rate.
    pub fn set_baudrate(&mut self, baudrate: u32) -> Result<(), ConfigError> {
        self.tx.set_baudrate(baudrate)
    }

    /// Send a break. See [`UartTx::send_break`].
    pub fn send_break(&mut self) {
        self.tx.send_break()
    }

    /// The transmitter is busy: a written byte is not sent yet.
    pub fn busy(&self) -> bool {
        self.tx.busy()
    }

    /// Write all bytes, blocking.
    pub fn blocking_write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        self.tx.blocking_write(buffer)
    }

    /// Wait until all written bytes are sent.
    pub fn blocking_flush(&mut self) -> Result<(), Error> {
        self.tx.blocking_flush()
    }

    /// Fill `buffer`, blocking.
    pub fn blocking_read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        self.rx.blocking_read(buffer)
    }

    /// Wait for at least one byte, then read up to `buffer.len()` bytes.
    pub fn blocking_read_buf(&mut self, buffer: &mut [u8]) -> Result<usize, Error> {
        self.rx.blocking_read_buf(buffer)
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

impl embedded_io::Error for Error {
    fn kind(&self) -> embedded_io::ErrorKind {
        embedded_io::ErrorKind::Other
    }
}

impl<M: Mode> embedded_io::ErrorType for Uart<'_, M> {
    type Error = Error;
}
impl<M: Mode> embedded_io::ErrorType for UartTx<'_, M> {
    type Error = Error;
}
impl<M: Mode> embedded_io::ErrorType for UartRx<'_, M> {
    type Error = Error;
}

impl<M: Mode> embedded_io::Write for UartTx<'_, M> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        self.blocking_write(buf)?;
        Ok(buf.len())
    }
    fn flush(&mut self) -> Result<(), Error> {
        self.blocking_flush()
    }
}

impl<M: Mode> embedded_io::Write for Uart<'_, M> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        self.blocking_write(buf)?;
        Ok(buf.len())
    }
    fn flush(&mut self) -> Result<(), Error> {
        self.blocking_flush()
    }
}

impl<M: Mode> embedded_io::Read for UartRx<'_, M> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        self.blocking_read_buf(buf)
    }
}

impl<M: Mode> embedded_io::Read for Uart<'_, M> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        self.blocking_read_buf(buf)
    }
}

impl<M: Mode> embedded_io::ReadReady for UartRx<'_, M> {
    fn read_ready(&mut self) -> Result<bool, Error> {
        rx_poll(&self.info.regs)
    }
}

impl<M: Mode> embedded_io::ReadReady for Uart<'_, M> {
    fn read_ready(&mut self) -> Result<bool, Error> {
        embedded_io::ReadReady::read_ready(&mut self.rx)
    }
}

impl<M: Mode> embedded_io::WriteReady for UartTx<'_, M> {
    fn write_ready(&mut self) -> Result<bool, Error> {
        Ok(self.info.regs.status().read().txbl())
    }
}

impl<M: Mode> embedded_io::WriteReady for Uart<'_, M> {
    fn write_ready(&mut self) -> Result<bool, Error> {
        embedded_io::WriteReady::write_ready(&mut self.tx)
    }
}

impl embedded_io_async::Write for UartTx<'_, Async> {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        UartTx::write(self, buf).await?;
        Ok(buf.len())
    }
    async fn flush(&mut self) -> Result<(), Error> {
        UartTx::flush(self).await
    }
}

impl embedded_io_async::Write for Uart<'_, Async> {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        Uart::write(self, buf).await?;
        Ok(buf.len())
    }
    async fn flush(&mut self) -> Result<(), Error> {
        Uart::flush(self).await
    }
}

impl embedded_io_async::Read for UartRx<'_, Async> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        UartRx::read_buf(self, buf).await
    }
}

impl embedded_io_async::Read for Uart<'_, Async> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        Uart::read_buf(self, buf).await
    }
}

impl<M: Mode> embedded_hal_02::blocking::serial::Write<u8> for UartTx<'_, M> {
    type Error = Error;
    fn bwrite_all(&mut self, buffer: &[u8]) -> Result<(), Error> {
        self.blocking_write(buffer)
    }
    fn bflush(&mut self) -> Result<(), Error> {
        self.blocking_flush()
    }
}

impl<M: Mode> embedded_hal_02::blocking::serial::Write<u8> for Uart<'_, M> {
    type Error = Error;
    fn bwrite_all(&mut self, buffer: &[u8]) -> Result<(), Error> {
        self.blocking_write(buffer)
    }
    fn bflush(&mut self) -> Result<(), Error> {
        self.blocking_flush()
    }
}

impl<M: Mode> embassy_embedded_hal::SetConfig for Uart<'_, M> {
    type Config = Config;
    type ConfigError = ConfigError;
    fn set_config(&mut self, config: &Config) -> Result<(), ConfigError> {
        Uart::set_config(self, config)
    }
}

impl<M: Mode> embassy_embedded_hal::SetConfig for UartTx<'_, M> {
    type Config = Config;
    type ConfigError = ConfigError;
    fn set_config(&mut self, config: &Config) -> Result<(), ConfigError> {
        UartTx::set_config(self, config)
    }
}

impl<M: Mode> embassy_embedded_hal::SetConfig for UartRx<'_, M> {
    type Config = Config;
    type ConfigError = ConfigError;
    fn set_config(&mut self, config: &Config) -> Result<(), ConfigError> {
        UartRx::set_config(self, config)
    }
}
