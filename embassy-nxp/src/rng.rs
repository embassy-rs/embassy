#![macro_use]

//! Random Number Generator (RNG) driver.

#[cfg_attr(lpc55, path = "./rng/lpc55.rs")]
mod inner;
pub use inner::*;
