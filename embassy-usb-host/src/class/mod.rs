//! USB host class drivers.

pub mod cdc_acm;
pub mod hid;

use embassy_usb_driver::Speed;

use crate::descriptor::DeviceDescriptor;

/// Trait for USB host class drivers.
///
/// A class driver is notified when a device is enumerated and can claim interfaces
/// that match its supported class/subclass/protocol.
pub trait HostClassDriver {
    /// Called when a device has been enumerated and configured.
    ///
    /// The driver should parse the configuration descriptor to find matching interfaces.
    /// Returns `true` if the driver has claimed one or more interfaces.
    ///
    /// # Arguments
    /// * `device` - The device descriptor
    /// * `config_desc` - The full configuration descriptor (all interfaces + endpoints)
    /// * `device_address` - The assigned USB device address
    /// * `speed` - The device speed
    fn bind(&mut self, device: &DeviceDescriptor, config_desc: &[u8], device_address: u8, speed: Speed) -> bool;

    /// Called when the device is disconnected.
    fn unbind(&mut self);
}
