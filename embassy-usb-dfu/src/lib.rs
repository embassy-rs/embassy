#![no_std]
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
mod fmt;

/// Re-export DFU constants from embassy-usb.
pub mod consts {
    pub use embassy_usb::class::dfu::device::consts::*;
}

#[cfg(feature = "dfu")]
pub mod dfu;
#[cfg(all(feature = "dfu", not(feature = "application")))]
pub use self::dfu::*;

#[cfg(feature = "application")]
pub mod application;
#[cfg(all(feature = "application", not(feature = "dfu")))]
pub use self::application::*;

/// Provides a platform-agnostic interface for initiating a system reset.
///
/// Implement it for your platform and pass it to `new_state`.
pub trait Reset {
    /// Reset the device.
    fn reset(&self);
}
