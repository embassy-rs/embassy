//! Utilities for writing USB descriptors.
use embassy_usb_driver::EndpointType;

use crate::builder::Config;
use crate::driver::EndpointInfo;
use crate::types::{InterfaceNumber, StringIndex};
use crate::CONFIGURATION_VALUE;

/// Standard descriptor types
#[allow(missing_docs)]
pub mod descriptor_type {
    pub const DEVICE: u8 = 1;
    pub const CONFIGURATION: u8 = 2;
    pub const STRING: u8 = 3;
    pub const INTERFACE: u8 = 4;
    pub const ENDPOINT: u8 = 5;
    pub const DEVICE_QUALIFIER: u8 = 6;
    pub const OTHER_SPEED_CONFIGURATION: u8 = 7;
    pub const IAD: u8 = 11;
    pub const BOS: u8 = 15;
    pub const CAPABILITY: u8 = 16;
}

/// String descriptor language IDs.
pub mod lang_id {
    /// English (US)
    ///
    /// Recommended for use as the first language ID for compatibility.
    pub const ENGLISH_US: u16 = 0x0409;
}

/// Standard capability descriptor types
#[allow(missing_docs)]
pub mod capability_type {
    pub const WIRELESS_USB: u8 = 1;
    pub const USB_2_0_EXTENSION: u8 = 2;
    pub const SS_USB_DEVICE: u8 = 3;
    pub const CONTAINER_ID: u8 = 4;
    pub const PLATFORM: u8 = 5;
}

/// USB endpoint synchronization type. The values of this enum can be directly
/// cast into `u8` to get the bmAttributes synchronization type bits.
/// Values other than `NoSynchronization` are only allowed on isochronous endpoints.
#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SynchronizationType {
    /// No synchronization is used.
    NoSynchronization = 0b00,
    /// Unsynchronized, although sinks provide data rate feedback.
    Asynchronous = 0b01,
    /// Synchronized using feedback or feedforward data rate information.
    Adaptive = 0b10,
    /// Synchronized to the USB’s SOF.
    Synchronous = 0b11,
}

/// USB endpoint usage type. The values of this enum can be directly cast into
/// `u8` to get the bmAttributes usage type bits.
/// Values other than `DataEndpoint` are only allowed on isochronous endpoints.
#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum UsageType {
    /// Use the endpoint for regular data transfer.
    DataEndpoint = 0b00,
    /// Endpoint conveys explicit feedback information for one or more data endpoints.
    FeedbackEndpoint = 0b01,
    /// A data endpoint that also serves as an implicit feedback endpoint for one or more data endpoints.
    ImplicitFeedbackDataEndpoint = 0b10,
    /// Reserved usage type.
    Reserved = 0b11,
}

/// A writer for USB descriptors.
pub(crate) struct DescriptorWriter<'a> {
    pub buf: &'a mut [u8],
    position: usize,
    num_interfaces_mark: Option<usize>,
    num_endpoints_mark: Option<usize>,
}

impl<'a> DescriptorWriter<'a> {
    pub(crate) fn new(buf: &'a mut [u8]) -> Self {
        DescriptorWriter {
            buf,
            position: 0,
            num_interfaces_mark: None,
            num_endpoints_mark: None,
        }
    }

    pub fn into_buf(self) -> &'a mut [u8] {
        &mut self.buf[..self.position]
    }

    /// Gets the current position in the buffer, i.e. the number of bytes written so far.
    pub const fn position(&self) -> usize {
        self.position
    }

    /// Writes an arbitrary (usually class-specific) descriptor with optional extra fields.
    pub fn write(&mut self, descriptor_type: u8, descriptor: &[u8], extra_fields: &[u8]) {
        let descriptor_length = descriptor.len();
        let extra_fields_length = extra_fields.len();
        let total_length = descriptor_length + extra_fields_length;

        assert!(
            (self.position + 2 + total_length) <= self.buf.len() && (total_length + 2) <= 255,
            "Descriptor buffer full"
        );

        self.buf[self.position] = (total_length + 2) as u8;
        self.buf[self.position + 1] = descriptor_type;

        let start = self.position + 2;

        self.buf[start..start + descriptor_length].copy_from_slice(descriptor);
        self.buf[start + descriptor_length..start + total_length].copy_from_slice(extra_fields);

        self.position = start + total_length;
    }

    pub(crate) fn configuration(&mut self, config: &Config) {
        self.num_interfaces_mark = Some(self.position + 4);

        self.write(
            descriptor_type::CONFIGURATION,
            &[
                0,
                0,                   // wTotalLength
                0,                   // bNumInterfaces
                CONFIGURATION_VALUE, // bConfigurationValue
                0,                   // iConfiguration
                0x80 | if config.self_powered { 0x40 } else { 0x00 }
                    | if config.supports_remote_wakeup { 0x20 } else { 0x00 }, // bmAttributes
                (config.max_power / 2) as u8, // bMaxPower
            ],
            &[],
        );
    }

    #[allow(unused)]
    pub(crate) fn end_class(&mut self) {
        self.num_endpoints_mark = None;
    }

    pub(crate) fn end_configuration(&mut self) {
        let position = self.position as u16;
        self.buf[2..4].copy_from_slice(&position.to_le_bytes());
    }

    /// Writes a interface association descriptor. Call from `UsbClass::get_configuration_descriptors`
    /// before writing the USB class or function's interface descriptors if your class has more than
    /// one interface and wants to play nicely with composite devices on Windows. If the USB device
    /// hosting the class was not configured as composite with IADs enabled, calling this function
    /// does nothing, so it is safe to call from libraries.
    ///
    /// # Arguments
    ///
    /// * `first_interface` - Number of the function's first interface, previously allocated with
    ///   [`UsbDeviceBuilder::interface`](crate::bus::UsbDeviceBuilder::interface).
    /// * `interface_count` - Number of interfaces in the function.
    /// * `function_class` - Class code assigned by USB.org. Use `0xff` for vendor-specific devices
    ///   that do not conform to any class.
    /// * `function_sub_class` - Sub-class code. Depends on class.
    /// * `function_protocol` - Protocol code. Depends on class and sub-class.
    pub fn iad(
        &mut self,
        first_interface: InterfaceNumber,
        interface_count: u8,
        function_class: u8,
        function_sub_class: u8,
        function_protocol: u8,
    ) {
        self.write(
            descriptor_type::IAD,
            &[
                first_interface.into(), // bFirstInterface
                interface_count,        // bInterfaceCount
                function_class,
                function_sub_class,
                function_protocol,
                0,
            ],
            &[],
        );
    }

    /// Writes a interface descriptor with a specific alternate setting and
    /// interface string identifier.
    ///
    /// # Arguments
    ///
    /// * `number` - Interface number previously allocated with
    ///   [`UsbDeviceBuilder::interface`](crate::bus::UsbDeviceBuilder::interface).
    /// * `alternate_setting` - Number of the alternate setting
    /// * `interface_class` - Class code assigned by USB.org. Use `0xff` for vendor-specific devices
    ///   that do not conform to any class.
    /// * `interface_sub_class` - Sub-class code. Depends on class.
    /// * `interface_protocol` - Protocol code. Depends on class and sub-class.
    /// * `interface_string` - Index of string descriptor describing this interface

    pub fn interface_alt(
        &mut self,
        number: InterfaceNumber,
        alternate_setting: u8,
        interface_class: u8,
        interface_sub_class: u8,
        interface_protocol: u8,
        interface_string: Option<StringIndex>,
    ) {
        if alternate_setting == 0 {
            match self.num_interfaces_mark {
                Some(mark) => self.buf[mark] += 1,
                None => {
                    panic!("you can only call `interface/interface_alt` after `configuration`.")
                }
            };
        }

        let str_index = interface_string.map_or(0, Into::into);

        self.num_endpoints_mark = Some(self.position + 4);

        self.write(
            descriptor_type::INTERFACE,
            &[
                number.into(),       // bInterfaceNumber
                alternate_setting,   // bAlternateSetting
                0,                   // bNumEndpoints
                interface_class,     // bInterfaceClass
                interface_sub_class, // bInterfaceSubClass
                interface_protocol,  // bInterfaceProtocol
                str_index,           // iInterface
            ],
            &[],
        );
    }

    /// Writes an endpoint descriptor.
    ///
    /// # Arguments
    ///
    /// * `endpoint` - Endpoint previously allocated with
    ///   [`UsbDeviceBuilder`](crate::bus::UsbDeviceBuilder).
    /// * `synchronization_type` - The synchronization type of the endpoint.
    /// * `usage_type` - The usage type of the endpoint.
    /// * `extra_fields` - Additional, class-specific entries at the end of the endpoint descriptor.
    pub fn endpoint(
        &mut self,
        endpoint: &EndpointInfo,
        synchronization_type: SynchronizationType,
        usage_type: UsageType,
        extra_fields: &[u8],
    ) {
        match self.num_endpoints_mark {
            Some(mark) => self.buf[mark] += 1,
            None => panic!("you can only call `endpoint` after `interface/interface_alt`."),
        };

        let mut bm_attributes = endpoint.ep_type as u8;

        // Synchronization types other than `NoSynchronization`,
        // and usage types other than `DataEndpoint`
        // are only allowed for isochronous endpoints.
        if endpoint.ep_type != EndpointType::Isochronous {
            assert_eq!(synchronization_type, SynchronizationType::NoSynchronization);
            assert_eq!(usage_type, UsageType::DataEndpoint);
        } else {
            if usage_type == UsageType::FeedbackEndpoint {
                assert_eq!(synchronization_type, SynchronizationType::NoSynchronization)
            }

            let synchronization_bm_attibutes: u8 = (synchronization_type as u8) << 2;
            let usage_bm_attibutes: u8 = (usage_type as u8) << 4;

            bm_attributes |= usage_bm_attibutes | synchronization_bm_attibutes;
        }

        self.write(
            descriptor_type::ENDPOINT,
            &[
                endpoint.addr.into(), // bEndpointAddress
                bm_attributes,        // bmAttributes
                endpoint.max_packet_size as u8,
                (endpoint.max_packet_size >> 8) as u8, // wMaxPacketSize
                endpoint.interval_ms,                  // bInterval
            ],
            extra_fields,
        );
    }

    /// Writes a string descriptor.
    #[allow(unused)]
    pub(crate) fn string(&mut self, string: &str) {
        let mut pos = self.position;

        assert!(pos + 2 <= self.buf.len(), "Descriptor buffer full");

        self.buf[pos] = 0; // length placeholder
        self.buf[pos + 1] = descriptor_type::STRING;

        pos += 2;

        for c in string.encode_utf16() {
            assert!(pos < self.buf.len(), "Descriptor buffer full");

            self.buf[pos..pos + 2].copy_from_slice(&c.to_le_bytes());
            pos += 2;
        }

        self.buf[self.position] = (pos - self.position) as u8;

        self.position = pos;
    }
}

/// Create a new Device Descriptor array.
///
/// All device descriptors are always 18 bytes, so there's no need for
/// a variable-length buffer or DescriptorWriter.
pub(crate) fn device_descriptor(config: &Config) -> [u8; 18] {
    [
        18,   // bLength
        0x01, // bDescriptorType
        config.bcd_usb as u8,
        (config.bcd_usb as u16 >> 8) as u8, // bcdUSB
        config.device_class,                // bDeviceClass
        config.device_sub_class,            // bDeviceSubClass
        config.device_protocol,             // bDeviceProtocol
        config.max_packet_size_0,           // bMaxPacketSize0
        config.vendor_id as u8,
        (config.vendor_id >> 8) as u8, // idVendor
        config.product_id as u8,
        (config.product_id >> 8) as u8, // idProduct
        config.device_release as u8,
        (config.device_release >> 8) as u8,    // bcdDevice
        config.manufacturer.map_or(0, |_| 1),  // iManufacturer
        config.product.map_or(0, |_| 2),       // iProduct
        config.serial_number.map_or(0, |_| 3), // iSerialNumber
        1,                                     // bNumConfigurations
    ]
}

/// Create a new Device Qualifier Descriptor array.
///
/// All device qualifier descriptors are always 10 bytes, so there's no need for
/// a variable-length buffer or DescriptorWriter.
pub(crate) fn device_qualifier_descriptor(config: &Config) -> [u8; 10] {
    [
        10,   // bLength
        0x06, // bDescriptorType
        config.bcd_usb as u8,
        (config.bcd_usb as u16 >> 8) as u8, // bcdUSB
        config.device_class,                // bDeviceClass
        config.device_sub_class,            // bDeviceSubClass
        config.device_protocol,             // bDeviceProtocol
        config.max_packet_size_0,           // bMaxPacketSize0
        1,                                  // bNumConfigurations
        0,                                  // Reserved
    ]
}

/// A writer for Binary Object Store descriptor.
pub struct BosWriter<'a> {
    pub(crate) writer: DescriptorWriter<'a>,
    num_caps_mark: Option<usize>,
}

impl<'a> BosWriter<'a> {
    pub(crate) const fn new(writer: DescriptorWriter<'a>) -> Self {
        Self {
            writer,
            num_caps_mark: None,
        }
    }

    pub(crate) fn bos(&mut self) {
        if (self.writer.buf.len() - self.writer.position) < 5 {
            return;
        }
        self.num_caps_mark = Some(self.writer.position + 4);
        self.writer.write(
            descriptor_type::BOS,
            &[
                0x00, 0x00, // wTotalLength
                0x00, // bNumDeviceCaps
            ],
            &[],
        );

        self.capability(capability_type::USB_2_0_EXTENSION, &[0; 4]);
    }

    /// Writes capability descriptor to a BOS
    ///
    /// # Arguments
    ///
    /// * `capability_type` - Type of a capability
    /// * `data` - Binary data of the descriptor
    pub fn capability(&mut self, capability_type: u8, data: &[u8]) {
        match self.num_caps_mark {
            Some(mark) => self.writer.buf[mark] += 1,
            None => panic!("called `capability` not between `bos` and `end_bos`."),
        }

        let mut start = self.writer.position;
        let blen = data.len();

        assert!(
            (start + blen + 3) <= self.writer.buf.len() && (blen + 3) <= 255,
            "Descriptor buffer full"
        );

        self.writer.buf[start] = (blen + 3) as u8;
        self.writer.buf[start + 1] = descriptor_type::CAPABILITY;
        self.writer.buf[start + 2] = capability_type;

        start += 3;
        self.writer.buf[start..start + blen].copy_from_slice(data);
        self.writer.position = start + blen;
    }

    pub(crate) fn end_bos(&mut self) {
        if self.writer.position == 0 {
            return;
        }
        self.num_caps_mark = None;
        let position = self.writer.position as u16;
        self.writer.buf[2..4].copy_from_slice(&position.to_le_bytes());
    }
}
