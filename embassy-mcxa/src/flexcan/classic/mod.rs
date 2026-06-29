pub mod frame;
mod mailbox;
mod meta;
mod timing;

use core::marker::PhantomData;
use core::sync::atomic::{AtomicU32, Ordering};

use embassy_sync::waitqueue::AtomicWaker;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::{Channel};
use embassy_hal_internal::Peri;

use crate::flexcan::classic::mailbox::tx;
use crate::flexcan::classic::frame::Frame;
use crate::flexcan::filter::FilterConfig;
use crate::flexcan::control::{Control};
use crate::flexcan::{RxPin, TxPin};
use crate::gpio::AnyPin;
use crate::interrupt::typelevel::{Handler, Interrupt};
use crate::flexcan::classic::meta::rx_queue_size::RX_QUEUE_SIZE;
use crate::clocks::{enable_and_reset, ClockError, PoweredClock, WakeGuard};
use crate::clocks::periph_helpers::{CanClockSel, CanConfig, CanInstance, Div4};
use nxp_pac::can as pac;

use crate::flexcan::classic::meta::docs::{doc_send, doc_try_send, doc_receive, doc_try_receive, doc_error_mode};

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
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SendError {
    /// The TX mailbox is currently full.
    TxMailboxFull,

    /// The bus is currently in BusOff mode and cannot immediately dispatch a message.
    BusOff,
}

/// Errors that may occur when calling try_receive().
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ReceiveError {
    /// There were no new messages to be received.
    NoMessages,
}

/// Errors that may occur when attempting to join a `FlexCanRx` and
/// `FlexCanTx` into a single `FlexCan`.
pub enum JoinError<'d> {
    /// You have tried to join together a `FlexCanRx` and `FlexCanTx`
    /// that come from different peripherals (e.g., `FlexCanRx` is from CAN0 while
    /// `FlexCanTx` is from CAN1). This is not valid.
    ///
    /// The original halves are returned unchanged so you can recover them and try again.
    DifferentPeripherals(FlexCanTx<'d>, FlexCanRx<'d>),
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
    /// we need the ISR to manually write TX-INACTIVE back to any message buffer where an REMOTE message was sent. These bits
    /// let the ISR track what buffers it needs to do this for.
    /// 
    /// TLDR: These bits are set by `dispatch()` for REMOTE frames, and cleared by the ISR once the buffer has been neutralized back to TX-INACTIVE.
    pub tx_remote: AtomicU32,

    /// Waker used to wake tasks awaiting on a CAN send() call.
    pub tx_waker: AtomicWaker,

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

    /// Software queue that holds received RX frames.
    pub rx_channel: Channel<CriticalSectionRawMutex, Frame, RX_QUEUE_SIZE>,
}

pub(crate) trait SealedInstance: crate::clocks::Gate<MrccPeriphConfig = CanConfig> {
    fn info() -> &'static Info;

    /// Which MRCC clock instance this peripheral maps to. 
    /// This is used to select the correct `MRCC_FLEXCANn_CLKSEL`/`CLKDIV` registers during clock setup.
    const CLOCK_INSTANCE: CanInstance;
}
#[allow(private_bounds)]
pub trait Instance: crate::flexcan::Instance + SealedInstance {}

/// RX-specific FlexCAN instance. Can be obtained by calling `.split()`
/// on your main `FlexCan` instance.
pub struct FlexCanRx<'d> {
    info: &'static Info,
    _rx: Peri<'d, AnyPin>,
}

impl<'d> FlexCanRx<'d> {
    /// Creates a new `FlexCanRx` instance.
    /// This isn't a public function, and should only be called via `.split()` in the `FlexCan` struct.
    fn new(info: &'static Info, rx: Peri<'d, AnyPin>) -> Self {
        Self { info, _rx: rx }
    }

    #[doc = doc_receive!()]
    pub async fn receive(&self) -> Frame { functions::receive(self.info).await }
    #[doc = doc_try_receive!()]
    pub fn try_receive(&self) -> Result<Frame, ReceiveError> { functions::try_receive(self.info) }
    #[doc = doc_error_mode!()]
    pub fn error_mode(&self) -> BusErrorMode { functions::error_mode(self.info) }
}

/// TX-specific FlexCAN instance. Can be obtained by calling `.split()`
/// on your main `FlexCan` instance.
pub struct FlexCanTx<'d> {
    info: &'static Info,
    _tx: Peri<'d, AnyPin>,

    /// Inhibits deep sleep while this driver is alive, if the selected clock source does not survive deep sleep. `None` if no guard is required.
    ///
    /// Note: When a `FlexCan` is split, the wake guard travels with the TX half. Dropping the TX half therefore
    /// releases the deep-sleep inhibition even if a `FlexCanRx` half is still alive.
    _wake_guard: Option<WakeGuard>,
}

impl<'d> FlexCanTx<'d> {
    /// Creates a new `FlexCanTx` instance.
    /// This isn't a public function, and should only be called via `.split()` in the `FlexCan` struct.
    fn new(info: &'static Info, tx: Peri<'d, AnyPin>, wake_guard: Option<WakeGuard>) -> Self {
        Self { info, _tx: tx, _wake_guard: wake_guard }
    }

    #[doc = doc_send!()]
    pub async fn send(&mut self, frame: &Frame) { functions::send(self.info, frame).await }
    #[doc = doc_try_send!()]
    pub fn try_send(&mut self, frame: &Frame) -> Result<(), SendError> { functions::try_send(self.info, frame) }
    #[doc = doc_error_mode!()]
    pub fn error_mode(&self) -> BusErrorMode { functions::error_mode(self.info) }
}

/// FlexCAN driver instance, in Classic CAN mode.
pub struct FlexCan<'d> {
    tx: FlexCanTx<'d>,
    rx: FlexCanRx<'d>,
}

impl<'d> FlexCan<'d> {
    /// Constructs a new FlexCAN driver instance, in Classic mode.
    /// 
    pub fn new<T: Instance>(
        _peri: Peri<'d, T>,
        rx: Peri<'d, impl RxPin<T>>,
        tx: Peri<'d, impl TxPin<T>>,
        config: FlexCanConfig,
    ) -> Result<Self, InitError> {
        use embassy_time::Duration;

        let info = T::info();

        // Software-only error checks to make sure stuff was configured correctly
        if config.protocol_exception && !info.prexcen_supported {return Err(InitError::ProtocolExceptionUnsupported);}

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

        // SAFETY: `_peri` gives us exclusive ownership of this peripheral, and we haven't touched it yet, so we should be good here.
        let parts = unsafe { enable_and_reset::<T>(&clock_cfg).map_err(InitError::ClockSetup)? };
        let src_clk_hz = parts.freq;
        let _wake_guard = parts.wake_guard;

        // Enable and freeze
        const ENABLE_TIMEOUT: u64 = 10; // Timeout for the `.enable()` call in ms
        info.control.enable(Some(Duration::from_millis(ENABLE_TIMEOUT))).map_err(|_| InitError::Timeout)?;
        const FREEZE_TIMEOUT: u64 = 10; // Timeout for the `.freeze()` call in ms
        info.control.freeze(Some(Duration::from_millis(FREEZE_TIMEOUT))).map_err(|_| InitError::Timeout)?;

        // If protocol_exception is supported, write whatever config value was passed in
        // Couldn't do this at the first protocol_exception check since this involves actually writing to a register
        if info.prexcen_supported {
            info.control.regs().ctrl2().modify(|m| m.set_prexcen(config.protocol_exception));
        }

        // Disable FDCAN
        info.control.regs().mcr().modify(|m| m.set_fden(pac::Fden::CanFdDisabled));

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
        timing::set_baudrate(info, src_clk_hz, config.bitrate).map_err(|e| InitError::TimingError(e))?;

        // As of right now, the whole HAL is based around us having 32 message buffers.
        // So, this isn't something the user should be able to configure.
        const NUM_MESSAGE_BUFFERS: u8 = 32;
        info.control.set_number_of_message_buffers(NUM_MESSAGE_BUFFERS);

        // Reset/setup the TX message buffers and the software state tracking.
        mailbox::tx::setup(info).map_err(|_| InitError::Timeout)?;

        // Reset/setup the Enhanced RX FIFO.
        mailbox::rx::setup(info, &config.filters).map_err(|_| InitError::Timeout)?;

        // Setup the interrupts
        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable(); }

        info.control.unfreeze();

        let tx = FlexCanTx::new(info, _tx, _wake_guard);
        let rx = FlexCanRx::new(info, _rx);
        Ok(Self { tx, rx })
    }

    /// Consumes this `FlexCan` and splits it into independent `FlexCanTx` and `FlexCanRx` halves. This is useful
    /// if you want to have separate dedicated tasks for CAN RX and CAN TX.
    pub fn split(self) -> (FlexCanTx<'d>, FlexCanRx<'d>) {
        (self.tx, self.rx)
    }

    /// Recombines a `FlexCanTx` and `FlexCanRx` (previously obtained from `split()`) back into a `FlexCan`.
    ///
    /// If the two halves come from different peripherals, this returns `Err(JoinError::DifferentPeripherals(tx, rx))`,
    /// where `tx` and `rx` are the same `tx`/`rx` you passed in. This means you can try again if you need to.
    pub fn join(tx: FlexCanTx<'d>, rx: FlexCanRx<'d>) -> Result<Self, JoinError<'d>> {
        if !core::ptr::eq(tx.info, rx.info) { return Err(JoinError::DifferentPeripherals(tx, rx)); }
        Ok(Self { tx, rx })
    }

    #[doc = doc_send!()]
    pub async fn send(&mut self, frame: &Frame) { self.tx.send(frame).await }
    #[doc = doc_try_send!()]
    pub fn try_send(&mut self, frame: &Frame) -> Result<(), SendError> { self.tx.try_send(frame) }
    #[doc = doc_receive!()]
    pub async fn receive(&self) -> Frame { self.rx.receive().await }
    #[doc = doc_try_receive!()]
    pub fn try_receive(&self) -> Result<Frame, ReceiveError> { self.rx.try_receive() }
    #[doc = doc_error_mode!()]
    pub fn error_mode(&self) -> BusErrorMode { self.tx.error_mode() } 
}

/// This module contains the actual implementations of the functions used by
/// `FlexCan`, `FlexCanTx`, and `FlexCanRx`. They're not implemented directly
/// on those structs because some functions are common to more than one of them.
mod functions {
    use super::{Frame, Info, tx, SendError, ReceiveError, BusErrorMode, pac};
    use super::{doc_send, doc_try_send, doc_receive, doc_try_receive, doc_error_mode};

    #[doc = doc_send!()]
    pub async fn send(info: &Info, frame: &Frame) {
        use core::future::poll_fn;
        use nb::Error::{WouldBlock, Other};
        use core::task::Poll;

        let message = tx::TxMessage::from(frame);
        poll_fn(|cx| {
            info.tx_waker.register(cx.waker());
            match tx::dispatch(info, &message) {
                Ok(()) => Poll::Ready(()),
                Err(WouldBlock) => Poll::Pending,
                Err(Other(e)) => match e {},
            }
        }).await
    }

    #[doc = doc_try_send!()]
    pub fn try_send(info: &Info, frame: &Frame) -> Result<(), SendError> {
        use nb::Error::{WouldBlock, Other};

        if error_mode(info) == BusErrorMode::BusOff {
            return Err(SendError::BusOff);
        }

        let message = tx::TxMessage::from(frame);
        match tx::dispatch(info, &message) {
            Ok(()) => Ok(()),
            Err(WouldBlock) => Err(SendError::TxMailboxFull),
            Err(Other(e)) => match e {},
        }
    }

    #[doc = doc_receive!()]
    pub async fn receive(info: &Info) -> Frame {
        info.rx_channel.receive().await
    }

    #[doc = doc_try_receive!()]
    pub fn try_receive(info: &Info) -> Result<Frame, ReceiveError> {
        info.rx_channel.try_receive().map_err(|_| ReceiveError::NoMessages)
    }

    #[doc = doc_error_mode!()]
    pub fn error_mode(info: &Info) -> BusErrorMode {
        match info.control.regs().esr1().read().fltconf() {
            pac::Fltconf::ErrorActive => BusErrorMode::ErrorActive,
            pac::Fltconf::ErrorPassive => BusErrorMode::ErrorPassive,
            pac::Fltconf::BusOff | pac::Fltconf::_RESERVED_3 => BusErrorMode::BusOff,
        }
    }
}

/// FlexCAN interrupt handler.
/// Construct this in a `bind_interrupts!` block to route an IRQ (e.g., CAN0, CAN1) here.
pub struct InterruptHandler<T: Instance> { _phantom: PhantomData<T> }

impl<T: Instance> Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let info = T::info();
        let can = info.control.regs();

        /* TX STUFF: */

        // Check what TX buffers have fired
        let tx_flags = can.iflag1().read().0;
        let tx_enabled = can.imask1().read().0;
        let tx_fired = tx_flags & tx_enabled; // Any TX buffers that have just fired and need to be reset will be marked as `1` here.

        // If any TX buffers have fired, we can reset them and mark them as available for re-use now.
        if tx_fired != 0 {

            // For more context about this following block, see the comment above `tx_remote`. TLDR: This block of code
            // is only relavent when we transmit REMOTE frames.
            let remote_fired = tx_fired & info.tx_remote.load(Ordering::Relaxed);
            if remote_fired != 0 {
                let mut bits = remote_fired;
                while bits != 0 {
                    let n = bits.trailing_zeros() as usize;
                    tx::buffer::set_inactive(info, n); // INACTIVE
                    bits &= bits - 1; // Clear the lowest set bit.
                }
                // Clear the remote markings before the buffers are advertised as available, so
                // that `dispatch()` never observes a free buffer that is still flagged remote.
                info.tx_remote.fetch_and(!remote_fired, Ordering::Relaxed);
            }

            // Actually clear the interrupt flag
            can.iflag1().write(|w| w.0 = tx_fired); // IFLAG1 is a "write 1 to clear" register. So, doing this basically just acknowledges that these interrupts fired, and clears them back to zero (so they can fire again in the future).
            let _ = can.iflag1().read(); // read back from the register so we make sure the write finished before we return from the ISR
            info.tx_available.fetch_or(tx_fired, Ordering::Release); // Update the `tx_available` tracker accordingly.
            info.tx_waker.wake(); // Tell sleepers that there's an available TX buffer now
        }

        /* RX STUFF: */

        // Check if any RX messages can be dequeued, and if so, dequeue them.
        while let Some(message) = mailbox::rx::fifo::get(info) {
            // Dequeue a frame from the hardware RX FIFO
            let frame: Frame = match message.try_into() {
                Ok(message) => message,

                // The try_into() shouldn't actually be able to fail since the PAC already ensures std()/ext() can't
                // exceed 11 bits/29 bits, but if it does somehow, just drop the frame.
                Err(_) => { continue; }
            };
            
            // Push the frame into the software RX queue
            if info.rx_channel.try_send(frame).is_err() {
                // if the software queue is full, do nothing (i.e., drop the frame)
                // eventually, it could be nice to increment a "dropped" counter or something that the
                // user of the HAL can look at on their own time
            }
        }

        /* BUSOFF STUFF: */
        let esr1 = can.esr1().read();

        // Handle when BusOff has triggered
        if esr1.boffint() {
            // Acknowledge the flag (write 1 to clear)
            can.esr1().write(|w| w.set_boffint(true));
            let _ = can.esr1().read(); // make sure the clear lands before returning
        }

        // Handle when BusOff autorecovery has finished
        if esr1.boffdoneint() == pac::Boffdoneint::BusOffDone {
            // Acknowledge the flag (write 1 to clear)
            can.esr1().write(|w| w.set_boffdoneint(pac::Boffdoneint::BusOffDone));
            let _ = can.esr1().read(); // Make surethe clear lands before returning
        }
    }
}