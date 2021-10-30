#![macro_use]

#[cfg_attr(dac_v1, path = "v1.rs")]
#[cfg_attr(dac_v2, path = "v2.rs")]
mod _version;
use crate::gpio::NoPin;
use crate::peripherals;
pub use _version::*;

pub(crate) mod sealed {
    use crate::gpio::OptionalPin;

    pub trait Instance {
        fn regs() -> &'static crate::pac::dac::Dac;
    }

    pub trait DacPin<T: Instance, const C: u8>: OptionalPin {}
}

pub trait Instance: sealed::Instance + 'static {}

pub trait DacPin<T: Instance, const C: u8>: sealed::DacPin<T, C> + 'static {}

impl<T: Instance, const C: u8> DacPin<T, C> for NoPin {}
impl<T: Instance, const C: u8> sealed::DacPin<T, C> for NoPin {}

crate::pac::peripherals!(
    (dac, $inst:ident) => {
        impl crate::dac::sealed::Instance for peripherals::$inst {
            fn regs() -> &'static crate::pac::dac::Dac {
                &crate::pac::$inst
            }
        }

        impl crate::dac::Instance for peripherals::$inst {}
    };
);

crate::pac::peripheral_pins!(
    ($inst:ident, dac, DAC, $pin:ident, OUT1) => {
        impl DacPin<peripherals::$inst, 1> for peripherals::$pin {}

        impl sealed::DacPin<peripherals::$inst, 1> for peripherals::$pin {
        }

    };

    ($inst:ident, dac, DAC, $pin:ident, OUT2) => {
        impl DacPin<peripherals::$inst, 2> for peripherals::$pin {}

        impl sealed::DacPin<peripherals::$inst, 2> for peripherals::$pin {
        }
    };
);
