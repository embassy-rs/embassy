#![no_std]
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

pub mod driver;

mod aes;
mod ct;
mod ec;
mod hash;

pub mod p256;
pub mod p384;
pub mod x25519;
pub mod ed25519;

pub use aes::*;
pub use driver::Error;
pub use hash::*;
