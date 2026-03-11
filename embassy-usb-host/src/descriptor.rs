//! USB descriptor parsers.

/// USB Device Descriptor (18 bytes).
#[derive(Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct DeviceDescriptor {
    /// USB specification version (BCD).
    pub usb_version: u16,
    /// Device class code.
    pub device_class: u8,
    /// Device subclass code.
    pub device_subclass: u8,
    /// Device protocol code.
    pub device_protocol: u8,
    /// Maximum packet size for endpoint 0.
    pub max_packet_size_0: u8,
    /// Vendor ID.
    pub vendor_id: u16,
    /// Product ID.
    pub product_id: u16,
    /// Device release number (BCD).
    pub device_version: u16,
    /// Number of configurations.
    pub num_configurations: u8,
}

impl DeviceDescriptor {
    /// Parse a device descriptor from a byte slice (must be at least 18 bytes).
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 18 || data[1] != 0x01 {
            return None;
        }
        Some(Self {
            usb_version: u16::from_le_bytes([data[2], data[3]]),
            device_class: data[4],
            device_subclass: data[5],
            device_protocol: data[6],
            max_packet_size_0: data[7],
            vendor_id: u16::from_le_bytes([data[8], data[9]]),
            product_id: u16::from_le_bytes([data[10], data[11]]),
            device_version: u16::from_le_bytes([data[12], data[13]]),
            num_configurations: data[17],
        })
    }
}

/// USB Configuration Descriptor header (9 bytes).
#[derive(Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ConfigDescriptor {
    /// Total length of config + interface + endpoint descriptors.
    pub total_length: u16,
    /// Number of interfaces.
    pub num_interfaces: u8,
    /// Configuration value to use with SET_CONFIGURATION.
    pub config_value: u8,
    /// Configuration attributes (bit 6 = self-powered, bit 5 = remote wakeup).
    pub attributes: u8,
    /// Maximum power consumption in 2mA units.
    pub max_power: u8,
}

impl ConfigDescriptor {
    /// Parse a configuration descriptor header from a byte slice (must be at least 9 bytes).
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 9 || data[1] != 0x02 {
            return None;
        }
        Some(Self {
            total_length: u16::from_le_bytes([data[2], data[3]]),
            num_interfaces: data[4],
            config_value: data[5],
            attributes: data[7],
            max_power: data[8],
        })
    }
}

/// USB Interface Descriptor (9 bytes).
#[derive(Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct InterfaceDescriptor {
    /// Interface number.
    pub interface_number: u8,
    /// Alternate setting.
    pub alternate_setting: u8,
    /// Number of endpoints (excluding EP0).
    pub num_endpoints: u8,
    /// Interface class code.
    pub interface_class: u8,
    /// Interface subclass code.
    pub interface_subclass: u8,
    /// Interface protocol code.
    pub interface_protocol: u8,
}

impl InterfaceDescriptor {
    /// Parse an interface descriptor from a byte slice (must be at least 9 bytes).
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 9 || data[1] != 0x04 {
            return None;
        }
        Some(Self {
            interface_number: data[2],
            alternate_setting: data[3],
            num_endpoints: data[4],
            interface_class: data[5],
            interface_subclass: data[6],
            interface_protocol: data[7],
        })
    }
}

/// USB Endpoint Descriptor (7 bytes).
#[derive(Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct EndpointDescriptor {
    /// Endpoint address (bit 7 = direction, bits 3:0 = number).
    pub endpoint_address: u8,
    /// Attributes (bits 1:0 = transfer type).
    pub attributes: u8,
    /// Maximum packet size.
    pub max_packet_size: u16,
    /// Polling interval (in frames/microframes).
    pub interval: u8,
}

impl EndpointDescriptor {
    /// Parse an endpoint descriptor from a byte slice (must be at least 7 bytes).
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 7 || data[1] != 0x05 {
            return None;
        }
        Some(Self {
            endpoint_address: data[2],
            attributes: data[3],
            max_packet_size: u16::from_le_bytes([data[4], data[5]]),
            interval: data[6],
        })
    }

    /// Endpoint number (0-15).
    pub fn ep_number(&self) -> u8 {
        self.endpoint_address & 0x0F
    }

    /// True if this is an IN endpoint.
    pub fn is_in(&self) -> bool {
        (self.endpoint_address & 0x80) != 0
    }

    /// Transfer type.
    pub fn transfer_type(&self) -> u8 {
        self.attributes & 0x03
    }
}

/// Descriptor type constants.
pub mod descriptor_type {
    /// Device descriptor.
    pub const DEVICE: u8 = 0x01;
    /// Configuration descriptor.
    pub const CONFIGURATION: u8 = 0x02;
    /// Interface descriptor.
    pub const INTERFACE: u8 = 0x04;
    /// Endpoint descriptor.
    pub const ENDPOINT: u8 = 0x05;
}

/// Iterator over descriptors in a configuration descriptor blob.
pub struct DescriptorIter<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> DescriptorIter<'a> {
    /// Create a new iterator over the raw config descriptor data.
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, offset: 0 }
    }
}

impl<'a> Iterator for DescriptorIter<'a> {
    /// (descriptor_type, descriptor_data)
    type Item = (u8, &'a [u8]);

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset + 2 > self.data.len() {
            return None;
        }
        let len = self.data[self.offset] as usize;
        if len < 2 || self.offset + len > self.data.len() {
            return None;
        }
        let desc_type = self.data[self.offset + 1];
        let desc_data = &self.data[self.offset..self.offset + len];
        self.offset += len;
        Some((desc_type, desc_data))
    }
}
