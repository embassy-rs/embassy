//! USB descriptor parsers.
#![allow(missing_docs)]

use embassy_usb_driver::host::HostError;
use embassy_usb_driver::{Direction, EndpointInfo, EndpointType};

/// Standard descriptor type constants.
pub mod descriptor_type {
    pub const DEVICE: u8 = 0x01;
    pub const CONFIGURATION: u8 = 0x02;
    pub const INTERFACE: u8 = 0x04;
    pub const ENDPOINT: u8 = 0x05;
}

pub type StringIndex = u8;

/// Maximum descriptor buffer size used during enumeration.
pub(crate) const DEFAULT_MAX_DESCRIPTOR_SIZE: usize = 512;

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum DescriptorError {
    BadDescriptorType,
    UnexpectedEndOfBuffer,
}

/// Trait for fixed-size USB descriptors that can be parsed from a byte slice.
pub trait USBDescriptor {
    const SIZE: usize;
    const DESC_TYPE: u8;
    type Error;
    fn try_from_bytes(bytes: &[u8]) -> Result<Self, Self::Error>
    where
        Self: Sized;
}

/// First 8 bytes of the DeviceDescriptor, used to read `max_packet_size0` before SET_ADDRESS.
#[derive(Debug)]
pub struct DeviceDescriptorPartial {
    _padding: [u8; 7],
    pub max_packet_size0: u8,
}

impl USBDescriptor for DeviceDescriptorPartial {
    const SIZE: usize = 8;
    const DESC_TYPE: u8 = descriptor_type::DEVICE;
    type Error = ();

    fn try_from_bytes(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() < Self::SIZE || bytes[1] != Self::DESC_TYPE {
            return Err(());
        }
        Ok(Self {
            _padding: [0; 7],
            max_packet_size0: bytes[7],
        })
    }
}

/// USB Device Descriptor (18 bytes).
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct DeviceDescriptor {
    pub len: u8,
    pub descriptor_type: u8,
    pub bcd_usb: u16,
    pub device_class: u8,
    pub device_subclass: u8,
    pub device_protocol: u8,
    pub max_packet_size0: u8,
    pub vendor_id: u16,
    pub product_id: u16,
    pub bcd_device: u16,
    pub manufacturer: StringIndex,
    pub product: StringIndex,
    pub serial_number: StringIndex,
    pub num_configurations: u8,
}

impl USBDescriptor for DeviceDescriptor {
    const SIZE: usize = 18;
    const DESC_TYPE: u8 = descriptor_type::DEVICE;
    type Error = ();

    fn try_from_bytes(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() < Self::SIZE || bytes[1] != Self::DESC_TYPE {
            return Err(());
        }
        Ok(Self {
            len: bytes[0],
            descriptor_type: bytes[1],
            bcd_usb: u16::from_le_bytes([bytes[2], bytes[3]]),
            device_class: bytes[4],
            device_subclass: bytes[5],
            device_protocol: bytes[6],
            max_packet_size0: bytes[7],
            vendor_id: u16::from_le_bytes([bytes[8], bytes[9]]),
            product_id: u16::from_le_bytes([bytes[10], bytes[11]]),
            bcd_device: u16::from_le_bytes([bytes[12], bytes[13]]),
            manufacturer: bytes[14],
            product: bytes[15],
            serial_number: bytes[16],
            num_configurations: bytes[17],
        })
    }
}

/// USB Configuration Descriptor header with a reference to the sub-descriptor buffer.
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ConfigurationDescriptor<'a> {
    pub len: u8,
    pub descriptor_type: u8,
    pub total_len: u16,
    pub num_interfaces: u8,
    pub configuration_value: u8,
    pub configuration_name: StringIndex,
    pub attributes: u8,
    pub max_power: u8,
    /// The raw bytes following the 9-byte header (interface + endpoint descriptors).
    pub buffer: &'a [u8],
}

impl USBDescriptor for ConfigurationDescriptor<'_> {
    const SIZE: usize = 9;
    const DESC_TYPE: u8 = descriptor_type::CONFIGURATION;
    type Error = ();

    fn try_from_bytes(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() < Self::SIZE || bytes[1] != Self::DESC_TYPE {
            return Err(());
        }
        Ok(Self {
            len: bytes[0],
            descriptor_type: bytes[1],
            total_len: u16::from_le_bytes([bytes[2], bytes[3]]),
            num_interfaces: bytes[4],
            configuration_value: bytes[5],
            configuration_name: bytes[6],
            attributes: bytes[7],
            max_power: bytes[8],
            buffer: &[],
        })
    }
}

impl<'a> ConfigurationDescriptor<'a> {
    /// Parse a full Configuration Descriptor blob, giving access to sub-descriptors via iterators.
    pub fn try_from_slice(buf: &'a [u8]) -> Result<Self, HostError> {
        if buf.len() < Self::SIZE || buf[1] != Self::DESC_TYPE {
            return Err(HostError::InvalidDescriptor);
        }
        let total_length = u16::from_le_bytes([buf[2], buf[3]]);
        Ok(Self {
            len: buf[0],
            descriptor_type: buf[1],
            total_len: total_length,
            num_interfaces: buf[4],
            configuration_value: buf[5],
            configuration_name: buf[6],
            attributes: buf[7],
            max_power: buf[8],
            buffer: &buf[buf[0] as usize..total_length as usize],
        })
    }

    /// Iterate over all raw descriptors in this Configuration.
    pub fn iter_descriptors(&self) -> RawDescriptorIterator<'_> {
        RawDescriptorIterator {
            buf: self.buffer,
            offset: 0,
        }
    }

    /// Iterate over all interface descriptors of this Configuration.
    pub fn iter_interface(&self) -> InterfaceIterator<'_> {
        let first_interface_offset = self
            .iter_descriptors()
            .find_map(|(offset, bytes)| {
                if bytes[1] == descriptor_type::INTERFACE {
                    Some(offset)
                } else {
                    None
                }
            })
            .unwrap_or(0);
        InterfaceIterator {
            offset: first_interface_offset,
            cfg_desc: self,
        }
    }
}

/// USB Interface Descriptor with a reference to the trailing sub-descriptor buffer.
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct InterfaceDescriptor<'a> {
    pub len: u8,
    pub descriptor_type: u8,
    pub interface_number: u8,
    pub alternate_setting: u8,
    pub num_endpoints: u8,
    pub interface_class: u8,
    pub interface_subclass: u8,
    pub interface_protocol: u8,
    pub interface_name: StringIndex,
    /// All bytes following this descriptor up to (but not including) the next interface descriptor.
    pub buffer: &'a [u8],
}

impl<'a> InterfaceDescriptor<'a> {
    const SIZE: usize = 9;
    const DESC_TYPE: u8 = descriptor_type::INTERFACE;

    pub(crate) fn try_from_bytes(bytes: &'a [u8]) -> Result<Self, ()> {
        if bytes.len() < Self::SIZE || bytes[1] != Self::DESC_TYPE {
            return Err(());
        }
        let endpoints = &bytes[bytes[0] as usize..];
        let mut raw = RawDescriptorIterator {
            buf: endpoints,
            offset: 0,
        };
        let next_iface_index = raw
            .find_map(|(index, v)| v.get(1).is_some_and(|v| *v == Self::DESC_TYPE).then_some(index))
            .unwrap_or(endpoints.len());
        Ok(Self {
            len: bytes[0],
            descriptor_type: bytes[1],
            interface_number: bytes[2],
            alternate_setting: bytes[3],
            num_endpoints: bytes[4],
            interface_class: bytes[5],
            interface_subclass: bytes[6],
            interface_protocol: bytes[7],
            interface_name: bytes[8],
            buffer: &endpoints[..next_iface_index],
        })
    }

    /// Iterate over raw descriptors inside this interface.
    pub fn iter_descriptors(&self) -> RawDescriptorIterator<'_> {
        RawDescriptorIterator {
            buf: self.buffer,
            offset: 0,
        }
    }

    /// Iterate over endpoint descriptors inside this interface.
    pub fn iter_endpoints(&'a self) -> EndpointIterator<'a> {
        EndpointIterator {
            index: 0,
            buffer_idx: 0,
            iface_desc: self,
        }
    }
}

/// Iterates over the InterfaceDescriptors of a configuration.
pub struct InterfaceIterator<'a> {
    offset: usize,
    cfg_desc: &'a ConfigurationDescriptor<'a>,
}

impl<'a> Iterator for InterfaceIterator<'a> {
    type Item = InterfaceDescriptor<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset >= self.cfg_desc.buffer.len() {
            return None;
        }
        let remaining = &self.cfg_desc.buffer[self.offset..];
        let iface = InterfaceDescriptor::try_from_bytes(remaining).ok()?;
        self.offset += iface.len as usize + iface.buffer.len();
        Some(iface)
    }
}

/// Iterates over raw descriptors, yielding `(byte_offset, &[u8])`.
pub struct RawDescriptorIterator<'a> {
    buf: &'a [u8],
    offset: usize,
}

impl<'a> Iterator for RawDescriptorIterator<'a> {
    type Item = (usize, &'a [u8]);

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset >= self.buf.len() {
            return None;
        }
        let pre = self.offset;
        let len = self.buf[pre] as usize;
        self.offset += len;
        if self.offset > self.buf.len() {
            return None;
        }
        Some((pre, &self.buf[pre..self.offset]))
    }
}

/// Iterates over the endpoint descriptors of an interface.
pub struct EndpointIterator<'a> {
    buffer_idx: usize,
    index: usize,
    iface_desc: &'a InterfaceDescriptor<'a>,
}

impl Iterator for EndpointIterator<'_> {
    type Item = EndpointDescriptor;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.iface_desc.num_endpoints as usize {
            return None;
        }
        while self.buffer_idx + 7 <= self.iface_desc.buffer.len() {
            let working = &self.iface_desc.buffer[self.buffer_idx..];
            self.buffer_idx += working[0] as usize;
            if let Ok(d) = EndpointDescriptor::try_from_bytes(working) {
                self.index += 1;
                return Some(d);
            }
        }
        None
    }
}

/// USB Endpoint Descriptor (7 bytes).
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct EndpointDescriptor {
    pub len: u8,
    pub descriptor_type: u8,
    pub endpoint_address: u8,
    pub attributes: u8,
    pub max_packet_size: u16,
    pub interval: u8,
}

impl EndpointDescriptor {
    /// Returns the endpoint direction.
    pub fn ep_dir(&self) -> Direction {
        match self.endpoint_address & 0x80 {
            0x00 => Direction::Out,
            _ => Direction::In,
        }
    }

    /// Returns the endpoint transfer type.
    pub fn ep_type(&self) -> EndpointType {
        match self.attributes & 0x03 {
            0 => EndpointType::Control,
            1 => EndpointType::Isochronous,
            2 => EndpointType::Bulk,
            _ => EndpointType::Interrupt,
        }
    }

    /// Endpoint number (0-15).
    pub fn ep_number(&self) -> u8 {
        self.endpoint_address & 0x0F
    }

    /// True if this is an IN endpoint.
    pub fn is_in(&self) -> bool {
        (self.endpoint_address & 0x80) != 0
    }

    /// Transfer type (0=Control, 1=Isochronous, 2=Bulk, 3=Interrupt).
    pub fn transfer_type(&self) -> u8 {
        self.attributes & 0x03
    }
}

impl USBDescriptor for EndpointDescriptor {
    const SIZE: usize = 7;
    const DESC_TYPE: u8 = descriptor_type::ENDPOINT;
    type Error = DescriptorError;

    fn try_from_bytes(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() < Self::SIZE || bytes.len() < bytes[0] as usize {
            return Err(DescriptorError::UnexpectedEndOfBuffer);
        }
        if bytes[1] != Self::DESC_TYPE {
            return Err(DescriptorError::BadDescriptorType);
        }
        Ok(Self {
            len: bytes[0],
            descriptor_type: bytes[1],
            endpoint_address: bytes[2],
            attributes: bytes[3],
            max_packet_size: u16::from_le_bytes([bytes[4], bytes[5]]),
            interval: bytes[6],
        })
    }
}

impl From<EndpointDescriptor> for EndpointInfo {
    fn from(value: EndpointDescriptor) -> Self {
        EndpointInfo {
            addr: value.endpoint_address.into(),
            ep_type: value.ep_type(),
            max_packet_size: value.max_packet_size,
            interval_ms: value.interval,
        }
    }
}

#[cfg(test)]
mod test {
    use heapless::Vec;

    use super::{ConfigurationDescriptor, EndpointDescriptor};

    #[test]
    fn test_parse_extended_endpoint_descriptor() {
        let desc_bytes = [
            9, 2, 76, 0, 2, 1, 0, 160, 101, 8, 11, 0, 1, 3, 0, 0, 0, 9, 4, 0, 0, 1, 3, 1, 1, 0, 9, 33, 16, 1, 0, 1, 34,
            63, 0, 9, 5, 129, 3, 8, 0, 1, 99, 99, 9, 4, 1, 0, 2, 3, 1, 0, 0, 9, 33, 16, 1, 0, 1, 34, 39, 0, 7, 5, 131,
            3, 64, 0, 1, 7, 5, 3, 3, 64, 0, 1,
        ];

        let cfg = ConfigurationDescriptor::try_from_slice(desc_bytes.as_slice()).unwrap();
        assert_eq!(cfg.num_interfaces, 2);

        let interface0 = cfg.iter_interface().next().unwrap();
        assert_eq!(interface0.interface_number, 0);
        assert_eq!(interface0.num_endpoints, 1);

        let endpoints: Vec<EndpointDescriptor, 2> = interface0.iter_endpoints().collect();
        assert_eq!(endpoints.len(), 1);
        assert_eq!(endpoints[0].endpoint_address, 0x81);
        assert_eq!(endpoints[0].max_packet_size, 8);

        let interface1 = cfg.iter_interface().nth(1).unwrap();
        assert_eq!(interface1.interface_number, 1);
        assert_eq!(interface1.num_endpoints, 2);

        let endpoints: Vec<EndpointDescriptor, 2> = interface1.iter_endpoints().collect();
        assert_eq!(endpoints.len(), 2);
    }

    #[test]
    fn test_parse_interface_descriptor() {
        let desc_bytes = [
            9, 2, 66, 0, 2, 1, 0, 160, 101, 9, 4, 0, 0, 1, 3, 1, 1, 0, 9, 33, 16, 1, 0, 1, 34, 63, 0, 7, 5, 129, 3, 8,
            0, 1, 9, 4, 1, 0, 2, 3, 1, 0, 0, 9, 33, 16, 1, 0, 1, 34, 39, 0, 7, 5, 131, 3, 64, 0, 1, 7, 5, 3, 3, 64, 0,
            1,
        ];

        let cfg = ConfigurationDescriptor::try_from_slice(desc_bytes.as_slice()).unwrap();
        assert_eq!(cfg.num_interfaces, 2);

        let interface0 = cfg.iter_interface().next().unwrap();
        assert_eq!(interface0.interface_number, 0);

        let interface0_buffer_ref = [9u8, 33, 16, 1, 0, 1, 34, 63, 0, 7, 5, 129, 3, 8, 0, 1];
        assert_eq!(interface0.buffer.len(), interface0_buffer_ref.len());

        let interface1 = cfg.iter_interface().nth(1).unwrap();
        assert_eq!(interface1.interface_number, 1);

        let interface1_buffer_ref = [
            9u8, 33, 16, 1, 0, 1, 34, 39, 0, 7, 5, 131, 3, 64, 0, 1, 7, 5, 3, 3, 64, 0, 1,
        ];
        assert_eq!(interface1.buffer.len(), interface1_buffer_ref.len());
    }
}
