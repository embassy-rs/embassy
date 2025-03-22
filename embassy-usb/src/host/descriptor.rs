use crate::descriptor::descriptor_type;
use embassy_usb_driver::host::EndpointDescriptor;
use heapless::Vec;

type StringIndex = u8;

/// First 8 bytes of the DeviceDescriptor. This is used to figure out the `max_packet_size0` value to reconfigure channel 0.
/// All USB devices support max_packet_size0=8 which is why the first 8 bytes of the descriptor can always be read.
#[allow(missing_docs)]
#[derive(Debug)]
pub struct DeviceDescriptorPartial {
    _padding: [u8; 7],
    pub max_packet_size0: u8,
}

#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[allow(missing_docs)]
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

#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[allow(missing_docs)]
pub struct ConfigurationDescriptor {
    pub len: u8,
    pub descriptor_type: u8,
    pub total_len: u16,
    pub num_interfaces: u8,
    pub configuration_value: u8,
    pub configuration_name: StringIndex,
    pub attributes: u8,
    pub max_power: u8,

    /// All additional bytes end up in this buffer.
    /// This includes the interface descriptors
    pub buffer: [u8; 256],
}

#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[allow(missing_docs)]
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

    /// All additional bytes end up in this buffer.
    /// This buffer can then be used to parse endpoint descriptors or class descriptors
    pub buffer: &'a [u8],
}

/// Trait to be implemented by fixed size descriptors for automatic parsing.
pub trait USBDescriptor {
    /// Fixed size of the descriptor
    /// For varying length descriptors, this cannot be used and they have to be parsed outside of this module.
    const SIZE: usize;

    /// The descriptor type that has to match the type of this descriptor.
    const DESC_TYPE: u8;

    /// The type returned on error
    type Error;

    /// Try to parse the descriptor from a byte slice
    fn try_from_bytes(bytes: &[u8]) -> Result<Self, Self::Error>
    where
        Self: Sized;
}

impl USBDescriptor for DeviceDescriptorPartial {
    const SIZE: usize = 8;

    const DESC_TYPE: u8 = descriptor_type::DEVICE;

    type Error = ();

    fn try_from_bytes(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() < Self::SIZE {
            return Err(());
        }
        if bytes[1] != Self::DESC_TYPE {
            return Err(());
        }
        Ok(Self {
            _padding: [0; 7],
            max_packet_size0: bytes[7],
        })
    }
}

impl USBDescriptor for DeviceDescriptor {
    const SIZE: usize = 18;

    const DESC_TYPE: u8 = descriptor_type::DEVICE;
    type Error = ();

    fn try_from_bytes(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() < Self::SIZE {
            return Err(());
        }
        if bytes[1] != Self::DESC_TYPE {
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

impl USBDescriptor for ConfigurationDescriptor {
    const SIZE: usize = 9;

    const DESC_TYPE: u8 = descriptor_type::CONFIGURATION;
    type Error = ();

    fn try_from_bytes(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() < Self::SIZE {
            return Err(());
        }
        if bytes[1] != Self::DESC_TYPE {
            return Err(());
        }
        let mut buffer = [0u8; 256];
        let rest_of_desc = &bytes[Self::SIZE..];
        buffer[..rest_of_desc.len()].copy_from_slice(rest_of_desc);

        Ok(Self {
            len: bytes[0],
            descriptor_type: bytes[1],
            total_len: u16::from_le_bytes([bytes[2], bytes[3]]),
            num_interfaces: bytes[4],
            configuration_value: bytes[5],
            configuration_name: bytes[6],
            attributes: bytes[7],
            max_power: bytes[8],
            buffer,
        })
    }
}

pub struct InterfaceIterator<'a> {
    num_interface: usize,
    index: usize,
    cfg_desc: &'a ConfigurationDescriptor,
}

impl<'a> Iterator for InterfaceIterator<'a> {
    type Item = InterfaceDescriptor<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.num_interface {
            None
        } else {
            let res = self.cfg_desc.parse_interface(self.index);
            self.index += 1;
            res
        }
    }
}

impl ConfigurationDescriptor {
    pub fn iter_interface<'a>(&'a self) -> InterfaceIterator<'a> {
        InterfaceIterator {
            num_interface: self.num_interfaces as usize,
            index: 0,
            cfg_desc: &self,
        }
    }

    /// Try to find and parse the interface with interface number `index`
    pub fn parse_interface<'a>(&'a self, index: usize) -> Option<InterfaceDescriptor<'a>> {
        if index >= self.num_interfaces as usize {
            return None;
        }

        let mut dest_buffer = self.buffer_sliced();

        let mut start = None;

        // Find interface that matches the requested index
        while let Some((offset, interface_number)) = Self::identify_interface(dest_buffer) {
            if interface_number == index as u8 {
                // start of interface
                start = Some(offset);
                break;
            }
            dest_buffer = &dest_buffer[offset + InterfaceDescriptor::SIZE..];
        }

        // start is relative to current dest_buffer.
        let Some(start) = start else { return None };

        // Find next interface if any
        let next_interface_buffer = &dest_buffer[start + InterfaceDescriptor::SIZE..];

        let interface_bytes = if let Some((offset, _)) = Self::identify_interface(next_interface_buffer) {
            let end = start + InterfaceDescriptor::SIZE + offset;
            &dest_buffer[start..end]
        } else {
            &dest_buffer[start..]
        };

        InterfaceDescriptor::try_from_bytes(interface_bytes).ok()
    }

    fn buffer_sliced(&self) -> &[u8] {
        // The confiuration descriptor's own bytes are already consumed.
        let end = self.total_len as usize - Self::SIZE;
        &self.buffer[..end]
    }

    // Returns the offset to the next interface descriptor as well as the interface_number (index in descriptor)
    fn identify_interface(slice: &[u8]) -> Option<(usize, u8)> {
        let mut offset = 0;
        let mut desc_len = slice[offset] as usize;
        let mut desc_type = slice[offset + 1];

        while desc_type != InterfaceDescriptor::DESC_TYPE || desc_len != InterfaceDescriptor::SIZE {
            // 'flush' buffer until end of descriptor
            offset += desc_len.max(1); // at least 1 byute to prevent infinite loop
            if offset + InterfaceDescriptor::SIZE > slice.len() {
                // end of slice
                return None;
            }

            desc_len = slice[offset] as usize;
            desc_type = slice[offset + 1];
        }

        let interface_number = slice[offset + 2];
        return Some((offset, interface_number));
    }
}

pub struct EndpointIterator<'a> {
    buffer_idx: usize,
    index: usize,
    iface_desc: &'a InterfaceDescriptor<'a>,
}

impl<'a> Iterator for EndpointIterator<'a> {
    type Item = EndpointDescriptor;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.iface_desc.num_endpoints as usize {
            None
        } else {
            let mut working_buffer = &self.iface_desc.buffer[self.buffer_idx..];
            if let Some(endpoint) = InterfaceDescriptor::identify_descriptor::<EndpointDescriptor>(working_buffer)
                .map(|i| {
                    working_buffer = &working_buffer[i..];
                    EndpointDescriptor::try_from_bytes(working_buffer).ok()
                })
                .flatten()
            {
                return Some(endpoint);
            }
            self.buffer_idx += EndpointDescriptor::SIZE;
            self.index += 1;
            None
        }
    }
}

/// InterfaceDescriptor does not implement [USBDescriptor] because it has a borrowed buffer.
/// Since we cannot request an interface decriptor from the device by itself it does not strictly need to implement [USBDescriptor].
impl<'a> InterfaceDescriptor<'a> {
    const SIZE: usize = 9;

    const DESC_TYPE: u8 = descriptor_type::INTERFACE;

    fn try_from_bytes(bytes: &'a [u8]) -> Result<Self, ()> {
        if bytes.len() < Self::SIZE {
            return Err(());
        }
        if bytes[1] != Self::DESC_TYPE {
            return Err(());
        }
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
            buffer: &bytes[Self::SIZE..],
        })
    }

    /// Try to parse a class descriptor of a given type
    pub fn parse_class_descriptor<T: USBDescriptor>(&self) -> Option<T> {
        Self::identify_descriptor::<T>(self.buffer)
            .map(|i| T::try_from_bytes(&self.buffer[i..]).ok())
            .flatten()
    }

    pub fn iter_endpoints(&'a self) -> EndpointIterator<'a> {
        EndpointIterator {
            index: 0,
            buffer_idx: 0,
            iface_desc: &self,
        }
    }

    /// Parse up to `L` endpoints corresponding to this interface.
    /// Returns a vector of EndpointDescriptors. The length of the vector is `min(L, self.num_endpoints)`.
    pub fn parse_endpoints<const L: usize>(&self) -> Vec<EndpointDescriptor, L> {
        let mut endpoints: Vec<EndpointDescriptor, L> = Vec::new();

        let mut working_buffer = &self.buffer[..];
        for _ in 0..self.num_endpoints.min(L as u8) {
            if let Some(endpoint) = Self::identify_descriptor::<EndpointDescriptor>(working_buffer)
                .map(|i| {
                    working_buffer = &working_buffer[i..];
                    EndpointDescriptor::try_from_bytes(working_buffer).ok()
                })
                .flatten()
            {
                // safe because we limited the iterations.
                endpoints.push(endpoint).ok();
            }
            working_buffer = &working_buffer[EndpointDescriptor::SIZE..];
        }

        endpoints
    }

    // Returns the offset to the first matching descriptor in the slice
    fn identify_descriptor<T: USBDescriptor>(slice: &[u8]) -> Option<usize> {
        let mut offset = 0;
        let mut desc_len = slice[offset] as usize;
        let mut desc_type = slice[offset + 1];

        while desc_type != T::DESC_TYPE || desc_len != T::SIZE {
            // 'flush' buffer until end of descriptor
            offset += desc_len.max(1); // at least 1 byute to prevent infinite loop
            if offset + T::SIZE > slice.len() {
                // end of slice
                return None;
            }

            desc_len = slice[offset] as usize;
            desc_type = slice[offset + 1];
        }

        return Some(offset);
    }
}

impl USBDescriptor for EndpointDescriptor {
    const SIZE: usize = 7;

    const DESC_TYPE: u8 = descriptor_type::ENDPOINT;
    type Error = ();

    fn try_from_bytes(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() < Self::SIZE {
            return Err(());
        }
        if bytes[1] != Self::DESC_TYPE {
            return Err(());
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

#[cfg(test)]
mod test {
    use super::{ConfigurationDescriptor, USBDescriptor};

    #[test]
    fn test_parse_interface_descriptor() {
        // This configuration descriptor has 2 HID interfaces with HID descriptors
        let desc_bytes = [
            9, 2, 66, 0, 2, 1, 0, 160, 101, 9, 4, 0, 0, 1, 3, 1, 1, 0, 9, 33, 16, 1, 0, 1, 34, 63, 0, 7, 5, 129, 3, 8,
            0, 1, 9, 4, 1, 0, 2, 3, 1, 0, 0, 9, 33, 16, 1, 0, 1, 34, 39, 0, 7, 5, 131, 3, 64, 0, 1, 7, 5, 3, 3, 64, 0,
            1,
        ];

        let cfg = ConfigurationDescriptor::try_from_bytes(desc_bytes.as_slice()).unwrap();
        assert_eq!(cfg.num_interfaces, 2);

        let interface0 = cfg.parse_interface(0).unwrap();
        assert_eq!(interface0.interface_number, 0);

        let interface0_buffer_ref = [9u8, 33, 16, 1, 0, 1, 34, 63, 0, 7, 5, 129, 3, 8, 0, 1];
        assert_eq!(interface0.buffer.len(), interface0_buffer_ref.len());

        let interface1 = cfg.parse_interface(1).unwrap();
        assert_eq!(interface1.interface_number, 1);

        let interface1_buffer_ref = [
            9u8, 33, 16, 1, 0, 1, 34, 39, 0, 7, 5, 131, 3, 64, 0, 1, 7, 5, 3, 3, 64, 0, 1,
        ];
        assert_eq!(interface1.buffer.len(), interface1_buffer_ref.len());
    }

    #[test]
    fn test_parse_endpoint_descriptor() {
        // This configuration descriptor has 2 HID interfaces with HID descriptors
        let desc_bytes = [
            9, 2, 66, 0, 2, 1, 0, 160, 101, 9, 4, 0, 0, 1, 3, 1, 1, 0, 9, 33, 16, 1, 0, 1, 34, 63, 0, 7, 5, 129, 3, 8,
            0, 1, 9, 4, 1, 0, 2, 3, 1, 0, 0, 9, 33, 16, 1, 0, 1, 34, 39, 0, 7, 5, 131, 3, 64, 0, 1, 7, 5, 3, 3, 64, 0,
            1,
        ];

        let cfg = ConfigurationDescriptor::try_from_bytes(desc_bytes.as_slice()).unwrap();
        assert_eq!(cfg.num_interfaces, 2);

        let interface0 = cfg.parse_interface(0).unwrap();
        assert_eq!(interface0.interface_number, 0);

        assert_eq!(interface0.num_endpoints, 1);

        let endpoints = interface0.parse_endpoints::<2>();
        assert_eq!(endpoints.len(), 1);

        let ep = endpoints[0];
        assert_eq!(ep.endpoint_address, 0x81);
        assert_eq!(ep.max_packet_size, 8);

        let interface1 = cfg.parse_interface(1).unwrap();
        assert_eq!(interface1.interface_number, 1);
        assert_eq!(interface1.num_endpoints, 2);

        let endpoints = interface1.parse_endpoints::<2>();
        assert_eq!(endpoints.len(), 2);
    }

    #[test]
    fn test_parse_custom_descriptor() {
        // Define a custom descriptor (HID descriptor in this case)
        struct HIDDescriptor {
            len: u8,
            descriptor_type: u8,
            bcd_hid: u16,
            country_code: u8,
            num_descriptors: u8,
            descriptor_type0: u8,
            descriptor_length0: u16,
        }

        impl USBDescriptor for HIDDescriptor {
            const SIZE: usize = 9; // only valid for 1 descriptor
            const DESC_TYPE: u8 = 33;
            type Error = ();
            fn try_from_bytes(bytes: &[u8]) -> Result<Self, Self::Error> {
                if bytes.len() < Self::SIZE {
                    return Err(());
                }
                if bytes[1] != Self::DESC_TYPE {
                    return Err(());
                }
                Ok(Self {
                    len: bytes[0],
                    descriptor_type: bytes[1],
                    bcd_hid: u16::from_le_bytes([bytes[2], bytes[3]]),
                    country_code: bytes[4],
                    num_descriptors: bytes[5],
                    descriptor_type0: bytes[6],
                    descriptor_length0: u16::from_le_bytes([bytes[7], bytes[8]]),
                })
            }
        }
        // This configuration descriptor has 2 HID interfaces with HID descriptors
        let desc_bytes = [
            9, 2, 66, 0, 2, 1, 0, 160, 101, 9, 4, 0, 0, 1, 3, 1, 1, 0, 9, 33, 16, 1, 0, 1, 34, 63, 0, 7, 5, 129, 3, 8,
            0, 1, 9, 4, 1, 0, 2, 3, 1, 0, 0, 9, 33, 16, 1, 0, 1, 34, 39, 0, 7, 5, 131, 3, 64, 0, 1, 7, 5, 3, 3, 64, 0,
            1,
        ];

        let cfg = ConfigurationDescriptor::try_from_bytes(desc_bytes.as_slice()).unwrap();
        assert_eq!(cfg.num_interfaces, 2);

        let interface0 = cfg.parse_interface(0).unwrap();
        assert_eq!(interface0.interface_number, 0);

        let hid_desc: HIDDescriptor = interface0.parse_class_descriptor().unwrap();

        assert_eq!(hid_desc.len, 9);
        assert_eq!(hid_desc.descriptor_type, 33);

        assert_eq!(hid_desc.bcd_hid, 0x0110);
        assert_eq!(hid_desc.country_code, 0);
        assert_eq!(hid_desc.num_descriptors, 1);
        assert_eq!(hid_desc.descriptor_type0, 34);
        assert_eq!(hid_desc.descriptor_length0, 63);
    }
}
