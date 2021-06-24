#![macro_use]

macro_rules! foreach_exti_irq {
    ($action:ident) => {
        crate::pac::interrupts!(
            (EXTI0)  => { $action!(EXTI0); };
            (EXTI1)  => { $action!(EXTI1); };
            (EXTI2)  => { $action!(EXTI2); };
            (EXTI3)  => { $action!(EXTI3); };
            (EXTI4)  => { $action!(EXTI4); };
            (EXTI5)  => { $action!(EXTI5); };
            (EXTI6)  => { $action!(EXTI6); };
            (EXTI7)  => { $action!(EXTI7); };
            (EXTI8)  => { $action!(EXTI8); };
            (EXTI9)  => { $action!(EXTI9); };
            (EXTI10) => { $action!(EXTI10); };
            (EXTI11) => { $action!(EXTI11); };
            (EXTI12) => { $action!(EXTI12); };
            (EXTI13) => { $action!(EXTI13); };
            (EXTI14) => { $action!(EXTI14); };
            (EXTI15) => { $action!(EXTI15); };

            // plus the weird ones
            (EXTI0_1)   => { $action!( EXTI0_1 ); };
            (EXTI15_10) => { $action!(EXTI15_10); };
            (EXTI15_4)  => { $action!(EXTI15_4); };
            (EXTI1_0)   => { $action!(EXTI1_0); };
            (EXTI2_3)   => { $action!(EXTI2_3); };
            (EXTI2_TSC) => { $action!(EXTI2_TSC); };
            (EXTI3_2)   => { $action!(EXTI3_2); };
            (EXTI4_15)  => { $action!(EXTI4_15); };
            (EXTI9_5)   => { $action!(EXTI9_5); };
        );
    };
}

#[cfg_attr(exti_v1, path = "v1.rs")]
#[cfg_attr(exti_wb55, path = "v2.rs")]
mod _version;

#[allow(unused)]
pub use _version::*;

use crate::peripherals;
use embassy_extras::unsafe_impl_unborrow;

pub(crate) mod sealed {
    pub trait Channel {}
}

pub trait Channel: sealed::Channel + Sized {
    fn number(&self) -> usize;
    fn degrade(self) -> AnyChannel {
        AnyChannel {
            number: self.number() as u8,
        }
    }
}

pub struct AnyChannel {
    number: u8,
}
unsafe_impl_unborrow!(AnyChannel);
impl sealed::Channel for AnyChannel {}
impl Channel for AnyChannel {
    fn number(&self) -> usize {
        self.number as usize
    }
}

macro_rules! impl_exti {
    ($type:ident, $number:expr) => {
        impl sealed::Channel for peripherals::$type {}
        impl Channel for peripherals::$type {
            fn number(&self) -> usize {
                $number as usize
            }
        }
    };
}

impl_exti!(EXTI0, 0);
impl_exti!(EXTI1, 1);
impl_exti!(EXTI2, 2);
impl_exti!(EXTI3, 3);
impl_exti!(EXTI4, 4);
impl_exti!(EXTI5, 5);
impl_exti!(EXTI6, 6);
impl_exti!(EXTI7, 7);
impl_exti!(EXTI8, 8);
impl_exti!(EXTI9, 9);
impl_exti!(EXTI10, 10);
impl_exti!(EXTI11, 11);
impl_exti!(EXTI12, 12);
impl_exti!(EXTI13, 13);
impl_exti!(EXTI14, 14);
impl_exti!(EXTI15, 15);

macro_rules! enable_irq {
    ($e:ident) => {
        crate::interrupt::$e::steal().enable();
    };
}

/// safety: must be called only once
pub(crate) unsafe fn init() {
    use embassy::interrupt::Interrupt;
    use embassy::interrupt::InterruptExt;

    foreach_exti_irq!(enable_irq);
}
