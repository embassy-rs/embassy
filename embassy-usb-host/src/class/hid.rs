//! HID (Human Interface Device) host class driver.
//!
//! This driver can communicate with USB HID devices (keyboards, mice, gamepads, etc.).

use embassy_usb_host_driver::{
    ChannelAllocError, DeviceEndpoint, DeviceSpeed, Direction, EndpointType, HostBus, HostChannel, TransferError,
};

use crate::descriptor::{DescriptorIter, EndpointDescriptor, InterfaceDescriptor, descriptor_type};

/// HID class code.
const USB_CLASS_HID: u8 = 0x03;
/// Interrupt transfer type.
const TRANSFER_INTERRUPT: u8 = 0x03;

/// HID class request: GET_REPORT.
const GET_REPORT: u8 = 0x01;
/// HID class request: SET_IDLE.
const SET_IDLE: u8 = 0x0A;
/// HID class request: SET_PROTOCOL.
const SET_PROTOCOL: u8 = 0x0B;

/// Boot protocol.
pub const PROTOCOL_BOOT: u8 = 0;
/// Report protocol.
pub const PROTOCOL_REPORT: u8 = 1;

/// Information about a HID interface found in a configuration descriptor.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct HidInfo {
    /// HID interface number.
    pub interface_number: u8,
    /// Interrupt IN endpoint address (raw, with direction bit).
    pub interrupt_in_ep: u8,
    /// Interrupt IN max packet size.
    pub interrupt_in_mps: u16,
}

/// Find the first HID interface in a configuration descriptor.
pub fn find_hid(config_desc: &[u8]) -> Option<HidInfo> {
    let mut hid_iface: Option<u8> = None;
    let mut interrupt_in: Option<(u8, u16)> = None;
    let mut in_hid_iface = false;

    for (desc_type, desc_data) in DescriptorIter::new(config_desc) {
        match desc_type {
            descriptor_type::INTERFACE => {
                if let Some(iface) = InterfaceDescriptor::parse(desc_data) {
                    if iface.interface_class == USB_CLASS_HID {
                        hid_iface = Some(iface.interface_number);
                        in_hid_iface = true;
                    } else {
                        in_hid_iface = false;
                    }
                }
            }
            descriptor_type::ENDPOINT => {
                if in_hid_iface {
                    if let Some(ep) = EndpointDescriptor::parse(desc_data) {
                        if ep.transfer_type() == TRANSFER_INTERRUPT && ep.is_in() {
                            interrupt_in = Some((ep.endpoint_address, ep.max_packet_size));
                            // We have both interface and endpoint; stop scanning
                            in_hid_iface = false;
                        }
                    }
                }
            }
            _ => {}
        }

        if hid_iface.is_some() && interrupt_in.is_some() {
            break;
        }
    }

    if let (Some(iface), Some((ep, mps))) = (hid_iface, interrupt_in) {
        Some(HidInfo {
            interface_number: iface,
            interrupt_in_ep: ep,
            interrupt_in_mps: mps,
        })
    } else {
        None
    }
}

/// HID host class driver error.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum HidError {
    /// Transfer error.
    Transfer(TransferError),
    /// No matching HID interface found in the device.
    NoInterface,
    /// Failed to allocate a channel.
    NoChannel,
}

impl From<TransferError> for HidError {
    fn from(e: TransferError) -> Self {
        Self::Transfer(e)
    }
}

impl From<ChannelAllocError> for HidError {
    fn from(_: ChannelAllocError) -> Self {
        Self::NoChannel
    }
}

impl core::fmt::Display for HidError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Transfer(e) => write!(f, "Transfer error: {}", e),
            Self::NoInterface => write!(f, "No HID interface found"),
            Self::NoChannel => write!(f, "No free channel"),
        }
    }
}

impl core::error::Error for HidError {}

/// HID host driver.
///
/// Provides report reading and optional class request access to a USB HID device.
pub struct HidHost<B: HostBus> {
    ctrl_ch: B::Channel,
    in_ch: B::Channel,
    interface: u8,
}

impl<B: HostBus> HidHost<B> {
    /// Create a new HID host driver.
    ///
    /// Parses the config descriptor to find the HID interface and its interrupt IN endpoint,
    /// then allocates the necessary channels.
    pub fn new(bus: &B, config_desc: &[u8], device_address: u8, speed: DeviceSpeed) -> Result<Self, HidError> {
        let info = find_hid(config_desc).ok_or(HidError::NoInterface)?;

        let ctrl_ep = DeviceEndpoint {
            device_address,
            ep_number: 0,
            direction: Direction::In,
            ep_type: EndpointType::Control,
            max_packet_size: 64,
            speed,
        };

        let in_ep = DeviceEndpoint {
            device_address,
            ep_number: info.interrupt_in_ep & 0x0F,
            direction: Direction::In,
            ep_type: EndpointType::Interrupt,
            max_packet_size: info.interrupt_in_mps,
            speed,
        };

        let ctrl_ch = bus.alloc_channel(&ctrl_ep)?;
        let in_ch = bus.alloc_channel(&in_ep)?;

        Ok(Self {
            ctrl_ch,
            in_ch,
            interface: info.interface_number,
        })
    }

    /// Set the idle rate for a report.
    ///
    /// `report_id = 0` applies to all reports. `idle_duration = 0` disables idle repeat.
    ///
    /// Note: SET_IDLE is optional; some devices STALL this request.
    /// A STALL is treated as success per the HID specification.
    pub async fn set_idle(&mut self, report_id: u8, idle_duration: u8) -> Result<(), HidError> {
        let value = (idle_duration as u16) << 8 | report_id as u16;
        let setup = crate::control::class_interface_out(SET_IDLE, value, self.interface as u16);
        match self.ctrl_ch.control_transfer(&setup, Direction::Out, &mut []).await {
            Ok(_) => Ok(()),
            Err(TransferError::Stall) => Ok(()),
            Err(e) => Err(HidError::Transfer(e)),
        }
    }

    /// Set the protocol (boot or report).
    pub async fn set_protocol(&mut self, protocol: u8) -> Result<(), HidError> {
        let setup = crate::control::class_interface_out(SET_PROTOCOL, protocol as u16, self.interface as u16);
        self.ctrl_ch.control_transfer(&setup, Direction::Out, &mut []).await?;
        Ok(())
    }

    /// Read an input report from the interrupt IN endpoint.
    ///
    /// Returns the number of bytes received.
    pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize, HidError> {
        let n = self.in_ch.in_transfer(buf).await?;
        Ok(n)
    }

    /// Issue a GET_REPORT control request.
    ///
    /// `report_type`: 1=Input, 2=Output, 3=Feature.
    /// `report_id`: 0 if the device uses a single report.
    ///
    /// Returns the number of bytes received.
    pub async fn get_report(&mut self, report_type: u8, report_id: u8, buf: &mut [u8]) -> Result<usize, HidError> {
        let value = (report_type as u16) << 8 | report_id as u16;
        let setup =
            crate::control::class_interface_in_with_data(GET_REPORT, value, self.interface as u16, buf.len() as u16);
        let n = self.ctrl_ch.control_transfer(&setup, Direction::In, buf).await?;
        Ok(n)
    }
}
