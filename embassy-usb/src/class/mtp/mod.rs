//! Media Transfer Protocol (MTP / PTP) USB class.
//!
//! MTP exposes storages, folders, and file objects over USB using PIMA 15740 /
//! MTP 1.1 container transactions rather than raw block-level access, allowing
//! the device firmware to retain ownership of its filesystem while the host
//! browses and transfers objects concurrently.

#[cfg(feature = "device")]
pub mod device;
pub mod types;

#[cfg(feature = "device")]
pub use device::*;
pub use types::*;
