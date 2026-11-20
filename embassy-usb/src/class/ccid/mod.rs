//! Smart card (CCID) class.

#[cfg(feature = "device")]
pub mod device;
#[cfg(feature = "host")]
pub mod host;
mod types;

pub use types::*;
