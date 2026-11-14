//! Utilities for writing USB descriptors.
use embassy_usb_driver::EndpointType;

use crate::driver::{EndpointAddress, EndpointConfig, EndpointInfo};
use crate::types::{InterfaceNumber, StringIndex};
use crate::{CONFIGURATION_VALUE, Config, EndpointOptions, UsbDeviceSpeed};

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
    pub const INTERFACE_POWER: u8 = 8;
    pub const OTG: u8 = 9;
    pub const DEBUG: u8 = 10;
    pub const IAD: u8 = 11;
    pub const BOS: u8 = 15;
    pub const CAPABILITY: u8 = 16;
    pub const SS_ENDPOINT_COMPANION: u8 = 48;
    pub const SSP_ISOCH_ENDPOINT_COMPANION: u8 = 49;
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
    pub const SS_PLUS_USB_DEVICE: u8 = 10;
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

/// USB endpoint interval configuration.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum EndpointInterval {
    /// Interval in milliseconds.
    ///
    /// - At Full Speed:
    ///   - For interrupt endpoints: linear polling period in frames (1..=255 ms). 0 is treated as 1 ms.
    ///   - For isochronous endpoints: exponential period $2^{(bInterval-1)}$ frames (1..=16, representing 1, 2, 4, ... ms).
    /// - At High Speed:
    ///   - For periodic endpoints (interrupt and isochronous): exponential period $2^{(bInterval-1)}$ microframes (125 µs units).
    ///     0 is treated as 1 microframe (125 µs).
    Milliseconds(u8),

    /// Interval in microframes (125 µs units).
    ///
    /// Valid range for high-speed periodic endpoints is 1 to 32768 microframes ($2^{(bInterval-1)}$ microframes, where bInterval is 1..=16).
    /// Non-power-of-two values are rounded up to the next power of two.
    ///
    /// At Full Speed:
    /// - Microframe intervals <= 8 (1 ms) map to 1 frame (1 ms).
    /// - Intervals > 8 map to frames: `(uframes + 7) / 8`.
    Microframes(u32),

    /// Raw `bInterval` byte to write directly to the endpoint descriptor without transformation or rewriting.
    ///
    /// This value will not be modified by full-speed or high-speed descriptor rewriting passes.
    Raw(u8),
}

impl EndpointInterval {
    /// Create an interval in milliseconds.
    pub const fn from_ms(ms: u8) -> Self {
        Self::Milliseconds(ms)
    }

    /// Create an interval in microframes (125 µs units).
    pub const fn from_microframes(microframes: u32) -> Self {
        Self::Microframes(microframes)
    }

    /// Create a raw `bInterval` byte that will not be rewritten.
    pub const fn raw(val: u8) -> Self {
        Self::Raw(val)
    }

    /// Compute the bInterval byte for full-speed operation.
    pub fn to_full_speed_b_interval(&self, ep_type: EndpointType) -> u8 {
        match *self {
            EndpointInterval::Raw(val) => val,
            EndpointInterval::Milliseconds(ms) => match ep_type {
                EndpointType::Isochronous => encode_full_speed_isochronous_interval(ms),
                EndpointType::Interrupt => ms.max(1),
                _ => 0,
            },
            EndpointInterval::Microframes(uframes) => match ep_type {
                EndpointType::Isochronous => {
                    if uframes <= 8 {
                        1
                    } else {
                        let frames = ((uframes + 7) / 8).min(32768) as u16;
                        encode_full_speed_isochronous_frames(frames)
                    }
                }
                EndpointType::Interrupt => {
                    if uframes <= 8 {
                        1
                    } else {
                        let frames = ((uframes + 7) / 8).min(255) as u8;
                        frames.max(1)
                    }
                }
                _ => 0,
            },
        }
    }

    /// Compute the bInterval byte for high-speed operation.
    pub fn to_high_speed_b_interval(&self, ep_type: EndpointType) -> u8 {
        match *self {
            EndpointInterval::Raw(val) => val,
            EndpointInterval::Milliseconds(ms) => match ep_type {
                EndpointType::Interrupt | EndpointType::Isochronous => encode_high_speed_interval(ms),
                _ => 0,
            },
            EndpointInterval::Microframes(uframes) => match ep_type {
                EndpointType::Interrupt | EndpointType::Isochronous => encode_high_speed_microframes(uframes),
                _ => 0,
            },
        }
    }
}

impl From<u8> for EndpointInterval {
    fn from(ms: u8) -> Self {
        Self::Milliseconds(ms)
    }
}

/// `wBytesPerInterval` of a SuperSpeed periodic endpoint that moves one packet per service
/// interval, and 0 for other endpoints.
fn periodic_bytes_per_interval(ep_type: EndpointType, max_packet_size: u16) -> u16 {
    match ep_type {
        EndpointType::Interrupt | EndpointType::Isochronous => max_packet_size & 0x7ff,
        EndpointType::Control | EndpointType::Bulk => 0,
    }
}

/// A writer for USB descriptors.
pub(crate) struct DescriptorWriter<'a> {
    pub buf: &'a mut [u8],
    position: usize,
    num_interfaces_mark: Option<usize>,
    num_endpoints_mark: Option<usize>,
    superspeed: bool,
}

pub(crate) struct DescriptorMark(usize);

impl<'a> DescriptorWriter<'a> {
    pub(crate) fn new(buf: &'a mut [u8]) -> Self {
        DescriptorWriter {
            buf,
            position: 0,
            num_interfaces_mark: None,
            num_endpoints_mark: None,
            superspeed: false,
        }
    }

    pub fn into_buf(self) -> &'a mut [u8] {
        &mut self.buf[..self.position]
    }

    /// Gets the current position in the buffer, i.e. the number of bytes written so far.
    pub const fn position(&self) -> usize {
        self.position
    }

    pub(crate) const fn mark(&self) -> DescriptorMark {
        DescriptorMark(self.position)
    }

    pub(crate) fn patch<R>(&mut self, mark: DescriptorMark, patch: impl FnOnce(&mut [u8]) -> R) -> R {
        patch(&mut self.buf[mark.0..self.position])
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

        let superspeed = config.max_speed >= UsbDeviceSpeed::Super;
        self.superspeed = superspeed;
        let max_power = if superspeed {
            config.max_power.div_ceil(8)
        } else {
            config.max_power / 2
        };

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
                max_power as u8,     // bMaxPower
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
    #[allow(unused)]
    pub fn endpoint(
        &mut self,
        endpoint: &EndpointInfo,
        synchronization_type: SynchronizationType,
        usage_type: UsageType,
        extra_fields: &[u8],
    ) {
        self.endpoint_raw(
            endpoint,
            endpoint.interval,
            synchronization_type,
            usage_type,
            extra_fields,
        );
    }

    /// Writes an endpoint descriptor with a raw `bInterval` byte.
    pub fn endpoint_raw(
        &mut self,
        endpoint: &EndpointInfo,
        b_interval: u8,
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

            let synchronization_bm_attributes: u8 = (synchronization_type as u8) << 2;
            let usage_bm_attributes: u8 = (usage_type as u8) << 4;

            bm_attributes |= usage_bm_attributes | synchronization_bm_attributes;
        }

        self.write(
            descriptor_type::ENDPOINT,
            &[
                endpoint.addr.into(), // bEndpointAddress
                bm_attributes,        // bmAttributes
                endpoint.max_packet_size as u8,
                (endpoint.max_packet_size >> 8) as u8, // wMaxPacketSize
                b_interval,                            // bInterval
            ],
            extra_fields,
        );

        if self.superspeed {
            // `max_streams` of bulk endpoints and `mult` of isochronous endpoints are always 0.
            let bytes_per_interval = periodic_bytes_per_interval(endpoint.ep_type, endpoint.max_packet_size);
            self.write(
                descriptor_type::SS_ENDPOINT_COMPANION,
                &[
                    endpoint.max_burst,
                    0,
                    bytes_per_interval as u8,
                    (bytes_per_interval >> 8) as u8,
                ],
                &[],
            );
        }
    }

    /// The configuration to allocate an endpoint with, with the SuperSpeed fields of the
    /// SuperSpeed Endpoint Companion descriptor this writer gives it.
    pub(crate) fn endpoint_config(
        &self,
        addr: Option<EndpointAddress>,
        ep_type: EndpointType,
        max_packet_size: u16,
        interval: u8,
        options: EndpointOptions,
    ) -> EndpointConfig {
        let mut config = EndpointConfig::new(addr, ep_type, max_packet_size, interval);
        config.buffers = options.buffers;
        if self.superspeed {
            if ep_type == EndpointType::Bulk {
                config.max_burst = options.max_burst;
            }
            config.bytes_per_interval = periodic_bytes_per_interval(ep_type, max_packet_size).into();
        }
        config
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

/// The control endpoint size the driver is started with.
pub(crate) fn control_max_packet_size(config: &Config) -> u16 {
    if config.max_speed >= UsbDeviceSpeed::Super {
        1 << config.max_packet_size_0
    } else {
        config.max_packet_size_0 as u16
    }
}

/// Returns the high-speed `bInterval` (2^(bInterval-1) microframes) for `interval_ms`.
/// 0 is every microframe.
pub(crate) fn encode_high_speed_interval(interval_ms: u8) -> u8 {
    if interval_ms == 0 {
        return 1;
    }
    let microframes = u32::from(interval_ms) * 8;
    let rounded = microframes.next_power_of_two().min(1 << 15);
    let exponent = (31 - rounded.leading_zeros()) as u8;
    exponent + 1
}

/// Returns the high-speed `bInterval` (2^(bInterval-1) microframes) for `microframes`.
/// 0 or 1 is every microframe (bInterval 1).
pub(crate) fn encode_high_speed_microframes(microframes: u32) -> u8 {
    if microframes <= 1 {
        return 1;
    }
    let rounded = microframes.next_power_of_two().min(1 << 15);
    let exponent = (31 - rounded.leading_zeros()) as u8;
    exponent + 1
}

/// Returns the full-speed isochronous `bInterval` (2^(bInterval-1) frames) for `interval_ms`.
/// 0 or 1 is every frame (bInterval 1).
pub(crate) fn encode_full_speed_isochronous_interval(interval_ms: u8) -> u8 {
    encode_full_speed_isochronous_frames(u16::from(interval_ms))
}

/// Returns the full-speed isochronous `bInterval` (2^(bInterval-1) frames) for `frames`.
/// 0 or 1 is every frame (bInterval 1).
pub(crate) fn encode_full_speed_isochronous_frames(frames: u16) -> u8 {
    if frames <= 1 {
        return 1;
    }
    let frames = u32::from(frames);
    let rounded = frames.next_power_of_two().min(1 << 15);
    let exponent = (31 - rounded.leading_zeros()) as u8;
    exponent + 1
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

    /// Writes the device capabilities a SuperSpeed or SuperSpeedPlus device must report.
    pub(crate) fn superspeed_capabilities(&mut self, config: &Config) {
        assert!(
            self.num_caps_mark.is_some(),
            "SuperSpeed devices require a BOS descriptor, but bos_descriptor_buf is too small"
        );

        let mut attributes = 0x00;
        if config.ltm_capable {
            attributes |= 0x02;
        }
        self.capability(
            capability_type::SS_USB_DEVICE,
            &[
                attributes, // bmAttributes
                0x08, 0x00, // wSpeedsSupported
                0x03, // bFunctionalitySupport
                0x0a, // bU1DevExitLat
                0xff, 0x07, // wU2DevExitLat
            ],
        );

        if config.max_speed == UsbDeviceSpeed::SuperPlus {
            self.capability(
                capability_type::SS_PLUS_USB_DEVICE,
                &[
                    0x00, // bReserved
                    0x01, 0x00, 0x00, 0x00, // bmAttributes
                    0x00, 0x11, // wFunctionalitySupport
                    0x00, 0x00, // wReserved
                    0x30, 0x40, 0x0a, 0x00, // bmSublinkSpeedAttr[0]
                    0xb0, 0x40, 0x0a, 0x00, // bmSublinkSpeedAttr[1]
                ],
            );
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn high_speed_interval() {
        assert_eq!(encode_high_speed_interval(0), 1); // 125 us
        assert_eq!(encode_high_speed_interval(1), 4); // 8 microframes
        assert_eq!(encode_high_speed_interval(2), 5);
        assert_eq!(encode_high_speed_interval(3), 6); // rounded up to 32 microframes
        assert_eq!(encode_high_speed_interval(255), 12); // 2048 microframes
    }

    #[test]
    fn high_speed_microframes() {
        assert_eq!(encode_high_speed_microframes(0), 1); // 125 us
        assert_eq!(encode_high_speed_microframes(1), 1); // 125 us
        assert_eq!(encode_high_speed_microframes(2), 2); // 250 us
        assert_eq!(encode_high_speed_microframes(3), 3); // rounded up to 4 microframes (500 us)
        assert_eq!(encode_high_speed_microframes(4), 3); // 500 us
        assert_eq!(encode_high_speed_microframes(8), 4); // 1 ms
        assert_eq!(encode_high_speed_microframes(16), 5); // 2 ms
        assert_eq!(encode_high_speed_microframes(32768), 16); // max 2^15 microframes
        assert_eq!(encode_high_speed_microframes(65536), 16); // clamped to 16
    }

    #[test]
    fn full_speed_isochronous_interval() {
        assert_eq!(encode_full_speed_isochronous_interval(0), 1); // 1 frame
        assert_eq!(encode_full_speed_isochronous_interval(1), 1); // 1 frame
        assert_eq!(encode_full_speed_isochronous_interval(2), 2); // 2 frames
        assert_eq!(encode_full_speed_isochronous_interval(3), 3); // rounded up to 4 frames
        assert_eq!(encode_full_speed_isochronous_interval(4), 3); // 4 frames = 2^(3-1)
        assert_eq!(encode_full_speed_isochronous_interval(8), 4); // 8 frames = 2^(4-1)
        assert_eq!(encode_full_speed_isochronous_interval(16), 5); // 16 frames = 2^(5-1)
        assert_eq!(encode_full_speed_isochronous_interval(255), 9); // 256 frames = 2^(9-1)
    }

    #[test]
    fn endpoint_interval_full_and_high_speed() {
        // Interrupt endpoint
        let ep_int = EndpointInterval::Milliseconds(4);
        assert_eq!(ep_int.to_full_speed_b_interval(EndpointType::Interrupt), 4);
        assert_eq!(ep_int.to_high_speed_b_interval(EndpointType::Interrupt), 6);

        // Isochronous endpoint (exponential for both)
        let ep_iso = EndpointInterval::Milliseconds(4);
        assert_eq!(ep_iso.to_full_speed_b_interval(EndpointType::Isochronous), 3);
        assert_eq!(ep_iso.to_high_speed_b_interval(EndpointType::Isochronous), 6);

        // Microframes endpoint
        let ep_uframes = EndpointInterval::Microframes(2); // 250 us
        assert_eq!(ep_uframes.to_full_speed_b_interval(EndpointType::Interrupt), 1); // <= 8 uframes clamped to 1 frame
        assert_eq!(ep_uframes.to_full_speed_b_interval(EndpointType::Isochronous), 1);
        assert_eq!(ep_uframes.to_high_speed_b_interval(EndpointType::Interrupt), 2);
        assert_eq!(ep_uframes.to_high_speed_b_interval(EndpointType::Isochronous), 2);

        // Raw endpoint
        let ep_raw = EndpointInterval::Raw(7);
        assert_eq!(ep_raw.to_full_speed_b_interval(EndpointType::Interrupt), 7);
        assert_eq!(ep_raw.to_full_speed_b_interval(EndpointType::Isochronous), 7);
        assert_eq!(ep_raw.to_high_speed_b_interval(EndpointType::Interrupt), 7);
        assert_eq!(ep_raw.to_high_speed_b_interval(EndpointType::Isochronous), 7);
    }
}
