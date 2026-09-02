//! Inter Integrated Circuit (I2C) driver

#![macro_use]

#[cfg_attr(lpc55, path = "./i2c/lpc55.rs")]
mod inner;
pub use inner::*;