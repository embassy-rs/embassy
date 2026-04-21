//! Host driver for HID boot-protocol keyboards.
#![allow(missing_docs)]

use core::num::NonZeroU8;

use bitflags::bitflags;
use embassy_usb_driver::host::{HostError, UsbHostDriver, UsbPipe, pipe};
use embassy_usb_driver::{Direction, EndpointInfo, EndpointType};

use crate::control::ControlPipeExt;
use crate::descriptor::{DEFAULT_MAX_DESCRIPTOR_SIZE, InterfaceDescriptor, USBDescriptor};
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
pub struct KbdHandler<'d, H: UsbHostDriver<'d>> {
    interrupt_channel: H::Pipe<pipe::Interrupt, pipe::In>,
    control_channel: H::Pipe<pipe::Control, pipe::InOut>,
    _phantom: core::marker::PhantomData<&'d ()>,
}

impl<'d, H: UsbHostDriver<'d>> KbdHandler<'d, H> {
    /// Attempt to register a keyboard handler for the given device.
    pub async fn try_register(bus: &H, enum_info: &EnumerationInfo) -> Result<Self, RegisterError> {
        let mut control_channel = bus.alloc_pipe::<pipe::Control, pipe::InOut>(
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
                    v,
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

        let interrupt_channel = bus.alloc_pipe::<pipe::Interrupt, pipe::In>(
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

/// HID class descriptor (type 0x21).
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct HIDDescriptor {
    pub len: u8,
    pub descriptor_type: u8,
    pub bcd_hid: u16,
    pub country_code: u8,
    pub num_descriptors: u8,
    pub descriptor_type0: u8,
    pub descriptor_length0: u16,
}

impl USBDescriptor for HIDDescriptor {
    const SIZE: usize = 9;
    const DESC_TYPE: u8 = 33;
    type Error = ();

    fn try_from_bytes(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() < Self::SIZE || bytes[1] != Self::DESC_TYPE {
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
