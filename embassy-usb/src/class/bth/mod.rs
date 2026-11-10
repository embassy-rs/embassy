//! Bluetooth Host Controller Interface (HCI) class.

#[cfg(feature = "device")]
pub mod device;

#[cfg(feature = "device")]
pub use device::*;
