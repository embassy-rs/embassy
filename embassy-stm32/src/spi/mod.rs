#![macro_use]

#[cfg_attr(feature = "_spi_v1", path = "spi_v1.rs")]
#[cfg_attr(feature = "_spi_v2", path = "spi_v2.rs")]
mod spi;

pub use spi::*;

use crate::gpio::Pin;

pub enum Error {
    Framing,
    Crc,
    Overrun,
}

// TODO move upwards in the tree
pub enum ByteOrder {
    LsbFirst,
    MsbFirst,
}

#[derive(Copy, Clone, PartialOrd, PartialEq)]
enum WordSize {
    EightBit,
    SixteenBit,
}

#[non_exhaustive]
pub struct Config {
    pub mode: Mode,
    pub byte_order: ByteOrder,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mode: MODE_0,
            byte_order: ByteOrder::MsbFirst,
        }
    }
}

pub(crate) mod sealed {
    use super::*;

    pub trait Instance {
        fn regs() -> &'static crate::pac::spi::Spi;
    }

    pub trait SckPin<T: Instance>: Pin {
        fn af_num(&self) -> u8;
    }

    pub trait MosiPin<T: Instance>: Pin {
        fn af_num(&self) -> u8;
    }

    pub trait MisoPin<T: Instance>: Pin {
        fn af_num(&self) -> u8;
    }
}

pub trait Instance: sealed::Instance + 'static {}

pub trait SckPin<T: Instance>: sealed::SckPin<T> + 'static {}

pub trait MosiPin<T: Instance>: sealed::MosiPin<T> + 'static {}

pub trait MisoPin<T: Instance>: sealed::MisoPin<T> + 'static {}

macro_rules! impl_spi {
    ($inst:ident, $clk:ident) => {
        impl crate::spi::sealed::Instance for peripherals::$inst {
            fn regs() -> &'static crate::pac::spi::Spi {
                &crate::pac::$inst
            }
        }

        impl crate::spi::Instance for peripherals::$inst {}
    };
}

macro_rules! impl_spi_pin {
    ($inst:ident, $pin_func:ident, $pin:ident, $af:expr) => {
        impl crate::spi::$pin_func<peripherals::$inst> for peripherals::$pin {}

        impl crate::spi::sealed::$pin_func<peripherals::$inst> for peripherals::$pin {
            fn af_num(&self) -> u8 {
                $af
            }
        }
    };
}
