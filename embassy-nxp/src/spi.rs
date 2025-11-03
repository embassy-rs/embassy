//! Serial Peripheral Interface (SPI) driver.
#![macro_use]

#[cfg_attr(lpc55, path = "./spi/lpc55.rs")]
mod inner;
pub use inner::*;
