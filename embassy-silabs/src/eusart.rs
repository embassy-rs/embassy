//! Enhanced Universal Synchronous/Asynchronous Receiver/Transmitter (EUSART).
//!
//! This driver currently implements **asynchronous UART** (start/stop framing)
//! in two flavours that share the same register setup:
//!
//! - **Blocking** ([`Uart::new_blocking`]) — polls the FIFO-level status flags.
//! - **Interrupt-driven async** ([`Uart::new`]) — wakes on the per-instance
//!   `EUSARTx_RX` / `EUSARTx_TX` NVIC vectors.
//!
//! # Pin routing
//!
//! Series 2 has a digital bus matrix: *any* GPIO can carry *any* EUSART signal.
//! The constructors therefore take plain [`Pin`]s and program the
//! `GPIO.eusartX_txroute` / `eusartX_rxroute` / `eusartX_routeen` registers at runtime.
//!
//! ```rust,ignore
//! use embassy_silabs::{bind_interrupts, eusart, peripherals};
//!
//! bind_interrupts!(struct Irqs {
//!     EUSART0_RX => eusart::RxInterruptHandler<peripherals::EUSART0>;
//!     EUSART0_TX => eusart::TxInterruptHandler<peripherals::EUSART0>;
//! });
//!
//! let mut uart = eusart::Uart::new(p.EUSART0, p.PA05, p.PA04, Irqs, eusart::Config::default()).unwrap();
//! uart.write(b"hello\r\n").await.unwrap();
//! ```

use core::future::poll_fn;
use core::marker::PhantomData;
use core::task::Poll;

use embassy_hal_internal::{Peri, PeripheralType};
use embassy_sync::waitqueue::AtomicWaker;
use eusart_mod::vals;

use crate::gpio::{AnyPin, Pin};
use crate::interrupt::typelevel::Interrupt;
use crate::mode::{Async, Blocking, Mode};
use crate::pac::{CMU, GPIO, eusart_v2 as eusart_mod};
use crate::time::Hertz;
use crate::{interrupt, peripherals};

/// Asynchronous-UART oversampling. 16× gives the best noise margin and is fixed
/// for now; the divider math assumes it.
const OVERSAMPLE: u32 = 16;

/// Number of data bits per frame.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum DataBits {
    /// 7 data bits.
    DataBits7,
    /// 8 data bits.
    DataBits8,
    /// 9 data bits.
    DataBits9,
}

impl DataBits {
    fn to_vals(self) -> vals::Databits {
        match self {
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
    /// Invert the TX line (idle-low instead of idle-high).
    pub invert_tx: bool,
    /// Invert the RX line.
    pub invert_rx: bool,
    /// Internal loopback: tie TX to RX inside the peripheral. Useful for
    /// self-test without external wiring.
    pub loopback: bool,
}

impl Default for Config {
    /// 115200 8N1, no inversion.
    fn default() -> Self {
        Self {
            baudrate: 115_200,
            data_bits: DataBits::DataBits8,
            parity: Parity::ParityNone,
            stop_bits: StopBits::STOP1,
            invert_tx: false,
            invert_rx: false,
            loopback: false,
        }
    }
}

/// Runtime UART error.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// Framing error - a stop bit was not seen when expected.
    Framing,
    /// Parity check failed.
    Parity,
    /// RX FIFO overflowed; data was lost.
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

/// Error returned by the constructors when [`Config`] cannot be realised.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum ConfigError {
    /// The requested baud rate is too low for the EUSART input clock (the
    /// fractional divider would overflow its 20-bit field).
    BaudrateTooLow,
    /// The requested baud rate is too high for the EUSART input clock.
    BaudrateTooHigh,
}

/// Compute the `CLKDIV.DIV` field value for a target baud rate.
///
/// Silabs EUSART asynchronous baud rate (reference-manual form):
/// `baud = f_clk / (ovs * (1 + DIV_hw / 256))`, where `DIV_hw` is the raw
/// register divider. The PAC's `CLKDIV.DIV` accessor sits at a 3-bit register
/// offset, so `DIV_hw = DIV_field * 8` and the field-level relation becomes
/// `baud = f_clk / (ovs * (1 + DIV_field / 32))`. Inverting:
///
/// `DIV_field = 32 * (f_clk / (ovs * baud) - 1)`
fn compute_clkdiv(src: Hertz, baudrate: u32, ovs: u32) -> Result<u32, ConfigError> {
    let f = src.0 as u64;
    let denom = (ovs as u64) * (baudrate as u64);
    // f/(ovs*baud) must exceed 1, otherwise the baud is too fast for the clock.
    if f <= denom {
        return Err(ConfigError::BaudrateTooHigh);
    }
    let div = (32u64 * (f - denom)) / denom;
    if div > 0xF_FFFF {
        return Err(ConfigError::BaudrateTooLow);
    }
    Ok(div as u32)
}

/// Per-instance shared state: the wakers signalled by the interrupt handler.
struct State {
    rx_waker: AtomicWaker,
    tx_waker: AtomicWaker,
}

impl State {
    const fn new() -> Self {
        Self {
            rx_waker: AtomicWaker::new(),
            tx_waker: AtomicWaker::new(),
        }
    }
}

trait SealedInstance {
    fn regs() -> eusart_mod::Eusart;
    fn state() -> &'static State;
    fn enable_bus_clock();
    fn source_freq() -> Hertz;
    /// Route TX/RX to the given pins (`port` 0..=3, `pin` 0..=15) and enable them.
    fn route(tx_port: u8, tx_pin: u8, rx_port: u8, rx_pin: u8);
}

/// An EUSART peripheral instance.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static {
    /// Interrupt for this instance's RX events.
    type RxInterrupt: interrupt::typelevel::Interrupt;
    /// Interrupt for this instance's TX events.
    type TxInterrupt: interrupt::typelevel::Interrupt;
}

/// Resolve EUSART0's input-clock frequency from its CMU mux selection.
fn eusart0_source_freq() -> Hertz {
    use crate::rcc::Eusart0ClkSource as Sel;
    // SAFETY: read-only; init_clocks has populated the frozen table by the time
    // any driver is constructed.
    let f = unsafe { crate::rcc::get_freqs() };
    match CMU.eusart0clkctrl().read().clksel() {
        Sel::Em01grpcclk => f.em01grpcclk,
        Sel::Hfrcoem23 => f.hfrcoem23,
        Sel::Lfrco => f.lfrco,
        Sel::Lfxo => f.lfxo.expect("EUSART0 clocked from LFXO but LFXO is not configured"),
        Sel::Disabled => panic!("EUSART0 clock source is Disabled"),
        _ => unreachable!(),
    }
}

/// EUSART1..3 are clocked directly from EM01GRPCCLK (no per-instance mux).
fn em01grpcclk_freq() -> Hertz {
    // SAFETY: read-only; the frozen clock table is populated by `init_clocks`.
    unsafe { crate::rcc::get_freqs() }.em01grpcclk
}

macro_rules! impl_eusart {
    ($inst:ident, $rxirq:ident, $txirq:ident, $txroute:ident, $rxroute:ident, $routeen:ident, $enable:block, $freq:expr $(,)?) => {
        impl SealedInstance for peripherals::$inst {
            fn regs() -> eusart_mod::Eusart {
                unsafe { eusart_mod::Eusart::from_ptr(crate::pac::$inst.as_ptr()) }
            }
            fn state() -> &'static State {
                static STATE: State = State::new();
                &STATE
            }
            fn enable_bus_clock() $enable
            fn source_freq() -> Hertz {
                $freq
            }
            fn route(tx_port: u8, tx_pin: u8, rx_port: u8, rx_pin: u8) {
                GPIO.$txroute().write(|w| {
                    w.set_port(tx_port);
                    w.set_pin(tx_pin);
                });
                GPIO.$rxroute().write(|w| {
                    w.set_port(rx_port);
                    w.set_pin(rx_pin);
                });
                GPIO.$routeen().modify(|w| {
                    w.set_txpen(true);
                    w.set_rxpen(true);
                });
            }
        }
        impl Instance for peripherals::$inst {
            type RxInterrupt = crate::interrupt::typelevel::$rxirq;
            type TxInterrupt = crate::interrupt::typelevel::$txirq;
        }
    };
}

// All four EUSART instances. EUSART0 is the LF-capable instance with its own
// clock mux (resolved by `eusart0_source_freq`); EUSART1..3 are HF-only on
// EM01GRPCCLK. Bus-clock gates live in CLKEN1 (EUSART0/1) and CLKEN2 (EUSART2/3).
impl_eusart!(
    EUSART0,
    EUSART0_RX,
    EUSART0_TX,
    eusart0_txroute,
    eusart0_rxroute,
    eusart0_routeen,
    {
        CMU.clken1().modify(|w| w.set_eusart0(true));
    },
    eusart0_source_freq(),
);
impl_eusart!(
    EUSART1,
    EUSART1_RX,
    EUSART1_TX,
    eusart1_txroute,
    eusart1_rxroute,
    eusart1_routeen,
    {
        CMU.clken1().modify(|w| w.set_eusart1(true));
    },
    em01grpcclk_freq(),
);
impl_eusart!(
    EUSART2,
    EUSART2_RX,
    EUSART2_TX,
    eusart2_txroute,
    eusart2_rxroute,
    eusart2_routeen,
    {
        CMU.clken2().modify(|w| w.set_eusart2(true));
    },
    em01grpcclk_freq(),
);
impl_eusart!(
    EUSART3,
    EUSART3_RX,
    EUSART3_TX,
    eusart3_txroute,
    eusart3_rxroute,
    eusart3_routeen,
    {
        CMU.clken2().modify(|w| w.set_eusart3(true));
    },
    em01grpcclk_freq(),
);

/// Interrupt handler for an EUSART instance's RX vector (`EUSARTx_RX`).
///
/// Both this and [`TxInterruptHandler`] run the same servicing routine, so a
/// wake on either vector progresses both directions.
pub struct RxInterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::RxInterrupt> for RxInterruptHandler<T> {
    unsafe fn on_interrupt() {
        on_interrupt::<T>();
    }
}

/// Interrupt handler for an EUSART instance's TX vector (`EUSARTx_TX`).
pub struct TxInterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::TxInterrupt> for TxInterruptHandler<T> {
    unsafe fn on_interrupt() {
        on_interrupt::<T>();
    }
}

fn on_interrupt<T: Instance>() {
    let r = T::regs();
    let s = T::state();
    let if_ = r.if_().read();
    let ien = r.ien().read();

    // RX side: data available or an error that the reader armed.
    let rx = (if_.rxfl() && ien.rxfl())
        || (if_.ferr() && ien.ferr())
        || (if_.perr() && ien.perr())
        || (if_.rxof() && ien.rxof());
    if rx {
        // Disable so the ISR doesn't re-enter before the future re-arms.
        r.ien_clr().write(|w| {
            w.set_rxfl(true);
            w.set_ferr(true);
            w.set_perr(true);
            w.set_rxof(true);
        });
        s.rx_waker.wake();
    }

    // TX side: FIFO has space, or transmission completed.
    let tx = (if_.txfl() && ien.txfl()) || (if_.txc() && ien.txc());
    if tx {
        r.ien_clr().write(|w| {
            w.set_txfl(true);
            w.set_txc(true);
        });
        s.tx_waker.wake();
    }
}

/// Configure clocks, routing, pins and registers for instance `T`.
fn init<T: Instance>(tx: &AnyPin, rx: &AnyPin, config: &Config) -> Result<(), ConfigError> {
    T::enable_bus_clock();

    let src = T::source_freq();
    let div = compute_clkdiv(src, config.baudrate, OVERSAMPLE)?;
    debug!(
        "eusart: src_hz={} baud={} ovs={} clkdiv={}",
        src.0, config.baudrate, OVERSAMPLE, div
    );

    // GPIO: TX is a push-pull output (idle high), RX is an input.
    crate::gpio::set_as_alternate_output(tx);
    crate::gpio::set_as_alternate_input(rx);
    T::route(
        tx.pin_port() >> 4,
        tx.pin_port() & 0xF,
        rx.pin_port() >> 4,
        rx.pin_port() & 0xF,
    );

    let r = T::regs();

    // EUSART config registers are writable only while EN.EN = 0.
    r.en().write(|w| w.set_en(false));
    while r.en().read().disabling() {}

    r.cfg0().write(|w| {
        w.set_sync(vals::Sync::Async);
        w.set_ovs(vals::Ovs::X16);
        w.set_msbf(false);
        w.set_rxinv(config.invert_rx);
        w.set_txinv(config.invert_tx);
        w.set_loopbk(config.loopback);
    });
    r.cfg1().write(|w| {
        // RXFL when ≥1 received frame; TXFL when space for ≥1 frame.
        w.set_rxfiw(vals::Rxfiw::Oneframe);
        w.set_txfiw(vals::Txfiw::Oneframe);
    });
    r.framecfg().write(|w| {
        w.set_databits(config.data_bits.to_vals());
        w.set_parity(config.parity.to_vals());
        w.set_stopbits(config.stop_bits.to_vals());
    });
    r.clkdiv().write(|w| w.set_div(div));

    r.en().write(|w| w.set_en(true));

    // Enable RX and TX, then wait for them to report enabled.
    r.cmd().write(|w| {
        w.set_rxen(true);
        w.set_txen(true);
    });
    while {
        let st = r.status().read();
        !(st.rxens() && st.txens())
    } {}

    Ok(())
}

/// UART transmitter.
pub struct UartTx<'d, M: Mode> {
    info: eusart_mod::Eusart,
    state: &'static State,
    _pin: Peri<'d, AnyPin>,
    _phantom: PhantomData<M>,
}

impl<'d, M: Mode> UartTx<'d, M> {
    /// Write all bytes, blocking until each enters the TX FIFO.
    pub fn blocking_write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        let r = self.info;
        for &b in buffer {
            while !r.status().read().txfl() {}
            r.txdata().write(|w| w.set_txdata(b as u16));
        }
        Ok(())
    }

    /// Block until all queued data has been shifted out.
    pub fn blocking_flush(&mut self) -> Result<(), Error> {
        while !self.info.status().read().txc() {}
        Ok(())
    }
}

impl<'d> UartTx<'d, Async> {
    /// Write all bytes, yielding to the executor while the TX FIFO is full.
    pub async fn write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        let r = self.info;
        for &b in buffer {
            poll_fn(|cx| {
                if r.status().read().txfl() {
                    return Poll::Ready(());
                }
                self.state.tx_waker.register(cx.waker());
                r.ien_set().write(|w| w.set_txfl(true));
                if r.status().read().txfl() {
                    Poll::Ready(())
                } else {
                    Poll::Pending
                }
            })
            .await;
            r.txdata().write(|w| w.set_txdata(b as u16));
        }
        Ok(())
    }

    /// Wait until all queued data has been shifted out.
    pub async fn flush(&mut self) -> Result<(), Error> {
        let r = self.info;
        poll_fn(|cx| {
            if r.status().read().txc() {
                return Poll::Ready(());
            }
            self.state.tx_waker.register(cx.waker());
            r.ien_set().write(|w| w.set_txc(true));
            if r.status().read().txc() {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;
        Ok(())
    }
}

/// Check/clear sticky RX error flags; returns whether ≥1 byte is available.
fn rx_poll(r: &eusart_mod::Eusart) -> Result<bool, Error> {
    let if_ = r.if_().read();
    if if_.ferr() {
        r.if_clr().write(|w| w.set_ferr(true));
        return Err(Error::Framing);
    }
    if if_.perr() {
        r.if_clr().write(|w| w.set_perr(true));
        return Err(Error::Parity);
    }
    if if_.rxof() {
        r.if_clr().write(|w| w.set_rxof(true));
        return Err(Error::Overrun);
    }
    Ok(r.status().read().rxfl())
}

/// UART receiver.
pub struct UartRx<'d, M: Mode> {
    info: eusart_mod::Eusart,
    state: &'static State,
    _pin: Peri<'d, AnyPin>,
    _phantom: PhantomData<M>,
}

impl<'d, M: Mode> UartRx<'d, M> {
    /// Fill `buffer` completely, blocking until enough bytes arrive.
    pub fn blocking_read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        let r = self.info;
        for slot in buffer {
            while !rx_poll(&r)? {}
            *slot = r.rxdata().read().rxdata() as u8;
        }
        Ok(())
    }

    /// Read at least one byte (blocking), up to `buffer.len()`. Returns the
    /// count read.
    pub fn blocking_read_buf(&mut self, buffer: &mut [u8]) -> Result<usize, Error> {
        if buffer.is_empty() {
            return Ok(0);
        }
        let r = self.info;
        while !rx_poll(&r)? {}
        let mut n = 0;
        while n < buffer.len() && r.status().read().rxfl() {
            buffer[n] = r.rxdata().read().rxdata() as u8;
            n += 1;
        }
        Ok(n)
    }
}

impl<'d> UartRx<'d, Async> {
    /// Fill `buffer` completely, yielding while waiting for data.
    pub async fn read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        let r = self.info;
        for slot in buffer {
            self.wait_rx_ready().await?;
            *slot = r.rxdata().read().rxdata() as u8;
        }
        Ok(())
    }

    /// Read at least one byte, up to `buffer.len()`. Returns the count read.
    pub async fn read_buf(&mut self, buffer: &mut [u8]) -> Result<usize, Error> {
        if buffer.is_empty() {
            return Ok(0);
        }
        self.wait_rx_ready().await?;
        let r = self.info;
        let mut n = 0;
        while n < buffer.len() && r.status().read().rxfl() {
            buffer[n] = r.rxdata().read().rxdata() as u8;
            n += 1;
        }
        Ok(n)
    }

    /// Yield until ≥1 byte is available, surfacing RX errors.
    async fn wait_rx_ready(&mut self) -> Result<(), Error> {
        let r = self.info;
        poll_fn(|cx| {
            match rx_poll(&r) {
                Ok(true) => return Poll::Ready(Ok(())),
                Ok(false) => {}
                Err(e) => return Poll::Ready(Err(e)),
            }
            self.state.rx_waker.register(cx.waker());
            r.ien_set().write(|w| {
                w.set_rxfl(true);
                w.set_ferr(true);
                w.set_perr(true);
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

/// Bidirectional UART.
pub struct Uart<'d, M: Mode> {
    tx: UartTx<'d, M>,
    rx: UartRx<'d, M>,
}

impl<'d> Uart<'d, Blocking> {
    /// Create a blocking UART on `tx`/`rx` for instance `peri`.
    pub fn new_blocking<T: Instance>(
        _peri: Peri<'d, T>,
        rx: Peri<'d, impl Pin>,
        tx: Peri<'d, impl Pin>,
        config: Config,
    ) -> Result<Self, ConfigError> {
        let tx: Peri<'d, AnyPin> = tx.into();
        let rx: Peri<'d, AnyPin> = rx.into();
        init::<T>(&tx, &rx, &config)?;
        Ok(Self::wrap::<T>(tx, rx))
    }
}

impl<'d> Uart<'d, Async> {
    /// Create an interrupt-driven async UART. Bind both `EUSARTx_RX` and
    /// `EUSARTx_TX` to [`InterruptHandler<T>`].
    pub fn new<T: Instance>(
        _peri: Peri<'d, T>,
        rx: Peri<'d, impl Pin>,
        tx: Peri<'d, impl Pin>,
        _irqs: impl interrupt::typelevel::Binding<T::RxInterrupt, RxInterruptHandler<T>>
        + interrupt::typelevel::Binding<T::TxInterrupt, TxInterruptHandler<T>>
        + 'd,
        config: Config,
    ) -> Result<Self, ConfigError> {
        let tx: Peri<'d, AnyPin> = tx.into();
        let rx: Peri<'d, AnyPin> = rx.into();
        init::<T>(&tx, &rx, &config)?;

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
        Self {
            tx: UartTx {
                info: T::regs(),
                state: T::state(),
                _pin: tx,
                _phantom: PhantomData,
            },
            rx: UartRx {
                info: T::regs(),
                state: T::state(),
                _pin: rx,
                _phantom: PhantomData,
            },
        }
    }

    /// Split into independent TX and RX halves.
    pub fn split(self) -> (UartTx<'d, M>, UartRx<'d, M>) {
        (self.tx, self.rx)
    }

    /// Write all bytes, blocking.
    pub fn blocking_write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        self.tx.blocking_write(buffer)
    }

    /// Block until all queued data has been shifted out.
    pub fn blocking_flush(&mut self) -> Result<(), Error> {
        self.tx.blocking_flush()
    }

    /// Fill `buffer` completely, blocking.
    pub fn blocking_read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        self.rx.blocking_read(buffer)
    }

    /// Read at least one byte (blocking), up to `buffer.len()`.
    pub fn blocking_read_buf(&mut self, buffer: &mut [u8]) -> Result<usize, Error> {
        self.rx.blocking_read_buf(buffer)
    }
}

impl<'d> Uart<'d, Async> {
    /// Write all bytes, async.
    pub async fn write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        self.tx.write(buffer).await
    }

    /// Wait until all queued data has been shifted out.
    pub async fn flush(&mut self) -> Result<(), Error> {
        self.tx.flush().await
    }

    /// Fill `buffer` completely, async.
    pub async fn read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        self.rx.read(buffer).await
    }

    /// Read at least one byte, up to `buffer.len()`.
    pub async fn read_buf(&mut self, buffer: &mut [u8]) -> Result<usize, Error> {
        self.rx.read_buf(buffer).await
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
