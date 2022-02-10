#![macro_use]

use embassy::interrupt::Interrupt;

#[cfg_attr(i2c_v1, path = "v1.rs")]
#[cfg_attr(i2c_v2, path = "v2.rs")]
mod _version;
use crate::peripherals;
pub use _version::*;

#[derive(Debug)]
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
    pub trait Instance: crate::rcc::RccPeripheral {
        fn regs() -> crate::pac::i2c::I2c;
        fn state_number() -> usize;
    }
}

pub trait Instance: sealed::Instance + 'static {
    type Interrupt: Interrupt;
}

pin_trait!(SclPin, Instance);
pin_trait!(SdaPin, Instance);
dma_trait!(RxDma, Instance);
dma_trait!(TxDma, Instance);

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

#[cfg(not(rcc_f1))]
crate::pac::peripheral_pins!(
    ($inst:ident, i2c, I2C, $pin:ident, SDA, $af:expr) => {
        pin_trait_impl!(SdaPin, $inst, $pin, $af);
    };
    ($inst:ident, i2c, I2C, $pin:ident, SCL, $af:expr) => {
        pin_trait_impl!(SclPin, $inst, $pin, $af);
    };
);

#[cfg(rcc_f1)]
crate::pac::peripheral_pins!(
    ($inst:ident, i2c, I2C, $pin:ident, SDA) => {
        pin_trait_impl!(SdaPin, $inst, $pin, 0);
    };
    ($inst:ident, i2c, I2C, $pin:ident, SCL) => {
        pin_trait_impl!(SdaPin, $inst, $pin, 0);
    };
);

crate::pac::peripheral_dma_channels! {
    ($peri:ident, i2c, $kind:ident, RX, $channel:tt, $request:expr) => {
        dma_trait_impl!(RxDma, $peri, $channel, $request);
    };
    ($peri:ident, i2c, $kind:ident, TX, $channel:tt, $request:expr) => {
        dma_trait_impl!(TxDma, $peri, $channel, $request);
    };
}
