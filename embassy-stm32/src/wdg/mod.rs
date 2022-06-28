use core::marker::PhantomData;

use embassy_hal_common::{unborrow, Unborrow};
use stm32_metapac::iwdg::vals::Key;
pub use stm32_metapac::iwdg::vals::Pr as Prescaler;

pub struct IndependentWatchdog<'d, T: Instance> {
    wdg: PhantomData<&'d mut T>,
}

impl<'d, T: Instance> IndependentWatchdog<'d, T> {
    pub fn new(_instance: impl Unborrow<Target = T> + 'd, presc: Prescaler) -> Self {
        unborrow!(_instance);

        let wdg = T::regs();
        unsafe {
            wdg.kr().write(|w| w.set_key(Key::ENABLE));
            wdg.pr().write(|w| w.set_pr(presc));
        }

        IndependentWatchdog {
            wdg: PhantomData::default(),
        }
    }

    pub unsafe fn unleash(&mut self) {
        T::regs().kr().write(|w| w.set_key(Key::START));
    }

    pub unsafe fn pet(&mut self) {
        T::regs().kr().write(|w| w.set_key(Key::RESET));
    }
}

mod sealed {
    pub trait Instance {
        fn regs() -> crate::pac::iwdg::Iwdg;
    }
}

pub trait Instance: sealed::Instance {}

impl sealed::Instance for crate::peripherals::IWDG {
    fn regs() -> crate::pac::iwdg::Iwdg {
        crate::pac::IWDG
    }
}

impl Instance for crate::peripherals::IWDG {}
