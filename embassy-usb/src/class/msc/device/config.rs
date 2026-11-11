//! [`MscClass`](super::MscClass) configuration.

use core::cmp::min;

use crate::EndpointOptions;

/// Configuration for the USB Mass Storage Class.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Config {
    /// Vendor identification (8 ASCII characters, space-padded per SPC-2).
    pub vendor_id: [u8; 8],
    /// Product identification (16 ASCII characters, space-padded per SPC-2).
    pub product_id: [u8; 16],
    /// Product revision level (4 ASCII characters, space-padded per SPC-2).
    pub product_revision_level: [u8; 4],
    /// Unit serial number (up to 32 ASCII characters).
    pub serial_number: [u8; 32],
    /// Length of the unit serial number in bytes (0..=32).
    pub serial_number_len: u8,
    /// Options for the bulk OUT endpoint, which receives the data the host sends.
    pub out_endpoint_options: EndpointOptions,
    /// Options for the bulk IN endpoint, which sends data to the host.
    pub in_endpoint_options: EndpointOptions,
}

impl Config {
    /// Creates a new `Config` with empty vendor/product/serial strings.
    pub const fn new() -> Self {
        Self {
            vendor_id: [b' '; 8],
            product_id: [b' '; 16],
            product_revision_level: [b' '; 4],
            serial_number: [0u8; 32],
            serial_number_len: 0,
            out_endpoint_options: EndpointOptions::new(),
            in_endpoint_options: EndpointOptions::new(),
        }
    }

    /// Sets the SCSI Vendor Identification (space-padded or truncated to 8 bytes).
    pub fn vendor_id(mut self, vendor_id: &str) -> Self {
        self.set_vendor_id(vendor_id);
        self
    }

    /// Sets the SCSI Vendor Identification (space-padded or truncated to 8 bytes).
    pub fn set_vendor_id(&mut self, vendor_id: &str) {
        copy_pad_ascii(&mut self.vendor_id, vendor_id);
    }

    /// Sets the SCSI Product Identification (space-padded or truncated to 16 bytes).
    pub fn product_id(mut self, product_id: &str) -> Self {
        self.set_product_id(product_id);
        self
    }

    /// Sets the SCSI Product Identification (space-padded or truncated to 16 bytes).
    pub fn set_product_id(&mut self, product_id: &str) {
        copy_pad_ascii(&mut self.product_id, product_id);
    }

    /// Sets the SCSI Product Revision Level (space-padded or truncated to 4 bytes).
    pub fn product_revision_level(mut self, revision: &str) -> Self {
        self.set_product_revision_level(revision);
        self
    }

    /// Sets the SCSI Product Revision Level (space-padded or truncated to 4 bytes).
    pub fn set_product_revision_level(&mut self, revision: &str) {
        copy_pad_ascii(&mut self.product_revision_level, revision);
    }

    /// Sets the Unit Serial Number (up to 32 ASCII characters).
    pub fn serial_number(mut self, serial_number: &str) -> Self {
        self.set_serial_number(serial_number);
        self
    }

    /// Sets the Unit Serial Number (up to 32 ASCII characters).
    pub fn set_serial_number(&mut self, serial_number: &str) {
        let bytes = serial_number.as_bytes();
        let len = min(bytes.len(), self.serial_number.len());
        self.serial_number[..len].copy_from_slice(&bytes[..len]);
        self.serial_number_len = len as u8;
    }

    /// Returns the unit serial number bytes.
    pub fn serial_number_bytes(&self) -> &[u8] {
        &self.serial_number[..self.serial_number_len as usize]
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

fn copy_pad_ascii(dest: &mut [u8], src: &str) {
    dest.fill(b' ');
    let len = min(dest.len(), src.len());
    dest[..len].copy_from_slice(&src.as_bytes()[..len]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pads_ascii() {
        let mut buf = [0u8; 8];
        copy_pad_ascii(&mut buf, "Embassy");
        core::assert_eq!(&buf, b"Embassy ");

        let mut buf16 = [0u8; 16];
        copy_pad_ascii(&mut buf16, "MSC Disk");
        core::assert_eq!(&buf16, b"MSC Disk        ");

        let mut buf4 = [0u8; 4];
        copy_pad_ascii(&mut buf4, "0.1");
        core::assert_eq!(&buf4, b"0.1 ");
    }

    #[test]
    fn default_config() {
        let config = Config::default();
        core::assert_eq!(&config.vendor_id, b"        ");
        core::assert_eq!(&config.product_id, b"                ");
        core::assert_eq!(&config.product_revision_level, b"    ");
        core::assert_eq!(config.serial_number_bytes(), b"");
    }

    #[test]
    fn config_builder_dynamic_strings() {
        let dynamic_serial = "CHIP-9876543210";
        let config = Config::new()
            .vendor_id("ACME")
            .product_id("Storage Device")
            .product_revision_level("2.0")
            .serial_number(dynamic_serial);

        core::assert_eq!(&config.vendor_id, b"ACME    ");
        core::assert_eq!(&config.product_id, b"Storage Device  ");
        core::assert_eq!(&config.product_revision_level, b"2.0 ");
        core::assert_eq!(config.serial_number_bytes(), dynamic_serial.as_bytes());
    }
}
