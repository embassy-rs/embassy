pub mod frame;
mod mailbox;

use core::marker::PhantomData;
use core::sync::atomic::{AtomicU32, Ordering};

use embassy_sync::waitqueue::AtomicWaker;
use embassy_hal_internal::Peri;

use crate::flexcan::classic::mailbox::tx;
use crate::flexcan::classic::frame::Frame;
use crate::flexcan::filter::{FilterConfig, FilterConfigError};
use crate::flexcan::control::{Control, ControlError};
use crate::interrupt::typelevel::Handler;
use nxp_pac::can as pac;

/// FlexCAN driver instance, in Classic CAN mode.
pub struct FlexCan<'d> {
    info: &'static Info,
    _phantom: PhantomData<&'d mut ()>,
}

pub(in crate::flexcan) trait SealedInstance { fn info() -> &'static Info; }
#[allow(private_bounds)]
pub trait Instance: crate::flexcan::Instance + SealedInstance {}

/// Info and state for a single `classic::FlexCan` instance.
pub(in crate::flexcan) struct Info {
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
    /// to specify this constraint internally via `impl_flexcan_instance!()`. This way,
    /// if a user tries to enable this feature in their config on an unsupported peripheral,
    /// they'll get an error at init-time.
    pub prexcen_supported: bool,
}

/// Errors that can return when initializing
/// a `FlexCan` instance.
#[non_exhaustive]
pub enum InitError {
    /// This error indicates that the hardware didn't response
    /// within a reasonable timeframe to a request the HAL made.
    Timeout,

    /// This error indicates an invalid FilterConfig. See the `FilterConfigError`
    /// enum for the specific possible errors.
    Filter(FilterConfigError),

    /// This error indicates that you attempted enabling the
    /// `protocol_exception` feature in `FlexCanConfig` on a
    /// peripheral where it is not supported.
    /// 
    /// Note: Protocol Exception is only supported on CAN0.
    ProtocolExceptionUnsupported,
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
}

impl<'d> FlexCan<'d> {
    /// Constructs a new FlexCAN driver instance, in Classic mode.
    /// 
    /// 
    pub fn new<T: Instance>(_peri: Peri<'d, T>, config: FlexCanConfig, /* rx/tx pins, Config, irq binding */) -> Result<Self, InitError> {
        use embassy_time::Duration;

        let info = T::info();

        // Software-only error checks to make sure stuff was configured correctly
        if config.protocol_exception && !info.prexcen_supported {return Err(InitError::ProtocolExceptionUnsupported);}
        config.filters.validate().map_err(|e| InitError::Filter(e))?;

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

        // As of right now, the whole HAL is based around us having 32 message buffers.
        // So, this isn't something the user should be able to configure.
        const NUM_MESSAGE_BUFFERS: u8 = 32;
        info.control.set_number_of_message_buffers(NUM_MESSAGE_BUFFERS);

        // Reset/setup the TX message buffers and the software state tracking.
        mailbox::tx::setup(info).map_err(|_| InitError::Timeout)?;

        // Reset/setup the Enhanced RX FIFO.
        mailbox::rx::setup(info, &config.filters).map_err(|_| InitError::Timeout)?;

        info.control.unfreeze();
        Ok(Self { info, _phantom: PhantomData })
    }

    /// Sends a CAN message.
    /// If there's no space left in the TX buffers, this
    /// call asynchronously waits for space to free up, and then tries again.
    pub async fn send(&mut self, frame: Frame) {
        use core::future::poll_fn;
        use nb::Error::{WouldBlock, Other};
        use core::task::Poll;

        let message = tx::TxMessage::from(frame);
        poll_fn(|cx| {
            self.info.tx_waker.register(cx.waker());
            match tx::dispatch(self.info, &message) {
                Ok(()) => Poll::Ready(()),
                Err(WouldBlock) => Poll::Pending,
                Err(Other(e)) => match e {},
            }
        }).await
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
            // uhh do this tomorrow
        }

        // u_TODO: when RX (Enhanced RX FIFO: erfsr/erfier) and error/bus-off (esr1/ctrl1) handling are added, demux them here
    }
}