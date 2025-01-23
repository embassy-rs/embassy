#![no_std]
#![allow(async_fn_in_trait)]
#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

pub mod adapter;
pub mod flash;
pub mod shared_bus;

/// Set the configuration of a peripheral driver.
///
/// This trait is intended to be implemented by peripheral drivers such as SPI
/// and I2C. It allows changing the configuration at runtime.
///
/// The exact type of the "configuration" is defined by each individual driver, since different
/// drivers support different options. Therefore it is defined as an associated type.
///
/// For example, it is used by [`SpiDeviceWithConfig`](crate::shared_bus::asynch::spi::SpiDeviceWithConfig) and
///  [`I2cDeviceWithConfig`](crate::shared_bus::asynch::i2c::I2cDeviceWithConfig) to allow different
/// devices on the same bus to use different communication settings.
pub trait SetConfig {
    /// The configuration type used by this driver.
    type Config;

    /// The error type that can occur if `set_config` fails.
    type ConfigError;

    /// Set the configuration of the driver.
    fn set_config(&mut self, config: &Self::Config) -> Result<(), Self::ConfigError>;
}

/// Get the configuration of a peripheral driver.
pub trait GetConfig {
    /// The configuration type used by this driver.
    type Config;

    /// Get the configuration of the driver.
    fn get_config(&self) -> Self::Config;
}
