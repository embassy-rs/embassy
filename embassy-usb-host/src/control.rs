//! Standard USB control request builders and the `ControlChannelExt` trait.

use core::num::NonZeroU8;

use embassy_time::Timer;
use embassy_usb::control::Request;
pub use embassy_usb_driver::host::channel;
use embassy_usb_driver::host::{ChannelError, HostError, UsbChannel};
use embassy_usb_driver::{Direction, EndpointInfo, EndpointType, Speed};

use crate::descriptor::{USBDescriptor, descriptor_type};
use crate::handler::EnumerationInfo;

/// Recipient of a USB control request.
///
/// This is the 5-bit Recipient sub-field of `bmRequestType`
/// (USB 2.0 spec Table 9-2, bits 4..0). The discriminant of each variant
/// matches the on-wire value.
#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Recipient {
    /// The request is intended for the entire device.
    Device = 0,
    /// The request is intended for an interface.
    Interface = 1,
    /// The request is intended for an endpoint.
    Endpoint = 2,
    /// The recipient of the request is unspecified.
    Other = 3,
    /// Any reserved recipient value (4..=31).
    Reserved = 4,
}

/// Type of a USB control request.
///
/// This is the 2-bit Type sub-field of `bmRequestType`
/// (USB 2.0 spec Table 9-2, bits 6..5). The discriminant of each variant
/// matches the on-wire value.
#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ControlType {
    /// A standard USB request (USB 2.0 spec §9.4).
    Standard = 0,
    /// A class-specific request.
    Class = 1,
    /// A vendor-specific request.
    Vendor = 2,
    /// Reserved.
    Reserved = 3,
}

/// USB control request type (`bmRequestType`).
///
/// Encodes the three sub-fields of `bmRequestType` (USB 2.0 spec Table 9-2):
/// direction (bit 7), type (bits 6..5) and recipient (bits 4..0).
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct RequestType {
    /// Transfer direction (IN = device→host, OUT = host→device).
    pub direction: Direction,

    /// Whether this is a standard, class, vendor, or reserved request.
    pub control_type: ControlType,

    /// Recipient of the request.
    pub recipient: Recipient,
}

impl RequestType {
    /// Encode this request type to its wire-format `bmRequestType` byte.
    pub const fn to_bits(self) -> u8 {
        let d = match self.direction {
            Direction::Out => 0,
            Direction::In => 1 << 7,
        };
        let t = (self.control_type as u8) << 5;
        let r = self.recipient as u8;
        d | t | r
    }

    /// Decode a wire-format `bmRequestType` byte.
    ///
    /// Reserved type values decode to [`ControlType::Reserved`]; reserved
    /// recipient values (4..=31) decode to [`Recipient::Reserved`].
    pub const fn from_bits(b: u8) -> Self {
        let direction = if b & 0x80 != 0 { Direction::In } else { Direction::Out };
        let control_type = match (b >> 5) & 0b11 {
            0 => ControlType::Standard,
            1 => ControlType::Class,
            2 => ControlType::Vendor,
            _ => ControlType::Reserved,
        };
        let recipient = match b & 0b1_1111 {
            0 => Recipient::Device,
            1 => Recipient::Interface,
            2 => Recipient::Endpoint,
            3 => Recipient::Other,
            _ => Recipient::Reserved,
        };
        Self {
            direction,
            control_type,
            recipient,
        }
    }
}

/// USB Control Setup Packet.
///
/// Convenience type for building SETUP packets; serialize with
/// [`SetupPacket::to_bytes`] before passing to a USB driver.
///
/// Setup data format is described in USB spec Table 9-2.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct SetupPacket {
    /// `bmRequestType` - Request characteristics: direction, type, recipient.
    ///
    /// See [`RequestType`] for details.
    pub request_type: RequestType,

    /// `bRequest` - Request code.
    ///
    /// See Table 9-3 of USB spec for standard ones.
    pub request: u8,

    /// `wValue` - Use depending on request field.
    pub value: u16,

    /// `wIndex` - Use depending on request field.
    pub index: u16,

    /// `wLength` - Number of bytes to transfer in data stage if there is one.
    pub length: u16,
}

/// HID class descriptor type: Report (HID 1.11 §7.1.1).
const HID_REPORT_DESCRIPTOR_TYPE: u8 = 0x22;

impl SetupPacket {
    /// Serialize this SETUP packet to its 8-byte wire format.
    ///
    /// Multi-byte fields are emitted in little-endian order, as required by
    /// USB 2.0 spec §8.1.
    pub const fn to_bytes(self) -> [u8; 8] {
        let v = self.value.to_le_bytes();
        let i = self.index.to_le_bytes();
        let l = self.length.to_le_bytes();
        [
            self.request_type.to_bits(),
            self.request,
            v[0],
            v[1],
            i[0],
            i[1],
            l[0],
            l[1],
        ]
    }

    /// Build a GET_DESCRIPTOR SETUP packet delivered to the Device recipient.
    ///
    /// `class` selects Standard (`false`) vs Class (`true`) request type.
    pub const fn get_descriptor(class: bool, desc_type: u8, index: u8, max_len: u16) -> Self {
        Self {
            request_type: RequestType {
                direction: Direction::In,
                control_type: if class {
                    ControlType::Class
                } else {
                    ControlType::Standard
                },
                recipient: Recipient::Device,
            },
            request: Request::GET_DESCRIPTOR,
            value: ((desc_type as u16) << 8) | index as u16,
            index: 0,
            length: max_len,
        }
    }

    /// Build a GET_DESCRIPTOR(Device) SETUP packet.
    pub const fn get_device_descriptor(max_len: u16) -> Self {
        Self::get_descriptor(false, descriptor_type::DEVICE, 0, max_len)
    }

    /// Build a GET_DESCRIPTOR(Configuration) SETUP packet.
    pub const fn get_config_descriptor(index: u8, max_len: u16) -> Self {
        Self::get_descriptor(false, descriptor_type::CONFIGURATION, index, max_len)
    }

    /// Build a standard GET_DESCRIPTOR SETUP packet delivered to an Interface recipient.
    ///
    /// Used for interface-owned descriptors such as the HID Report Descriptor.
    pub const fn get_interface_descriptor(desc_type: u8, interface: u16, max_len: u16) -> Self {
        Self {
            request_type: RequestType {
                direction: Direction::In,
                control_type: ControlType::Standard,
                recipient: Recipient::Interface,
            },
            request: Request::GET_DESCRIPTOR,
            value: (desc_type as u16) << 8,
            index: interface,
            length: max_len,
        }
    }

    /// Build a GET_DESCRIPTOR(HID Report Descriptor) SETUP packet.
    ///
    /// `interface` is the HID interface number; `len` is from `HidInfo::report_descriptor_len`.
    pub const fn get_hid_report_descriptor(interface: u8, len: u16) -> Self {
        Self::get_interface_descriptor(HID_REPORT_DESCRIPTOR_TYPE, interface as u16, len)
    }

    /// Build a SET_ADDRESS SETUP packet.
    pub const fn set_address(address: u8) -> Self {
        Self {
            request_type: RequestType {
                direction: Direction::Out,
                control_type: ControlType::Standard,
                recipient: Recipient::Device,
            },
            request: Request::SET_ADDRESS,
            value: address as u16,
            index: 0,
            length: 0,
        }
    }

    /// Build a SET_CONFIGURATION SETUP packet.
    pub const fn set_configuration(config_value: u8) -> Self {
        Self {
            request_type: RequestType {
                direction: Direction::Out,
                control_type: ControlType::Standard,
                recipient: Recipient::Device,
            },
            request: Request::SET_CONFIGURATION,
            value: config_value as u16,
            index: 0,
            length: 0,
        }
    }

    /// Build a GET_CONFIGURATION SETUP packet.
    pub const fn get_configuration() -> Self {
        Self {
            request_type: RequestType {
                direction: Direction::In,
                control_type: ControlType::Standard,
                recipient: Recipient::Device,
            },
            request: Request::GET_CONFIGURATION,
            value: 0,
            index: 0,
            length: 1,
        }
    }

    /// Build a class-specific interface request SETUP packet, host-to-device.
    ///
    /// Pass `length = 0` for requests with no data stage.
    pub const fn class_interface_out(request: u8, value: u16, interface: u16, length: u16) -> Self {
        Self {
            request_type: RequestType {
                direction: Direction::Out,
                control_type: ControlType::Class,
                recipient: Recipient::Interface,
            },
            request,
            value,
            index: interface,
            length,
        }
    }

    /// Build a class-specific interface request SETUP packet, device-to-host.
    pub const fn class_interface_in(request: u8, value: u16, interface: u16, length: u16) -> Self {
        Self {
            request_type: RequestType {
                direction: Direction::In,
                control_type: ControlType::Class,
                recipient: Recipient::Interface,
            },
            request,
            value,
            index: interface,
            length,
        }
    }
}

// ── ControlChannelExt ──────────────────────────────────────────────────────────

/// Extension trait providing higher-level control request methods on a USB control channel.
pub trait ControlChannelExt<D: channel::Direction>: UsbChannel<channel::Control, D> {
    /// Request and parse a fixed-size descriptor.
    async fn request_descriptor<T: USBDescriptor, const SIZE: usize>(
        &mut self,
        index: u8,
        class: bool,
    ) -> Result<T, HostError>
    where
        D: channel::IsIn,
    {
        let mut buf = [0u8; SIZE];
        let setup = SetupPacket::get_descriptor(class, T::DESC_TYPE, index, SIZE as u16);
        self.control_in(&setup.to_bytes(), &mut buf).await?;
        trace!("Descriptor {}: {:?}", core::any::type_name::<T>(), buf);
        T::try_from_bytes(&buf).map_err(|_| HostError::InvalidDescriptor)
    }

    /// Request the raw bytes of a descriptor by type and index.
    async fn request_descriptor_bytes(&mut self, desc_type: u8, index: u8, buf: &mut [u8]) -> Result<usize, HostError>
    where
        D: channel::IsIn,
    {
        let setup = SetupPacket::get_descriptor(false, desc_type, index, buf.len() as u16);
        self.control_in(&setup.to_bytes(), buf)
            .await
            .map_err(HostError::ChannelError)
    }

    /// Request the raw bytes of a class-specific interface descriptor.
    async fn interface_request_descriptor_bytes<T: USBDescriptor>(
        &mut self,
        interface_num: u8,
        buf: &mut [u8],
    ) -> Result<usize, HostError>
    where
        D: channel::IsIn,
    {
        let setup = SetupPacket::get_interface_descriptor(T::DESC_TYPE, interface_num as u16, buf.len() as u16);
        self.control_in(&setup.to_bytes(), buf)
            .await
            .map_err(HostError::ChannelError)
    }

    /// GET_CONFIGURATION — returns the active configuration value, or `None` if unconfigured.
    async fn active_configuration_value(&mut self) -> Result<Option<NonZeroU8>, HostError>
    where
        D: channel::IsIn,
    {
        let setup = SetupPacket::get_configuration();
        let mut buf = [0u8; 1];
        self.control_in(&setup.to_bytes(), &mut buf).await?;
        Ok(NonZeroU8::new(buf[0]))
    }

    /// SET_CONFIGURATION.
    async fn set_configuration(&mut self, config_no: u8) -> Result<(), HostError>
    where
        D: channel::IsOut,
    {
        let setup = SetupPacket::set_configuration(config_no);
        self.control_out(&setup.to_bytes(), &[]).await?;
        Ok(())
    }

    /// SET_ADDRESS — assign the device a new address.
    ///
    /// # Warning
    /// Breaks host channel state; use only during enumeration.
    async fn device_set_address(&mut self, new_addr: u8) -> Result<(), HostError>
    where
        D: channel::IsOut,
    {
        let setup = SetupPacket::set_address(new_addr);
        self.control_out(&setup.to_bytes(), &[]).await?;
        Ok(())
    }

    /// Class + Interface OUT request (no data stage).
    async fn class_request_out(&mut self, request: u8, value: u16, index: u16, buf: &[u8]) -> Result<(), HostError>
    where
        D: channel::IsOut,
    {
        let setup = SetupPacket::class_interface_out(request, value, index, buf.len() as u16);
        self.control_out(&setup.to_bytes(), buf).await?;
        Ok(())
    }

    /// Enumerate the currently pending device and return an [`EnumerationInfo`].
    ///
    /// The device must have been reset immediately before this call.
    async fn enumerate_device(
        &mut self,
        speed: Speed,
        new_device_address: u8,
        ls_over_fs: bool,
    ) -> Result<EnumerationInfo, HostError>
    where
        D: channel::IsIn + channel::IsOut,
    {
        use crate::descriptor::DeviceDescriptorPartial;

        self.retarget_channel(
            0,
            &EndpointInfo {
                addr: 0.into(),
                ep_type: EndpointType::Control,
                max_packet_size: speed.max_packet_size(),
                interval_ms: 0,
            },
            ls_over_fs,
        )?;

        trace!("[enum] Getting max_packet_size for new device");
        let max_packet_size0 = {
            let mut max_retries = 10;
            loop {
                match self
                    .request_descriptor::<DeviceDescriptorPartial, { DeviceDescriptorPartial::SIZE }>(0, false)
                    .await
                {
                    Ok(desc) => break desc.max_packet_size0,
                    Err(e) => {
                        warn!("Request descriptor error: {:?}, retries: {}", e, max_retries);
                        if max_retries > 0 {
                            max_retries -= 1;
                            Timer::after_millis(1).await;
                            continue;
                        } else {
                            return Err(HostError::RequestFailed);
                        }
                    }
                }
            }
        };
        // USB 2.0 §9.6.1: legal EP0 max packet sizes are 8, 16, 32, 64.
        if !matches!(max_packet_size0, 8 | 16 | 32 | 64) {
            return Err(HostError::InvalidDescriptor);
        }

        self.device_set_address(new_device_address).await?;
        // USB 2.0 §9.2.6.3: allow the device a 2ms recovery interval after SET_ADDRESS.
        Timer::after_millis(2).await;

        self.retarget_channel(
            new_device_address,
            &EndpointInfo {
                addr: 0.into(),
                ep_type: EndpointType::Control,
                max_packet_size: max_packet_size0 as u16,
                interval_ms: 0,
            },
            ls_over_fs,
        )?;

        let retries = 5;
        let device_desc = async {
            for _ in 0..retries {
                match self
                    .request_descriptor::<crate::descriptor::DeviceDescriptor, { crate::descriptor::DeviceDescriptor::SIZE }>(0, false)
                    .await
                {
                    Err(HostError::ChannelError(ChannelError::Timeout)) => {
                        Timer::after_millis(1).await;
                        continue;
                    }
                    v => return v,
                }
            }
            Err(HostError::ChannelError(ChannelError::Timeout))
        }
        .await?;

        trace!("Device Descriptor: {:?}", device_desc);

        Ok(EnumerationInfo {
            device_address: new_device_address,
            ls_over_fs,
            speed,
            device_desc,
        })
    }
}

impl<D: channel::Direction, C> ControlChannelExt<D> for C where C: UsbChannel<channel::Control, D> {}
