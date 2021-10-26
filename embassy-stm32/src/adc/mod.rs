#![macro_use]

#[cfg_attr(adc_v3, path = "v3.rs")]
#[cfg_attr(adc_v2, path = "v2.rs")]
#[cfg_attr(adc_g0, path = "v3.rs")]
mod _version;

#[allow(unused)]
pub use _version::*;

use crate::peripherals;

pub(crate) mod sealed {
    pub trait Instance {
        fn regs() -> &'static crate::pac::adc::Adc;
        fn common_regs() -> &'static crate::pac::adccommon::AdcCommon;
    }

    pub trait Common {
        fn regs() -> &'static crate::pac::adccommon::AdcCommon;
    }

    pub trait AdcPin<T: Instance> {
        fn channel(&self) -> u8;
    }
}

pub trait Instance: sealed::Instance + 'static {}
pub trait Common: sealed::Common + 'static {}
pub trait AdcPin<T: Instance>: sealed::AdcPin<T> {}

crate::pac::peripherals!(
    (adc, $inst:ident) => {
        impl crate::adc::sealed::Instance for peripherals::$inst {
            fn regs() -> &'static crate::pac::adc::Adc {
                &crate::pac::$inst
            }
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

macro_rules! impl_pin {
    ($inst:ident, $pin:ident, $ch:expr) => {
        impl AdcPin<peripherals::$inst> for peripherals::$pin {}

        impl sealed::AdcPin<peripherals::$inst> for peripherals::$pin {
            fn channel(&self) -> u8 {
                $ch
            }
        }
    };
}

crate::pac::peripheral_pins!(
    ($inst:ident, adc, ADC, $pin:ident, IN0) => {
        impl_pin!($inst, $pin, 0);
    };
    ($inst:ident, adc, ADC, $pin:ident, IN1) => {
        impl_pin!($inst, $pin, 1);
    };
    ($inst:ident, adc, ADC, $pin:ident, IN2) => {
        impl_pin!($inst, $pin, 2);
    };
    ($inst:ident, adc, ADC, $pin:ident, IN3) => {
        impl_pin!($inst, $pin, 3);
    };
    ($inst:ident, adc, ADC, $pin:ident, IN4) => {
        impl_pin!($inst, $pin, 4);
    };
    ($inst:ident, adc, ADC, $pin:ident, IN5) => {
        impl_pin!($inst, $pin, 5);
    };
    ($inst:ident, adc, ADC, $pin:ident, IN6) => {
        impl_pin!($inst, $pin, 6);
    };
    ($inst:ident, adc, ADC, $pin:ident, IN7) => {
        impl_pin!($inst, $pin, 7);
    };
    ($inst:ident, adc, ADC, $pin:ident, IN8) => {
        impl_pin!($inst, $pin, 8);
    };
    ($inst:ident, adc, ADC, $pin:ident, IN9) => {
        impl_pin!($inst, $pin, 9);
    };
    ($inst:ident, adc, ADC, $pin:ident, IN10) => {
        impl_pin!($inst, $pin, 10);
    };
    ($inst:ident, adc, ADC, $pin:ident, IN11) => {
        impl_pin!($inst, $pin, 11);
    };
    ($inst:ident, adc, ADC, $pin:ident, IN12) => {
        impl_pin!($inst, $pin, 12);
    };
    ($inst:ident, adc, ADC, $pin:ident, IN13) => {
        impl_pin!($inst, $pin, 13);
    };
    ($inst:ident, adc, ADC, $pin:ident, IN14) => {
        impl_pin!($inst, $pin, 14);
    };
    ($inst:ident, adc, ADC, $pin:ident, IN15) => {
        impl_pin!($inst, $pin, 15);
    };
    ($inst:ident, adc, ADC, $pin:ident, IN16) => {
        impl_pin!($inst, $pin, 16);
    };
);
