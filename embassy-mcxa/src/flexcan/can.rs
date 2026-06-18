use core::marker::PhantomData;
use core::sync::atomic::{AtomicU32, Ordering};

use embassy_hal_internal::PeripheralType;
use embassy_sync::waitqueue::AtomicWaker;

use crate::interrupt::typelevel::{Handler, Interrupt};
use nxp_pac::can as pac;

/// Info and state for a single FlexCAN instance.
pub(crate) struct Info {
    pub regs: pac::Can,

    /// Each bit indicates the if that message buffer (one of the 32) is available for TX use.
    /// If `1`, the message buffer can be used to transmit a new TX message.
    /// If `0`, the message buffer is currently claimed and in-use.
    /// These bits get set by the ISR, and cleared by the TX future.
    pub tx_available: AtomicU32,
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
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

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
            can.iflag1().write(|w| w.0 = tx_fired); // IFLAG1 is a "write 1 to clear" register. So, doing this basically just acknowledges that these interrupts fired, and clears them back to zero (so they can fire again in the future).
            info.tx_available.fetch_or(tx_fired, Ordering::Release); // Update the `tx_available` tracker accordingly.
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
                        tx_done: core::sync::atomic::AtomicU32::new(0),
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