use super::{EnumerationInfo, HandlerEvent, RegisterError, UsbHostHandler};
use crate::{
    control::Request,
    host::{
        descriptor::{
            ConfigurationDescriptor, DeviceDescriptor, DeviceDescriptorPartial, InterfaceDescriptor, USBDescriptor,
        },
        ControlChannelExt,
    },
};
use core::num::NonZeroU8;

use bitflags::Flags;
use embassy_time::Timer;
use embassy_usb_driver::{
    host::{channel, HostError, RequestType, SetupPacket, UsbChannel, UsbHostDriver},
    Direction, EndpointInfo, EndpointType, Speed,
};
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum KbdEvent {
    KeyPressUpdate([Option<NonZeroU8>; 8]),
}

pub struct KbdHandler<H: UsbHostDriver> {
    interrupt_channel: H::Channel<channel::Interrupt, channel::In>,
    control_channel: H::Channel<channel::Control, channel::InOut>,
}

impl<H: UsbHostDriver> UsbHostHandler for KbdHandler<H> {
    type PollEvent = KbdEvent;
    type Driver = H;

    fn static_spec() -> super::StaticHandlerSpec {
        super::StaticHandlerSpec { device_filter: None }
    }

    async fn try_register(bus: &H, enum_info: EnumerationInfo) -> Result<Self, RegisterError> {
        let iface = enum_info
            .cfg_desc
            .iter_interface()
            .find(|v| match v {
                InterfaceDescriptor {
                    interface_class: 0x03,
                    interface_subclass: 0x1,
                    interface_protocol: 0x1,
                    ..
                } => true,
                _ => false,
            })
            .ok_or(RegisterError::NoSupportedInterface)?;

        let interrupt_ep = iface
            .iter_endpoints()
            .find(|v| v.ep_type() == EndpointType::Interrupt && v.ep_dir() == Direction::In)
            .ok_or(RegisterError::NoSupportedInterface)?;

        let interrupt_channel = bus.alloc_channel::<channel::Interrupt, channel::In>(
            enum_info.device_address,
            &interrupt_ep.into(),
            enum_info.ls_over_fs,
        )?;

        let mut control_channel = bus.alloc_channel::<channel::Control, channel::InOut>(
            enum_info.device_address,
            &EndpointInfo::new(
                0.into(),
                EndpointType::Control,
                enum_info
                    .device_desc
                    .max_packet_size0
                    .min(if enum_info.ls_over_fs { 8 } else { 64 }) as u16,
            ),
            enum_info.ls_over_fs,
        )?;

        const SET_PROTOCOL: u8 = 0x0B;
        const BOOT_PROTOCOL: u16 = 0x0000;
        control_channel
            .class_request_out(SET_PROTOCOL, BOOT_PROTOCOL, iface.interface_number as u16, &mut [])
            .await?;

        const SET_IDLE: u8 = 0x0A;
        control_channel
            .class_request_out(SET_IDLE, 0, iface.interface_number as u16, &mut [])
            .await?;

        Ok(KbdHandler {
            interrupt_channel,
            control_channel,
        })
    }

    async fn wait_for_event(&mut self) -> Result<HandlerEvent<Self::PollEvent>, HostError> {
        loop {
            let mut buffer = [0u8; 8];
            self.interrupt_channel.request_in(&mut buffer[..]).await?;
            let keycodes = parse_payload(buffer);

            return Ok(HandlerEvent::HandlerEvent(KbdEvent::KeyPressUpdate(keycodes)));
        }
    }
}

bitflags! {
    struct KeyboardState: u8 {
        const NumLock   = 1 << 0;
        const CapsLock     = 1 << 1;
        const ScrollLock   = 1 << 2;
        const Compose = 1 << 3;
        const Kana    = 1 << 4;
    }
}

impl<H: UsbHostDriver> KbdHandler<H> {
    async fn set_state(&mut self, state: &KeyboardState) -> Result<(), HostError> {
        const SET_REPORT: u8 = 0x09;
        const OUTPUT_REPORT: u16 = 2 << 8;
        self.control_channel
            .class_request_out(SET_REPORT, OUTPUT_REPORT, 0, &[state.bits()])
            .await
    }
}

fn parse_payload(buffer: [u8; 8]) -> [Option<NonZeroU8>; 8] {
    // SAFETY: Option<NonZeroU8> is None when u8 = 0
    unsafe { core::mem::transmute(buffer) }
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct HIDDescriptor {
    pub len: u8,
    pub descriptor_type: u8,
    pub bcd_hid: u16,
    pub country_code: u8,
    pub num_descriptors: u8,

    // num_descriptors determines how many pairs of descriptor_typeI/descriptor_lengthI follow.
    pub descriptor_type0: u8,
    pub descriptor_length0: u16,
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
