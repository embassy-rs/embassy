//! Generic Vendor-specific bulk USB class with optional WinUSB and WebUSB support.

#[cfg(feature = "device")]
pub mod device;

#[cfg(feature = "device")]
pub use device::*;
