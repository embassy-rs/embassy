#![macro_use]

#[cfg_attr(adc_v3, path = "v3.rs")]
#[cfg_attr(adc_v2, path = "v2.rs")]
#[cfg_attr(adc_g0, path = "v3.rs")]
#[cfg_attr(adc_f1, path = "f1.rs")]
mod _version;

#[allow(unused)]
pub use _version::*;

use crate::peripherals;

pub(crate) mod sealed {
    pub trait Instance {
        fn regs() -> &'static crate::pac::adc::Adc;
        #[cfg(not(adc_f1))]
        fn common_regs() -> &'static crate::pac::adccommon::AdcCommon;
    }

    #[cfg(not(adc_f1))]
    pub trait Common {
        fn regs() -> &'static crate::pac::adccommon::AdcCommon;
    }

    pub trait AdcPin<T: Instance> {
        fn channel(&self) -> u8;
    }
}

#[cfg(not(adc_f1))]
pub trait Instance: sealed::Instance + 'static {}
#[cfg(adc_f1)]
pub trait Instance: sealed::Instance + crate::rcc::RccPeripheral + 'static {}
#[cfg(not(adc_f1))]
pub trait Common: sealed::Common + 'static {}
pub trait AdcPin<T: Instance>: sealed::AdcPin<T> {}

crate::pac::peripherals!(
    (adc, $inst:ident) => {
        impl crate::adc::sealed::Instance for peripherals::$inst {
            fn regs() -> &'static crate::pac::adc::Adc {
                &crate::pac::$inst
            }
            #[cfg(not(adc_f1))]
            fn common_regs() -> &'static crate::pac::adccommon::AdcCommon {
                crate::pac::peripherals!{
                    (adccommon, $common_inst:ident) => {
                        return &crate::pac::$common_inst
                    };
                }
            }
        }

        impl crate::adc::Instance for peripherals::$inst {}
    };
);

#[cfg(not(adc_f1))]
crate::pac::peripherals!(
    (adccommon, $inst:ident) => {
        impl sealed::Common for peripherals::$inst {
            fn regs() -> &'static crate::pac::adccommon::AdcCommon {
                &crate::pac::$inst
            }
        }

        impl crate::adc::Common for peripherals::$inst {}
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
