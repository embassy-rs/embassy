#![macro_use]

#[cfg_attr(spi_v1, path = "v1.rs")]
#[cfg_attr(spi_v2, path = "v2.rs")]
#[cfg_attr(spi_v3, path = "v3.rs")]
mod _version;
use crate::peripherals;
pub use _version::*;

use crate::gpio::Pin;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    Framing,
    Crc,
    ModeFault,
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

crate::pac::peripherals!(
    (spi, $inst:ident) => {
        impl sealed::Instance for peripherals::$inst {
            fn regs() -> &'static crate::pac::spi::Spi {
                &crate::pac::$inst
            }
        }

        impl Instance for peripherals::$inst {}
    };
);

crate::pac::peripheral_pins!(
    ($inst:ident, spi, SPI, $pin:ident, SCK, $af:expr) => {
        impl SckPin<peripherals::$inst> for peripherals::$pin {}

        impl sealed::SckPin<peripherals::$inst> for peripherals::$pin {
            fn af_num(&self) -> u8 {
                $af
            }
        }
    };

    ($inst:ident, spi, SPI, $pin:ident, MOSI, $af:expr) => {
        impl MosiPin<peripherals::$inst> for peripherals::$pin {}

        impl sealed::MosiPin<peripherals::$inst> for peripherals::$pin {
            fn af_num(&self) -> u8 {
                $af
            }
        }
    };

    ($inst:ident, spi, SPI, $pin:ident, MISO, $af:expr) => {
        impl MisoPin<peripherals::$inst> for peripherals::$pin {}

        impl sealed::MisoPin<peripherals::$inst> for peripherals::$pin {
            fn af_num(&self) -> u8 {
                $af
            }
        }
    };
);
