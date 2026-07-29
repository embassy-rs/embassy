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

// Async example for cargo doc
#![doc = concat!(
    "<details>\n\n",
    "<summary><h4>Async Example</h4></summary>\n\n",
    "Here's a short example program that demonstrates how to set up a FlexCAN peripheral in Async mode for Classic CAN using this HAL:\n",
    "```rust,no_run\n",
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../examples/mcxa2xx/src/bin/flexcan-classic-async.rs")),
    "\n```\n",
    "</details>",
)]
// Blocking example for cargo doc
#![doc = concat!(
    "<details>\n\n",
    "<summary><h4>Blocking Example</h4></summary>\n\n",
    "Here's a short example program that demonstrates how to set up a FlexCAN peripheral in Blocking mode for Classic CAN using this HAL:\n",
    "```rust,no_run\n",
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../examples/mcxa2xx/src/bin/flexcan-classic-blocking.rs")),
    "\n```\n",
    "</details>",
)]

pub(crate) mod asynchronous;
pub(crate) mod blocking;
pub mod frame;
mod mailbox;
mod timing;

use core::cell::Cell;
use core::sync::atomic::{AtomicU8, AtomicU32, Ordering};

pub use asynchronous::{Async, InterruptHandler, RxQueue};
pub use blocking::{Blocking, ReceiveErrorWithTimeout, SendErrorWithTimeout};
use embassy_hal_internal::Peri;
use embassy_hal_internal::drop::OnDrop;
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::{Channel, SendDynamicSender};
use maitake_sync::WaitCell;
use nxp_pac::can as pac;

use crate::clocks::periph_helpers::{CanClockSel, CanConfig, CanInstance, Div4};
use crate::clocks::{ClockError, PoweredClock, WakeGuard, enable_and_reset};
use crate::flexcan::classic::frame::Frame;
use crate::flexcan::classic::halves::FlexCanPins;
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

    /// This error indicates that, when reconstructing a previously-dropped FlexCAN peripheral,
    /// stale descriptors were encountered for the TX and RX pins. This means that there has been a state
    /// management error internal to the FlexCAN HAL itself.
    ReconstructionError,
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

    /// Waker used to wake tasks awaiting on a CAN send() call. Only relavent when in `Async` mode.
    pub tx_waker: WaitCell,

    /// Handle to the RX queue's sender. Only relavent when in `Async` mode.
    pub rx_sender: Mutex<CriticalSectionRawMutex, Cell<Option<SendDynamicSender<'static, Frame>>>>,

    /// Stores a count of the number of RX frames dropped so far due to the Software RX Channel being full. Only relavent when in `Async` mode.
    pub rx_dropped_count_channel: AtomicU32,

    /// Stores a count of the number of RX frames dropped so far due to the Hardware RX FIFO being full. Only relavent when in `Async` mode
    ///
    /// Note: Technically the hardware RX FIFO still exists in blocking mode, but we can't use any interrupts to track it. So that's why this is an async-only feature.
    pub rx_dropped_count_fifo: AtomicU32,

    /// Tracks which havles (TX, RX, or both) are currently active.
    pub halves: halves::HalfMask,

    /// Callback to disable the FlexCAN interrupt (used for teardown).
    pub interrupt_disable: fn(),
    /// Callback to unpend the FlexCAN interrupt (used for teardown).
    pub interrupt_unpend: fn(),
    /// Callback to disable the FlexCAN clock (used for teardown).
    pub clock_disable: unsafe fn(),

    /// Stores the descriptors for the TX and RX pins.
    pub pins: halves::Pins,
}

/// Errors that can arise when disabling/dropping the FlexCAN peripheral.
#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum DisableError {
    /// The hardware did not respond within a reasonable time period, so full
    /// disabling was not able to be completed.
    Timeout,
}

impl Info {
    /// Resets software state that persists across driver constructions.
    pub(crate) fn reset(&'static self) {
        self.rx_sender.lock(|sender| sender.set(None));
        self.tx_waker.wake();
        self.tx_available.store(0, Ordering::Relaxed);
        self.tx_remote.store(0, Ordering::Relaxed);
        self.tx_mailbox_full_count.store(0, Ordering::Relaxed);
        self.rx_dropped_count_channel.store(0, Ordering::Relaxed);
        self.rx_dropped_count_fifo.store(0, Ordering::Relaxed);
    }

    /// Disables shared FlexCAN resources after the final TX/RX half is dropped.
    pub(crate) fn disable_peripheral(&'static self) -> Result<(), DisableError> {
        use embassy_time::Duration;

        // Prevent a peripheral source from racing with teardown or repending the NVIC line.
        (self.interrupt_disable)();
        self.control.regs().imask1().write(|w| w.0 = 0);
        self.control.regs().erfier().modify(|w| {
            w.set_erfufwie(false);
            w.set_erfovfie(false);
            w.set_erfwmiie(false);
            w.set_erfdaie(false);
        });
        self.control
            .regs()
            .ctrl1()
            .modify(|w| w.set_boffmsk(pac::Boffmsk::BusOffIntDisabled));
        self.control.regs().ctrl2().modify(|w| w.set_boffdonemsk(false));

        // Clear all sticky sources before the next construction enables this IRQ again.
        self.control.regs().iflag1().write(|w| w.0 = u32::MAX);
        self.control.regs().erfsr().write(|w| {
            w.set_erfufw(true);
            w.set_erfovf(true);
            w.set_erfwmi(pac::Erfwmi::WatermarkYes);
            w.set_erfda(true);
        });
        self.control.regs().esr1().write(|w| {
            w.set_boffint(true);
            w.set_boffdoneint(pac::Boffdoneint::BusOffDone);
        });

        self.rx_sender.lock(|sender| sender.set(None));
        self.tx_waker.wake();

        // Do not gate the peripheral clock until FlexCAN confirms that it is inactive.
        const DISABLE_TIMEOUT: Duration = Duration::from_millis(10);
        let result = match self.control.disable(Some(DISABLE_TIMEOUT)) {
            Ok(()) => {
                unsafe {
                    (self.clock_disable)();
                }
                Ok(())
            }
            Err(_) => Err(DisableError::Timeout),
        };
        (self.interrupt_unpend)();

        result
    }

    /// Disables the shared pins in `Info`.
    /// This is meant to be called only when the last living half (`FlexCanTx` or `FlexCanRx`) is dropped.
    fn disable_registered_pins(&self) {
        if let Some(pins) = self.pins.take() {
            pins.disable();
        }
    }
}

/// Contains stuff related to tracking FlexCAN halves. Modelled after `TxRxRef` and `TxRxRefMask` from embassy-mcxa LPUART.
///
/// A big part of this module is the tracking of the FlexCAN TX and RX pins (see `FlexCanPins`). Because CAN TX and RX are quite closely
/// coupled, it isn't really possible to fully drop a TX or RX half independently by disabling its repsective pin as much of the rest of this HAL does,
/// since that would disrupt the remaining half's ability to ACK/detect bus errors. The solution to this is having `Info` store the `AnyPin` descriptors
/// of the FlexCAN pins, while the actual TX/RX halves still hold the owned `Peri` instances. This is probably kind of janky, but it makes it possible to
/// wait for the last TX/RX half to get dropped before actually disabling the pins.
pub(crate) mod halves {
    use super::{AnyPin, AtomicU8, Cell, CriticalSectionRawMutex, Mutex, Ordering};
    use crate::gpio::SealedPin;

    /// TX and RX pins.
    #[derive(Clone, Copy)]
    pub struct FlexCanPins {
        rx: AnyPin,
        tx: AnyPin,
    }
    impl FlexCanPins {
        /// Creates a new `FlexCanPins` from an RX and a TX pin.
        pub fn new(tx: AnyPin, rx: AnyPin) -> Self {
            Self { tx, rx }
        }

        /// Disables the `FlexCanPins` by calling `set_as_disabled()` on them.
        pub fn disable(&self) {
            self.rx.set_as_disabled();
            self.tx.set_as_disabled();
        }
    }

    /// Stores the descriptors of the TX and RX pins for use in `Info`.
    pub struct Pins {
        pins: Mutex<CriticalSectionRawMutex, Cell<Option<FlexCanPins>>>,
    }
    impl Pins {
        /// Creates an empty/`None` `Pins`.
        pub const fn new() -> Self {
            Self {
                pins: Mutex::new(Cell::new(None)),
            }
        }

        /// Sets the pins stored in a `Pins`.
        ///
        /// This function is meant to be called during construction to set up the pins. If this function is called
        /// on a `Pins` that has already been set up, it will return Err(()). This indicates that the HAL has a lifecycle
        /// issue in how it is constructing/deconstructing `Pins`.
        pub fn set(&self, pins: FlexCanPins) -> Result<(), ()> {
            self.pins.lock(|cell| {
                // if cell is `Some`, then `set()` has already been called for these `Pins`, which is not allowed
                if cell.get().is_some() {
                    return Err(());
                }
                cell.set(Some(pins));
                Ok(())
            })
        }

        /// Takes the internal `FlexCanPins`.
        ///
        /// This function will return `Some(FlexCanPins)` if the pins have already been set (i.e., `set()` has been called), and will
        /// return `None` if the pins have already been taken or never set.
        pub fn take(&self) -> Option<FlexCanPins> {
            self.pins.lock(|cell| cell.take())
        }
    }

    /// Masks representing which halves are active (bit set to `1` means active).
    /// This is used for the `Drop`/teardown stuff.
    #[repr(u8)]
    pub enum Half {
        Rx = 0b01,
        Tx = 0b10,
    }
    /// Tracks which halves of FlexCAN (TX, RX, or both) are currently active.
    pub struct HalfMask(AtomicU8);
    impl HalfMask {
        pub const fn new() -> Self {
            Self(AtomicU8::new(0))
        }

        /// Atomically signal that either the Tx or Rx has been dropped.
        ///
        /// Returns `true` if after this call all parts are inactive.
        pub fn set_inactive_fetch_last(&self, value: Half) -> bool {
            let value = value as u8;
            self.0.fetch_and(!value, Ordering::AcqRel) & !value == 0
        }

        /// Atomically signal that either the Tx or Rx has been created.
        pub fn set_active(&self, value: Half) {
            let value = value as u8;
            self.0.fetch_or(value, Ordering::AcqRel);
        }

        /// Atomically determine if either channels have been created, but not dropped. Clears the state.
        ///
        /// Should only be relevant when any of the parts have been leaked using [core::mem::forget].
        pub fn fetch_any_alive_reset(&self) -> bool {
            self.0.swap(0, Ordering::AcqRel) != 0
        }
    }
}

pub(crate) trait SealedInstance: crate::clocks::Gate<MrccPeriphConfig = CanConfig> {
    fn info() -> &'static Info;

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
        info.halves.set_active(halves::Half::Rx);

        Self {
            info,
            _rx: rx,
            _wake_guard: wake_guard,
            mode,
        }
    }

    /// Returns the error mode the FlexCAN is currently in.
    /// See `BusErrorMode`.
    pub fn error_mode(&self) -> BusErrorMode {
        functions::error_mode(self.info)
    }
}

/// Teardown for `FlexCanRx`.
impl<M: Mode> Drop for FlexCanRx<'_, M> {
    fn drop(&mut self) {
        // Stop ISR delivery into an RX queue that no longer has a receiver.
        self.info.control.regs().erfier().modify(|w| {
            w.set_erfdaie(false);
            w.set_erfovfie(false);
        });
        self.info.rx_sender.lock(|sender| sender.set(None));

        let last = self.info.halves.set_inactive_fetch_last(halves::Half::Rx);
        if last {
            match self.info.disable_peripheral() {
                Ok(_) => (),
                Err(_err) => {
                    #[cfg(feature = "defmt")]
                    defmt::error!(
                        "FlexCAN: FlexCanRx Drop failed: Could not disable FlexCAN peripheral due to a hardware timeout: {}",
                        _err
                    );
                }
            };

            // only disable both pins when we are the last one standing
            self.info.disable_registered_pins();
        }
    }
}

/// Teardown for `FlexCanTx`.
impl<M: Mode> Drop for FlexCanTx<'_, M> {
    fn drop(&mut self) {
        // Stop TX mailbox completion interrupts.
        self.info.control.regs().imask1().write(|w| w.0 = 0);
        self.info.tx_waker.wake();

        let last = self.info.halves.set_inactive_fetch_last(halves::Half::Tx);
        if last {
            match self.info.disable_peripheral() {
                Ok(_) => (),
                Err(_err) => {
                    #[cfg(feature = "defmt")]
                    defmt::error!(
                        "FlexCAN: FlexCanTx Drop failed: Could not disable FlexCAN peripheral due to a hardware timeout: {}",
                        _err
                    );
                }
            }

            // only disable both pins when we are the last one standing
            self.info.disable_registered_pins();
        }
    }
}

/// TX-specific FlexCAN instance. Can be obtained by calling `.split()`
/// on your main `FlexCan` instance.
///
/// Note: Dropping this handle does not flush pending frames. Call the mode-specific
/// flush method first (before dropping) when delivery must complete before teardown.
pub struct FlexCanTx<'d, M: Mode> {
    info: &'static Info,
    _tx: Peri<'d, AnyPin>,
    _wake_guard: Option<WakeGuard>,
    _mode: M,
}

/// General `FlexCanTx` functions (available for both `Async` and `Blocking` mode).
impl<'d, M: Mode> FlexCanTx<'d, M> {
    /// Creates a new `FlexCanTx` instance.
    /// This isn't a public function, and should only be called via the mode-specific constructors.
    fn new(info: &'static Info, tx: Peri<'d, AnyPin>, wake_guard: Option<WakeGuard>, mode: M) -> Self {
        info.halves.set_active(halves::Half::Tx);

        Self {
            info,
            _tx: tx,
            _wake_guard: wake_guard,
            _mode: mode,
        }
    }

    /// Returns the error mode the FlexCAN is currently in.
    /// See `BusErrorMode`.
    pub fn error_mode(&self) -> BusErrorMode {
        functions::error_mode(self.info)
    }

    /// Indicates the number of times new transmissions have been blocked or deferred due to the TX mailbox being full so far.
    ///
    /// Note: See the docs for `send()` and `try_send()` for each function's behavior when it encounters a full mailbox.
    pub fn tx_mailbox_full_count(&self) -> u32 {
        self.info.tx_mailbox_full_count.load(Ordering::Relaxed)
    }
}

/// FlexCAN driver instance, in Classic CAN mode.
pub struct FlexCan<'d, M: Mode> {
    tx: FlexCanTx<'d, M>,
    rx: FlexCanRx<'d, M>,
}

/// General `FlexCan` functions (available for both `Async` and `Blocking` mode).
impl<'d, M: Mode> FlexCan<'d, M> {
    /// Consumes this `FlexCan` and splits it into separately owned `FlexCanTx` and `FlexCanRx` halves. This is useful
    /// if you want to have separate dedicated tasks for CAN RX and CAN TX.
    pub fn split(self) -> (FlexCanTx<'d, M>, FlexCanRx<'d, M>) {
        (self.tx, self.rx)
    }

    /// Recombines a `FlexCanTx` and `FlexCanRx` (previously obtained from `split()`) back into a `FlexCan`.
    ///
    /// If the two halves come from different peripherals, this returns `Err((tx, rx))`,
    /// where `tx` and `rx` are the same `tx`/`rx` you passed in. This means you can try again if you need to.
    pub fn join(tx: FlexCanTx<'d, M>, rx: FlexCanRx<'d, M>) -> Result<Self, (FlexCanTx<'d, M>, FlexCanRx<'d, M>)> {
        if !core::ptr::eq(tx.info, rx.info) {
            return Err((tx, rx));
        }
        Ok(Self { tx, rx })
    }

    /// Returns the error mode the FlexCAN is currently in.
    /// See `BusErrorMode`.
    pub fn error_mode(&self) -> BusErrorMode {
        self.tx.error_mode()
    }

    /// Indicates the number of times new transmissions have been blocked or deferred due to the TX mailbox being full so far.
    ///
    /// Note: See the docs for `send()` and `try_send()` for each function's behavior when it encounters a full mailbox.
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

    // Recover shared resources if a previous driver was leaked with `mem::forget`,
    // or if a previous construction was interrupted after registering its pins.
    let stale_halves = info.halves.fetch_any_alive_reset();
    let stale_pins = info.pins.take();

    if stale_halves && stale_pins.is_none() {
        #[cfg(feature = "defmt")]
        defmt::warn!("FlexCAN: stale half state found without registered pins");
    } else if !stale_halves && stale_pins.is_some() {
        #[cfg(feature = "defmt")]
        defmt::warn!("FlexCAN: registered pins found without stale half state");
    }

    if stale_halves || stale_pins.is_some() {
        match info.disable_peripheral() {
            Ok(()) => (),
            Err(_err) => {
                // Preserve the cleanup recipe so a later construction can retry.
                if let Some(pins) = stale_pins {
                    let _ = info.pins.set(pins);
                }

                #[cfg(feature = "defmt")]
                defmt::error!(
                    "FlexCAN: Reconstruction failed: Previous FlexCAN driver could not be disabled due to a hardware timeout: {}",
                    _err
                );

                return Err(InitError::Timeout);
            }
        }
    }

    if let Some(pins) = stale_pins {
        pins.disable();
    }
    info.reset();

    // Mux the pins to their CAN function and take ownership for the driver's lifetime.
    rx.as_rx();
    tx.as_tx();
    let rx: Peri<'_, AnyPin> = rx.into();
    let tx: Peri<'_, AnyPin> = tx.into();
    let pin_rollback = OnDrop::new(|| {
        use crate::gpio::SealedPin;

        rx.set_as_disabled();
        tx.set_as_disabled();
    });

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
    let peripheral_rollback = OnDrop::new(|| match info.disable_peripheral() {
        Ok(()) => (),
        Err(_err) => {
            #[cfg(feature = "defmt")]
            defmt::error!(
                "FlexCAN: Peripheral Rollback failed after Drop: FlexCAN peripheral could not be disabled due to a hardware timeout: {}",
                _err
            );
        }
    });

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

    info.pins
        .set(FlexCanPins::new(*tx, *rx))
        .map_err(|_| InitError::ReconstructionError)?;

    peripheral_rollback.defuse();
    pin_rollback.defuse();
    Ok((info, rx, tx, _wake_guard))
}

/// This module contains implementations for functions that are needed on all `FlexCan` structs,
/// and can't be delegated to TX or RX specifically.
mod functions {
    use super::{BusErrorMode, Info, pac};

    /// Returns the error mode the FlexCAN is currently in.
    /// See `BusErrorMode`.
    pub fn error_mode(info: &Info) -> BusErrorMode {
        match info.control.regs().esr1().read().fltconf() {
            pac::Fltconf::ErrorActive => BusErrorMode::ErrorActive,
            pac::Fltconf::ErrorPassive => BusErrorMode::ErrorPassive,
            pac::Fltconf::BusOff | pac::Fltconf::_RESERVED_3 => BusErrorMode::BusOff,
        }
    }
}
