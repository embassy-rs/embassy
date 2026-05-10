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

    pub const INTERFACE_ASSOCIATION: u8 = 0x0B;
    pub const CS_INTERFACE: u8 = 0x24;
    pub const CS_ENDPOINT: u8 = 0x25;
}

/// String descriptor index.
///
/// If the index is 0, then there is no string descriptor for that field.
pub type StringIndex = u8;

/// Maximum descriptor buffer size used during enumeration.
pub(crate) const DEFAULT_MAX_DESCRIPTOR_SIZE: usize = 512;

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum DescriptorError {
    BadDescriptorData,
    BadDescriptorSize,
    BadDescriptorType,
    NotImplemented,
    UnexpectedEndOfBuffer,
}

/// Error returned by [`ConfigurationDescriptor::visit_descriptors`].
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum VisitError<E> {
    /// An interface or endpoint descriptor in the configuration buffer was malformed.
    BadDescriptor,
    /// The visitor itself returned an error.
    Visitor(E),
}

/// Trait for a USB descriptor that can be parsed from a byte slice.
pub trait USBDescriptor {
    /// Size of the byte buffer.
    ///
    /// This is the size of the byte buffer that should be used to read or write the descriptor.
    /// This is not the size of the descriptor.
    const BUF_SIZE: usize;

    /// Descriptor type.
    ///
    /// This constant is compared against byte 1 of the buffer while reading.
    const DESC_TYPE: u8;

    /// Descriptor subtype.
    ///
    /// If this constant is `None`, then it is ignored.
    /// If this constant is `Some(subtype)`, then `subtype` is compared against byte 2 of the buffer while reading.
    ///
    /// This constant is `None` by default.
    const DESC_SUBTYPE: Option<u8> = None;

    type Error;
    fn try_from_bytes(bytes: &[u8]) -> Result<Self, Self::Error>
    where
        Self: Sized;
}

/// Fixed size descriptor.
///
/// Implementors of this trait only allow the correct size while reading or writing.
///
/// If you are not sure that the length is always the same, implement [ExtendableDescriptor] instead.
pub trait FixedSizeDescriptor: USBDescriptor {
    /// Length of the descriptor.
    ///
    /// This constant is compared against byte 0 of the buffer.
    const LEN: u8;

    /// Matches `bytes` with this descriptor.
    ///
    /// On success it returns `Ok(())`.
    /// On error it returns a [DescriptorError].
    #[inline(always)]
    fn match_bytes(bytes: &[u8]) -> Result<(), DescriptorError> {
        if bytes.len() < Self::LEN as usize {
            Err(DescriptorError::UnexpectedEndOfBuffer)
        } else if bytes[0] != Self::LEN {
            Err(DescriptorError::BadDescriptorSize)
        } else if bytes[1] != Self::DESC_TYPE {
            Err(DescriptorError::BadDescriptorType)
        } else if let Some(subtype) = Self::DESC_SUBTYPE
            && bytes[2] != subtype
        {
            Err(DescriptorError::BadDescriptorType)
        } else {
            Ok(())
        }
    }
}

/// Extendable fixed size descriptor.
///
/// Implementors of this trait allow extra bytes in the descriptor while reading or writing.
/// The origin and purpose of the extra bytes is undefined, it might be a class extension,
/// a vendor extension, or anything else.
pub trait ExtendableDescriptor: USBDescriptor {
    /// Minimum length of the descriptor.
    ///
    /// This value is compared against byte 0 of the buffer.
    /// All bytes after this length are considered an extension of the descriptor.
    const MIN_LEN: u8;

    /// Matches `bytes` with this descriptor.
    ///
    /// On success it returns `Ok(())`.
    /// On error it returns a [DescriptorError].
    #[inline(always)]
    fn match_bytes(bytes: &[u8]) -> Result<(), DescriptorError> {
        if bytes.len() < Self::MIN_LEN as usize {
            Err(DescriptorError::UnexpectedEndOfBuffer)
        } else if bytes[0] < Self::MIN_LEN {
            Err(DescriptorError::BadDescriptorSize)
        } else if bytes[1] != Self::DESC_TYPE {
            Err(DescriptorError::BadDescriptorType)
        } else if let Some(subtype) = Self::DESC_SUBTYPE
            && bytes[2] != subtype
        {
            Err(DescriptorError::BadDescriptorType)
        } else {
            Ok(())
        }
    }
}

/// Variable size descriptor.
///
/// Implementors of this trait accept multiple sizes while reading or writing.
///
/// The minimum length and the maximum length restrictions are always checked.
/// Other restrictions should be implemented in [match_bytes_len](Self::match_bytes_len).
pub trait VariableSizeDescriptor: USBDescriptor {
    /// Minimum length of the descriptor.
    ///
    /// This constant is compared against byte 0 of the buffer.
    const MIN_LEN: u8;

    /// Maximum length of the descriptor.
    ///
    /// This constant is compared against byte 0 of the buffer.
    const MAX_LEN: u8;

    /// Matches `bytes` with this descriptor.
    ///
    /// On success it returns `Ok(())`.
    /// On error it returns a [DescriptorError].
    #[inline(always)]
    fn match_bytes(bytes: &[u8]) -> Result<(), DescriptorError> {
        if bytes.len() < Self::MIN_LEN as usize {
            Err(DescriptorError::UnexpectedEndOfBuffer)
        } else if !(Self::MIN_LEN..=Self::MAX_LEN).contains(&bytes[0]) || !Self::match_bytes_len(bytes) {
            Err(DescriptorError::BadDescriptorSize)
        } else if bytes[1] != Self::DESC_TYPE {
            Err(DescriptorError::BadDescriptorType)
        } else if let Some(subtype) = Self::DESC_SUBTYPE
            && bytes[2] != subtype
        {
            Err(DescriptorError::BadDescriptorType)
        } else {
            Ok(())
        }
    }

    /// Matches additional restrictions of the length.
    ///
    /// By default, there are no additional restrictions.
    #[inline(always)]
    fn match_bytes_len(_bytes: &[u8]) -> bool {
        true
    }
}

/// Partial version of [DeviceDescriptor].
///
/// This descriptor is used to read `max_packet_size0` before SET_ADDRESS.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DeviceDescriptorPartial {
    pub max_packet_size0: u8,
}

impl ExtendableDescriptor for DeviceDescriptorPartial {
    const MIN_LEN: u8 = DeviceDescriptor::MIN_LEN;
}

impl USBDescriptor for DeviceDescriptorPartial {
    // `max_packet_size0` is at byte 7.
    const BUF_SIZE: usize = 8;
    const DESC_TYPE: u8 = DeviceDescriptor::DESC_TYPE;
    type Error = DescriptorError;

    fn try_from_bytes(buf: &[u8]) -> Result<Self, Self::Error> {
        Self::match_bytes(buf)?;
        Ok(Self {
            max_packet_size0: buf[7],
        })
    }
}

/// Standard USB Device Descriptor.
///
/// Each USB device has exactly one device descriptor, which contains information that
/// applies globally to the device and all of it's configurations (USB 2.0 §9.6.1).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct DeviceDescriptor {
    /// USB specification version that the device and it's descriptors comply to.
    pub bcd_usb: u16,
    /// Class code.
    ///
    /// If the class is 0, then each configuration interface has an independent class code.
    ///
    /// If the class is 0xff, then the device class is vendor-specific.
    pub device_class: u8,
    /// Subclass code.
    ///
    /// If the class is 0, then the subclass must be 0.
    ///
    /// If the subclass is 0xff, then the device subclass is vendor-specific.
    pub device_subclass: u8,
    /// Protocol code.
    ///
    /// If the protocol is 0, then there is no class-specific device protocol.
    /// However, individual interfaces may still use a class-specific protocol.
    ///
    /// If the protocol is 0xff, then the device protocol is vendor-specific.
    pub device_protocol: u8,
    /// Maximum packet size for endpoint 0.
    ///
    /// For USB 2.0, the only valid sizes are 8, 16, 32, 64.
    /// For USB 3.2, this value is a 2-based exponent.
    pub max_packet_size0: u8,
    /// Vendor ID.
    pub vendor_id: u16,
    /// Product ID.
    pub product_id: u16,
    /// Device version.
    pub bcd_device: u16,
    /// Manufacturer string.
    pub manufacturer: StringIndex,
    /// Product string.
    pub product: StringIndex,
    /// Serial number string.
    pub serial_number: StringIndex,
    /// Number of possible configurations.
    pub num_configurations: u8,
}

impl ExtendableDescriptor for DeviceDescriptor {
    const MIN_LEN: u8 = 18;
}

impl USBDescriptor for DeviceDescriptor {
    const BUF_SIZE: usize = Self::MIN_LEN as usize;
    const DESC_TYPE: u8 = descriptor_type::DEVICE;
    type Error = DescriptorError;

    fn try_from_bytes(bytes: &[u8]) -> Result<Self, Self::Error> {
        Self::match_bytes(bytes)?;
        Ok(Self {
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

/// Standard USB Configuration Descriptor.
///
/// When a configuration descriptor is requested, all related descriptors are returned. (USB 2.0 §9.6.3)
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ConfigurationDescriptor {
    /// Total length of data returned for this configuration.
    ///
    /// The data includes this descriptor, interface descriptors,
    /// endpoint descriptors, and possibly other descriptors.
    pub total_len: u16,
    /// Number of interface descriptors.
    pub num_interfaces: u8,
    /// Configuration ID.
    pub configuration_value: u8,
    /// Configuration string.
    pub configuration_name: StringIndex,
    /// Configuration attribute bitmap.
    pub attributes: u8,
    /// Maximum bus power that will be consumed in 2mA units.
    pub max_power: u8,
}

impl ExtendableDescriptor for ConfigurationDescriptor {
    const MIN_LEN: u8 = 9;
}

impl USBDescriptor for ConfigurationDescriptor {
    const BUF_SIZE: usize = Self::MIN_LEN as usize;
    const DESC_TYPE: u8 = descriptor_type::CONFIGURATION;
    type Error = DescriptorError;

    fn try_from_bytes(bytes: &[u8]) -> Result<Self, Self::Error> {
        Self::match_bytes(bytes)?;
        Ok(Self {
            total_len: u16::from_le_bytes([bytes[2], bytes[3]]),
            num_interfaces: bytes[4],
            configuration_value: bytes[5],
            configuration_name: bytes[6],
            attributes: bytes[7],
            max_power: bytes[8],
        })
    }
}

/// A chain of descriptors.
///
/// Holds the current descriptor and a reference to the bytes after the descriptor.
/// Deferences to the descriptor.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DescriptorChain<'a, T> {
    /// The current descriptor.
    pub descriptor: T,
    /// The raw bytes following the descriptor.
    pub buffer: &'a [u8],
}

impl<T> Copy for DescriptorChain<'_, T> where T: Copy {}

#[cfg(feature = "defmt")]
impl<T> defmt::Format for DescriptorChain<'_, T>
where
    T: defmt::Format,
{
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "DescriptorChain<{}>{{descriptor: {}, buffer: {=[u8]}}}",
            core::any::type_name::<T>(),
            self.descriptor,
            self.buffer
        )
    }
}

impl<'a, T> core::ops::Deref for DescriptorChain<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.descriptor
    }
}

impl<'a, T> core::ops::DerefMut for DescriptorChain<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.descriptor
    }
}
/// The chain of descriptors of a [ConfigurationDescriptor].
///
/// When you request the configuration descriptor of a usb device you get a chain of descriptors (USB 2.0 §9.6.3).
/// The chain includes interface descriptors, endpoint descriptors, and possibly other descriptors.
///
/// The total length of the chain is in [ConfigurationDescriptor::total_len].
pub type ConfigurationDescriptorChain<'a> = DescriptorChain<'a, ConfigurationDescriptor>;

impl<'a> ConfigurationDescriptorChain<'a> {
    /// Parse a full Configuration Descriptor blob, giving access to sub-descriptors via iterators.
    pub fn try_from_slice(buf: &'a [u8]) -> Result<Self, HostError> {
        let descriptor = ConfigurationDescriptor::try_from_bytes(buf).map_err(|_| HostError::InvalidDescriptor)?;
        if let Some(buffer) = buf.get(buf[0] as usize..descriptor.total_len as usize) {
            Ok(Self { descriptor, buffer })
        } else {
            Err(HostError::InvalidDescriptor)
        }
    }

    /// Iterate over all raw descriptors in this Configuration.
    pub fn iter_descriptors(&self) -> RawDescriptorIterator<'a> {
        RawDescriptorIterator::new(self.buffer)
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

    /// Iterate over all descriptors of this Configuration, passing to Visitor callbacks.
    /// Returns `Ok(())` on completion (including early stop), or `Err(e)` on error.
    pub fn visit_descriptors<V: DescriptorVisitor<'a>>(&self, visitor: &mut V) -> Result<(), VisitError<V::Error>> {
        if !visitor.on_configuration(self) {
            return Ok(());
        }
        let mut current_iface: Option<InterfaceDescriptorChain<'a>> = None;
        for (_, bytes) in self.iter_descriptors() {
            if bytes.len() < 2 {
                continue;
            }
            match bytes[1] {
                descriptor_type::INTERFACE => {
                    let iface =
                        InterfaceDescriptorChain::try_from_bytes(bytes).map_err(|_| VisitError::BadDescriptor)?;
                    current_iface = Some(iface);
                    if !visitor.on_interface(&iface) {
                        return Ok(());
                    }
                }
                descriptor_type::ENDPOINT => {
                    let ep = EndpointDescriptor::try_from_bytes(bytes).map_err(|_| VisitError::BadDescriptor)?;
                    if let Some(iface) = current_iface.as_ref() {
                        if !visitor.on_endpoint(iface, &ep) {
                            return Ok(());
                        }
                    }
                }
                _ => {
                    if !visitor
                        .on_other(current_iface.as_ref(), bytes)
                        .map_err(VisitError::Visitor)?
                    {
                        return Ok(());
                    }
                }
            }
        }
        Ok(())
    }
}

/// Callback-based visitor for a configuration's descriptor tree.
///
/// Implement only the methods you care about.
pub trait DescriptorVisitor<'a> {
    type Error;

    /// Return `false` to stop iteration early
    fn on_configuration(&mut self, _c: &ConfigurationDescriptor) -> bool {
        true
    }

    /// Return `false` to stop iteration early
    fn on_interface(&mut self, _i: &InterfaceDescriptorChain<'a>) -> bool {
        true
    }

    /// Return `false` to stop iteration early
    fn on_endpoint(&mut self, _iface: &InterfaceDescriptorChain<'a>, _e: &EndpointDescriptor) -> bool {
        true
    }

    /// Catches every sub-descriptor that isn't an interface or endpoint:
    /// CS_INTERFACE, CS_ENDPOINT, HID, vendor-specific, etc.
    /// Return `Ok(false)` to stop iteration early without an error, or `Err(e)` to stop with one.
    fn on_other(&mut self, _iface: Option<&InterfaceDescriptorChain<'a>>, _raw: &[u8]) -> Result<bool, Self::Error> {
        Ok(true)
    }
}

/// [`A DescriptorVisitor`] that just logs the descriptors to the debug stream
pub struct ShowDescriptors;

impl<'a> DescriptorVisitor<'a> for ShowDescriptors {
    type Error = core::convert::Infallible;

    fn on_configuration(&mut self, c: &ConfigurationDescriptor) -> bool {
        debug!("{:?}", c);
        true
    }
    fn on_interface(&mut self, i: &InterfaceDescriptorChain) -> bool {
        debug!("  {:?}", i);
        true
    }
    fn on_endpoint(&mut self, _i: &InterfaceDescriptorChain, e: &EndpointDescriptor) -> bool {
        debug!("    {:?}", e);
        true
    }
    fn on_other(&mut self, _i: Option<&InterfaceDescriptorChain>, d: &[u8]) -> Result<bool, Self::Error> {
        let dlen = d[0];
        let dtype = d[1];
        let domain = match dtype & 0x60 {
            0x00 => "standard",
            0x20 => "class",
            0x40 => "vendor",
            _ => "reserved",
        };
        debug!("  {} type 0x{:02X} len {}", domain, dtype, dlen);
        Ok(true)
    }
}

/// Standard USB Interface Descriptor.
///
/// A configuration provides one or more interfaces. (USB 2.0 §9.6.5)
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct InterfaceDescriptor {
    /// Interface index in this configuration (0-based).
    pub interface_number: u8,
    /// Alternate setting ID of this interface.
    pub alternate_setting: u8,
    /// Number of endpoints used by this interface.
    pub num_endpoints: u8,
    /// Class code.
    ///
    /// If the class is 0, then the behavior is undefined (value is reserved).
    ///
    /// If the class is 0xff, then the interface class is vendor-specific.
    pub interface_class: u8,
    /// Subclass code.
    ///
    /// If the class is 0, then the subclass must be 0.
    ///
    /// If the subclass is 0xff, then the interface subclass is vendor-specific.
    pub interface_subclass: u8,
    /// Protocol code.
    ///
    /// If the protocol is 0, then there is no class-specific interface protocol.
    ///
    /// If the protocol is 0xff, then the interface protocol is vendor-specific.
    pub interface_protocol: u8,
    /// Interface string.
    pub interface_name: StringIndex,
}

impl ExtendableDescriptor for InterfaceDescriptor {
    const MIN_LEN: u8 = 9;
}

impl USBDescriptor for InterfaceDescriptor {
    const BUF_SIZE: usize = Self::MIN_LEN as usize;
    const DESC_TYPE: u8 = descriptor_type::INTERFACE;
    type Error = DescriptorError;

    fn try_from_bytes(bytes: &[u8]) -> Result<Self, Self::Error> {
        Self::match_bytes(bytes)?;
        Ok(Self {
            interface_number: bytes[2],
            alternate_setting: bytes[3],
            num_endpoints: bytes[4],
            interface_class: bytes[5],
            interface_subclass: bytes[6],
            interface_protocol: bytes[7],
            interface_name: bytes[8],
        })
    }
}

/// The chain of descriptors of a [InterfaceDescriptor].
///
/// A [ConfigurationDescriptorChain] provides one or more interface descriptors (USB 2.0 §9.6.5).
/// Each interface chain includes endpoint descriptors, and possibly other descriptors.
///
/// The buffer goes up to the next interface descriptor.
pub type InterfaceDescriptorChain<'a> = DescriptorChain<'a, InterfaceDescriptor>;

impl<'a> InterfaceDescriptorChain<'a> {
    pub(crate) fn try_from_bytes(bytes: &'a [u8]) -> Result<Self, DescriptorError> {
        let descriptor = InterfaceDescriptor::try_from_bytes(bytes)?;
        if let Some(endpoints) = bytes.get(bytes[0] as usize..) {
            let mut next_iface_index = endpoints.len();
            for (index, bytes) in RawDescriptorIterator::new(endpoints) {
                if bytes.get(1) == Some(&InterfaceDescriptor::DESC_TYPE) {
                    next_iface_index = index;
                    break;
                }
            }
            // up to the next interface descriptor
            let buffer = &endpoints[..next_iface_index];
            Ok(Self { descriptor, buffer })
        } else {
            Err(DescriptorError::UnexpectedEndOfBuffer)
        }
    }

    /// Iterate over raw descriptors inside this interface.
    pub fn iter_descriptors(&self) -> RawDescriptorIterator<'_> {
        RawDescriptorIterator::new(self.buffer)
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

impl<'a> From<&InterfaceDescriptorChain<'a>> for InterfaceDescriptor {
    fn from(chain: &InterfaceDescriptorChain<'a>) -> Self {
        chain.descriptor
    }
}

/// Iterates over the InterfaceDescriptors of a configuration.
pub struct InterfaceIterator<'a> {
    offset: usize,
    cfg_desc: &'a ConfigurationDescriptorChain<'a>,
}

impl<'a> Iterator for InterfaceIterator<'a> {
    type Item = InterfaceDescriptorChain<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset >= self.cfg_desc.buffer.len() {
            return None;
        }
        let remaining = &self.cfg_desc.buffer[self.offset..];
        let iface = InterfaceDescriptorChain::try_from_bytes(remaining).ok()?;
        self.offset += remaining[0] as usize + iface.buffer.len();
        Some(iface)
    }
}

/// Iterates over raw descriptors, yielding `(byte_offset, &[u8])`.
pub struct RawDescriptorIterator<'a> {
    buf: &'a [u8],
    offset: usize,
}

impl<'a> RawDescriptorIterator<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Self { buf, offset: 0 }
    }
}

impl<'a> Iterator for RawDescriptorIterator<'a> {
    type Item = (usize, &'a [u8]);

    fn next(&mut self) -> Option<Self::Item> {
        let offset = self.offset;
        if let Some(&len) = self.buf.get(offset)
            && len > 0
        {
            self.offset += len as usize;
            if let Some(bytes) = self.buf.get(offset..self.offset) {
                return Some((offset, bytes));
            }
        }
        None
    }
}

/// Iterates over the endpoint descriptors of an interface.
pub struct EndpointIterator<'a> {
    buffer_idx: usize,
    index: usize,
    iface_desc: &'a InterfaceDescriptorChain<'a>,
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

/// Standard USB Endpoint Descriptor.
///
/// Contains information to determine the bandwidth requirements. (USB 2.0 §9.6.6)
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct EndpointDescriptor {
    /// Endpoint address.
    ///
    /// Contains the endpoint number and direction.
    pub endpoint_address: u8,
    /// Endpoint attribute bitmap.
    pub attributes: u8,
    /// Maximum packet size (11 bits).
    ///
    /// For high-speed isochronous and interrupt endpoints,
    /// it also specifies aditional transaction opportunities.
    pub max_packet_size: u16,
    /// Polling interval.
    ///
    /// The meaning of this value depends on the transfer type and speed.
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

impl ExtendableDescriptor for EndpointDescriptor {
    const MIN_LEN: u8 = 7;
}

impl USBDescriptor for EndpointDescriptor {
    const BUF_SIZE: usize = Self::MIN_LEN as usize;
    const DESC_TYPE: u8 = descriptor_type::ENDPOINT;
    type Error = DescriptorError;

    fn try_from_bytes(bytes: &[u8]) -> Result<Self, Self::Error> {
        Self::match_bytes(bytes)?;
        Ok(Self {
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

    use super::*;

    struct TestInterface<'a> {
        interface: InterfaceDescriptorChain<'a>,
        endpoints: Vec<EndpointDescriptor, 4>,
    }

    const MAX_INTERFACES: usize = 4;
    const MAX_DESCRIPTOR_SIZE: usize = 256;
    const MAX_OTHERS: usize = 8;

    struct TestVisitor<'a> {
        configuration: Option<ConfigurationDescriptor>,
        interfaces: Vec<TestInterface<'a>, MAX_INTERFACES>,
        others: Vec<Vec<u8, MAX_DESCRIPTOR_SIZE>, MAX_OTHERS>,
    }

    impl<'a> Default for TestVisitor<'a> {
        fn default() -> Self {
            Self {
                configuration: None,
                interfaces: Vec::new(),
                others: Vec::new(),
            }
        }
    }

    impl<'a> DescriptorVisitor<'a> for TestVisitor<'a> {
        type Error = core::convert::Infallible;

        fn on_configuration(&mut self, c: &ConfigurationDescriptor) -> bool {
            assert!(self.configuration.is_none());
            self.configuration = Some(*c);
            true
        }

        fn on_interface(&mut self, i: &InterfaceDescriptorChain<'a>) -> bool {
            assert!(self.configuration.is_some());
            let _ = self.interfaces.push(TestInterface {
                interface: *i,
                endpoints: Vec::new(),
            });
            true
        }

        fn on_endpoint(&mut self, _iface: &InterfaceDescriptorChain<'a>, e: &EndpointDescriptor) -> bool {
            assert!(!self.interfaces.is_empty());
            let _ = self.interfaces.last_mut().unwrap().endpoints.push(*e);
            true
        }

        fn on_other(&mut self, _iface: Option<&InterfaceDescriptorChain<'a>>, d: &[u8]) -> Result<bool, Self::Error> {
            assert!(self.configuration.is_some());
            let _ = self.others.push(Vec::from_slice(d).unwrap_or_default());
            Ok(true)
        }
    }

    #[test]
    fn test_parse_extended_endpoint_descriptor() {
        let desc_bytes = [
            9, 2, 76, 0, 2, 1, 0, 160, 101, 8, 11, 0, 1, 3, 0, 0, 0, 9, 4, 0, 0, 1, 3, 1, 1, 0, 9, 33, 16, 1, 0, 1, 34,
            63, 0, 9, 5, 129, 3, 8, 0, 1, 99, 99, 9, 4, 1, 0, 2, 3, 1, 0, 0, 9, 33, 16, 1, 0, 1, 34, 39, 0, 7, 5, 131,
            3, 64, 0, 1, 7, 5, 3, 3, 64, 0, 1,
        ];

        let cfg = ConfigurationDescriptorChain::try_from_slice(desc_bytes.as_slice()).unwrap();
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

        let cfg = ConfigurationDescriptorChain::try_from_slice(desc_bytes.as_slice()).unwrap();
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

    #[test]
    fn test_parse_visit_midi_descriptor() {
        let _ = env_logger::builder().is_test(true).try_init();

        let desc_bytes = [
            9, 2, 101, 0, 2, 1, 0, 128, 50, 9, 4, 0, 0, 0, 1, 1, 0, 0, 9, 36, 1, 0, 1, 9, 0, 1, 1, 9, 4, 1, 0, 2, 1, 3,
            0, 0, 7, 36, 1, 0, 1, 65, 0, 6, 36, 2, 1, 1, 0, 6, 36, 2, 2, 2, 0, 9, 36, 3, 1, 3, 1, 2, 1, 0, 9, 36, 3, 2,
            4, 1, 1, 1, 0, 9, 5, 2, 2, 32, 0, 0, 0, 0, 5, 37, 1, 1, 1, 9, 5, 129, 2, 32, 0, 0, 0, 0, 5, 37, 1, 1, 3,
        ];

        let cfg = ConfigurationDescriptorChain::try_from_slice(desc_bytes.as_slice()).unwrap();
        assert_eq!(cfg.num_interfaces, 2);

        let mut v = TestVisitor::default();
        cfg.visit_descriptors(&mut v).unwrap();

        assert!(v.configuration.is_some());
        assert_eq!(cfg.num_interfaces, 2);
        assert_eq!(v.interfaces.len(), 2);
        assert_eq!(v.interfaces[0].interface.interface_class, 1);
        assert_eq!(v.interfaces[0].endpoints.len(), 0);
        assert_eq!(v.interfaces[1].endpoints.len(), 2);
        assert_eq!(v.interfaces[1].endpoints[0].attributes, 2);
        assert_eq!(v.interfaces[1].endpoints[0].endpoint_address, 0x02);
        assert_eq!(v.interfaces[1].endpoints[1].endpoint_address, 0x81);
        assert_eq!(v.others.len(), 8);

        let mut sv = ShowDescriptors {};
        cfg.visit_descriptors(&mut sv).unwrap();
    }
}
