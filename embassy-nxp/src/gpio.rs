//! General purpose input/output (GPIO) driver.

#[cfg_attr(feature = "lpc55", path = "./gpio/lpc55.rs")]
mod inner;
pub use inner::*;
