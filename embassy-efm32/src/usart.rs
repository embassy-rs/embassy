//! Universal Synchronous/Asynchronous Receiver Transmitter (USART) driver.
//!
//! This uses the USART (and, on chips that have them, the UART) peripherals in asynchronous
//! (UART) mode.
//! [`Uart`] can be split into a [`UartTx`] and a [`UartRx`], which can also be created on their
//! own.
//!
//! ## Pin routing
//!
//! Unlike MCUs with a flexible alternate-function mux, EFM32's USART pins are connected through
//! a fixed `ROUTE.LOCATION` selector: each location wires one specific pair of GPIO pins to
//! `TX`/`RX`. The [`TxPin`] and [`RxPin`] traits are only implemented for the pins that exist at
//! some location, and the constructors pick that location. Since there is a single `LOCATION`
//! field per peripheral, the TX and RX pins of a [`Uart`] must belong to the same location,
//! otherwise the constructor returns [`ConfigError::LocationMismatch`].
//!
//! ## Hardware flow control
//!
//! The EFM32 Series 0 USART has no RTS/CTS hardware flow control, so there are no
//! `_with_rtscts` constructors.

#![macro_use]

use core::future::poll_fn;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use core::task::Poll;

use embassy_hal_internal::{Peri, PeripheralType};
use embassy_sync::waitqueue::AtomicWaker;

use crate::gpio::{AnyPin, Pin, Pull};
use crate::interrupt::InterruptExt;
use crate::interrupt::typelevel::{Binding, Interrupt as _};
use crate::mode::{Async, Blocking, Mode};
use crate::{cmu, interrupt, pac};

/// Number of data bits.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
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
}

/// Parity.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Parity {
    /// No parity.
    ParityNone,
    /// Even parity.
    ParityEven,
    /// Odd parity.
    ParityOdd,
}

/// Number of stop bits.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum StopBits {
    /// 0.5 stop bits.
    #[doc(alias = "0.5")]
    STOP0P5,
    /// 1 stop bit.
    #[doc(alias = "1")]
    STOP1,
    /// 1.5 stop bits.
    #[doc(alias = "1.5")]
    STOP1P5,
    /// 2 stop bits.
    #[doc(alias = "2")]
    STOP2,
}

/// UART configuration.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Config {
    /// Baud rate, in bits per second.
    pub baudrate: u32,
    /// Number of data bits.
    pub data_bits: DataBits,
    /// Parity.
    pub parity: Parity,
    /// Number of stop bits.
    pub stop_bits: StopBits,
}

impl Default for Config {
    /// 115200 baud, 8N1.
    fn default() -> Self {
        Self {
            baudrate: 115_200,
            data_bits: DataBits::DataBits8,
            parity: Parity::ParityNone,
            stop_bits: StopBits::STOP1,
        }
    }
}

/// UART configuration error.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum ConfigError {
    /// The baud rate is too low for the current `HFPERCLK` frequency.
    BaudrateTooLow,
    /// The baud rate is too high for the current `HFPERCLK` frequency.
    BaudrateTooHigh,
    /// The closest baud rate the current `HFPERCLK` frequency allows is more than 2.5% off the
    /// requested one.
    ///
    /// This only covers the divider's rounding. The HFRCO clocking `HFPERCLK` has an error of its
    /// own on top (see [`HfrcoBand`](crate::cmu::HfrcoBand)).
    BaudrateInaccurate,
    /// The TX and RX pins don't belong to the same `ROUTE` location.
    LocationMismatch,
}

impl core::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::BaudrateTooLow => write!(f, "baud rate too low"),
            Self::BaudrateTooHigh => write!(f, "baud rate too high"),
            Self::BaudrateInaccurate => write!(f, "baud rate can't be generated accurately enough"),
            Self::LocationMismatch => write!(f, "TX and RX pins are at different locations"),
        }
    }
}

impl core::error::Error for ConfigError {}

/// UART error.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// Received data overran a full receive buffer.
    Overrun,
    /// Framing error (missing stop bit).
    Framing,
    /// Parity error.
    Parity,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Overrun => write!(f, "receive buffer overrun"),
            Self::Framing => write!(f, "framing error"),
            Self::Parity => write!(f, "parity error"),
        }
    }
}

impl core::error::Error for Error {}

impl embedded_io::Error for Error {
    fn kind(&self) -> embedded_io::ErrorKind {
        embedded_io::ErrorKind::Other
    }
}

/// RX interrupt handler.
///
/// These are two separate types (rather than one handler implementing both interrupts) because
/// `T::RxInterrupt` and `T::TxInterrupt` are associated types: the compiler can't rule out a
/// (hypothetical) future `Instance` impl setting them to the same interrupt, which would make a
/// single generic type's two `Handler` impls conflict.
pub struct RxInterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::RxInterrupt> for RxInterruptHandler<T> {
    unsafe fn on_interrupt() {
        let r = T::info().regs();
        if r.ien.read().rxdatav().bit_is_set() {
            // Mask the (level-triggered) source; the woken future re-arms it if it still waits.
            critical_section::with(|_| r.ien.modify(|_, w| w.rxdatav().clear_bit()));
            T::state().rx_waker.wake();
        }
    }
}

/// TX interrupt handler. See [`RxInterruptHandler`] for why this isn't merged with it.
pub struct TxInterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::TxInterrupt> for TxInterruptHandler<T> {
    unsafe fn on_interrupt() {
        let r = T::info().regs();
        let ien = r.ien.read();
        if ien.txbl().bit_is_set() || ien.txc().bit_is_set() {
            // Either TXBL (space free in the buffer) or TXC (shift register empty, used by
            // `flush()`) may have fired; mask both and let whichever future is polling re-arm the
            // one it still needs.
            critical_section::with(|_| r.ien.modify(|_, w| w.txbl().clear_bit().txc().clear_bit()));
            T::state().tx_waker.wake();
        }
    }
}

/// Read-only per-instance information. Lives in flash.
pub(crate) struct Info {
    regs: *const pac::usart0::RegisterBlock,
    clock: cmu::PeripheralClock,
    rx_interrupt: interrupt::Interrupt,
    tx_interrupt: interrupt::Interrupt,
}

// SAFETY: `regs` is a fixed MMIO address; accessing the registers is what the rest of the driver
// synchronizes.
unsafe impl Sync for Info {}

impl Info {
    /// Create the info for a USART/UART instance.
    ///
    /// # Safety
    ///
    /// `regs` must point at a USART or UART register block. All of them share the `usart0` layout
    /// (checked against the SVDs: only the base address differs).
    pub(crate) const unsafe fn new(
        regs: *const pac::usart0::RegisterBlock,
        clock: cmu::PeripheralClock,
        rx_interrupt: interrupt::Interrupt,
        tx_interrupt: interrupt::Interrupt,
    ) -> Self {
        Self {
            regs,
            clock,
            rx_interrupt,
            tx_interrupt,
        }
    }

    fn regs(&self) -> &'static pac::usart0::RegisterBlock {
        unsafe { &*self.regs }
    }
}

/// Mutable per-instance state. Lives in RAM (`.bss`: all-zero at reset).
pub(crate) struct State {
    rx_waker: AtomicWaker,
    tx_waker: AtomicWaker,
    /// Number of live halves (TX and/or RX). The peripheral clock is gated when it drops to 0.
    tx_rx_refcount: AtomicU8,
    /// Whether anything was written since construction: `STATUS.TXC` is only set once a
    /// transmission completes, so it can't tell an idle, never used transmitter from a busy one.
    tx_written: AtomicBool,
}

impl State {
    pub(crate) const fn new() -> Self {
        Self {
            rx_waker: AtomicWaker::new(),
            tx_waker: AtomicWaker::new(),
            tx_rx_refcount: AtomicU8::new(0),
            tx_written: AtomicBool::new(false),
        }
    }
}

pub(crate) trait SealedInstance {
    fn info() -> &'static Info;
    fn state() -> &'static State;
}

/// USART/UART instance.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static + Send {
    /// RX interrupt for this instance.
    type RxInterrupt: interrupt::typelevel::Interrupt;
    /// TX interrupt for this instance.
    type TxInterrupt: interrupt::typelevel::Interrupt;
}

pub(crate) trait SealedTxPin<T: Instance> {
    /// The `ROUTE.LOCATION` this pin is `TX` at.
    fn location(&self) -> u8;
}

pub(crate) trait SealedRxPin<T: Instance> {
    /// The `ROUTE.LOCATION` this pin is `RX` at.
    fn location(&self) -> u8;
}

/// TX pin trait.
#[allow(private_bounds)]
pub trait TxPin<T: Instance>: SealedTxPin<T> + Pin {}

/// RX pin trait.
#[allow(private_bounds)]
pub trait RxPin<T: Instance>: SealedRxPin<T> + Pin {}

/// Implement [`Instance`] for a USART/UART peripheral singleton.
///
/// Used by each chip's `chips/*.rs` module.
macro_rules! impl_usart {
    ($name:ident, $rx_irq:ident, $tx_irq:ident) => {
        impl $crate::usart::SealedInstance for peripherals::$name {
            fn info() -> &'static $crate::usart::Info {
                // SAFETY: all USART/UART peripherals share the `usart0` register layout.
                static INFO: $crate::usart::Info = unsafe {
                    $crate::usart::Info::new(
                        $crate::pac::$name::PTR as *const _,
                        $crate::cmu::PeripheralClock::$name,
                        $crate::interrupt::Interrupt::$rx_irq,
                        $crate::interrupt::Interrupt::$tx_irq,
                    )
                };
                &INFO
            }

            fn state() -> &'static $crate::usart::State {
                static STATE: $crate::usart::State = $crate::usart::State::new();
                &STATE
            }
        }

        impl $crate::usart::Instance for peripherals::$name {
            type RxInterrupt = $crate::interrupt::typelevel::$rx_irq;
            type TxInterrupt = $crate::interrupt::typelevel::$tx_irq;
        }
    };
}

/// Implement [`TxPin`] or [`RxPin`] for a pin, at the given `ROUTE` location.
macro_rules! impl_usart_pin {
    ($inst:ident, TxPin, $pin:ident, $loc:expr) => {
        impl $crate::usart::SealedTxPin<peripherals::$inst> for peripherals::$pin {
            fn location(&self) -> u8 {
                $loc
            }
        }
        impl $crate::usart::TxPin<peripherals::$inst> for peripherals::$pin {}
    };
    ($inst:ident, RxPin, $pin:ident, $loc:expr) => {
        impl $crate::usart::SealedRxPin<peripherals::$inst> for peripherals::$pin {
            fn location(&self) -> u8 {
                $loc
            }
        }
        impl $crate::usart::RxPin<peripherals::$inst> for peripherals::$pin {}
    };
}

/// How far off (in ‰) the generated baud rate may be from the requested one.
///
/// Both ends of the line contribute to the timing error a receiver sees; at 16x oversampling it
/// fails somewhere above 4%, so each end gets a bit more than half of that budget.
const BAUDRATE_TOLERANCE_PERMILLE: u32 = 25;

/// Compute the `CLKDIV.DIV` field value for `baudrate`.
fn calc_clkdiv(hfperclk_hz: u32, baudrate: u32) -> Result<u16, ConfigError> {
    if baudrate == 0 {
        return Err(ConfigError::BaudrateTooLow);
    }
    // The CLKDIV register value is 256 * (fHFPERCLK / (16 * baudrate) - 1), with the usual 16x
    // oversampling (OVS reset default). Only bits 20:6 are implemented (the 15 bit `DIV` field),
    // so the field value, which the accessor shifts into place, is that divided by 64: 2
    // fractional bits. This is the same computation as emlib's `USART_BaudrateAsyncSet`, rounded
    // to the nearest representable step.
    let numerator = 4u64 * hfperclk_hz as u64;
    let denominator = 16u64 * baudrate as u64;
    let div = (numerator + denominator / 2) / denominator;
    let field = match div.checked_sub(4) {
        None => return Err(ConfigError::BaudrateTooHigh),
        Some(field) if field > 0x7FFF => return Err(ConfigError::BaudrateTooLow),
        Some(field) => field,
    };

    // The rate the divider actually gives: fHFPERCLK / (16 * (1 + field / 4)).
    let actual = numerator / (16 * (4 + field));
    if actual.abs_diff(baudrate as u64) * 1000 > baudrate as u64 * BAUDRATE_TOLERANCE_PERMILLE as u64 {
        return Err(ConfigError::BaudrateInaccurate);
    }
    Ok(field as u16)
}

/// Validate `config` and apply it to the frame format and baud rate registers.
///
/// Nothing is written if `config` is invalid.
fn apply_config(info: &Info, config: &Config) -> Result<(), ConfigError> {
    let div = calc_clkdiv(cmu::clocks().hfperclk.0, config.baudrate)?;
    let r = info.regs();
    r.frame.write(|w| {
        match config.data_bits {
            DataBits::DataBits5 => w.databits().five(),
            DataBits::DataBits6 => w.databits().six(),
            DataBits::DataBits7 => w.databits().seven(),
            DataBits::DataBits8 => w.databits().eight(),
        };
        match config.parity {
            Parity::ParityNone => w.parity().none(),
            Parity::ParityEven => w.parity().even(),
            Parity::ParityOdd => w.parity().odd(),
        };
        match config.stop_bits {
            StopBits::STOP0P5 => w.stopbits().half(),
            StopBits::STOP1 => w.stopbits().one(),
            StopBits::STOP1P5 => w.stopbits().oneandahalf(),
            StopBits::STOP2 => w.stopbits().two(),
        }
    });
    r.clkdiv.write(|w| unsafe { w.div().bits(div) });
    Ok(())
}

/// Bring the peripheral up from scratch: clock on, reset to a known state, configured.
///
/// `config` must already have been validated, see [`calc_clkdiv`].
fn init_peripheral(info: &Info, state: &State, config: &Config, location: u8, tx: bool, rx: bool) {
    cmu::enable(info.clock);

    let r = info.regs();
    // Don't assume reset state: a bootloader, or an earlier driver, may have used it.
    r.cmd.write(|w| {
        w.rxdis()
            .set_bit()
            .txdis()
            .set_bit()
            .masterdis()
            .set_bit()
            .clearrx()
            .set_bit()
            .cleartx()
            .set_bit()
    });
    r.ctrl.reset();
    r.ien.reset();
    r.ifc.write(|w| {
        w.txc()
            .set_bit()
            .rxfull()
            .set_bit()
            .rxof()
            .set_bit()
            .rxuf()
            .set_bit()
            .txof()
            .set_bit()
            .txuf()
            .set_bit()
            .perr()
            .set_bit()
            .ferr()
            .set_bit()
            .mpaf()
            .set_bit()
            .ssm()
            .set_bit()
            .ccf()
            .set_bit()
    });
    unwrap!(apply_config(info, config));

    r.route
        .write(|w| unsafe { w.location().bits(location).txpen().bit(tx).rxpen().bit(rx) });

    state.tx_written.store(false, Ordering::Relaxed);
    state.tx_rx_refcount.store(tx as u8 + rx as u8, Ordering::Relaxed);
}

fn enable_tx(info: &Info, pin: &AnyPin) {
    // Idle-high, to avoid a glitch on the line before the peripheral takes over.
    pin.set_high();
    pin.set_as_output();
    info.regs().cmd.write(|w| w.txen().set_bit());
}

fn enable_rx(info: &Info, pin: &AnyPin) {
    pin.set_as_input(Pull::None);
    info.regs().cmd.write(|w| w.rxen().set_bit());
}

/// Release a half's share of the peripheral: gate its clock once both halves are gone.
fn drop_tx_rx(info: &Info, state: &State) {
    if state.tx_rx_refcount.fetch_sub(1, Ordering::AcqRel) == 1 {
        info.regs().route.reset();
        cmu::disable(info.clock);
    }
}

/// Bidirectional UART driver.
///
/// Dropping it waits until the bytes already written have been sent.
#[doc(alias = "USART")]
pub struct Uart<'d, M: Mode> {
    tx: UartTx<'d, M>,
    rx: UartRx<'d, M>,
}

/// Transmitter half of a UART driver.
///
/// Dropping it waits until the bytes already written have been sent.
pub struct UartTx<'d, M: Mode> {
    info: &'static Info,
    state: &'static State,
    pin: Peri<'d, AnyPin>,
    /// Created by [`Uart::split_ref`]: the owning driver does the teardown.
    reborrowed: bool,
    _phantom: PhantomData<M>,
}

/// Receiver half of a UART driver.
pub struct UartRx<'d, M: Mode> {
    info: &'static Info,
    state: &'static State,
    pin: Peri<'d, AnyPin>,
    /// Created by [`Uart::split_ref`]: the owning driver does the teardown.
    reborrowed: bool,
    _phantom: PhantomData<M>,
}

impl<'d> Uart<'d, Async> {
    /// Create a new UART driver, with interrupt-driven async methods.
    pub fn new<T: Instance>(
        peri: Peri<'d, T>,
        tx: Peri<'d, impl TxPin<T>>,
        rx: Peri<'d, impl RxPin<T>>,
        _irq: impl Binding<T::RxInterrupt, RxInterruptHandler<T>> + Binding<T::TxInterrupt, TxInterruptHandler<T>> + 'd,
        config: Config,
    ) -> Result<Self, ConfigError> {
        let this = Self::new_inner(peri, tx, rx, config)?;
        T::RxInterrupt::unpend();
        T::TxInterrupt::unpend();
        unsafe {
            T::RxInterrupt::enable();
            T::TxInterrupt::enable();
        }
        Ok(this)
    }

    /// Write a buffer, waiting until all bytes have been queued for transmission.
    ///
    /// Use [`Self::flush`] to wait until they have actually been sent.
    pub async fn write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        self.tx.write(buffer).await
    }

    /// Wait until all written bytes have been transmitted.
    pub async fn flush(&mut self) -> Result<(), Error> {
        self.tx.flush().await
    }

    /// Read a buffer, filling it completely.
    pub async fn read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        self.rx.read(buffer).await
    }
}

impl<'d> Uart<'d, Blocking> {
    /// Create a new UART driver, with blocking methods only.
    pub fn new_blocking<T: Instance>(
        peri: Peri<'d, T>,
        tx: Peri<'d, impl TxPin<T>>,
        rx: Peri<'d, impl RxPin<T>>,
        config: Config,
    ) -> Result<Self, ConfigError> {
        Self::new_inner(peri, tx, rx, config)
    }
}

impl<'d, M: Mode> Uart<'d, M> {
    fn new_inner<T: Instance>(
        _peri: Peri<'d, T>,
        tx: Peri<'d, impl TxPin<T>>,
        rx: Peri<'d, impl RxPin<T>>,
        config: Config,
    ) -> Result<Self, ConfigError> {
        let location = tx.location();
        if rx.location() != location {
            return Err(ConfigError::LocationMismatch);
        }
        calc_clkdiv(cmu::clocks().hfperclk.0, config.baudrate)?;

        let (info, state) = (T::info(), T::state());
        init_peripheral(info, state, &config, location, true, true);

        let tx: Peri<'d, AnyPin> = tx.into();
        let rx: Peri<'d, AnyPin> = rx.into();
        enable_tx(info, &tx);
        enable_rx(info, &rx);

        Ok(Self {
            tx: UartTx::from_parts(info, state, tx),
            rx: UartRx::from_parts(info, state, rx),
        })
    }

    /// Write a buffer, blocking until all bytes have been queued for transmission.
    pub fn blocking_write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        self.tx.blocking_write(buffer)
    }

    /// Block until all written bytes have been transmitted.
    pub fn blocking_flush(&mut self) -> Result<(), Error> {
        self.tx.blocking_flush()
    }

    /// Read a buffer, blocking until it's completely filled.
    pub fn blocking_read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        self.rx.blocking_read(buffer)
    }

    /// Split into transmitter and receiver, consuming the driver.
    pub fn split(self) -> (UartTx<'d, M>, UartRx<'d, M>) {
        (self.tx, self.rx)
    }

    /// Split by reference, borrowing the driver.
    pub fn split_ref(&mut self) -> (UartTx<'_, M>, UartRx<'_, M>) {
        (
            UartTx {
                info: self.tx.info,
                state: self.tx.state,
                pin: self.tx.pin.reborrow(),
                reborrowed: true,
                _phantom: PhantomData,
            },
            UartRx {
                info: self.rx.info,
                state: self.rx.state,
                pin: self.rx.pin.reborrow(),
                reborrowed: true,
                _phantom: PhantomData,
            },
        )
    }

    /// Change the configuration. This affects both directions.
    ///
    /// Frames being transferred while this is called may be corrupted.
    pub fn set_config(&mut self, config: &Config) -> Result<(), ConfigError> {
        self.tx.set_config(config)
    }

    /// Change the baud rate. This affects both directions.
    pub fn set_baudrate(&mut self, baudrate: u32) -> Result<(), ConfigError> {
        self.tx.set_baudrate(baudrate)
    }

    /// Send a break: one frame with the line held low, including the stop bits.
    pub fn send_break(&mut self) {
        self.tx.send_break()
    }

    /// Whether the transmitter is still sending data.
    pub fn busy(&self) -> bool {
        self.tx.busy()
    }
}

impl<'d> UartTx<'d, Async> {
    /// Create a new transmit-only UART driver, with interrupt-driven async methods.
    pub fn new<T: Instance>(
        peri: Peri<'d, T>,
        tx: Peri<'d, impl TxPin<T>>,
        _irq: impl Binding<T::TxInterrupt, TxInterruptHandler<T>> + 'd,
        config: Config,
    ) -> Result<Self, ConfigError> {
        let this = Self::new_inner(peri, tx, config)?;
        T::TxInterrupt::unpend();
        unsafe { T::TxInterrupt::enable() };
        Ok(this)
    }

    /// Write a buffer, waiting until all bytes have been queued for transmission.
    ///
    /// Use [`Self::flush`] to wait until they have actually been sent.
    pub async fn write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        let r = self.info.regs();
        for &b in buffer {
            poll_fn(|cx| {
                self.state.tx_waker.register(cx.waker());
                if r.status.read().txbl().bit_is_set() {
                    Poll::Ready(())
                } else {
                    critical_section::with(|_| r.ien.modify(|_, w| w.txbl().set_bit()));
                    Poll::Pending
                }
            })
            .await;
            self.write_byte(b);
        }
        Ok(())
    }

    /// Wait until all written bytes have been transmitted.
    pub async fn flush(&mut self) -> Result<(), Error> {
        let r = self.info.regs();
        poll_fn(|cx| {
            self.state.tx_waker.register(cx.waker());
            // `IF.TXC` is sticky: drop a stale one before (maybe) arming the interrupt on it, so a
            // completion from before this call can't wake us over and over.
            r.ifc.write(|w| w.txc().set_bit());
            if !self.busy() {
                Poll::Ready(())
            } else {
                critical_section::with(|_| r.ien.modify(|_, w| w.txc().set_bit()));
                Poll::Pending
            }
        })
        .await;
        Ok(())
    }
}

impl<'d> UartTx<'d, Blocking> {
    /// Create a new transmit-only UART driver, with blocking methods only.
    pub fn new_blocking<T: Instance>(
        peri: Peri<'d, T>,
        tx: Peri<'d, impl TxPin<T>>,
        config: Config,
    ) -> Result<Self, ConfigError> {
        Self::new_inner(peri, tx, config)
    }
}

impl<'d, M: Mode> UartTx<'d, M> {
    fn new_inner<T: Instance>(
        _peri: Peri<'d, T>,
        tx: Peri<'d, impl TxPin<T>>,
        config: Config,
    ) -> Result<Self, ConfigError> {
        calc_clkdiv(cmu::clocks().hfperclk.0, config.baudrate)?;

        let (info, state) = (T::info(), T::state());
        init_peripheral(info, state, &config, tx.location(), true, false);

        let tx: Peri<'d, AnyPin> = tx.into();
        enable_tx(info, &tx);

        Ok(Self::from_parts(info, state, tx))
    }

    fn from_parts(info: &'static Info, state: &'static State, pin: Peri<'d, AnyPin>) -> Self {
        Self {
            info,
            state,
            pin,
            reborrowed: false,
            _phantom: PhantomData,
        }
    }

    fn write_byte(&mut self, b: u8) {
        // Set before writing, so `busy()` can't miss this byte.
        self.state.tx_written.store(true, Ordering::Relaxed);
        self.info.regs().txdata.write(|w| unsafe { w.txdata().bits(b) });
    }

    /// Write a buffer, blocking until all bytes have been queued for transmission.
    pub fn blocking_write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        let r = self.info.regs();
        for &b in buffer {
            while r.status.read().txbl().bit_is_clear() {}
            self.write_byte(b);
        }
        Ok(())
    }

    /// Block until all written bytes have been transmitted.
    pub fn blocking_flush(&mut self) -> Result<(), Error> {
        while self.busy() {}
        Ok(())
    }

    /// Whether the transmitter is still sending data.
    pub fn busy(&self) -> bool {
        self.state.tx_written.load(Ordering::Relaxed) && self.info.regs().status.read().txc().bit_is_clear()
    }

    /// Send a break: one frame with the line held low, including the stop bits.
    pub fn send_break(&mut self) {
        let r = self.info.regs();
        while r.status.read().txbl().bit_is_clear() {}
        self.state.tx_written.store(true, Ordering::Relaxed);
        r.txdatax.write(|w| unsafe { w.txdatax().bits(0).txbreak().set_bit() });
    }

    /// Change the configuration. On a split driver, this affects the receiver, too.
    ///
    /// Frames being transferred while this is called may be corrupted.
    pub fn set_config(&mut self, config: &Config) -> Result<(), ConfigError> {
        apply_config(self.info, config)
    }

    /// Change the baud rate. On a split driver, this affects the receiver, too.
    pub fn set_baudrate(&mut self, baudrate: u32) -> Result<(), ConfigError> {
        set_baudrate(self.info, baudrate)
    }
}

impl<'d, M: Mode> Drop for UartTx<'d, M> {
    fn drop(&mut self) {
        if self.reborrowed {
            return;
        }
        // Don't cut off what's still being sent: at most the few frames in the transmit buffer
        // and shift register.
        while self.busy() {}
        let r = self.info.regs();
        self.info.tx_interrupt.disable();
        critical_section::with(|_| {
            r.ien.modify(|_, w| w.txbl().clear_bit().txc().clear_bit());
            r.route.modify(|_, w| w.txpen().clear_bit());
        });
        r.cmd.write(|w| w.txdis().set_bit().cleartx().set_bit());
        self.pin.set_as_disconnected();
        drop_tx_rx(self.info, self.state);
    }
}

impl<'d> UartRx<'d, Async> {
    /// Create a new receive-only UART driver, with interrupt-driven async methods.
    pub fn new<T: Instance>(
        peri: Peri<'d, T>,
        rx: Peri<'d, impl RxPin<T>>,
        _irq: impl Binding<T::RxInterrupt, RxInterruptHandler<T>> + 'd,
        config: Config,
    ) -> Result<Self, ConfigError> {
        let this = Self::new_inner(peri, rx, config)?;
        T::RxInterrupt::unpend();
        unsafe { T::RxInterrupt::enable() };
        Ok(this)
    }

    /// Read a single byte, waiting for one to become available.
    async fn read_byte(&mut self) -> Result<u8, Error> {
        let r = self.info.regs();
        poll_fn(|cx| {
            self.state.rx_waker.register(cx.waker());
            match self.try_read_byte() {
                Some(res) => Poll::Ready(res),
                None => {
                    critical_section::with(|_| r.ien.modify(|_, w| w.rxdatav().set_bit()));
                    Poll::Pending
                }
            }
        })
        .await
    }

    /// Read a buffer, filling it completely.
    pub async fn read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        for slot in buffer.iter_mut() {
            *slot = self.read_byte().await?;
        }
        Ok(())
    }
}

impl<'d> UartRx<'d, Blocking> {
    /// Create a new receive-only UART driver, with blocking methods only.
    pub fn new_blocking<T: Instance>(
        peri: Peri<'d, T>,
        rx: Peri<'d, impl RxPin<T>>,
        config: Config,
    ) -> Result<Self, ConfigError> {
        Self::new_inner(peri, rx, config)
    }
}

impl<'d, M: Mode> UartRx<'d, M> {
    fn new_inner<T: Instance>(
        _peri: Peri<'d, T>,
        rx: Peri<'d, impl RxPin<T>>,
        config: Config,
    ) -> Result<Self, ConfigError> {
        calc_clkdiv(cmu::clocks().hfperclk.0, config.baudrate)?;

        let (info, state) = (T::info(), T::state());
        init_peripheral(info, state, &config, rx.location(), false, true);

        let rx: Peri<'d, AnyPin> = rx.into();
        enable_rx(info, &rx);

        Ok(Self::from_parts(info, state, rx))
    }

    fn from_parts(info: &'static Info, state: &'static State, pin: Peri<'d, AnyPin>) -> Self {
        Self {
            info,
            state,
            pin,
            reborrowed: false,
            _phantom: PhantomData,
        }
    }

    /// Whether a receive error is pending: an overrun, or a frame error on the next byte.
    fn rx_error_pending(&self) -> bool {
        let r = self.info.regs();
        if r.if_.read().rxof().bit_is_set() {
            return true;
        }
        if r.status.read().rxdatav().bit_is_clear() {
            return false;
        }
        // Peek, without popping the byte from the buffer.
        let next = r.rxdataxp.read();
        next.ferrp().bit_is_set() || next.perrp().bit_is_set()
    }

    /// Take a pending receive error, or a received byte, without waiting.
    ///
    /// An overrun (bytes were lost before the ones in the buffer) is reported before the buffered
    /// bytes, which are returned by the next calls. A byte with a framing or parity error is
    /// consumed, and reported as that error instead of being returned.
    fn try_read_byte(&mut self) -> Option<Result<u8, Error>> {
        let r = self.info.regs();
        // Clear all the error flags at once, so none is left behind. Only the overrun is reported
        // from here: the framing and parity errors are taken per frame, from `RXDATAX` below.
        let flags = r.if_.read();
        let (rxof, ferr, perr) = (
            flags.rxof().bit_is_set(),
            flags.ferr().bit_is_set(),
            flags.perr().bit_is_set(),
        );
        if rxof || ferr || perr {
            r.ifc.write(|w| w.rxof().bit(rxof).ferr().bit(ferr).perr().bit(perr));
        }
        if rxof {
            return Some(Err(Error::Overrun));
        }
        if r.status.read().rxdatav().bit_is_clear() {
            return None;
        }
        // `RXDATAX` carries the error flags of this very frame, unlike the sticky `IF` flags.
        let frame = r.rxdatax.read();
        Some(if frame.ferr().bit_is_set() {
            Err(Error::Framing)
        } else if frame.perr().bit_is_set() {
            Err(Error::Parity)
        } else {
            Ok(frame.rxdata().bits() as u8)
        })
    }

    /// Take the bytes already received, up to `buf.len()`, without waiting.
    ///
    /// Stops before a pending error, so the next read reports it rather than it being lost here.
    fn read_available(&mut self, buf: &mut [u8]) -> usize {
        let r = self.info.regs();
        let mut n = 0;
        while n < buf.len() && !self.rx_error_pending() && r.status.read().rxdatav().bit_is_set() {
            // Checked by `rx_error_pending`: this frame has no framing or parity error.
            buf[n] = r.rxdatax.read().rxdata().bits() as u8;
            n += 1;
        }
        n
    }

    /// Whether [`Self::try_read_byte`] would return something.
    fn read_ready(&self) -> bool {
        self.rx_error_pending() || self.info.regs().status.read().rxdatav().bit_is_set()
    }

    /// Read a buffer, blocking until it's completely filled.
    pub fn blocking_read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        for slot in buffer.iter_mut() {
            *slot = loop {
                if let Some(res) = self.try_read_byte() {
                    break res?;
                }
            };
        }
        Ok(())
    }

    /// Change the configuration. On a split driver, this affects the transmitter, too.
    ///
    /// Frames being transferred while this is called may be corrupted.
    pub fn set_config(&mut self, config: &Config) -> Result<(), ConfigError> {
        apply_config(self.info, config)
    }

    /// Change the baud rate. On a split driver, this affects the transmitter, too.
    pub fn set_baudrate(&mut self, baudrate: u32) -> Result<(), ConfigError> {
        set_baudrate(self.info, baudrate)
    }
}

impl<'d, M: Mode> Drop for UartRx<'d, M> {
    fn drop(&mut self) {
        if self.reborrowed {
            return;
        }
        let r = self.info.regs();
        self.info.rx_interrupt.disable();
        critical_section::with(|_| {
            r.ien.modify(|_, w| w.rxdatav().clear_bit());
            r.route.modify(|_, w| w.rxpen().clear_bit());
        });
        r.cmd.write(|w| w.rxdis().set_bit().clearrx().set_bit());
        self.pin.set_as_disconnected();
        drop_tx_rx(self.info, self.state);
    }
}

fn set_baudrate(info: &Info, baudrate: u32) -> Result<(), ConfigError> {
    let div = calc_clkdiv(cmu::clocks().hfperclk.0, baudrate)?;
    info.regs().clkdiv.write(|w| unsafe { w.div().bits(div) });
    Ok(())
}

// `SetConfig`, for use with `embassy-embedded-hal`.

impl<'d, M: Mode> embassy_embedded_hal::SetConfig for Uart<'d, M> {
    type Config = Config;
    type ConfigError = ConfigError;

    fn set_config(&mut self, config: &Self::Config) -> Result<(), Self::ConfigError> {
        self.set_config(config)
    }
}

impl<'d, M: Mode> embassy_embedded_hal::SetConfig for UartTx<'d, M> {
    type Config = Config;
    type ConfigError = ConfigError;

    fn set_config(&mut self, config: &Self::Config) -> Result<(), Self::ConfigError> {
        self.set_config(config)
    }
}

impl<'d, M: Mode> embassy_embedded_hal::SetConfig for UartRx<'d, M> {
    type Config = Config;
    type ConfigError = ConfigError;

    fn set_config(&mut self, config: &Self::Config) -> Result<(), Self::ConfigError> {
        self.set_config(config)
    }
}

// embedded-io, embedded-io-async and embedded-hal 0.2 trait implementations. These forward to
// the inherent methods.

impl<'d, M: Mode> embedded_io::ErrorType for Uart<'d, M> {
    type Error = Error;
}

impl<'d, M: Mode> embedded_io::ErrorType for UartTx<'d, M> {
    type Error = Error;
}

impl<'d, M: Mode> embedded_io::ErrorType for UartRx<'d, M> {
    type Error = Error;
}

impl<'d, M: Mode> embedded_io::Read for UartRx<'d, M> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        if buf.is_empty() {
            return Ok(0);
        }
        // Block for the first byte, then return whatever else is already available.
        self.blocking_read(&mut buf[..1])?;
        Ok(1 + self.read_available(&mut buf[1..]))
    }
}

impl<'d, M: Mode> embedded_io::Read for Uart<'d, M> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        embedded_io::Read::read(&mut self.rx, buf)
    }
}

impl<'d, M: Mode> embedded_io::ReadReady for UartRx<'d, M> {
    fn read_ready(&mut self) -> Result<bool, Error> {
        Ok(UartRx::read_ready(self))
    }
}

impl<'d, M: Mode> embedded_io::ReadReady for Uart<'d, M> {
    fn read_ready(&mut self) -> Result<bool, Error> {
        Ok(self.rx.read_ready())
    }
}

impl<'d, M: Mode> embedded_io::Write for UartTx<'d, M> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        self.blocking_write(buf)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<(), Error> {
        self.blocking_flush()
    }
}

impl<'d, M: Mode> embedded_io::Write for Uart<'d, M> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        embedded_io::Write::write(&mut self.tx, buf)
    }

    fn flush(&mut self) -> Result<(), Error> {
        self.blocking_flush()
    }
}

impl<'d, M: Mode> embedded_io::WriteReady for UartTx<'d, M> {
    fn write_ready(&mut self) -> Result<bool, Error> {
        Ok(self.info.regs().status.read().txbl().bit_is_set())
    }
}

impl<'d, M: Mode> embedded_io::WriteReady for Uart<'d, M> {
    fn write_ready(&mut self) -> Result<bool, Error> {
        embedded_io::WriteReady::write_ready(&mut self.tx)
    }
}

impl<'d> embedded_io_async::Read for UartRx<'d, Async> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        // Per the trait contract: wait for at least one byte, then return what's already
        // available without waiting for more.
        if buf.is_empty() {
            return Ok(0);
        }
        buf[0] = self.read_byte().await?;
        Ok(1 + self.read_available(&mut buf[1..]))
    }
}

impl<'d> embedded_io_async::Read for Uart<'d, Async> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        embedded_io_async::Read::read(&mut self.rx, buf).await
    }
}

impl<'d> embedded_io_async::Write for UartTx<'d, Async> {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        UartTx::write(self, buf).await?;
        Ok(buf.len())
    }

    async fn flush(&mut self) -> Result<(), Error> {
        UartTx::flush(self).await
    }
}

impl<'d> embedded_io_async::Write for Uart<'d, Async> {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        embedded_io_async::Write::write(&mut self.tx, buf).await
    }

    async fn flush(&mut self) -> Result<(), Error> {
        self.tx.flush().await
    }
}

impl<'d, M: Mode> embedded_hal_02::blocking::serial::Write<u8> for UartTx<'d, M> {
    type Error = Error;

    fn bwrite_all(&mut self, buffer: &[u8]) -> Result<(), Self::Error> {
        self.blocking_write(buffer)
    }

    fn bflush(&mut self) -> Result<(), Self::Error> {
        self.blocking_flush()
    }
}

impl<'d, M: Mode> embedded_hal_02::blocking::serial::Write<u8> for Uart<'d, M> {
    type Error = Error;

    fn bwrite_all(&mut self, buffer: &[u8]) -> Result<(), Self::Error> {
        self.blocking_write(buffer)
    }

    fn bflush(&mut self) -> Result<(), Self::Error> {
        self.blocking_flush()
    }
}

#[cfg(test)]
mod test {
    use super::{ConfigError, calc_clkdiv};

    #[test]
    fn test_calc_clkdiv() {
        // Same `CLKDIV.DIV` values as emlib's `USART_BaudrateAsyncSet` (whose register value is
        // these times 64).
        assert_eq!(calc_clkdiv(14_000_000, 115_200), Ok(26)); // +1.3%
        assert_eq!(calc_clkdiv(7_000_000, 115_200), Ok(11)); // +1.3%
        assert_eq!(calc_clkdiv(1_000_000, 9_600), Ok(22)); // +0.2%
        assert_eq!(calc_clkdiv(28_000_000, 300), Ok(23_329));

        assert_eq!(calc_clkdiv(28_000_000, 0), Err(ConfigError::BaudrateTooLow));
        // The divider would need more than its 15 bits.
        assert_eq!(calc_clkdiv(28_000_000, 100), Err(ConfigError::BaudrateTooLow));
        // Faster than fHFPERCLK / 16.
        assert_eq!(calc_clkdiv(28_000_000, 10_000_000), Err(ConfigError::BaudrateTooHigh));
        // Representable, but the closest divider gives 875000 baud: -5%.
        assert_eq!(calc_clkdiv(28_000_000, 921_600), Err(ConfigError::BaudrateInaccurate));
    }
}
