pub mod classic;
pub mod filter;
mod control;

pub use control::ControlError;

use embassy_hal_internal::PeripheralType;
use crate::interrupt::typelevel::Interrupt;

/// Shared, mode-agnostic peripheral identity. Each FlexCAN instance implements this
/// regardless of whether it is used in Classic or (future) FD mode.
pub trait Instance: PeripheralType + 'static + Send {
    type Interrupt: Interrupt;
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_flexcan_instance {
    ($n:expr) => {
        paste::paste! {
            // mode-agnostic peripheral identity
            impl crate::flexcan::Instance for crate::peripherals::[<CAN $n>] {
                type Interrupt = crate::interrupt::typelevel::[<CAN $n>];
            }

            // stuff for classic CAN mode
            impl crate::flexcan::classic::SealedInstance for crate::peripherals::[<CAN $n>] {
                fn info() -> &'static crate::flexcan::classic::Info {
                    static INFO: crate::flexcan::classic::Info = crate::flexcan::classic::Info {
                        control: crate::flexcan::control::Control::new(crate::pac::[<CAN $n>]),
                        tx_available: core::sync::atomic::AtomicU32::new(0),
                        tx_remote: core::sync::atomic::AtomicU32::new(0),
                        tx_waker: embassy_sync::waitqueue::AtomicWaker::new(),
                        prexcen_supported: $n == 0, // Protocol Exception is only supported on CAN0.
                    };
                    &INFO
                }
            }
            impl crate::flexcan::classic::Instance for crate::peripherals::[<CAN $n>] {}

            // u_TODO FDCAN mode: uncomment this block once a `fdcan` module exists alongside `classic`
            // impl crate::flexcan::fdcan::SealedInstance for crate::peripherals::[<CAN $n>] {
            //     fn info() -> &'static crate::flexcan::fdcan::Info {
            //         static INFO: crate::flexcan::fdcan::Info = crate::flexcan::fdcan::Info {
            //             control: crate::flexcan::control::Control::new(crate::pac::[<CAN $n>]),
            //             // other FDCAN stuff
            //         };
            //         &INFO
            //     }
            // }
            // impl crate::flexcan::fdcan::Instance for crate::peripherals::[<CAN $n>] {}
        }
    };
}

crate::impl_flexcan_instance!(0); // u_Note: There is an interrrupt for CAN0, but none for CAN1 for some reason. Probably should look into the PAC crate again to see if it can be generated. If CAN1 is ever able to exist, implement it w/ `crate::impl_flexcan_instance!(1);`
// u_Note: also, it seems like the other drivers (i.e., lpuart) get their impl_xxx_instance!() macros called in the codegen, so probably should look into doing that