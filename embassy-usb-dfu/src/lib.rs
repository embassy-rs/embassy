#![no_std]

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
