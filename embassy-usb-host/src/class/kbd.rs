//! Host driver for HID boot-protocol keyboards.
#![allow(missing_docs)]

use core::num::NonZeroU8;
use core::ops::Deref;

use bitflags::bitflags;
use embassy_usb_driver::host::{HostError, UsbHostAllocator, UsbPipe, pipe};
use embassy_usb_driver::{Direction, EndpointInfo, EndpointType};

use crate::control::ControlPipeExt;
use crate::descriptor::{
    DEFAULT_MAX_DESCRIPTOR_SIZE, DescriptorError, InterfaceDescriptor, USBDescriptor, VariableSizeDescriptor,
    WritableDescriptor,
};
use crate::handler::{EnumerationInfo, HandlerEvent, RegisterError};

#[repr(C)]
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct KeyStatusUpdate {
    /// Modifier keys bitmask (LeftCtrl, LeftShift, LeftAlt, LeftGUI, RightCtrl, RightShift, RightAlt, RightGUI).
    pub modifiers: u8,
    /// Reserved (OEM).
    pub reserved: u8,
    /// Keycodes of currently pressed keys (0 = not pressed, 1 = rollover).
    pub keypress: [Option<NonZeroU8>; 6],
}

impl KeyStatusUpdate {
    fn from_buffer_unchecked(value: [u8; 8]) -> Self {
        // SAFETY: Option<NonZeroU8> is None when the u8 value is 0.
        unsafe { core::mem::transmute(value) }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum KbdEvent {
    KeyStatusUpdate(KeyStatusUpdate),
}

/// Host-side HID boot-keyboard driver.
pub struct KbdHandler<'d, A: UsbHostAllocator<'d>> {
    interrupt_channel: A::Pipe<pipe::Interrupt, pipe::In>,
    control_channel: A::Pipe<pipe::Control, pipe::InOut>,
    _phantom: core::marker::PhantomData<&'d ()>,
}

impl<'d, A: UsbHostAllocator<'d>> KbdHandler<'d, A> {
    /// Attempt to register a keyboard handler for the given device.
    pub async fn try_register(alloc: &A, enum_info: &EnumerationInfo) -> Result<Self, RegisterError> {
        let mut control_channel = alloc.alloc_pipe::<pipe::Control, pipe::InOut>(
            enum_info.device_address,
            &EndpointInfo {
                addr: 0.into(),
                ep_type: EndpointType::Control,
                max_packet_size: (enum_info.device_desc.max_packet_size0 as u16)
                    .min(enum_info.speed().max_packet_size()),
                interval_ms: 0,
            },
            enum_info.split(),
        )?;

        let mut cfg_desc_buf = [0u8; DEFAULT_MAX_DESCRIPTOR_SIZE];
        let configuration = enum_info
            .active_config_or_set_default(&mut control_channel, &mut cfg_desc_buf)
            .await?;

        let iface = configuration
            .iter_interface()
            .find(|v| {
                matches!(
                    v.deref(),
                    InterfaceDescriptor {
                        interface_class: 0x03,
                        interface_subclass: 0x1,
                        interface_protocol: 0x1,
                        ..
                    }
                )
            })
            .ok_or(RegisterError::NoSupportedInterface)?;

        let interrupt_ep = iface
            .iter_endpoints()
            .find(|v| v.ep_type() == EndpointType::Interrupt && v.ep_dir() == Direction::In)
            .ok_or(RegisterError::NoSupportedInterface)?;

        control_channel
            .set_configuration(configuration.configuration_value)
            .await?;

        let interrupt_channel = alloc.alloc_pipe::<pipe::Interrupt, pipe::In>(
            enum_info.device_address,
            &interrupt_ep.into(),
            enum_info.split(),
        )?;

        debug!("[kbd]: Setting PROTOCOL & idle");
        const SET_PROTOCOL: u8 = 0x0B;
        const BOOT_PROTOCOL: u16 = 0x0000;
        if let Err(err) = control_channel
            .class_request_out(SET_PROTOCOL, BOOT_PROTOCOL, iface.interface_number as u16, &[])
            .await
        {
            error!("[kbd]: Failed to set protocol: {:?}", err);
        }

        const SET_IDLE: u8 = 0x0A;
        if let Err(err) = control_channel
            .class_request_out(SET_IDLE, 0, iface.interface_number as u16, &[])
            .await
        {
            error!("[kbd]: Failed to set idle: {:?}", err);
        }

        Ok(KbdHandler {
            interrupt_channel,
            control_channel,
            _phantom: core::marker::PhantomData,
        })
    }

    /// Wait for the next keyboard event.
    pub async fn wait_for_event(&mut self) -> Result<HandlerEvent<KbdEvent>, HostError> {
        let mut buffer = [0u8; 8];
        debug!("[kbd]: Requesting interrupt IN");
        self.interrupt_channel.request_in(&mut buffer[..]).await?;
        debug!("[kbd]: Got interrupt {:?}", buffer);
        Ok(HandlerEvent::HandlerEvent(KbdEvent::KeyStatusUpdate(
            KeyStatusUpdate::from_buffer_unchecked(buffer),
        )))
    }

    /// SET_REPORT — update keyboard LEDs.
    pub async fn set_state(&mut self, state: &KeyboardState) -> Result<(), HostError> {
        const SET_REPORT: u8 = 0x09;
        const OUTPUT_REPORT: u16 = 2 << 8;
        self.control_channel
            .class_request_out(SET_REPORT, OUTPUT_REPORT, 0, &[state.bits()])
            .await
    }
}

bitflags! {
    /// Keyboard LED state.
    pub struct KeyboardState: u8 {
        const NUM_LOCK    = 1 << 0;
        const CAPS_LOCK   = 1 << 1;
        const SCROLL_LOCK = 1 << 2;
        const COMPOSE     = 1 << 3;
        const KANA        = 1 << 4;
    }
}

/// USB HID class descriptor.
///
/// This descriptor identifies subordinate class descriptors. (USB HID 1.11 §6.2.1)
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct HIDDescriptor {
    /// USB HID specification version that the class descriptors comply to.
    pub bcd_hid: u16,
    /// Country code.
    ///
    /// For country code 0, the hardware is not localized.
    ///
    /// Keyboards may use this field to identify the language of the key caps.
    pub country_code: u8,
    /// Number of class descriptors.
    pub num_descriptors: u8,
    /// Type of the class descriptor 0.
    pub descriptor_type0: u8,
    /// Length of the class descriptor 0.
    pub descriptor_length0: u16,
}

impl HIDDescriptor {
    /// Maximum number of class descriptors that we support (at most 83).
    pub const SUPPORTED_DESCRIPTORS: u8 = 1;
}

impl VariableSizeDescriptor for HIDDescriptor {
    const MIN_LEN: u8 = 6 + 3; // a Report descriptor is always present
    const MAX_LEN: u8 = 6 + (u8::MAX - 6) / 3;

    /// Matches length with the number of class descriptors.
    fn match_bytes_len(bytes: &[u8]) -> bool {
        if bytes.len() < 6 {
            return false;
        }
        let len = bytes[0] as usize;
        let num_descriptors = bytes[5] as usize;
        len == 6 + 3 * num_descriptors
    }
}

impl USBDescriptor for HIDDescriptor {
    const BUF_SIZE: usize = 6 + 3 * Self::SUPPORTED_DESCRIPTORS as usize;
    const DESC_TYPE: u8 = 0x21;
    type Error = DescriptorError;

    fn try_from_bytes(bytes: &[u8]) -> Result<Self, Self::Error> {
        Self::match_bytes(bytes)?;
        let num_descriptors = bytes[5];
        if num_descriptors > Self::SUPPORTED_DESCRIPTORS {
            return Err(DescriptorError::NotImplemented);
        }
        Ok(Self {
            bcd_hid: u16::from_le_bytes([bytes[2], bytes[3]]),
            country_code: bytes[4],
            num_descriptors,
            descriptor_type0: bytes[6],
            descriptor_length0: u16::from_le_bytes([bytes[7], bytes[8]]),
        })
    }
}

impl WritableDescriptor for HIDDescriptor {
    fn write_to_bytes(&self, bytes: &mut [u8]) -> Result<usize, Self::Error> {
        if self.num_descriptors > Self::SUPPORTED_DESCRIPTORS {
            return Err(DescriptorError::NotImplemented);
        }
        Self::prepare_bytes(bytes, 6 + 3 * self.num_descriptors)?;
        [bytes[2], bytes[3]] = self.bcd_hid.to_le_bytes();
        bytes[4] = self.country_code;
        bytes[5] = self.num_descriptors;
        bytes[6] = self.descriptor_type0;
        [bytes[7], bytes[8]] = self.descriptor_length0.to_le_bytes();
        Ok(bytes[0] as usize)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn roundtrip_hid_descriptor() {
        let descriptor = HIDDescriptor {
            bcd_hid: 0x1122,
            country_code: 0x33,
            num_descriptors: 1,
            descriptor_type0: 0x55,
            descriptor_length0: 0x6677,
        };
        let mut bytes = [0u8; HIDDescriptor::BUF_SIZE];
        assert_eq!(descriptor.write_to_bytes(&mut bytes), Ok(6 + 3 * 1));
        assert_eq!(HIDDescriptor::try_from_bytes(&bytes), Ok(descriptor));
    }
}
