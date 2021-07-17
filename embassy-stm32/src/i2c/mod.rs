#![macro_use]

#[cfg_attr(i2c_v1, path = "v1.rs")]
#[cfg_attr(i2c_v2, path = "v2.rs")]
mod _version;
use crate::peripherals;
pub use _version::*;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    Bus,
    Arbitration,
    Nack,
    Timeout,
    Crc,
    Overrun,
    ZeroLengthTransfer,
}

pub(crate) mod sealed {
    use crate::gpio::Pin;
    use crate::rcc::RccPeripheral;

    pub trait Instance: RccPeripheral {
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

crate::pac::peripherals!(
    (i2c, $inst:ident) => {
        impl sealed::Instance for peripherals::$inst {
            fn regs() -> &'static crate::pac::i2c::I2c {
                &crate::pac::$inst
            }
        }

        impl Instance for peripherals::$inst {}

    };
);

macro_rules! impl_pin {
    ($inst:ident, $pin:ident, $signal:ident, $af:expr) => {
        impl $signal<peripherals::$inst> for peripherals::$pin {}

        impl sealed::$signal<peripherals::$inst> for peripherals::$pin {
            fn af_num(&self) -> u8 {
                $af
            }
        }
    };
}

crate::pac::peripheral_pins!(
    ($inst:ident, i2c, I2C, $pin:ident, SDA, $af:expr) => {
        impl_pin!($inst, $pin, SdaPin, $af);
    };

    ($inst:ident, i2c, I2C, $pin:ident, SCL, $af:expr) => {
        impl_pin!($inst, $pin, SclPin, $af);
    };
);
