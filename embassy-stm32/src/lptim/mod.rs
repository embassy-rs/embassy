//! Low-power timer (LPTIM)

pub mod pwm;
pub mod timer;

use crate::rcc::RccPeripheral;

/// Timer channel.
#[cfg(any(lptim_v2a, lptim_v2b))]
mod channel;
#[cfg(any(lptim_v2a, lptim_v2b))]
pub use channel::Channel;

pin_trait!(OutputPin, Instance);
pin_trait!(Channel1Pin, Instance);
pin_trait!(Channel2Pin, Instance);

pub(crate) trait SealedInstance: RccPeripheral {
    fn regs() -> crate::pac::lptim::Lptim;
}

/// LPTIM instance trait.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + 'static {}
foreach_interrupt! {
    ($inst:ident, lptim, LPTIM, UP, $irq:ident) => {
        impl SealedInstance for crate::peripherals::$inst {
            fn regs() -> crate::pac::lptim::Lptim {
                crate::pac::$inst
            }
        }

        impl Instance for crate::peripherals::$inst {

        }
    };
}
