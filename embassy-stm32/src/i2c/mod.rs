#![macro_use]

use embassy_time::Duration;

use crate::interrupt::Interrupt;

#[cfg_attr(i2c_v1, path = "v1.rs")]
#[cfg_attr(i2c_v2, path = "v2.rs")]
mod _version;
pub use _version::*;

use crate::peripherals;

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

#[non_exhaustive]
#[derive(Copy, Clone)]
pub struct Config {
    /// Enables SDA pullup, if supported by hardware.
    pub sda_pullup: bool,
    /// Enabled SCL pullup, if supported by hardware.
    pub scl_pullup: bool,
    /// If provided, I2C methods will fail with [Error::Timeout] after a given amount of time instead of spinning infinitely.
    ///
    /// It is possible to provide different timeout for each operation with `xxx_timeout()` methods.
    pub timeout: Option<Duration>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            sda_pullup: false,
            scl_pullup: false,
            timeout: None,
        }
    }
}

pub(crate) mod sealed {
    use super::*;
    pub trait Instance: crate::rcc::RccPeripheral {
        fn regs() -> crate::pac::i2c::I2c;
        fn state() -> &'static State;
    }
}

pub trait Instance: sealed::Instance + 'static {
    type Interrupt: Interrupt;
}

pin_trait!(SclPin, Instance);
pin_trait!(SdaPin, Instance);
dma_trait!(RxDma, Instance);
dma_trait!(TxDma, Instance);

foreach_interrupt!(
    ($inst:ident, i2c, $block:ident, EV, $irq:ident) => {
        impl sealed::Instance for peripherals::$inst {
            fn regs() -> crate::pac::i2c::I2c {
                crate::pac::$inst
            }

            fn state() -> &'static State {
                static STATE: State = State::new();
                &STATE
            }
        }

        impl Instance for peripherals::$inst {
            type Interrupt = crate::interrupt::$irq;
        }
    };
);
