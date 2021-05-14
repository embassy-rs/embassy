#![macro_use]

use embassy::interrupt::Interrupt;
use embassy::util::Unborrow;

use crate::pac;

pub(crate) mod sealed {
    use super::*;

    pub trait Instance {
        fn regs(&self) -> &pac::timer0::RegisterBlock;
    }
    pub trait ExtendedInstance {}
}

pub trait Instance: Unborrow<Target = Self> + sealed::Instance + 'static {
    type Interrupt: Interrupt;
}
pub trait ExtendedInstance: Instance + sealed::ExtendedInstance {}

macro_rules! impl_timer {
    ($type:ident, $pac_type:ident, $irq:ident) => {
        impl crate::timer::sealed::Instance for peripherals::$type {
            fn regs(&self) -> &pac::timer0::RegisterBlock {
                unsafe { &*(pac::$pac_type::ptr() as *const pac::timer0::RegisterBlock) }
            }
        }
        impl crate::timer::Instance for peripherals::$type {
            type Interrupt = crate::interrupt::$irq;
        }
    };
    ($type:ident, $pac_type:ident, $irq:ident, extended) => {
        impl_timer!($type, $pac_type, $irq);
        impl crate::timer::sealed::ExtendedInstance for peripherals::$type {}
        impl crate::timer::ExtendedInstance for peripherals::$type {}
    };
}
