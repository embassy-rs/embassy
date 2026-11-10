//! Human Interface Device (HID) class.

mod boot;
pub mod composite;
pub mod consumer;
pub mod gamepad;
mod types;

#[cfg(feature = "device")]
pub mod device;
#[cfg(feature = "host")]
pub mod host;

pub use boot::{KeyboardReport, MouseButtons, MouseReport};
pub use composite::{
    COMPOSITE_DESCRIPTOR, IdConsumerReport, IdGamepadReport, IdKeyboardReport, IdMouseReport, REPORT_ID_CONSUMER,
    REPORT_ID_GAMEPAD, REPORT_ID_KEYBOARD, REPORT_ID_MOUSE,
};
pub use consumer::ConsumerReport;
pub use gamepad::{GamepadButtons, GamepadReport};
pub use types::*;

pub(crate) const USB_CLASS_HID: u8 = 0x03;

// Class descriptor types (HID 1.11 §7.1).
pub(crate) const HID_DESC_TYPE_HID: u8 = 0x21;
pub(crate) const HID_DESC_TYPE_REPORT: u8 = 0x22;

// Class requests (HID 1.11 §7.2).
pub(crate) const HID_REQ_GET_REPORT: u8 = 0x01;
#[allow(dead_code)] // Used by the device role only.
pub(crate) const HID_REQ_GET_IDLE: u8 = 0x02;
#[allow(dead_code)] // Used by the device role only.
pub(crate) const HID_REQ_GET_PROTOCOL: u8 = 0x03;
pub(crate) const HID_REQ_SET_REPORT: u8 = 0x09;
pub(crate) const HID_REQ_SET_IDLE: u8 = 0x0a;
pub(crate) const HID_REQ_SET_PROTOCOL: u8 = 0x0b;
