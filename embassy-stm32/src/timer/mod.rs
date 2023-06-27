#[cfg(hrtim_v1)]
use core::ops;

use stm32_metapac::timer::vals;

use crate::interrupt;
use crate::rcc::sealed::RccPeripheral as __RccPeri;
use crate::rcc::RccPeripheral;
use crate::time::Hertz;

#[cfg(feature = "unstable-pac")]
pub mod low_level {
    pub use super::sealed::*;
}

pub(crate) mod sealed {
    use super::*;
    pub trait Basic16bitInstance: RccPeripheral {
        type Interrupt: interrupt::typelevel::Interrupt;

        fn regs() -> crate::pac::timer::TimBasic;

        fn start(&mut self);

        fn stop(&mut self);

        fn reset(&mut self);

        fn set_frequency(&mut self, frequency: Hertz);

        fn clear_update_interrupt(&mut self) -> bool;

        fn enable_update_interrupt(&mut self, enable: bool);
    }

    pub trait GeneralPurpose16bitInstance: Basic16bitInstance {
        fn regs_gp16() -> crate::pac::timer::TimGp16;
    }

    pub trait GeneralPurpose32bitInstance: GeneralPurpose16bitInstance {
        fn regs_gp32() -> crate::pac::timer::TimGp32;

        fn set_frequency(&mut self, frequency: Hertz);
    }

    pub trait AdvancedControlInstance: GeneralPurpose16bitInstance {
        fn regs_advanced() -> crate::pac::timer::TimAdv;
    }

    #[cfg(hrtim_v1)]
    pub trait HighResolutionControlInstance: RccPeripheral {
        type Interrupt: interrupt::typelevel::Interrupt;

        fn regs() -> crate::pac::hrtim::Hrtim;
    }
}

pub trait GeneralPurpose16bitInstance: sealed::GeneralPurpose16bitInstance + 'static {}

pub trait GeneralPurpose32bitInstance: sealed::GeneralPurpose32bitInstance + 'static {}

pub trait AdvancedControlInstance: sealed::AdvancedControlInstance + 'static {}

#[cfg(hrtim_v1)]
pub trait HighResolutionControlInstance: sealed::HighResolutionControlInstance + 'static {}

pub trait Basic16bitInstance: sealed::Basic16bitInstance + 'static {}

#[allow(unused)]
macro_rules! impl_basic_16bit_timer {
    ($inst:ident, $irq:ident) => {
        impl sealed::Basic16bitInstance for crate::peripherals::$inst {
            type Interrupt = crate::interrupt::typelevel::$irq;

            fn regs() -> crate::pac::timer::TimBasic {
                unsafe { crate::pac::timer::TimBasic::from_ptr(crate::pac::$inst.as_ptr()) }
            }

            fn start(&mut self) {
                Self::regs().cr1().modify(|r| r.set_cen(true));
            }

            fn stop(&mut self) {
                Self::regs().cr1().modify(|r| r.set_cen(false));
            }

            fn reset(&mut self) {
                Self::regs().cnt().write(|r| r.set_cnt(0));
            }

            fn set_frequency(&mut self, frequency: Hertz) {
                use core::convert::TryInto;
                let f = frequency.0;
                let timer_f = Self::frequency().0;
                let pclk_ticks_per_timer_period = timer_f / f;
                let psc: u16 = unwrap!(((pclk_ticks_per_timer_period - 1) / (1 << 16)).try_into());
                let arr: u16 = unwrap!((pclk_ticks_per_timer_period / (u32::from(psc) + 1)).try_into());

                let regs = Self::regs();
                regs.psc().write(|r| r.set_psc(psc));
                regs.arr().write(|r| r.set_arr(arr));

                regs.cr1().modify(|r| r.set_urs(vals::Urs::COUNTERONLY));
                regs.egr().write(|r| r.set_ug(true));
                regs.cr1().modify(|r| r.set_urs(vals::Urs::ANYEVENT));
            }

            fn clear_update_interrupt(&mut self) -> bool {
                let regs = Self::regs();
                let sr = regs.sr().read();
                if sr.uif() {
                    regs.sr().modify(|r| {
                        r.set_uif(false);
                    });
                    true
                } else {
                    false
                }
            }

            fn enable_update_interrupt(&mut self, enable: bool) {
                Self::regs().dier().write(|r| r.set_uie(enable));
            }
        }
    };
}

#[allow(unused)]
macro_rules! impl_32bit_timer {
    ($inst:ident) => {
        impl sealed::GeneralPurpose32bitInstance for crate::peripherals::$inst {
            fn regs_gp32() -> crate::pac::timer::TimGp32 {
                crate::pac::$inst
            }

            fn set_frequency(&mut self, frequency: Hertz) {
                use core::convert::TryInto;
                let f = frequency.0;
                let timer_f = Self::frequency().0;
                let pclk_ticks_per_timer_period = (timer_f / f) as u64;
                let psc: u16 = unwrap!(((pclk_ticks_per_timer_period - 1) / (1 << 32)).try_into());
                let arr: u32 = unwrap!(((pclk_ticks_per_timer_period / (psc as u64 + 1)).try_into()));

                let regs = Self::regs_gp32();
                regs.psc().write(|r| r.set_psc(psc));
                regs.arr().write(|r| r.set_arr(arr));

                regs.cr1().modify(|r| r.set_urs(vals::Urs::COUNTERONLY));
                regs.egr().write(|r| r.set_ug(true));
                regs.cr1().modify(|r| r.set_urs(vals::Urs::ANYEVENT));
            }
        }
    };
}

foreach_interrupt! {
    ($inst:ident, timer, TIM_BASIC, UP, $irq:ident) => {
        impl_basic_16bit_timer!($inst, $irq);

        impl Basic16bitInstance for crate::peripherals::$inst {
        }
    };
    ($inst:ident, timer, TIM_GP16, UP, $irq:ident) => {
        impl_basic_16bit_timer!($inst, $irq);

        impl Basic16bitInstance for crate::peripherals::$inst {
        }

        impl sealed::GeneralPurpose16bitInstance for crate::peripherals::$inst {
            fn regs_gp16() -> crate::pac::timer::TimGp16 {
                crate::pac::$inst
            }
        }

        impl GeneralPurpose16bitInstance for crate::peripherals::$inst {
        }
    };

    ($inst:ident, timer, TIM_GP32, UP, $irq:ident) => {
        impl_basic_16bit_timer!($inst, $irq);

        impl Basic16bitInstance for crate::peripherals::$inst {
        }

        impl sealed::GeneralPurpose16bitInstance for crate::peripherals::$inst {
            fn regs_gp16() -> crate::pac::timer::TimGp16 {
                unsafe { crate::pac::timer::TimGp16::from_ptr(crate::pac::$inst.as_ptr()) }
            }
        }

        impl GeneralPurpose16bitInstance for crate::peripherals::$inst {
        }

        impl_32bit_timer!($inst);

        impl GeneralPurpose32bitInstance for crate::peripherals::$inst {
        }
    };

    ($inst:ident, timer, TIM_ADV, UP, $irq:ident) => {
        impl_basic_16bit_timer!($inst, $irq);

        impl Basic16bitInstance for crate::peripherals::$inst {
        }

        impl sealed::GeneralPurpose16bitInstance for crate::peripherals::$inst {
            fn regs_gp16() -> crate::pac::timer::TimGp16 {
                unsafe { crate::pac::timer::TimGp16::from_ptr(crate::pac::$inst.as_ptr()) }
            }
        }

        impl GeneralPurpose16bitInstance for crate::peripherals::$inst {
        }

        impl sealed::AdvancedControlInstance for crate::peripherals::$inst {
            fn regs_advanced() -> crate::pac::timer::TimAdv {
                crate::pac::$inst
            }
        }

        impl AdvancedControlInstance for crate::peripherals::$inst {
        }
    };

    ($inst:ident, hrtim, HRTIM, MASTER, $irq:ident) => {
        impl sealed::HighResolutionControlInstance for crate::peripherals::$inst {
            type Interrupt = crate::interrupt::typelevel::$irq;

            fn regs() -> crate::pac::hrtim::Hrtim {
                crate::pac::$inst
            }
        }

        impl HighResolutionControlInstance for crate::peripherals::$inst {
        }
    };
}
