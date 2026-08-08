//! Universal Serial Bus (USB) device driver.

#[cfg_attr(lpc55, path = "./usb/lpc55.rs")]
mod inner;
pub use inner::*;
