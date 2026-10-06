//! Secure Hash Algorithm (SHA) driver.
//!
//! Currently supported:
//! - SHA-256 on LPC55 HASHCRYPT.

#[cfg_attr(lpc55, path = "./sha/lpc55.rs")]
mod inner;
pub use inner::*;
