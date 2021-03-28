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

macro_rules! make_impl {
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
        make_impl!($type, $irq);
        impl sealed::ExtendedInstance for peripherals::$type {}
        impl ExtendedInstance for peripherals::$type {}
    };
}

make_impl!(TIMER0, TIMER0);
make_impl!(TIMER1, TIMER1);
make_impl!(TIMER2, TIMER2);
#[cfg(any(feature = "52832", feature = "52833", feature = "52840"))]
make_impl!(TIMER3, TIMER3, extended);
#[cfg(any(feature = "52832", feature = "52833", feature = "52840"))]
make_impl!(TIMER4, TIMER4, extended);
