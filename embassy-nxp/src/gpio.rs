//! General purpose input/output (GPIO) driver.
#![macro_use]

#[cfg_attr(feature = "lpc55-core0", path = "./gpio/lpc55.rs")]
#[cfg_attr(rt1xxx, path = "./gpio/rt1xxx.rs")]
mod inner;
pub use inner::*;
