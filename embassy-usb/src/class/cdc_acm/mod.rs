//! CDC ACM (serial over USB) class.

mod line_coding;

#[cfg(feature = "device")]
pub mod device;
#[cfg(feature = "host")]
pub mod host;

pub use line_coding::*;

/// This should be used as `device_class` when building the `UsbDevice`.
pub const USB_CLASS_CDC: u8 = 0x02;

pub(crate) const USB_CLASS_CDC_DATA: u8 = 0x0a;
pub(crate) const CDC_SUBCLASS_ACM: u8 = 0x02;
#[allow(dead_code)] // Used by the device role only.
pub(crate) const CDC_PROTOCOL_NONE: u8 = 0x00;

// Class requests (CDC PSTN 1.2 Table 13).
pub(crate) const REQ_SET_LINE_CODING: u8 = 0x20;
#[allow(dead_code)] // Used by the device role only.
pub(crate) const REQ_GET_LINE_CODING: u8 = 0x21;
pub(crate) const REQ_SET_CONTROL_LINE_STATE: u8 = 0x22;
