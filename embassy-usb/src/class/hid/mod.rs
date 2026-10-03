//! Human Interface Device (HID) class.

mod types;

pub mod device;
pub mod host;

pub use types::*;

pub(crate) const USB_CLASS_HID: u8 = 0x03;

// Class descriptor types (HID 1.11 §7.1).
pub(crate) const HID_DESC_TYPE_HID: u8 = 0x21;
pub(crate) const HID_DESC_TYPE_REPORT: u8 = 0x22;

// Class requests (HID 1.11 §7.2).
pub(crate) const HID_REQ_GET_REPORT: u8 = 0x01;
pub(crate) const HID_REQ_GET_IDLE: u8 = 0x02;
pub(crate) const HID_REQ_GET_PROTOCOL: u8 = 0x03;
pub(crate) const HID_REQ_SET_REPORT: u8 = 0x09;
pub(crate) const HID_REQ_SET_IDLE: u8 = 0x0a;
pub(crate) const HID_REQ_SET_PROTOCOL: u8 = 0x0b;
