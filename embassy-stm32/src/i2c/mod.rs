#![macro_use]

use embassy::interrupt::Interrupt;

#[cfg_attr(i2c_v1, path = "v1.rs")]
#[cfg_attr(i2c_v2, path = "v2.rs")]
mod _version;
use crate::{dma, peripherals};
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
    use super::dma;
    use crate::gpio::Pin;
    use crate::rcc::RccPeripheral;

    pub trait Instance: RccPeripheral {
        fn regs() -> crate::pac::i2c::I2c;

        fn state_number() -> usize;
    }

    pub trait SclPin<T: Instance>: Pin {
        fn af_num(&self) -> u8;
    }

    pub trait SdaPin<T: Instance>: Pin {
        fn af_num(&self) -> u8;
    }

    pub trait RxDma<T: Instance> {
        fn request(&self) -> dma::Request;
    }

    pub trait TxDma<T: Instance> {
        fn request(&self) -> dma::Request;
    }
}

pub trait Instance: sealed::Instance + 'static {
    type Interrupt: Interrupt;
}

pub trait SclPin<T: Instance>: sealed::SclPin<T> + 'static {}

pub trait SdaPin<T: Instance>: sealed::SdaPin<T> + 'static {}

pub trait RxDma<T: Instance>: sealed::RxDma<T> + dma::Channel {}

pub trait TxDma<T: Instance>: sealed::TxDma<T> + dma::Channel {}

macro_rules! i2c_state {
    (I2C1) => {
        0
    };
    (I2C2) => {
        1
    };
    (I2C3) => {
        2
    };
    (I2C4) => {
        3
    };
    (I2C5) => {
        4
    };
}

crate::pac::interrupts!(
    ($inst:ident, i2c, $block:ident, EV, $irq:ident) => {
        impl sealed::Instance for peripherals::$inst {
            fn regs() -> crate::pac::i2c::I2c {
                crate::pac::$inst
            }

            fn state_number() -> usize {
                i2c_state!($inst)
            }
        }

        impl Instance for peripherals::$inst {
            type Interrupt = crate::interrupt::$irq;
        }

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
    ($inst:ident, i2c, I2C, $pin:ident, SDA, $af:expr) => {
        impl_pin!($inst, $pin, SdaPin, $af);
    };

    ($inst:ident, i2c, I2C, $pin:ident, SCL, $af:expr) => {
        impl_pin!($inst, $pin, SclPin, $af);
    };
);

#[cfg(rcc_f1)]
crate::pac::peripheral_pins!(
    ($inst:ident, i2c, I2C, $pin:ident, SDA) => {
        impl_pin!($inst, $pin, SdaPin, 0);
    };

    ($inst:ident, i2c, I2C, $pin:ident, SCL) => {
        impl_pin!($inst, $pin, SclPin, 0);
    };
);

#[allow(unused)]
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
    ($peri:ident, i2c, $kind:ident, RX, $channel:tt, $request:expr) => {
        impl_dma!($peri, $channel, RxDma, $request);
    };
    ($peri:ident, i2c, $kind:ident, TX, $channel:tt, $request:expr) => {
        impl_dma!($peri, $channel, TxDma, $request);
    };
}
