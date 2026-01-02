//! Low-power timer (LPTIM)

pub mod pwm;
pub mod timer;

use crate::rcc::RccPeripheral;

/// Timer channel.
#[cfg(any(lptim_v2a, lptim_v2b))]
mod channel;
#[cfg(any(lptim_v2a, lptim_v2b))]
pub use channel::Channel;
use embassy_hal_internal::PeripheralType;

pin_trait!(OutputPin, BasicInstance);
pin_trait!(Channel1Pin, BasicInstance);
pin_trait!(Channel2Pin, BasicInstance);

pub(crate) trait SealedInstance: RccPeripheral {
    fn regs() -> crate::pac::lptim::Lptim;
}
pub(crate) trait SealedBasicInstance: RccPeripheral {}

/// LPTIM basic instance trait.
#[allow(private_bounds)]
pub trait BasicInstance: PeripheralType + SealedBasicInstance + 'static {}

/// LPTIM instance trait.
#[allow(private_bounds)]
pub trait Instance: BasicInstance + SealedInstance + 'static {}

foreach_interrupt! {
    ($inst:ident, lptim, LPTIM, GLOBAL, $irq:ident) => {
        impl SealedInstance for crate::peripherals::$inst {
            fn regs() -> crate::pac::lptim::Lptim {
                crate::pac::$inst
            }
        }
        impl SealedBasicInstance for crate::peripherals::$inst {
        }
        impl BasicInstance for crate::peripherals::$inst {}
        impl Instance for crate::peripherals::$inst {}
    };
    ($inst:ident, lptim, LPTIM_BASIC, GLOBAL, $irq:ident) => {
        impl SealedBasicInstance for crate::peripherals::$inst {
        }
        impl BasicInstance for crate::peripherals::$inst {}
    };
}
