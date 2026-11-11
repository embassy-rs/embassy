//! USB Printer Class 1.1 implementation.
//!
//! Supports both Device mode (acting as a USB thermal/receipt/label printer) and
//! Host mode (connecting to USB printers, such as ESC/POS thermal printers).

macro_rules! bitflags {
    ($($tt:tt)*) => {
        #[cfg(feature = "defmt")]
        defmt::bitflags! { $($tt)* }
        #[cfg(not(feature = "defmt"))]
        bitflags::bitflags! { #[derive(Debug, Clone, Copy, PartialEq, Eq)] $($tt)* }
    };
}

/// USB Printer base class code.
pub const USB_CLASS_PRINTER: u8 = 0x07;
/// USB Printer subclass code.
pub const PRINTER_SUBCLASS_PRINTER: u8 = 0x01;

/// Unidirectional printer protocol (Bulk OUT only).
pub const PROTOCOL_UNIDIRECTIONAL: u8 = 0x01;
/// Bi-directional printer protocol (Bulk OUT and Bulk IN).
pub const PROTOCOL_BIDIRECTIONAL: u8 = 0x02;
/// 1284.4 compatible bi-directional printer protocol.
pub const PROTOCOL_1284_4: u8 = 0x03;

/// Class-specific request: read IEEE 1284 Device ID string.
pub const REQ_GET_DEVICE_ID: u8 = 0x00;
/// Class-specific request: query parallel port status bits.
pub const REQ_GET_PORT_STATUS: u8 = 0x01;
/// Class-specific request: perform a soft reset.
pub const REQ_SOFT_RESET: u8 = 0x02;

bitflags! {
    /// Port status byte returned by `GET_PORT_STATUS` (USB Printer Class 1.1 §4.2.2).
    pub struct PortStatus: u8 {
        /// No error condition (1 = OK / no error, 0 = error condition).
        const NOT_ERROR   = 1 << 3;
        /// Selected / on-line (1 = selected / on-line, 0 = off-line).
        const SELECTED    = 1 << 4;
        /// Paper empty (1 = out of paper, 0 = paper present).
        const PAPER_EMPTY = 1 << 5;
    }
}

impl Default for PortStatus {
    fn default() -> Self {
        Self::NOT_ERROR | Self::SELECTED
    }
}

#[cfg(feature = "device")]
pub mod device;

#[cfg(feature = "host")]
pub mod host;
