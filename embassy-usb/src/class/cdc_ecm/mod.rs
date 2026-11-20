//! CDC-ECM (Ethernet Control Model) class implementation.
//!
//! Provides raw Ethernet-over-USB networking for both Device and Host modes.

#[cfg(feature = "device")]
pub mod device;

#[cfg(feature = "host")]
pub mod host;
