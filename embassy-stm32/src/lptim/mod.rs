//! Low-power timer (LPTIM)

pub mod pwm;
pub mod timer;

use crate::rcc::RccPeripheral;

/// Timer channel.
#[derive(Clone, Copy)]
pub enum Channel {
    /// Channel 1.
    Ch1,
    /// Channel 2.
    Ch2,
}

impl Channel {
    /// Get the channel index (0..1)
    pub fn index(&self) -> usize {
        match self {
            Channel::Ch1 => 0,
            Channel::Ch2 => 1,
        }
    }
}

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
