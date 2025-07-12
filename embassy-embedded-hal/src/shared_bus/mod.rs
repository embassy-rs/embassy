//! Shared bus implementations
use core::fmt::Debug;

use embedded_hal_1::{i2c, spi};

pub mod asynch;
pub mod blocking;

/// Error returned by I2C device implementations in this crate.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum I2cDeviceError<BUS> {
    /// An operation on the inner I2C bus failed.
    I2c(BUS),
    /// Configuration of the inner I2C bus failed.
    Config,
}

impl<BUS> i2c::Error for I2cDeviceError<BUS>
where
    BUS: i2c::Error + Debug,
{
    fn kind(&self) -> i2c::ErrorKind {
        match self {
            Self::I2c(e) => e.kind(),
            Self::Config => i2c::ErrorKind::Other,
        }
    }
}

/// Error returned by SPI device implementations in this crate.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum SpiDeviceError<BUS, CS> {
    /// An operation on the inner SPI bus failed.
    Spi(BUS),
    /// Setting the value of the Chip Select (CS) pin failed.
    Cs(CS),
    /// Delay operations are not supported when the `time` Cargo feature is not enabled.
    DelayNotSupported,
    /// The SPI bus could not be configured.
    Config,
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
            Self::DelayNotSupported => spi::ErrorKind::Other,
            Self::Config => spi::ErrorKind::Other,
        }
    }
}
