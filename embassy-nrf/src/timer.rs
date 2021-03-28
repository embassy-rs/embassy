use embassy::interrupt::Interrupt;

use crate::{interrupt, pac, peripherals};

mod sealed {
    use super::*;

    pub trait Instance {
        fn regs(&self) -> &pac::timer0::RegisterBlock;
    }
    pub trait ExtendedInstance {}
}

pub trait Instance: sealed::Instance + 'static {
    type Interrupt: Interrupt;
}
pub trait ExtendedInstance: Instance + sealed::ExtendedInstance {}

macro_rules! impl_instance {
    ($type:ident, $irq:ident) => {
        impl sealed::Instance for peripherals::$type {
            fn regs(&self) -> &pac::timer0::RegisterBlock {
                unsafe { &*(pac::$type::ptr() as *const pac::timer0::RegisterBlock) }
            }
        }
        impl Instance for peripherals::$type {
            type Interrupt = interrupt::$irq;
        }
    };
    ($type:ident, $irq:ident, extended) => {
        impl_instance!($type, $irq);
        impl sealed::ExtendedInstance for peripherals::$type {}
        impl ExtendedInstance for peripherals::$type {}
    };
}

impl_instance!(TIMER0, TIMER0);
impl_instance!(TIMER1, TIMER1);
impl_instance!(TIMER2, TIMER2);
#[cfg(any(feature = "52832", feature = "52833", feature = "52840"))]
impl_instance!(TIMER3, TIMER3, extended);
#[cfg(any(feature = "52832", feature = "52833", feature = "52840"))]
impl_instance!(TIMER4, TIMER4, extended);
