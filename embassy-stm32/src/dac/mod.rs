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
}

pub trait Instance: sealed::Instance + 'static {}

pub trait DacPin<T: Instance, const C: u8>: crate::gpio::Pin + 'static {}

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

macro_rules! impl_dac_pin {
    ($inst:ident, $pin:ident, $ch:expr) => {
        impl crate::dac::DacPin<peripherals::$inst, $ch> for crate::peripherals::$pin {}
    };
}
