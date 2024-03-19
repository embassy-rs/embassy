//! Universal Serial Bus (USB)

#[cfg_attr(usb, path = "usb.rs")]
#[cfg_attr(otg, path = "otg.rs")]
mod _version;
pub use _version::*;
