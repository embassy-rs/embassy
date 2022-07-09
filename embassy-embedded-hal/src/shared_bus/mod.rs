//! Shared bus implementations
#[cfg(feature = "nightly")]
pub mod asynch;

pub mod blocking;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum I2cBusDeviceError<BUS> {
    I2c(BUS),
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum SpiBusDeviceError<BUS, CS> {
    Spi(BUS),
    Cs(CS),
}
