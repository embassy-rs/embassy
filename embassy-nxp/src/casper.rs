//! Cryptographic Accelerator and Signaling Processing Engine with RAM-sharing (CASPER) driver

#[cfg_attr(lpc55, path = "./casper/lpc55.rs")]
mod inner;
pub use inner::*;
