#![no_std]
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
mod fmt;

/// Re-export DFU constants from embassy-usb.
pub mod consts {
    pub use embassy_usb::class::dfu::consts::*;
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
/// This crate exposes `ResetImmediate` when compiled with cortex-m or esp32c3 support, which immediately issues a
/// reset request without interfacing with any other peripherals.
///
/// If alternate behaviour is desired, a custom implementation of Reset can be provided as an argument to the usb_dfu function.
pub trait Reset {
    /// Reset the device.
    fn sys_reset(&self);
}

/// Reset immediately.
#[cfg(feature = "esp32c3-hal")]
pub struct ResetImmediate;

#[cfg(feature = "esp32c3-hal")]
impl Reset for ResetImmediate {
    fn sys_reset(&self) {
        esp32c3_hal::reset::software_reset();
        loop {}
    }
}

/// Reset immediately.
#[cfg(feature = "cortex-m")]
pub struct ResetImmediate;

#[cfg(feature = "cortex-m")]
impl Reset for ResetImmediate {
    fn sys_reset(&self) {
        cortex_m::peripheral::SCB::sys_reset()
    }
}
