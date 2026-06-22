use core::marker::PhantomData;
use core::sync::atomic::{AtomicU32, Ordering};

use embassy_hal_internal::PeripheralType;
use embassy_sync::waitqueue::AtomicWaker;
use embassy_hal_internal::Peri;

use crate::flexcan::mailbox::tx;
use crate::flexcan::control::{Control, ControlError};
use crate::flexcan::frame::Frame;
use crate::interrupt::typelevel::{Handler, Interrupt};
use nxp_pac::can as pac;

// u_Note: eventually when an init function exists, set CTRL2[RRS] = 1
// u_Note: also need to write IMASK1 to all 1s at boot time, since we're dedicating the whole 32 message buffers to TX
// u_Note: also need to write all 1s to tx_available in init

// u_Note: eventually, when handling BusOff, im basically just going to reset the TX state (so set all 32 MBs back to INACTIVE, clear all IFLAG1 bits, clear all the bits in tx_remote, and set all the bits in tx_available).
// ^^^^ probably make a mailbox::init() function or maybe even tx::init() to handle this stuff
// (this way, can call those functions in both new() and whatever the busoff handling ends up looking like so i dont need to repeat that code)

/// FlexCAN driver bound to a single instance (CAN0/CAN1).
pub struct Can<'d> {
    info: &'static Info,
    _phantom: PhantomData<&'d mut ()>,
}

impl<'d> Can<'d> {
    pub fn new<T: Instance>(_peri: Peri<'d, T>, /* rx/tx pins, Config, irq binding */) -> Result<Self, ControlError> {
        use embassy_time::Duration;
        use crate::flexcan::mailbox;

        let info = T::info();
        let mut can = Self { info, _phantom: PhantomData };

        const ENABLE_TIMEOUT: u64 = 10; // Timeout for the `.enable()` call in ms
        can.control().enable(Some(Duration::from_millis(ENABLE_TIMEOUT)))?;

        // As of right now, the whole HAL is based around us having 32 message buffers.
        // So, this isn't something the user should be able to configure.
        const NUM_MESSAGE_BUFFERS: u8 = 32; 
        can.control().set_number_of_message_buffers(NUM_MESSAGE_BUFFERS);

        // enable_and_reset clocks, then the init steps from the u_Notes:
        //   CTRL2[RRS] = 1, IMASK1 = all 1s, tx_available = all 1s, etc.
        mailbox::tx::setup(info);


        can.control().unfreeze();
        Ok(can)
    }

    /// Access the `Control` sub-handler, which contains a bunch of random helpers for controlling/configuring the FlexCAN peripheral.
    #[allow(dead_code)]
    fn control(&mut self) -> Control<'_> {
        Control::new(self.info)
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

/// Info and state for a single FlexCAN instance.
pub(crate) struct Info {
    /// Raw FlexCAN registers.
    pub regs: pac::Can,

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
}

impl Info {
    #[inline(always)]
    fn regs(&self) -> pac::Can {
        self.regs
    }
}

pub(crate) trait SealedInstance {
    fn info() -> &'static Info;
}

/// Trait implemented by each FlexCAN peripheral instance.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static + Send {
    type Interrupt: Interrupt;
}

/// FlexCAN interrupt handler.
/// Construct this in a `bind_interrupts!` block to route an IRQ (e.g., CAN0, CAN1) here.
pub struct InterruptHandler<T: Instance> { _phantom: PhantomData<T> }

impl<T: Instance> Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let info = T::info();
        let can = info.regs();

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

        // u_TODO: when RX (Enhanced RX FIFO: erfsr/erfier) and error/bus-off (esr1/ctrl1) handling are added, demux them here
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_flexcan_instance {
    ($n:expr) => {
        paste::paste! {
            impl crate::flexcan::can::SealedInstance for crate::peripherals::[<CAN $n>] {
                fn info() -> &'static crate::flexcan::can::Info {
                    static INFO: crate::flexcan::can::Info = crate::flexcan::can::Info {
                        regs: crate::pac::[<CAN $n>],
                        tx_available: core::sync::atomic::AtomicU32::new(0),
                        tx_remote: core::sync::atomic::AtomicU32::new(0),
                        tx_waker: embassy_sync::waitqueue::AtomicWaker::new(),
                    };
                    &INFO
                }
            }

            impl crate::flexcan::can::Instance for crate::peripherals::[<CAN $n>] {
                type Interrupt = crate::interrupt::typelevel::[<CAN $n>];
            }
        }
    };
}

crate::impl_flexcan_instance!(0); // u_Note: There is an interrrupt for CAN0, but none for CAN1 for some reason. Probably should look into the PAC crate again to see if it can be generated. If CAN1 is ever able to exist, implement it w/ `crate::impl_flexcan_instance!(1);`
// u_Note: also, it seems like the other drivers (i.e., lpuart) get their impl_xxx_instance!() macros called in the codegen, so probably should look into doing that