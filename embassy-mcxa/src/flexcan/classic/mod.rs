//! Module for Classic CAN.
//!
//! This module allows you to initialize and configure a classic-mode `FlexCan` instance.
//!
//! This module provides two modes for FlexCAN: `Async` and `Blocking`. The `Async` mode is interrupt-based and provides an `async` API
//! for sending/receiving frames. The `Blocking` mode does not use any interrupts at all, and uses blocking polls on the hardware in place
//! of async awaiting.
//!
//! For most cases, you should probably just use the `Async` mode, unless you specifically need Blocking functionality
//! and are okay with accepting the risks.
//!
#![doc = asynchronous::docs::doc_async_example!()]
#![doc = blocking::docs::doc_blocking_example!()]

pub(crate) mod asynchronous;
pub(crate) mod blocking;
pub mod frame;
mod mailbox;
mod timing;

use core::cell::Cell;
use core::sync::atomic::{AtomicU32, Ordering};

use asynchronous::AsyncState;
pub use asynchronous::{Async, InterruptHandler, RxQueue};
pub use blocking::Blocking;
use embassy_hal_internal::Peri;
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use maitake_sync::WaitCell;
use nxp_pac::can as pac;

use crate::clocks::periph_helpers::{CanClockSel, CanConfig, CanInstance, Div4};
use crate::clocks::{ClockError, PoweredClock, WakeGuard, enable_and_reset};
use crate::flexcan::control::Control;
use crate::flexcan::filter::FilterConfig;
use crate::flexcan::{RxPin, TxPin};
use crate::gpio::AnyPin;

mod sealed {
    pub trait Sealed {}
}

/// Driver mode.
#[allow(private_bounds)]
pub trait Mode: sealed::Sealed {}

/// Errors that can return when initializing
/// a `FlexCan` instance.
#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum InitError {
    /// This error indicates that the hardware didn't response
    /// within a reasonable timeframe to a request the HAL made.
    Timeout,

    /// This error indicates that you attempted enabling the
    /// `protocol_exception` feature in `FlexCanConfig` on a
    /// peripheral where it is not supported.
    ///
    /// Note: Protocol Exception is only supported on CAN0.
    ProtocolExceptionUnsupported,

    /// You have attempted to configure an invalid bitrate.
    /// See the `TimingError` struct docs for more info.
    TimingError(timing::TimingError),

    /// Setting up the FlexCAN peripheral clock failed. This usually means the
    /// requested clock source was not enabled/configured by `embassy_mcxa::init()`,
    /// or the system clocks were never initialized. See the `ClockError` docs.
    ClockSetup(ClockError),
}

/// Configuration settings for a Classic-mode FlexCAN driver instance.
pub struct FlexCanConfig<'a> {
    /// This setting allows you to enable/disable the
    /// FlexCAN's Protocol Exception feature. This may be
    /// useful if you want this FlexCAN to coexist on a FD-enabled
    /// bus while staying in Classic mode.
    ///
    /// Note: This feature is only available on CAN0. If you attempt to enable this
    /// feature on an instance where it isn't supported, you will get an error
    /// when calling `FlexCan::new()`.
    pub protocol_exception: bool,

    /// This setting allows you to configure your peripheral's RX filters.
    /// See the `FilterConfig` struct docs for more information.
    pub filters: FilterConfig<'a>,

    /// CAN bit rate, in bits per second (e.g. `500_000` for 500 kbit/s).
    ///
    /// Your requested bitrate must conform to these constraints:
    /// - `bitrate <= 1_000_000` (i.e., your bitrate can't be greater than 1Mbps).
    /// - Your source clock rate must be an exact integer multiple of the bitrate.
    /// - At least 8 source clocks per bit are required.
    /// - Validity depends on clock divisibility, so prefer standard rates with a matching
    ///   clock (or else you might get a `InitError::TimingError`).
    ///
    /// Here are some common/standard rates people use (after applying `clock_div`):
    /// #### 45 MHz Source Clock:
    /// - `bitrate: 1_000_000` (1 Mbps)
    /// - `bitrate: 500_000` (500 kbps)
    /// - `bitrate: 250_000` (250 kbps)
    /// - `bitrate: 125_000` (125 kbps)
    ///  
    /// #### 48 MHz Source Clock:
    /// - `bitrate: 1_000_000` (1 Mbps)
    /// - `bitrate: 500_000` (500 kbps)
    /// - `bitrate: 250_000` (250 kbps)
    /// - `bitrate: 125_000` (125 kbps)
    pub bitrate: u32,

    /// Clock source feeding the FlexCAN protocol engine.
    ///
    /// The selected source, after applying `clock_div`, must
    /// be an integer multiple of `bitrate`, or `FlexCan::new()`
    /// returns a `InitError::TimingError`. See the docs for `CanClockSel`.
    pub clock_source: CanClockSel,

    /// Divider applied to `clock_source`.
    /// Use `Div4::no_div()` for no division.
    pub clock_div: Div4,

    /// Deep-sleep behavior for the FlexCAN clock. Use
    /// `PoweredClock::NormalEnabledDeepSleepDisabled` unless you want the FlexCAN to keep running through deep sleep.
    pub power: PoweredClock,
}

impl<'a> Default for FlexCanConfig<'a> {
    /// Returns a default `FlexCanConfig` instance
    /// with this configuration:
    /// ```rust
    /// FlexCanConfig {
    ///     protocol_exception: false,
    ///     filters: FilterConfig::default(),
    ///     bitrate: 500_000,
    ///     clock_source: CanClockSel::FroHf,
    ///     clock_div: Div4::no_div(),
    ///     power: PoweredClock::NormalEnabledDeepSleepDisabled,
    /// }
    /// ```
    fn default() -> Self {
        FlexCanConfig {
            protocol_exception: false,
            filters: FilterConfig::default(),
            bitrate: 500_000,
            clock_source: CanClockSel::FroHf,
            clock_div: Div4::no_div(),
            power: PoweredClock::NormalEnabledDeepSleepDisabled,
        }
    }
}

/// Bus error modes.
#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum BusErrorMode {
    /// Error active mode (default). Controller will transmit an
    /// active error frame upon protocol error.
    ErrorActive,

    /// Error passive mode. An error coutner exceeded 127. Controller will
    /// transmit a passive error frame upon protocol error.
    ErrorPassive,

    /// Bus off mode. The transmit error counter exceeded 255. Controller is
    /// not participating in bus traffic.
    BusOff,
}

/// Errors that may occur when sending a CAN message.
#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SendError {
    /// The TX mailbox is currently full.
    TxMailboxFull,

    /// The bus is currently in BusOff mode and cannot immediately dispatch a message.
    BusOff,
}

/// Errors that may occur when calling try_receive().
#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ReceiveError {
    /// There were no new messages to be received.
    NoMessages,
}

/// Info and state for a single `classic::FlexCan` instance.
///
/// Note: This stuff is common to both blocking and async mode. The async-specific stuff's in `AsyncState`.
pub(crate) struct Info {
    /// Mode-agnostic hardware access.
    /// Lets you call `.regs()` to access individual hardware registers, plus contains
    /// some extra helper functions for random stuff.
    pub control: Control,

    /// Each bit indicates the if that message buffer (one of the 32) is available for TX use.
    /// If `1`, the message buffer can be used to transmit a new TX message.
    /// If `0`, the message buffer is currently claimed and in-use.
    pub tx_available: AtomicU32,

    /// Each bit indicates whether that message buffer was last used to transmit a REMOTE
    /// (RTR = 1) frame. This is needed because after a REMOTE frame is transmitted, the hardware automatically
    /// flips the message buffer to RX-EMPTY instead of TX-INACTIVE (see page 1548 of the datasheet). Because of that,
    /// we need to manually write TX-INACTIVE back to any message buffer where an REMOTE message was sent. These bits
    /// let us track what buffers we need to do this for.
    ///
    /// TLDR: These bits are set by `dispatch()` for REMOTE frames, and cleared by `reclaim_completed()` once the buffer has been neutralized back to TX-INACTIVE.
    pub tx_remote: AtomicU32,

    /// This flag indicates whether or not Protocol Exception is supported
    /// by the FlexCAN peripheral. Protocol Exception allows a FlexCAN in
    /// Classic mode (MCR[FDEN] = 0) to coexist on an FD-enabled bus without
    /// throwing a bunch of error frames. In other words, it allows the FlexCAN
    /// to recognize that a frame is FD and ignore it, even when not in FD mode.
    /// See page 1492 of the datasheet.
    ///
    /// This feature is supported by CAN0, but not CAN1. This flag allows the HAL
    /// to specify this constraint internally via `impl_can_instance!()`. This way,
    /// if a user tries to enable this feature in their config on an unsupported peripheral,
    /// they'll get an error at init-time.
    pub prexcen_supported: bool,

    /// Stores a count of the number of times the TX mailbox has filled up so far.
    pub tx_mailbox_full_count: AtomicU32,
}

pub(crate) trait SealedInstance: crate::clocks::Gate<MrccPeriphConfig = CanConfig> {
    fn info() -> &'static Info;

    /// Async-only per-instance state.
    fn async_state() -> &'static AsyncState;

    /// This is used to select the correct `MRCC_FLEXCANn_CLKSEL`/`CLKDIV` registers during clock setup.
    const CLOCK_INSTANCE: CanInstance;
}

/// Represents a hardware CAN instance (e.g., CAN0, CAN1).
#[allow(private_bounds)]
pub trait Instance: crate::flexcan::Instance + SealedInstance {}

/// RX-specific FlexCAN instance. Can be obtained by calling `.split()`
/// on your main `FlexCan` instance.
pub struct FlexCanRx<'d, M: Mode> {
    info: &'static Info,
    _rx: Peri<'d, AnyPin>,
    _wake_guard: Option<WakeGuard>,
    mode: M,
}

/// General `FlexCanRx` functions (available for both `Async` and `Blocking` mode).
impl<'d, M: Mode> FlexCanRx<'d, M> {
    /// Creates a new `FlexCanRx` instance.
    /// This isn't a public function, and should only be called via the mode-specific constructors.
    fn new(info: &'static Info, rx: Peri<'d, AnyPin>, wake_guard: Option<WakeGuard>, mode: M) -> Self {
        Self {
            info,
            _rx: rx,
            _wake_guard: wake_guard,
            mode,
        }
    }

    #[doc = docs::doc_error_mode!()]
    pub fn error_mode(&self) -> BusErrorMode {
        functions::error_mode(self.info)
    }
}

/// TX-specific FlexCAN instance. Can be obtained by calling `.split()`
/// on your main `FlexCan` instance.
pub struct FlexCanTx<'d, M: Mode> {
    info: &'static Info,
    _tx: Peri<'d, AnyPin>,
    _wake_guard: Option<WakeGuard>,
    mode: M,
}

/// General `FlexCanTx` functions (available for both `Async` and `Blocking` mode).
impl<'d, M: Mode> FlexCanTx<'d, M> {
    /// Creates a new `FlexCanTx` instance.
    /// This isn't a public function, and should only be called via the mode-specific constructors.
    fn new(info: &'static Info, tx: Peri<'d, AnyPin>, wake_guard: Option<WakeGuard>, mode: M) -> Self {
        Self {
            info,
            _tx: tx,
            _wake_guard: wake_guard,
            mode,
        }
    }

    #[doc = docs::doc_error_mode!()]
    pub fn error_mode(&self) -> BusErrorMode {
        functions::error_mode(self.info)
    }
    #[doc = docs::doc_tx_mailbox_full_count!()]
    pub fn tx_mailbox_full_count(&self) -> u32 {
        self.info.tx_mailbox_full_count.load(Ordering::Acquire)
    }
}

/// FlexCAN driver instance, in Classic CAN mode.
pub struct FlexCan<'d, M: Mode> {
    tx: FlexCanTx<'d, M>,
    rx: FlexCanRx<'d, M>,
}

/// General `FlexCan` functions (available for both `Async` and `Blocking` mode).
impl<'d, M: Mode> FlexCan<'d, M> {
    /// Consumes this `FlexCan` and splits it into independent `FlexCanTx` and `FlexCanRx` halves. This is useful
    /// if you want to have separate dedicated tasks for CAN RX and CAN TX.
    pub fn split(self) -> (FlexCanTx<'d, M>, FlexCanRx<'d, M>) {
        (self.tx, self.rx)
    }

    /// Recombines a `FlexCanTx` and `FlexCanRx` (previously obtained from `split()`) back into a `FlexCan`.
    ///
    /// If the two halves come from different peripherals, this returns `Err((tx, rx))`,
    /// where `tx` and `rx` are the same `tx`/`rx` you passed in. This means you can try again if you need to.
    pub fn join(tx: FlexCanTx<'d, M>, rx: FlexCanRx<'d, M>) -> Result<Self, (FlexCanRx<'d, M>, FlexCanTx<'d, M>)> {
        if !core::ptr::eq(tx.info, rx.info) {
            return Err((rx, tx));
        }
        Ok(Self { tx, rx })
    }

    #[doc = docs::doc_error_mode!()]
    pub fn error_mode(&self) -> BusErrorMode {
        self.tx.error_mode()
    }
    #[doc = docs::doc_tx_mailbox_full_count!()]
    pub fn tx_mailbox_full_count(&self) -> u32 {
        self.tx.tx_mailbox_full_count()
    }
}

/// Big tuple for the pieces produced by `init()` and consumed by the mode-specific constructors.
type InitParts<'d> = (&'static Info, Peri<'d, AnyPin>, Peri<'d, AnyPin>, Option<WakeGuard>);

/// Shared initialization for both the async and blocking Classic drivers.
fn init<'d, T: Instance>(
    _peri: Peri<'d, T>,
    rx: Peri<'d, impl RxPin<T>>,
    tx: Peri<'d, impl TxPin<T>>,
    config: &FlexCanConfig<'_>,
) -> Result<InitParts<'d>, InitError> {
    use embassy_time::Duration;

    let info = T::info();

    // Software-only error checks to make sure stuff was configured correctly
    if config.protocol_exception && !info.prexcen_supported {
        return Err(InitError::ProtocolExceptionUnsupported);
    }

    // Mux the pins to their CAN function and take ownership for the driver's lifetime.
    rx.as_rx();
    tx.as_tx();
    let _rx = rx.into();
    let _tx = tx.into();

    // Set up the FlexCAN clock (before messing w/ any registers)
    let clock_cfg = CanConfig {
        power: config.power,
        source: config.clock_source,
        div: config.clock_div,
        instance: T::CLOCK_INSTANCE,
    };

    // SAFETY: the `Peri<'d, T>` token proves we exclusively own this peripheral
    // and it is not yet in use.
    let parts = unsafe { enable_and_reset::<T>(&clock_cfg).map_err(InitError::ClockSetup)? };
    let src_clk_hz = parts.freq;
    let _wake_guard = parts.wake_guard;

    // Enable and freeze
    const ENABLE_TIMEOUT: u64 = 10; // Timeout for the `.enable()` call in ms
    info.control
        .enable(Some(Duration::from_millis(ENABLE_TIMEOUT)))
        .map_err(|_| InitError::Timeout)?;
    const FREEZE_TIMEOUT: u64 = 10; // Timeout for the `.freeze()` call in ms
    info.control
        .freeze(Some(Duration::from_millis(FREEZE_TIMEOUT)))
        .map_err(|_| InitError::Timeout)?;

    // If protocol_exception is supported, write whatever config value was passed in
    // Couldn't do this at the first protocol_exception check since this involves actually writing to a register
    if info.prexcen_supported {
        info.control
            .regs()
            .ctrl2()
            .modify(|m| m.set_prexcen(config.protocol_exception));
    }

    // Disable FDCAN
    info.control
        .regs()
        .mcr()
        .modify(|m| m.set_fden(pac::Fden::CanFdDisabled));

    // Route bus-off events to the ISR, and enable autorecovery.
    info.control.regs().ctrl1().modify(|w| {
        w.set_boffmsk(pac::Boffmsk::BusOffIntEnabled);
        w.set_boffrec(pac::Boffrec::AutoRecoverEnabled);
    });

    // Enable BOFFDONE interrupt
    info.control.regs().ctrl2().modify(|w| {
        w.set_boffdonemsk(true);
    });

    // Use expanded bit timing
    info.control.regs().ctrl2().modify(|m| m.set_bte(true));

    // Set the bit rate. The internal `timing::` functions use the term "baud rate" to keep parity
    // with NXP's C FlexCAN timing code, but everything exposes publicly by this module uses the term "bitrate".
    timing::set_baudrate(info, src_clk_hz, config.bitrate).map_err(InitError::TimingError)?;

    // As of rn this FlexCAN driver is based around the idea that all 32 message buffers are dedicated to TX.
    // So, this isn't something the user should be able to configure.
    const NUM_MESSAGE_BUFFERS: u8 = 32;
    info.control.set_number_of_message_buffers(NUM_MESSAGE_BUFFERS);

    // Setup rx and tx stuff
    mailbox::tx::setup(info).map_err(|_| InitError::Timeout)?;
    mailbox::rx::setup(info, &config.filters).map_err(|_| InitError::Timeout)?;

    Ok((info, _rx, _tx, _wake_guard))
}

/// This module contains implementations for functions that are needed on all `FlexCan` structs,
/// and can't be delegated to TX or RX specifically.
mod functions {
    use super::{BusErrorMode, Info, docs, pac};

    #[doc = docs::doc_error_mode!()]
    pub fn error_mode(info: &Info) -> BusErrorMode {
        match info.control.regs().esr1().read().fltconf() {
            pac::Fltconf::ErrorActive => BusErrorMode::ErrorActive,
            pac::Fltconf::ErrorPassive => BusErrorMode::ErrorPassive,
            pac::Fltconf::BusOff | pac::Fltconf::_RESERVED_3 => BusErrorMode::BusOff,
        }
    }
}

/// Shared rustdocs used multiple times for functions that are identical
/// between structs.
pub(in crate::flexcan::classic) mod docs {
    macro_rules! doc_error_mode {
        () => {
            concat!(
                "Returns the error mode the FlexCAN is currently in.\n",
                "See `BusErrorMode`.",
            )
        };
    }
    pub(in crate::flexcan::classic) use doc_error_mode;

    macro_rules! doc_tx_mailbox_full_count {
        () => { concat!(
            "Indicates the number of times new transmissions have been blocked or deferred due to the TX mailbox being full so far.\n\n",
            "Note: See the docs for `send()` and `try_send()` for each function's behavior when it encounters a full mailbox.",
        )};
    }
    pub(in crate::flexcan::classic) use doc_tx_mailbox_full_count;
}
