#![macro_use]

#[cfg_attr(dac_v1, path = "v1.rs")]
#[cfg_attr(dac_v2, path = "v2.rs")]
mod _version;
use crate::peripherals;
pub use _version::*;

pub(crate) mod sealed {
    pub trait Instance {
        fn regs() -> &'static crate::pac::dac::Dac;
    }

    pub trait DacPin<T: Instance, const C: u8>: crate::gpio::Pin {}
}

pub trait Instance: sealed::Instance + 'static {}

pub trait DacPin<T: Instance, const C: u8>: sealed::DacPin<T, C> + 'static {}

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
