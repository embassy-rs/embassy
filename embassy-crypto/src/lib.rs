#![no_std]
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

pub mod driver;

mod aes;
mod chacha;
mod ct;
mod ec;
mod hash;

pub mod ed25519;
pub mod p256;
pub mod p384;
pub mod x25519;

pub use aes::*;
pub use chacha::*;
pub use driver::Error;
pub use hash::*;

/// Fill `buf` with cryptographically secure random bytes.
pub fn rng_fill_bytes(buf: &mut [u8]) {
    driver::RngImpl::fill_bytes(buf)
}
