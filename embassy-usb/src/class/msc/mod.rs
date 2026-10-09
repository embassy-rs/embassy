//! Mass storage (MSC) class.

pub(crate) mod bot;
mod scsi;

pub mod device;
pub mod host;

pub use scsi::{SenseData, SenseKey};

/// This should be used as `device_class` when building a pure MSC device.
pub const USB_CLASS_MSC: u8 = 0x08;

pub(crate) const USB_SUBCLASS_SCSI_TRANSPARENT: u8 = 0x06;
pub(crate) const USB_PROTOCOL_BULK_ONLY: u8 = 0x50;

// Bulk-Only class requests (MSC BBB r1.0 §3).
pub(crate) const BOT_REQ_RESET: u8 = 0xff;
pub(crate) const BOT_REQ_GET_MAX_LUN: u8 = 0xfe;
