#![macro_use]

#[cfg_attr(dac_v2, path = "v2.rs")]
mod _version;
use crate::gpio::NoPin;
pub use _version::*;

pub(crate) mod sealed {
    use super::*;
    use crate::gpio::{OptionalPin, Pin};

    pub trait Instance {
        fn regs() -> &'static crate::pac::dac::Dac;
    }

    pub trait DacPin<T: Instance, const C: u8>: OptionalPin {}
}

pub trait Instance: sealed::Instance + 'static {}

pub trait DacPin<T: Instance, const C: u8>: sealed::DacPin<T, C> + 'static {}

impl<T: Instance, const C: u8> DacPin<T, C> for NoPin {}
impl<T: Instance, const C: u8> sealed::DacPin<T, C> for NoPin {}

macro_rules! impl_dac {
    ($inst:ident) => {
        impl crate::dac::sealed::Instance for peripherals::$inst {
            fn regs() -> &'static crate::pac::dac::Dac {
                &crate::pac::$inst
            }
        }

        impl crate::dac::Instance for peripherals::$inst {}
    };
}

macro_rules! impl_dac_pin {
    ($inst:ident, $channel:expr, $pin:ident ) => {
        impl crate::dac::DacPin<peripherals::$inst, $channel> for peripherals::$pin {}

        impl crate::dac::sealed::DacPin<peripherals::$inst, $channel> for peripherals::$pin {
            //fn af_num(&self) -> u8 {
            //$af
            //}
        }
    };
}
