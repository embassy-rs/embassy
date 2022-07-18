//! Shared bus implementations
use core::fmt::Debug;

use embedded_hal_1::{i2c, spi};

#[cfg(feature = "nightly")]
pub mod asynch;

pub mod blocking;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum I2cDeviceError<BUS> {
    I2c(BUS),
}

impl<BUS> i2c::Error for I2cDeviceError<BUS>
where
    BUS: i2c::Error + Debug,
{
    fn kind(&self) -> i2c::ErrorKind {
        match self {
            Self::I2c(e) => e.kind(),
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum SpiDeviceError<BUS, CS> {
    Spi(BUS),
    Cs(CS),
}

impl<BUS, CS> spi::Error for SpiDeviceError<BUS, CS>
where
    BUS: spi::Error + Debug,
    CS: Debug,
{
    fn kind(&self) -> spi::ErrorKind {
        match self {
            Self::Spi(e) => e.kind(),
            Self::Cs(_) => spi::ErrorKind::Other,
        }
    }
}
