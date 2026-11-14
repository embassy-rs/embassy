//! CDC-ECM (Ethernet Control Model) host driver.
//!
//! Enables connecting to USB Ethernet dongles, 4G/LTE cellular modems in ECM mode,
//! and embedded devices running ECM gadget mode. Raw Ethernet frames are transmitted
//! and received directly over USB bulk endpoints.

use embassy_usb_driver::host::{PipeError, UsbHostAllocator, UsbPipe, pipe};
use embassy_usb_driver::{Direction, Direction as UsbDirection, EndpointAddress, EndpointInfo, EndpointType};

use crate::control::Request;
use crate::host::class::ClassMatch;
use crate::host::control::{ControlType, Recipient, RequestType, SetupPacket};
use crate::host::descriptor::{ClassCode, ConfigurationDescriptorChain, InterfaceDescriptor};
use crate::host::handler::EnumerationInfo;

pub mod embassy_net;

const USB_CLASS_CDC: u8 = 0x02;
const CDC_SUBCLASS_ECM: u8 = 0x06;
const USB_CLASS_CDC_DATA: u8 = 0x0A;

const CS_INTERFACE: u8 = 0x24;
const CDC_TYPE_ETHERNET: u8 = 0x0F;

const REQ_SET_ETHERNET_PACKET_FILTER: u8 = 0x43;

/// Packet filter bitmask flags for `SET_ETHERNET_PACKET_FILTER`.
pub mod filter {
    /// Accept promiscuous frames.
    pub const PROMISCUOUS: u16 = 1 << 0;
    /// Accept all multicast frames.
    pub const ALL_MULTICAST: u16 = 1 << 1;
    /// Accept directed unicast frames.
    pub const DIRECTED: u16 = 1 << 2;
    /// Accept broadcast frames.
    pub const BROADCAST: u16 = 1 << 3;
    /// Accept multicast frames matching multicast filter.
    pub const MULTICAST: u16 = 1 << 4;
}

/// Matcher for CDC-ECM network control interface.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct EcmMatcher;

impl ClassMatch for EcmMatcher {
    const NAME: &'static str = "CDC-ECM";
    const CODE: Option<ClassCode> = Some(ClassCode::new(0x02, 0x06, 0x00));

    fn matches(iface: &InterfaceDescriptor) -> bool {
        iface.interface_class == USB_CLASS_CDC && iface.interface_subclass == CDC_SUBCLASS_ECM
    }
}

/// CDC-ECM host driver error.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum EcmError {
    /// Pipe transfer error.
    Transfer(PipeError),
    /// No matching CDC-ECM interface found.
    NoInterface,
    /// Failed to allocate pipe.
    NoPipe,
    /// Invalid response from device.
    InvalidResponse,
    /// Invalid argument.
    InvalidArgument,
}

impl From<PipeError> for EcmError {
    fn from(e: PipeError) -> Self {
        Self::Transfer(e)
    }
}

impl core::fmt::Display for EcmError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Transfer(_) => write!(f, "Transfer error"),
            Self::NoInterface => write!(f, "No CDC-ECM interface found"),
            Self::NoPipe => write!(f, "No free pipe"),
            Self::InvalidResponse => write!(f, "Invalid response from device"),
            Self::InvalidArgument => write!(f, "Invalid argument"),
        }
    }
}

impl core::error::Error for EcmError {}

#[cfg(feature = "embedded-io")]
impl embedded_io_async::Error for EcmError {
    fn kind(&self) -> embedded_io_async::ErrorKind {
        match self {
            Self::Transfer(e) => match e {
                PipeError::Disconnected => embedded_io_async::ErrorKind::NotConnected,
                PipeError::BufferOverflow => embedded_io_async::ErrorKind::OutOfMemory,
                PipeError::Timeout => embedded_io_async::ErrorKind::TimedOut,
                _ => embedded_io_async::ErrorKind::Other,
            },
            Self::NoInterface => embedded_io_async::ErrorKind::NotFound,
            Self::NoPipe => embedded_io_async::ErrorKind::OutOfMemory,
            Self::InvalidResponse => embedded_io_async::ErrorKind::InvalidData,
            Self::InvalidArgument => embedded_io_async::ErrorKind::InvalidInput,
        }
    }
}

/// Descriptor-located info for CDC-ECM device.
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct EcmInfo {
    /// Control interface number.
    pub comm_if: u8,
    /// Interrupt notification endpoint address.
    pub notif_ep: u8,
    /// Interrupt notification max packet size.
    pub notif_mps: u16,
    /// Interrupt notification poll interval in ms.
    pub notif_interval: u8,
    /// Data interface number.
    pub data_if: u8,
    /// Data alternate setting ID with bulk endpoints.
    pub data_alt: u8,
    /// Bulk IN endpoint address.
    pub bulk_in_ep: u8,
    /// Bulk IN max packet size.
    pub bulk_in_mps: u16,
    /// Bulk OUT endpoint address.
    pub bulk_out_ep: u8,
    /// Bulk OUT max packet size.
    pub bulk_out_mps: u16,
    /// MAC address string descriptor index, if provided.
    pub mac_string_idx: Option<u8>,
    /// Maximum segment size (typically 1514).
    pub max_segment_size: u16,
}

/// Parse configuration descriptor to find CDC-ECM interfaces and endpoints.
pub fn find_ecm(config_desc: &[u8]) -> Option<EcmInfo> {
    let cfg = ConfigurationDescriptorChain::try_from_slice(config_desc).ok()?;

    let mut comm = None;
    let mut data = None;

    for iface in cfg.iter_interface() {
        if iface.interface_class == USB_CLASS_CDC && iface.interface_subclass == CDC_SUBCLASS_ECM {
            let mut notif_ep = None;
            for ep in iface.iter_endpoints() {
                if ep.ep_type() == EndpointType::Interrupt && ep.is_in() {
                    notif_ep = Some((ep.endpoint_address, ep.max_packet_size, ep.interval));
                    break;
                }
            }

            if let Some((ep, mps, interval)) = notif_ep {
                let mut mac_string_idx = None;
                let mut max_segment_size = 1514u16;

                for (_offset, desc) in iface.iter_descriptors() {
                    if desc.len() >= 13 && desc.get(1) == Some(&CS_INTERFACE) && desc.get(2) == Some(&CDC_TYPE_ETHERNET)
                    {
                        if desc[3] != 0 {
                            mac_string_idx = Some(desc[3]);
                        }
                        max_segment_size = u16::from_le_bytes([desc[8], desc[9]]);
                    }
                }

                comm = Some((
                    iface.interface_number,
                    ep,
                    mps,
                    interval,
                    mac_string_idx,
                    max_segment_size,
                ));
            }
        } else if iface.interface_class == USB_CLASS_CDC_DATA {
            let mut bulk_in = None;
            let mut bulk_out = None;

            for ep in iface.iter_endpoints() {
                if ep.ep_type() == EndpointType::Bulk {
                    if ep.is_in() {
                        bulk_in = Some((ep.endpoint_address, ep.max_packet_size));
                    } else {
                        bulk_out = Some((ep.endpoint_address, ep.max_packet_size));
                    }
                }
            }

            if let (Some((in_ep, in_mps)), Some((out_ep, out_mps))) = (bulk_in, bulk_out) {
                data = Some((
                    iface.interface_number,
                    iface.alternate_setting,
                    in_ep,
                    in_mps,
                    out_ep,
                    out_mps,
                ));
            }
        }
    }

    let (comm_if, notif_ep, notif_mps, notif_interval, mac_string_idx, max_segment_size) = comm?;
    let (data_if, data_alt, bulk_in_ep, bulk_in_mps, bulk_out_ep, bulk_out_mps) = data?;

    Some(EcmInfo {
        comm_if,
        notif_ep,
        notif_mps,
        notif_interval,
        data_if,
        data_alt,
        bulk_in_ep,
        bulk_in_mps,
        bulk_out_ep,
        bulk_out_mps,
        mac_string_idx,
        max_segment_size,
    })
}

/// CDC-ECM USB host driver.
pub struct EcmHost<'d, A: UsbHostAllocator<'d>> {
    ctrl: A::Pipe<pipe::Control, pipe::InOut>,
    notif: A::Pipe<pipe::Interrupt, pipe::In>,
    sender: EcmHostSender<'d, A>,
    receiver: EcmHostReceiver<'d, A>,
    info: EcmInfo,
}

impl<'d, A: UsbHostAllocator<'d>> EcmHost<'d, A> {
    /// Create and initialize a new CDC-ECM host driver.
    pub fn new(alloc: &A, enum_info: &EnumerationInfo, config_desc: &[u8]) -> Result<Self, EcmError> {
        let info = find_ecm(config_desc).ok_or(EcmError::NoInterface)?;

        let ctrl_ep_info = EndpointInfo {
            addr: EndpointAddress::from_parts(0, UsbDirection::In),
            ep_type: EndpointType::Control,
            max_packet_size: enum_info.max_packet_size0(),
            interval_ms: 0,
            companion: None,
        };
        let ctrl = alloc
            .alloc_pipe::<pipe::Control, pipe::InOut>(enum_info.device_address, &ctrl_ep_info, enum_info.split())
            .map_err(|_| EcmError::NoPipe)?;

        let notif_ep_info = EndpointInfo {
            addr: EndpointAddress::from(info.notif_ep),
            ep_type: EndpointType::Interrupt,
            max_packet_size: info.notif_mps,
            interval_ms: info.notif_interval,
            companion: None,
        };
        let notif = alloc
            .alloc_pipe::<pipe::Interrupt, pipe::In>(enum_info.device_address, &notif_ep_info, enum_info.split())
            .map_err(|_| EcmError::NoPipe)?;

        let in_ep_info = EndpointInfo {
            addr: EndpointAddress::from(info.bulk_in_ep),
            ep_type: EndpointType::Bulk,
            max_packet_size: info.bulk_in_mps,
            interval_ms: 0,
            companion: None,
        };
        let in_ch = alloc
            .alloc_pipe::<pipe::Bulk, pipe::In>(enum_info.device_address, &in_ep_info, enum_info.split())
            .map_err(|_| EcmError::NoPipe)?;

        let out_ep_info = EndpointInfo {
            addr: EndpointAddress::from(info.bulk_out_ep),
            ep_type: EndpointType::Bulk,
            max_packet_size: info.bulk_out_mps,
            interval_ms: 0,
            companion: None,
        };
        let out_ch = alloc
            .alloc_pipe::<pipe::Bulk, pipe::Out>(enum_info.device_address, &out_ep_info, enum_info.split())
            .map_err(|_| EcmError::NoPipe)?;

        Ok(Self {
            ctrl,
            notif,
            sender: EcmHostSender { out_ch },
            receiver: EcmHostReceiver { in_ch },
            info,
        })
    }

    /// Return descriptor-located info.
    pub fn info(&self) -> &EcmInfo {
        &self.info
    }

    /// Enable the network interface: selects the active data alternate setting
    /// and configures packet filters (directed + broadcast + multicast).
    pub async fn enable(&mut self) -> Result<(), EcmError> {
        // 1. Select the active alternate setting on the data interface
        let set_iface_pkt = SetupPacket {
            request_type: RequestType {
                direction: Direction::Out,
                control_type: ControlType::Standard,
                recipient: Recipient::Interface,
            },
            request: Request::SET_INTERFACE,
            value: self.info.data_alt as u16,
            index: self.info.data_if as u16,
            length: 0,
        };
        self.ctrl.control_out(&set_iface_pkt.to_bytes(), &[]).await?;

        // 2. Set Ethernet packet filter: directed, broadcast, multicast
        let filter_val = filter::DIRECTED | filter::BROADCAST | filter::MULTICAST;
        let filter_pkt =
            SetupPacket::class_interface_out(REQ_SET_ETHERNET_PACKET_FILTER, filter_val, self.info.comm_if as u16, 0);
        self.ctrl.control_out(&filter_pkt.to_bytes(), &[]).await?;

        Ok(())
    }

    /// Read an incoming Ethernet frame.
    pub async fn read_packet(&mut self, buf: &mut [u8]) -> Result<usize, EcmError> {
        self.receiver.read_packet(buf).await
    }

    /// Send an outgoing Ethernet frame.
    pub async fn write_packet(&mut self, data: &[u8]) -> Result<(), EcmError> {
        self.sender.write_packet(data).await
    }

    /// Poll for an interrupt notification packet.
    pub async fn poll_notification(&mut self, buf: &mut [u8]) -> Result<usize, EcmError> {
        Ok(self.notif.request_in(buf).await?)
    }

    /// Split into independent sender and receiver halves.
    pub fn split(self) -> (EcmHostSender<'d, A>, EcmHostReceiver<'d, A>) {
        (self.sender, self.receiver)
    }
}

/// Host transmitter half.
pub struct EcmHostSender<'d, A: UsbHostAllocator<'d>> {
    out_ch: A::Pipe<pipe::Bulk, pipe::Out>,
}

impl<'d, A: UsbHostAllocator<'d>> EcmHostSender<'d, A> {
    /// Send an Ethernet frame.
    pub async fn write_packet(&mut self, data: &[u8]) -> Result<(), EcmError> {
        self.out_ch.request_out(data, true).await?;
        Ok(())
    }
}

/// Host receiver half.
pub struct EcmHostReceiver<'d, A: UsbHostAllocator<'d>> {
    in_ch: A::Pipe<pipe::Bulk, pipe::In>,
}

impl<'d, A: UsbHostAllocator<'d>> EcmHostReceiver<'d, A> {
    /// Read an incoming Ethernet frame.
    pub async fn read_packet(&mut self, buf: &mut [u8]) -> Result<usize, EcmError> {
        Ok(self.in_ch.request_in(buf).await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ecm_matcher() {
        let iface = InterfaceDescriptor {
            interface_number: 0,
            alternate_setting: 0,
            num_endpoints: 1,
            interface_class: 0x02,
            interface_subclass: 0x06,
            interface_protocol: 0x00,
            interface_name: 0,
        };
        assert!(EcmMatcher::matches(&iface));

        let non_ecm = InterfaceDescriptor {
            interface_number: 0,
            alternate_setting: 0,
            num_endpoints: 1,
            interface_class: 0x02,
            interface_subclass: 0x02, // ACM
            interface_protocol: 0x00,
            interface_name: 0,
        };
        assert!(!EcmMatcher::matches(&non_ecm));
    }
}
