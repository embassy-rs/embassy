// HASHCRYPT driver
#![macro_use]
#[cfg_attr(lpc55, path = "./hashcrypt/lpc55.rs")]
mod inner;
pub use inner::*;
