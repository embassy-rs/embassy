#![no_std]
mod fmt;

pub mod consts;

#[cfg(feature = "bootloader")]
mod bootloader;
#[cfg(feature = "bootloader")]
pub use self::bootloader::*;

#[cfg(feature = "application")]
mod application;
#[cfg(feature = "application")]
pub use self::application::*;

#[cfg(any(
    all(feature = "bootloader", feature = "application"),
    not(any(feature = "bootloader", feature = "application"))
))]
compile_error!("usb-dfu must be compiled with exactly one of `bootloader`, or `application` features");

/// Provides a platform-agnostic interface for initiating a system reset.
///
/// This crate exposes `ResetImmediate` when compiled with cortex-m or esp32c3 support, which immediately issues a
/// reset request without interfacing with any other peripherals.
///
/// If alternate behaviour is desired, a custom implementation of Reset can be provided as a type argument to the usb_dfu function.
pub trait Reset {
    fn sys_reset() -> !;
}

#[cfg(feature = "esp32c3-hal")]
pub struct ResetImmediate;

#[cfg(feature = "esp32c3-hal")]
impl Reset for ResetImmediate {
    fn sys_reset() -> ! {
        esp32c3_hal::reset::software_reset();
        loop {}
    }
}

#[cfg(feature = "cortex-m")]
pub struct ResetImmediate;

#[cfg(feature = "cortex-m")]
impl Reset for ResetImmediate {
    fn sys_reset() -> ! {
        cortex_m::peripheral::SCB::sys_reset()
    }
}
