#![macro_use]

#[cfg_attr(feature = "_i2c_v2", path = "v2.rs")]
mod _version;
pub use _version::*;

pub enum Error {
    Bus,
    Arbitration,
    Nack,
}

pub(crate) mod sealed {
    use super::*;
    use crate::gpio::Pin;

    pub trait Instance {
        fn regs() -> &'static crate::pac::i2c::I2c;
    }

    pub trait SclPin<T: Instance>: Pin {
        fn af_num(&self) -> u8;
    }

    pub trait SdaPin<T: Instance>: Pin {
        fn af_num(&self) -> u8;
    }
}

pub trait Instance: sealed::Instance + 'static {}

pub trait SclPin<T: Instance>: sealed::SclPin<T> + 'static {}

pub trait SdaPin<T: Instance>: sealed::SdaPin<T> + 'static {}

macro_rules! impl_i2c {
    ($inst:ident) => {
        impl crate::i2c::sealed::Instance for peripherals::$inst {
            fn regs() -> &'static crate::pac::i2c::I2c {
                &crate::pac::$inst
            }
        }

        impl crate::i2c::Instance for peripherals::$inst {}
    };
}

macro_rules! impl_i2c_pin {
    ($inst:ident, $pin_func:ident, $pin:ident, $af:expr) => {
        impl crate::i2c::$pin_func<peripherals::$inst> for peripherals::$pin {}

        impl crate::i2c::sealed::$pin_func<peripherals::$inst> for peripherals::$pin {
            fn af_num(&self) -> u8 {
                $af
            }
        }
    };
}
