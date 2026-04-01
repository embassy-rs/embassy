//! Standard USB control request builders and the `ControlChannelExt` trait.

use core::num::NonZeroU8;

use embassy_time::Timer;
pub use embassy_usb_driver::host::channel;
use embassy_usb_driver::host::{ChannelError, HostError, RequestType, SetupPacket, UsbChannel};
use embassy_usb_driver::{EndpointInfo, EndpointType, Speed};

use crate::descriptor::{USBDescriptor, descriptor_type};
use crate::handler::EnumerationInfo;

/// USB request type direction bit.
const DIR_DEVICE_TO_HOST: u8 = 0x80;

/// USB request type: standard.
const TYPE_STANDARD: u8 = 0x00;

/// USB request type: class.
#[allow(dead_code)]
const TYPE_CLASS: u8 = 0x20;

/// Recipient: device.
const RECIPIENT_DEVICE: u8 = 0x00;

/// Recipient: interface.
const RECIPIENT_INTERFACE: u8 = 0x01;

/// Standard request codes.
pub(crate) const GET_DESCRIPTOR: u8 = 0x06;
pub(crate) const SET_ADDRESS: u8 = 0x05;
pub(crate) const SET_CONFIGURATION: u8 = 0x09;
pub(crate) const GET_CONFIGURATION: u8 = 0x08;
pub(crate) const SET_FEATURE: u8 = 0x03;
pub(crate) const CLEAR_FEATURE: u8 = 0x01;
pub(crate) const GET_STATUS: u8 = 0x00;

/// Build a GET_DESCRIPTOR(Device) SETUP packet.
pub fn get_device_descriptor(max_len: u16) -> [u8; 8] {
    make_setup(
        DIR_DEVICE_TO_HOST | TYPE_STANDARD | RECIPIENT_DEVICE,
        GET_DESCRIPTOR,
        (descriptor_type::DEVICE as u16) << 8,
        0,
        max_len,
    )
}

/// Build a GET_DESCRIPTOR(Configuration) SETUP packet.
pub fn get_config_descriptor(index: u8, max_len: u16) -> [u8; 8] {
    make_setup(
        DIR_DEVICE_TO_HOST | TYPE_STANDARD | RECIPIENT_DEVICE,
        GET_DESCRIPTOR,
        ((descriptor_type::CONFIGURATION as u16) << 8) | index as u16,
        0,
        max_len,
    )
}

/// Build a SET_ADDRESS SETUP packet.
pub fn set_address(address: u8) -> [u8; 8] {
    make_setup(TYPE_STANDARD | RECIPIENT_DEVICE, SET_ADDRESS, address as u16, 0, 0)
}

/// Build a SET_CONFIGURATION SETUP packet.
pub fn set_configuration(config_value: u8) -> [u8; 8] {
    make_setup(
        TYPE_STANDARD | RECIPIENT_DEVICE,
        SET_CONFIGURATION,
        config_value as u16,
        0,
        0,
    )
}

/// Build a GET_DESCRIPTOR(HID Report Descriptor) SETUP packet (Standard, Interface).
///
/// `interface` is the HID interface number; `len` is from `HidInfo::report_descriptor_len`.
pub fn get_hid_report_descriptor(interface: u8, len: u16) -> [u8; 8] {
    // wValue = descriptor_type(0x22) << 8 | index(0)
    make_setup(
        DIR_DEVICE_TO_HOST | TYPE_STANDARD | RECIPIENT_INTERFACE,
        0x06,
        0x2200,
        interface as u16,
        len,
    )
}

/// Build a class-specific interface request SETUP packet (OUT, no data).
pub fn class_interface_out(request: u8, value: u16, interface: u16) -> [u8; 8] {
    make_setup(TYPE_CLASS | RECIPIENT_INTERFACE, request, value, interface, 0)
}

/// Build a class-specific interface request SETUP packet (OUT, with data).
pub fn class_interface_out_with_data(request: u8, value: u16, interface: u16, length: u16) -> [u8; 8] {
    make_setup(TYPE_CLASS | RECIPIENT_INTERFACE, request, value, interface, length)
}

/// Build a class-specific interface request SETUP packet (IN, with data).
pub fn class_interface_in_with_data(request: u8, value: u16, interface: u16, length: u16) -> [u8; 8] {
    make_setup(
        DIR_DEVICE_TO_HOST | TYPE_CLASS | RECIPIENT_INTERFACE,
        request,
        value,
        interface,
        length,
    )
}

fn make_setup(bm_request_type: u8, b_request: u8, w_value: u16, w_index: u16, w_length: u16) -> [u8; 8] {
    let value_bytes = w_value.to_le_bytes();
    let index_bytes = w_index.to_le_bytes();
    let length_bytes = w_length.to_le_bytes();
    [
        bm_request_type,
        b_request,
        value_bytes[0],
        value_bytes[1],
        index_bytes[0],
        index_bytes[1],
        length_bytes[0],
        length_bytes[1],
    ]
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
        let value = ((T::DESC_TYPE as u16) << 8) | index as u16;
        let ty = if class {
            RequestType::TYPE_CLASS
        } else {
            RequestType::TYPE_STANDARD
        };
        let packet = SetupPacket {
            request_type: RequestType::IN | ty | RequestType::RECIPIENT_DEVICE,
            request: GET_DESCRIPTOR,
            value,
            index: 0,
            length: SIZE as u16,
        };
        self.control_in(&packet, &mut buf).await?;
        trace!("Descriptor {}: {:?}", core::any::type_name::<T>(), buf);
        T::try_from_bytes(&buf).map_err(|_| HostError::InvalidDescriptor)
    }

    /// Request the raw bytes of a descriptor by type and index.
    async fn request_descriptor_bytes(&mut self, desc_type: u8, index: u8, buf: &mut [u8]) -> Result<usize, HostError>
    where
        D: channel::IsIn,
    {
        let value = ((desc_type as u16) << 8) | index as u16;
        let packet = SetupPacket {
            request_type: RequestType::IN | RequestType::TYPE_STANDARD | RequestType::RECIPIENT_DEVICE,
            request: GET_DESCRIPTOR,
            value,
            index: 0,
            length: buf.len() as u16,
        };
        self.control_in(&packet, buf).await.map_err(HostError::ChannelError)
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
        let value = (T::DESC_TYPE as u16) << 8;
        let packet = SetupPacket {
            request_type: RequestType::IN | RequestType::TYPE_STANDARD | RequestType::RECIPIENT_INTERFACE,
            request: GET_DESCRIPTOR,
            value,
            index: interface_num as u16,
            length: buf.len() as u16,
        };
        self.control_in(&packet, buf).await.map_err(HostError::ChannelError)
    }

    /// GET_CONFIGURATION — returns the active configuration value, or `None` if unconfigured.
    async fn active_configuration_value(&mut self) -> Result<Option<NonZeroU8>, HostError>
    where
        D: channel::IsIn,
    {
        let packet = SetupPacket {
            request_type: RequestType::IN | RequestType::TYPE_STANDARD | RequestType::RECIPIENT_DEVICE,
            request: GET_CONFIGURATION,
            value: 0,
            index: 0,
            length: 1,
        };
        let mut buf = [0u8; 1];
        self.control_in(&packet, &mut buf).await?;
        Ok(NonZeroU8::new(buf[0]))
    }

    /// SET_CONFIGURATION.
    async fn set_configuration(&mut self, config_no: u8) -> Result<(), HostError>
    where
        D: channel::IsOut,
    {
        let packet = SetupPacket {
            request_type: RequestType::OUT | RequestType::TYPE_STANDARD | RequestType::RECIPIENT_DEVICE,
            request: SET_CONFIGURATION,
            value: config_no as u16,
            index: 0,
            length: 0,
        };
        self.control_out(&packet, &[]).await?;
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
        let packet = SetupPacket {
            request_type: RequestType::OUT | RequestType::TYPE_STANDARD | RequestType::RECIPIENT_DEVICE,
            request: SET_ADDRESS,
            value: new_addr as u16,
            index: 0,
            length: 0,
        };
        self.control_out(&packet, &[]).await?;
        Ok(())
    }

    /// Class + Interface OUT request (no data stage).
    async fn class_request_out(&mut self, request: u8, value: u16, index: u16, buf: &[u8]) -> Result<(), HostError>
    where
        D: channel::IsOut,
    {
        let packet = SetupPacket {
            request_type: RequestType::OUT | RequestType::TYPE_CLASS | RequestType::RECIPIENT_INTERFACE,
            request,
            value,
            index,
            length: buf.len() as u16,
        };
        self.control_out(&packet, buf).await?;
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
