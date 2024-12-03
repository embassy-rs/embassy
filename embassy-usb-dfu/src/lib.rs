#![no_std]
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
mod fmt;

pub mod consts;

#[cfg(feature = "dfu")]
mod dfu;
#[cfg(feature = "dfu")]
pub use self::dfu::*;

#[cfg(feature = "application")]
mod application;
#[cfg(feature = "application")]
pub use self::application::*;

#[cfg(any(
    all(feature = "dfu", feature = "application"),
    not(any(feature = "dfu", feature = "application"))
))]
compile_error!("usb-dfu must be compiled with exactly one of `dfu`, or `application` features");

/// Provides a platform-agnostic interface for initiating a system reset.
///
/// This crate exposes `ResetImmediate` when compiled with cortex-m or esp32c3 support, which immediately issues a
/// reset request without interfacing with any other peripherals.
///
/// If alternate behaviour is desired, a custom implementation of Reset can be provided as a type argument to the usb_dfu function.
pub trait Reset {
    /// Reset the device.
    fn sys_reset() -> !;
}

/// Reset immediately.
#[cfg(feature = "esp32c3-hal")]
pub struct ResetImmediate;

#[cfg(feature = "esp32c3-hal")]
impl Reset for ResetImmediate {
    fn sys_reset() -> ! {
        esp32c3_hal::reset::software_reset();
        loop {}
    }
}

/// Reset immediately.
#[cfg(feature = "cortex-m")]
pub struct ResetImmediate;

#[cfg(feature = "cortex-m")]
impl Reset for ResetImmediate {
    fn sys_reset() -> ! {
        cortex_m::peripheral::SCB::sys_reset()
    }
}
