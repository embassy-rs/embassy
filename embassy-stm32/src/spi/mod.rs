#![macro_use]

#[cfg_attr(spi_v1, path = "v1.rs")]
#[cfg_attr(spi_f1, path = "v1.rs")]
#[cfg_attr(spi_v2, path = "v2.rs")]
#[cfg_attr(spi_v3, path = "v3.rs")]
mod _version;
use crate::{dma, peripherals, rcc::RccPeripheral};
pub use _version::*;

use crate::gpio::Pin;

#[derive(Debug)]
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

    pub trait TxDmaChannel<T: Instance> {
        fn request(&self) -> dma::Request;
    }

    pub trait RxDmaChannel<T: Instance> {
        fn request(&self) -> dma::Request;
    }
}

pub trait Instance: sealed::Instance + RccPeripheral {}
pub trait SckPin<T: Instance>: sealed::SckPin<T> {}
pub trait MosiPin<T: Instance>: sealed::MosiPin<T> {}
pub trait MisoPin<T: Instance>: sealed::MisoPin<T> {}
pub trait TxDmaChannel<T: Instance>: sealed::TxDmaChannel<T> + dma::Channel {}
pub trait RxDmaChannel<T: Instance>: sealed::RxDmaChannel<T> + dma::Channel {}

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

#[cfg(not(rcc_f1))]
crate::pac::peripheral_pins!(
    ($inst:ident, spi, SPI, $pin:ident, SCK, $af:expr) => {
        impl_pin!($inst, $pin, SckPin, $af);
    };

    ($inst:ident, spi, SPI, $pin:ident, MOSI, $af:expr) => {
        impl_pin!($inst, $pin, MosiPin, $af);
    };

    ($inst:ident, spi, SPI, $pin:ident, MISO, $af:expr) => {
        impl_pin!($inst, $pin, MisoPin, $af);
    };
);

#[cfg(rcc_f1)]
crate::pac::peripheral_pins!(
    ($inst:ident, spi, SPI, $pin:ident, SCK) => {
        impl_pin!($inst, $pin, SckPin, 0);
    };

    ($inst:ident, spi, SPI, $pin:ident, MOSI) => {
        impl_pin!($inst, $pin, MosiPin, 0);
    };

    ($inst:ident, spi, SPI, $pin:ident, MISO) => {
        impl_pin!($inst, $pin, MisoPin, 0);
    };
);

macro_rules! impl_dma {
    ($inst:ident, {dmamux: $dmamux:ident}, $signal:ident, $request:expr) => {
        impl<T> sealed::$signal<peripherals::$inst> for T
        where
            T: crate::dma::MuxChannel<Mux = crate::dma::$dmamux>,
        {
            fn request(&self) -> dma::Request {
                $request
            }
        }

        impl<T> $signal<peripherals::$inst> for T where
            T: crate::dma::MuxChannel<Mux = crate::dma::$dmamux>
        {
        }
    };
    ($inst:ident, {channel: $channel:ident}, $signal:ident, $request:expr) => {
        impl sealed::$signal<peripherals::$inst> for peripherals::$channel {
            fn request(&self) -> dma::Request {
                $request
            }
        }

        impl $signal<peripherals::$inst> for peripherals::$channel {}
    };
}

crate::pac::peripheral_dma_channels! {
    ($peri:ident, spi, $kind:ident, RX, $channel:tt, $request:expr) => {
        impl_dma!($peri, $channel, RxDmaChannel, $request);
    };
    ($peri:ident, spi, $kind:ident, TX, $channel:tt, $request:expr) => {
        impl_dma!($peri, $channel, TxDmaChannel, $request);
    };
}
