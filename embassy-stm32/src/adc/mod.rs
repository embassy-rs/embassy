#![macro_use]

#[cfg(not(adc_f3))]
#[cfg_attr(adc_f1, path = "f1.rs")]
#[cfg_attr(adc_v1, path = "v1.rs")]
#[cfg_attr(adc_v2, path = "v2.rs")]
#[cfg_attr(any(adc_v3, adc_g0), path = "v3.rs")]
#[cfg_attr(adc_v4, path = "v4.rs")]
mod _version;

#[cfg(not(any(adc_f1, adc_f3)))]
mod resolution;
mod sample_time;

#[cfg(not(adc_f3))]
#[allow(unused)]
pub use _version::*;
#[cfg(not(any(adc_f1, adc_f3)))]
pub use resolution::Resolution;
#[cfg(not(adc_f3))]
pub use sample_time::SampleTime;

use crate::peripherals;

pub struct Adc<'d, T: Instance> {
    #[allow(unused)]
    adc: crate::PeripheralRef<'d, T>,
    #[cfg(not(adc_f3))]
    sample_time: SampleTime,
}

pub(crate) mod sealed {
    pub trait Instance {
        fn regs() -> crate::pac::adc::Adc;
        #[cfg(not(any(adc_f1, adc_v1, adc_f3)))]
        fn common_regs() -> crate::pac::adccommon::AdcCommon;
    }

    pub trait AdcPin<T: Instance> {
        fn channel(&self) -> u8;
    }

    pub trait InternalChannel<T> {
        fn channel(&self) -> u8;
    }
}

#[cfg(not(any(adc_f1, adc_v1, adc_v2, adc_v4)))]
pub trait Instance: sealed::Instance + crate::Peripheral<P = Self> {}
#[cfg(any(adc_f1, adc_v1, adc_v2, adc_v4))]
pub trait Instance: sealed::Instance + crate::Peripheral<P = Self> + crate::rcc::RccPeripheral {}

pub trait AdcPin<T: Instance>: sealed::AdcPin<T> {}
pub trait InternalChannel<T>: sealed::InternalChannel<T> {}

#[cfg(not(stm32h7))]
foreach_peripheral!(
    (adc, $inst:ident) => {
        impl crate::adc::sealed::Instance for peripherals::$inst {
            fn regs() -> crate::pac::adc::Adc {
                crate::pac::$inst
            }
            #[cfg(not(any(adc_f1, adc_v1, adc_f3)))]
            fn common_regs() -> crate::pac::adccommon::AdcCommon {
                foreach_peripheral!{
                    (adccommon, $common_inst:ident) => {
                        return crate::pac::$common_inst
                    };
                }
            }
        }

        impl crate::adc::Instance for peripherals::$inst {}
    };
);

#[cfg(stm32h7)]
foreach_peripheral!(
    (adc, ADC3) => {
        impl crate::adc::sealed::Instance for peripherals::ADC3 {
            fn regs() -> crate::pac::adc::Adc {
                crate::pac::ADC3
            }
            #[cfg(all(not(adc_f1), not(adc_v1)))]
            fn common_regs() -> crate::pac::adccommon::AdcCommon {
                foreach_peripheral!{
                    (adccommon, ADC3_COMMON) => {
                        return crate::pac::ADC3_COMMON
                    };
                }
            }
        }

        impl crate::adc::Instance for peripherals::ADC3 {}
    };
    (adc, $inst:ident) => {
        impl crate::adc::sealed::Instance for peripherals::$inst {
            fn regs() -> crate::pac::adc::Adc {
                crate::pac::$inst
            }
            #[cfg(all(not(adc_f1), not(adc_v1)))]
            fn common_regs() -> crate::pac::adccommon::AdcCommon {
                foreach_peripheral!{
                    (adccommon, ADC_COMMON) => {
                        return crate::pac::ADC_COMMON
                    };
                }
            }
        }

        impl crate::adc::Instance for peripherals::$inst {}
    };
);

macro_rules! impl_adc_pin {
    ($inst:ident, $pin:ident, $ch:expr) => {
        impl crate::adc::AdcPin<peripherals::$inst> for crate::peripherals::$pin {}

        impl crate::adc::sealed::AdcPin<peripherals::$inst> for crate::peripherals::$pin {
            fn channel(&self) -> u8 {
                $ch
            }
        }
    };
}
