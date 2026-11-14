//! CDC-NCM class implementation, aka Ethernet over USB.
//!
//! # Compatibility
//!
//! Windows: NOT supported in Windows 10 (though there's apparently a driver you can install?). Supported out of the box in Windows 11.
//!
//! Linux: Well-supported since forever.
//!
//! macOS: Supported. We declare the D0 `bmNetworkCapabilities` bit with no actual filtering,
//! otherwise macOS intermittently leaves the data interface disabled (Apple bug r.170072016).
//!
//! Android: Support for CDC-NCM is spotty and varies across manufacturers.
//!
//! - On Pixel 4a, it refused to work on Android 11, worked on Android 12.
//! - if the host's MAC address has the "locally-administered" bit set (bit 1 of first byte),
//!   it doesn't work! The "Ethernet tethering" option in settings doesn't get enabled.
//!   This is due to regex spaghetti: <https://android.googlesource.com/platform/frameworks/base/+/refs/tags/android-mainline-12.0.0_r84/core/res/res/values/config.xml#417>
//!   and this nonsense in the linux kernel: <https://github.com/torvalds/linux/blob/c00c5e1d157bec0ef0b0b59aa5482eb8dc7e8e49/drivers/net/usb/usbnet.c#L1751-L1757>

use core::mem::MaybeUninit;

use crate::control::{self, InResponse, OutResponse, Recipient, Request, RequestType};
use crate::driver::{Driver, Endpoint, EndpointError, EndpointIn, EndpointOut};
use crate::types::{InterfaceNumber, StringIndex};
use crate::{Builder, EndpointOptions, Handler, InterfaceHandler};

pub mod embassy_net;

/// This should be used as `device_class` when building the `UsbDevice`.
pub const USB_CLASS_CDC: u8 = 0x02;

const USB_CLASS_CDC_DATA: u8 = 0x0a;
const CDC_SUBCLASS_NCM: u8 = 0x0d;

const CDC_PROTOCOL_NONE: u8 = 0x00;
const CDC_PROTOCOL_NTB: u8 = 0x01;

const CS_INTERFACE: u8 = 0x24;
const CDC_TYPE_HEADER: u8 = 0x00;
const CDC_TYPE_UNION: u8 = 0x06;
const CDC_TYPE_ETHERNET: u8 = 0x0F;
const CDC_TYPE_NCM: u8 = 0x1A;

const REQ_SEND_ENCAPSULATED_COMMAND: u8 = 0x00;
//const REQ_GET_ENCAPSULATED_COMMAND: u8 = 0x01;
//const REQ_SET_ETHERNET_MULTICAST_FILTERS: u8 = 0x40;
//const REQ_SET_ETHERNET_POWER_MANAGEMENT_PATTERN_FILTER: u8 = 0x41;
//const REQ_GET_ETHERNET_POWER_MANAGEMENT_PATTERN_FILTER: u8 = 0x42;
const REQ_SET_ETHERNET_PACKET_FILTER: u8 = 0x43;
//const REQ_GET_ETHERNET_STATISTIC: u8 = 0x44;
const REQ_GET_NTB_PARAMETERS: u8 = 0x80;
//const REQ_GET_NET_ADDRESS: u8 = 0x81;
//const REQ_SET_NET_ADDRESS: u8 = 0x82;
//const REQ_GET_NTB_FORMAT: u8 = 0x83;
//const REQ_SET_NTB_FORMAT: u8 = 0x84;
//const REQ_GET_NTB_INPUT_SIZE: u8 = 0x85;
const REQ_SET_NTB_INPUT_SIZE: u8 = 0x86;
//const REQ_GET_MAX_DATAGRAM_SIZE: u8 = 0x87;
//const REQ_SET_MAX_DATAGRAM_SIZE: u8 = 0x88;
//const REQ_GET_CRC_MODE: u8 = 0x89;
//const REQ_SET_CRC_MODE: u8 = 0x8A;

//const NOTIF_MAX_PACKET_SIZE: u16 = 8;
//const NOTIF_POLL_INTERVAL: u8 = 20;

/// bmNetworkCapabilities D0. Claiming it also works around a macOS bug (Apple's r.170072016) that
/// leaves the data interface disabled if the function declares no capabilities.
const NCAP_ETH_PACKET_FILTER: u8 = 1 << 0;

const NTB_MAX_SIZE: usize = 2048;
const SIG_NTH: u32 = 0x484d_434e;
const SIG_NDP_NO_FCS: u32 = 0x304d_434e;
const SIG_NDP_WITH_FCS: u32 = 0x314d_434e;

const ALTERNATE_SETTING_DISABLED: u8 = 0x00;
const ALTERNATE_SETTING_ENABLED: u8 = 0x01;

/// Simple NTB header (NTH+NDP all in one) for sending packets
struct NtbOutHeader {
    // NTH
    nth_sig: u32,
    nth_len: u16,
    nth_seq: u16,
    nth_total_len: u16,
    nth_first_index: u16,

    // NDP
    ndp_sig: u32,
    ndp_len: u16,
    ndp_next_index: u16,
    ndp_datagram_index: u16,
    ndp_datagram_len: u16,
    ndp_term1: u16,
    ndp_term2: u16,
}

impl NtbOutHeader {
    /// The header as sent: each field little-endian, in declaration order.
    fn to_le_bytes(&self) -> [u8; 28] {
        let mut bytes = [0; 28];
        let mut at = 0;
        let mut put = |field: &[u8]| {
            bytes[at..at + field.len()].copy_from_slice(field);
            at += field.len();
        };
        put(&self.nth_sig.to_le_bytes());
        put(&self.nth_len.to_le_bytes());
        put(&self.nth_seq.to_le_bytes());
        put(&self.nth_total_len.to_le_bytes());
        put(&self.nth_first_index.to_le_bytes());
        put(&self.ndp_sig.to_le_bytes());
        put(&self.ndp_len.to_le_bytes());
        put(&self.ndp_next_index.to_le_bytes());
        put(&self.ndp_datagram_index.to_le_bytes());
        put(&self.ndp_datagram_len.to_le_bytes());
        put(&self.ndp_term1.to_le_bytes());
        put(&self.ndp_term2.to_le_bytes());
        bytes
    }
}

struct NtbParameters {
    length: u16,
    formats_supported: u16,
    in_params: NtbParametersDir,
    out_params: NtbParametersDir,
}

impl NtbParameters {
    const LEN: usize = 28;

    /// The NTB Parameter Structure (CDC NCM 1.0, §6.2.1): each field little-endian.
    fn to_le_bytes(&self) -> [u8; Self::LEN] {
        let mut bytes = [0; Self::LEN];
        let mut at = 0;
        let mut put = |field: &[u8]| {
            bytes[at..at + field.len()].copy_from_slice(field);
            at += field.len();
        };
        put(&self.length.to_le_bytes());
        put(&self.formats_supported.to_le_bytes());
        for dir in [&self.in_params, &self.out_params] {
            put(&dir.max_size.to_le_bytes());
            put(&dir.divisor.to_le_bytes());
            put(&dir.payload_remainder.to_le_bytes());
            put(&dir.out_alignment.to_le_bytes());
            put(&dir.max_datagram_count.to_le_bytes());
        }
        bytes
    }
}

struct NtbParametersDir {
    max_size: u32,
    divisor: u16,
    payload_remainder: u16,
    out_alignment: u16,
    max_datagram_count: u16,
}

/// Largest bulk IN packet the class can send, the size of its packet buffer.
const ABS_MAX_PACKET_SIZE: usize = 512;

/// Configuration for [`CdcNcmClass`].
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub struct Config {
    /// MAC address of the host side of the link, reported to the host.
    pub mac_address: [u8; 6],
    /// Options for the bulk OUT endpoint, which receives the packets the host sends.
    pub out_endpoint_options: EndpointOptions,
    /// Options for the bulk IN endpoint, which sends packets to the host.
    pub in_endpoint_options: EndpointOptions,
}

impl Config {
    /// Create a configuration with the given MAC address and default endpoint options.
    pub const fn new(mac_address: [u8; 6]) -> Self {
        Self {
            mac_address,
            out_endpoint_options: EndpointOptions::new(),
            in_endpoint_options: EndpointOptions::new(),
        }
    }
}

/// Internal state for the CDC-NCM class.
pub struct State<'a> {
    control: MaybeUninit<Control<'a>>,
    shared: ControlShared,
}

impl<'a> Default for State<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> State<'a> {
    /// Create a new `State`.
    pub fn new() -> Self {
        Self {
            control: MaybeUninit::uninit(),
            shared: ControlShared::default(),
        }
    }
}

/// Shared data between Control and `CdcAcmClass`
#[derive(Default)]
struct ControlShared {
    mac_addr: [u8; 6],
}

struct Control<'a> {
    mac_addr_string: StringIndex,
    shared: &'a ControlShared,
    mac_addr_str: [u8; 12],
    comm_if: InterfaceNumber,
    data_if: InterfaceNumber,
}

impl<'d> Handler for Control<'d> {
    fn set_alternate_setting(&mut self, iface: InterfaceNumber, alternate_setting: u8) {
        if iface != self.data_if {
            return;
        }

        match alternate_setting {
            ALTERNATE_SETTING_ENABLED => info!("ncm: interface enabled"),
            ALTERNATE_SETTING_DISABLED => info!("ncm: interface disabled"),
            _ => unreachable!(),
        }
    }

    fn control_out(&mut self, req: control::Request, _data: &[u8]) -> Option<OutResponse> {
        if (req.request_type, req.recipient, req.index)
            != (RequestType::Class, Recipient::Interface, self.comm_if.0 as u16)
        {
            return None;
        }

        match req.request {
            REQ_SEND_ENCAPSULATED_COMMAND => {
                // We don't actually support encapsulated commands but pretend we do for standards
                // compatibility.
                Some(OutResponse::Accepted)
            }
            REQ_SET_ETHERNET_PACKET_FILTER => {
                // Inhibiting packets is optional per [USBECM12], so we accept any filter, keep
                // forwarding everything, and leave the host to filter in software.
                if req.length != 0 {
                    return Some(OutResponse::Rejected);
                }
                info!("ncm: host set packet filter to {:04x}", req.value);
                Some(OutResponse::Accepted)
            }
            REQ_SET_NTB_INPUT_SIZE => {
                // TODO
                Some(OutResponse::Accepted)
            }
            _ => Some(OutResponse::Rejected),
        }
    }

    fn control_in<'a>(&'a mut self, req: Request, buf: &'a mut [u8]) -> Option<InResponse<'a>> {
        if (req.request_type, req.recipient, req.index)
            != (RequestType::Class, Recipient::Interface, self.comm_if.0 as u16)
        {
            return None;
        }

        match req.request {
            REQ_GET_NTB_PARAMETERS => {
                let res = NtbParameters {
                    length: NtbParameters::LEN as _,
                    formats_supported: 1, // only 16bit,
                    in_params: NtbParametersDir {
                        max_size: NTB_MAX_SIZE as _,
                        divisor: 4,
                        payload_remainder: 0,
                        out_alignment: 4,
                        max_datagram_count: 0, // not used
                    },
                    out_params: NtbParametersDir {
                        max_size: NTB_MAX_SIZE as _,
                        divisor: 4,
                        payload_remainder: 0,
                        out_alignment: 4,
                        max_datagram_count: 1, // We only decode 1 packet per NTB
                    },
                };
                let bytes = res.to_le_bytes();
                let Some(buf) = buf.get_mut(..bytes.len()) else {
                    warn!("Control buffer too small for the NTB parameters.");
                    return Some(InResponse::Rejected);
                };
                buf.copy_from_slice(&bytes);
                Some(InResponse::Accepted(buf))
            }
            _ => Some(InResponse::Rejected),
        }
    }

    fn get_string(&mut self, index: StringIndex, _lang_id: u16) -> Option<&str> {
        if index == self.mac_addr_string {
            let mac_addr = self.shared.mac_addr;
            let s = &mut self.mac_addr_str;
            for i in 0..12 {
                let n = (mac_addr[i / 2] >> ((1 - i % 2) * 4)) & 0xF;
                s[i] = match n {
                    0x0..=0x9 => b'0' + n,
                    0xA..=0xF => b'A' + n - 0xA,
                    _ => unreachable!(),
                }
            }

            Some(unsafe { core::str::from_utf8_unchecked(s) })
        } else {
            warn!("unknown string index requested");
            None
        }
    }
}

impl<'d> InterfaceHandler for Control<'d> {
    fn owns(&self, iface: InterfaceNumber) -> bool {
        iface == self.comm_if || iface == self.data_if
    }
}

/// CDC-NCM class
pub struct CdcNcmClass<'d, D: Driver<'d>> {
    comm_if: InterfaceNumber,
    comm_ep: D::EndpointIn,

    read_ep: D::EndpointOut,
    write_ep: D::EndpointIn,

    _control: &'d ControlShared,

    max_packet_size: usize,
}

impl<'d, D: Driver<'d>> CdcNcmClass<'d, D> {
    /// Create a new CDC NCM class.
    pub fn new(builder: &mut Builder<'d, D>, state: &'d mut State<'d>, config: Config) -> Self {
        let Config {
            mac_address,
            out_endpoint_options,
            in_endpoint_options,
        } = config;
        state.shared.mac_addr = mac_address;

        let mut func = builder.function(USB_CLASS_CDC, CDC_SUBCLASS_NCM, CDC_PROTOCOL_NONE);

        // Control interface
        let mut iface = func.interface();
        let mac_addr_string = iface.string();
        let comm_if = iface.interface_number();
        let mut alt = iface.alt_setting(USB_CLASS_CDC, CDC_SUBCLASS_NCM, CDC_PROTOCOL_NONE, None);

        alt.descriptor(
            CS_INTERFACE,
            &[
                CDC_TYPE_HEADER, // bDescriptorSubtype
                0x10,
                0x01, // bcdCDC (1.10)
            ],
        );
        alt.descriptor(
            CS_INTERFACE,
            &[
                CDC_TYPE_UNION,        // bDescriptorSubtype
                comm_if.into(),        // bControlInterface
                u8::from(comm_if) + 1, // bSubordinateInterface
            ],
        );
        alt.descriptor(
            CS_INTERFACE,
            &[
                CDC_TYPE_ETHERNET,      // bDescriptorSubtype
                mac_addr_string.into(), // iMACAddress
                0,                      // bmEthernetStatistics
                0,                      // |
                0,                      // |
                0,                      // |
                0xea,                   // wMaxSegmentSize = 1514
                0x05,                   // |
                0,                      // wNumberMCFilters
                0,                      // |
                0,                      // bNumberPowerFilters
            ],
        );
        alt.descriptor(
            CS_INTERFACE,
            &[
                CDC_TYPE_NCM,           // bDescriptorSubtype
                0x00,                   // bcdNCMVersion
                0x01,                   // |
                NCAP_ETH_PACKET_FILTER, // bmNetworkCapabilities
            ],
        );

        let comm_ep = alt.endpoint_interrupt_in(None, 8, 255);

        // Data interface
        let mut iface = func.interface();
        let data_if = iface.interface_number();
        let _alt = iface.alt_setting(USB_CLASS_CDC_DATA, 0x00, CDC_PROTOCOL_NTB, None);
        let mut alt = iface.alt_setting(USB_CLASS_CDC_DATA, 0x00, CDC_PROTOCOL_NTB, None);
        let read_ep = alt.endpoint_bulk_out(None, out_endpoint_options);
        let write_ep = alt.endpoint_bulk_in(None, in_endpoint_options);
        let max_packet_size = write_ep.info().max_packet_size as usize;
        assert!(
            max_packet_size <= ABS_MAX_PACKET_SIZE,
            "CDC-NCM supports bulk packets of up to 512 bytes"
        );

        drop(func);

        let control = state.control.write(Control {
            mac_addr_string,
            shared: &state.shared,
            mac_addr_str: [0; 12],
            comm_if,
            data_if,
        });
        builder.interface_handler(control);

        CdcNcmClass {
            comm_if,
            comm_ep,
            read_ep,
            write_ep,
            _control: &state.shared,
            max_packet_size,
        }
    }

    /// Split the class into a sender and receiver.
    ///
    /// This allows concurrently sending and receiving packets from separate tasks.
    pub fn split(self) -> (Sender<'d, D>, Receiver<'d, D>) {
        (
            Sender {
                write_ep: self.write_ep,
                seq: 0,
                max_packet_size: self.max_packet_size,
            },
            Receiver {
                comm_if: self.comm_if,
                comm_ep: self.comm_ep,
                read_ep: self.read_ep,
            },
        )
    }
}

/// CDC NCM class packet sender.
///
/// You can obtain a `Sender` with [`CdcNcmClass::split`]
pub struct Sender<'d, D: Driver<'d>> {
    write_ep: D::EndpointIn,
    seq: u16,
    max_packet_size: usize,
}

impl<'d, D: Driver<'d>> Sender<'d, D> {
    /// Write a packet.
    ///
    /// This waits until the packet is successfully stored in the CDC-NCM endpoint buffers.
    pub async fn write_packet(&mut self, data: &[u8]) -> Result<(), EndpointError> {
        const OUT_HEADER_LEN: usize = 28;

        let seq = self.seq;
        self.seq = self.seq.wrapping_add(1);

        let header = NtbOutHeader {
            nth_sig: SIG_NTH,
            nth_len: 0x0c,
            nth_seq: seq,
            nth_total_len: (data.len() + OUT_HEADER_LEN) as u16,
            nth_first_index: 0x0c,

            ndp_sig: SIG_NDP_NO_FCS,
            ndp_len: 0x10,
            ndp_next_index: 0x00,
            ndp_datagram_index: OUT_HEADER_LEN as u16,
            ndp_datagram_len: data.len() as u16,
            ndp_term1: 0x00,
            ndp_term2: 0x00,
        };

        // Build first packet on a buffer, send next packets straight from `data`.
        let mut buf = [0; ABS_MAX_PACKET_SIZE];
        buf[..OUT_HEADER_LEN].copy_from_slice(&header.to_le_bytes());

        if OUT_HEADER_LEN + data.len() < self.max_packet_size {
            // First packet is not full, just send it.
            // No need to send ZLP because it's short for sure.
            buf[OUT_HEADER_LEN..][..data.len()].copy_from_slice(data);
            self.write_ep.write(&buf[..OUT_HEADER_LEN + data.len()]).await?;
        } else {
            let (d1, d2) = data.split_at(self.max_packet_size - OUT_HEADER_LEN);

            buf[OUT_HEADER_LEN..self.max_packet_size].copy_from_slice(d1);
            self.write_ep.write(&buf[..self.max_packet_size]).await?;

            self.write_ep.write_transfer(d2, true).await?;
        }

        Ok(())
    }
}

/// CDC NCM class packet receiver.
///
/// You can obtain a `Receiver` with [`CdcNcmClass::split`]
pub struct Receiver<'d, D: Driver<'d>> {
    comm_if: InterfaceNumber,
    comm_ep: D::EndpointIn,
    read_ep: D::EndpointOut,
}

impl<'d, D: Driver<'d>> Receiver<'d, D> {
    /// Write a network packet.
    ///
    /// This waits until a packet is successfully received from the endpoint buffers.
    pub async fn read_packet(&mut self, buf: &mut [u8]) -> Result<usize, EndpointError> {
        // Retry loop
        loop {
            // read NTB
            let mut ntb = [0u8; NTB_MAX_SIZE];
            let pos = self.read_ep.read_transfer(&mut ntb).await?;

            let ntb = &ntb[..pos];

            // Process NTB header (NTH)
            let Some(nth) = ntb.get(..12) else {
                warn!("Received too short NTB");
                continue;
            };
            let sig = u32::from_le_bytes(nth[0..4].try_into().unwrap());
            if sig != SIG_NTH {
                warn!("Received bad NTH sig.");
                continue;
            }
            let ndp_idx = u16::from_le_bytes(nth[10..12].try_into().unwrap()) as usize;

            // Process NTB Datagram Pointer (NDP)
            let Some(ndp) = ntb.get(ndp_idx..ndp_idx + 12) else {
                warn!("NTH has an NDP pointer out of range.");
                continue;
            };
            let sig = u32::from_le_bytes(ndp[0..4].try_into().unwrap());
            if sig != SIG_NDP_NO_FCS && sig != SIG_NDP_WITH_FCS {
                warn!("Received bad NDP sig.");
                continue;
            }
            let datagram_index = u16::from_le_bytes(ndp[8..10].try_into().unwrap()) as usize;
            let datagram_len = u16::from_le_bytes(ndp[10..12].try_into().unwrap()) as usize;

            if datagram_index == 0 || datagram_len == 0 {
                // empty, ignore. This is allowed by the spec, so don't warn.
                continue;
            }

            // Process actual datagram, finally.
            let Some(datagram) = ntb.get(datagram_index..datagram_index + datagram_len) else {
                warn!("NDP has a datagram pointer out of range.");
                continue;
            };
            let Some(buf) = buf.get_mut(..datagram_len) else {
                warn!("Received datagram longer than the buffer, dropped.");
                continue;
            };
            buf.copy_from_slice(datagram);

            return Ok(datagram_len);
        }
    }

    /// Waits for the USB host to enable this interface
    pub async fn wait_connection(&mut self) -> Result<(), EndpointError> {
        loop {
            self.read_ep.wait_enabled().await;
            self.comm_ep.wait_enabled().await;

            let buf = [
                0xA1, //bmRequestType
                0x00, //bNotificationType = NETWORK_CONNECTION
                0x01, // wValue = connected
                0x00,
                self.comm_if.into(), // wIndex = communication interface
                0x00,
                0x00, // wLength
                0x00,
            ];
            match self.comm_ep.write(&buf).await {
                Ok(()) => break,                   // Done!
                Err(EndpointError::Disabled) => {} // Got disabled again, wait again.
                Err(e) => return Err(e),
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use core::cell::RefCell;

    use embassy_futures::block_on;

    use super::*;
    use crate::driver::{
        Bus, ControlPipe, EndpointAddress, EndpointAllocError, EndpointConfig, EndpointError, EndpointIn, EndpointInfo,
        EndpointOut, Event, Unsupported,
    };

    /// An enabled interrupt endpoint that keeps what the class writes to it.
    struct Notifications<'a>(&'a RefCell<[u8; 8]>);

    impl Endpoint for Notifications<'_> {
        fn info(&self) -> &EndpointInfo {
            unreachable!()
        }

        async fn wait_enabled(&mut self) {}

        async fn is_enabled(&mut self) -> bool {
            true
        }

        async fn set_stall(&mut self, _: bool) -> Result<(), Unsupported> {
            Err(Unsupported)
        }

        async fn is_stalled(&mut self) -> Result<bool, Unsupported> {
            Err(Unsupported)
        }
    }

    impl EndpointIn for Notifications<'_> {
        async fn write_transfer(&mut self, buf: &[u8], _needs_zlp: bool) -> Result<(), EndpointError> {
            self.0.borrow_mut().copy_from_slice(buf);
            Ok(())
        }
    }

    /// An enabled bulk OUT endpoint that is never read here.
    struct Enabled;

    impl Endpoint for Enabled {
        fn info(&self) -> &EndpointInfo {
            unreachable!()
        }

        async fn wait_enabled(&mut self) {}

        async fn is_enabled(&mut self) -> bool {
            true
        }

        async fn set_stall(&mut self, _: bool) -> Result<(), Unsupported> {
            Err(Unsupported)
        }

        async fn is_stalled(&mut self) -> Result<bool, Unsupported> {
            Err(Unsupported)
        }
    }

    impl EndpointOut for Enabled {
        async fn read_transfer(&mut self, _: &mut [u8]) -> Result<usize, EndpointError> {
            unreachable!()
        }
    }

    struct TestDriver;

    impl<'a> Driver<'a> for TestDriver {
        type EndpointOut = Enabled;
        type EndpointIn = Notifications<'a>;
        type ControlPipe = Unused;
        type Bus = Unused;

        fn alloc_endpoint_out(&mut self, _: &EndpointConfig) -> Result<Self::EndpointOut, EndpointAllocError> {
            unreachable!()
        }

        fn alloc_endpoint_in(&mut self, _: &EndpointConfig) -> Result<Self::EndpointIn, EndpointAllocError> {
            unreachable!()
        }

        fn start(self, _: u16) -> (Self::Bus, Self::ControlPipe) {
            unreachable!()
        }
    }

    /// One NTB16 with a single datagram of `len` bytes, as a host sends it.
    fn ntb(len: usize) -> ([u8; 128], usize) {
        let block = 12 + 16 + len;
        let mut ntb = [0; 128];
        ntb[0..4].copy_from_slice(&SIG_NTH.to_le_bytes());
        ntb[4..6].copy_from_slice(&12u16.to_le_bytes()); // wHeaderLength
        ntb[8..10].copy_from_slice(&(block as u16).to_le_bytes()); // wBlockLength
        ntb[10..12].copy_from_slice(&12u16.to_le_bytes()); // wNdpIndex
        ntb[12..16].copy_from_slice(&SIG_NDP_NO_FCS.to_le_bytes());
        ntb[16..18].copy_from_slice(&16u16.to_le_bytes()); // wLength
        ntb[20..22].copy_from_slice(&28u16.to_le_bytes()); // wDatagramIndex
        ntb[22..24].copy_from_slice(&(len as u16).to_le_bytes()); // wDatagramLength
        for (i, byte) in ntb[28..block].iter_mut().enumerate() {
            *byte = i as u8;
        }
        (ntb, block)
    }

    /// The host's NTBs, one per transfer, in order.
    struct Transfers<const N: usize> {
        ntbs: [([u8; 128], usize); N],
        next: usize,
    }

    impl<const N: usize> Endpoint for Transfers<N> {
        fn info(&self) -> &EndpointInfo {
            unreachable!()
        }

        async fn wait_enabled(&mut self) {
            unreachable!()
        }

        async fn is_enabled(&mut self) -> bool {
            true
        }

        async fn set_stall(&mut self, _: bool) -> Result<(), Unsupported> {
            Err(Unsupported)
        }

        async fn is_stalled(&mut self) -> Result<bool, Unsupported> {
            Err(Unsupported)
        }
    }

    impl<const N: usize> EndpointOut for Transfers<N> {
        async fn read_transfer(&mut self, buf: &mut [u8]) -> Result<usize, EndpointError> {
            let (ntb, len) = self.ntbs[self.next];
            self.next += 1;
            buf[..len].copy_from_slice(&ntb[..len]);
            Ok(len)
        }
    }

    struct NtbDriver<const N: usize>;

    impl<'a, const N: usize> Driver<'a> for NtbDriver<N> {
        type EndpointOut = Transfers<N>;
        type EndpointIn = Unused;
        type ControlPipe = Unused;
        type Bus = Unused;

        fn alloc_endpoint_out(&mut self, _: &EndpointConfig) -> Result<Self::EndpointOut, EndpointAllocError> {
            unreachable!()
        }

        fn alloc_endpoint_in(&mut self, _: &EndpointConfig) -> Result<Self::EndpointIn, EndpointAllocError> {
            unreachable!()
        }

        fn start(self, _: u16) -> (Self::Bus, Self::ControlPipe) {
            unreachable!()
        }
    }

    struct Unused;

    impl Endpoint for Unused {
        fn info(&self) -> &EndpointInfo {
            unreachable!()
        }

        async fn wait_enabled(&mut self) {
            unreachable!()
        }

        async fn is_enabled(&mut self) -> bool {
            unreachable!()
        }

        async fn set_stall(&mut self, _: bool) -> Result<(), Unsupported> {
            unreachable!()
        }

        async fn is_stalled(&mut self) -> Result<bool, Unsupported> {
            unreachable!()
        }
    }

    impl EndpointIn for Unused {
        async fn write_transfer(&mut self, _: &[u8], _: bool) -> Result<(), EndpointError> {
            unreachable!()
        }
    }

    impl ControlPipe for Unused {
        fn max_packet_size(&self) -> usize {
            unreachable!()
        }

        async fn setup(&mut self) -> [u8; 8] {
            unreachable!()
        }

        async fn data_out_transfer(&mut self, _: &mut [u8]) -> Result<usize, EndpointError> {
            unreachable!()
        }

        async fn data_in_transfer(&mut self, _: &[u8], _: bool) -> Result<(), EndpointError> {
            unreachable!()
        }

        async fn accept(&mut self) {
            unreachable!()
        }

        async fn reject(&mut self) {
            unreachable!()
        }

        async fn accept_set_address(&mut self, _: u8) {
            unreachable!()
        }
    }

    impl Bus for Unused {
        async fn enable(&mut self) {
            unreachable!()
        }

        async fn disable(&mut self) {
            unreachable!()
        }

        async fn poll(&mut self) -> Event {
            unreachable!()
        }

        async fn endpoint_set_enabled(&mut self, _: EndpointAddress, _: bool) {
            unreachable!()
        }

        async fn endpoint_is_enabled(&mut self, _: EndpointAddress) -> bool {
            unreachable!()
        }

        async fn endpoint_set_stalled(&mut self, _: EndpointAddress, _: bool) {
            unreachable!()
        }

        async fn endpoint_is_stalled(&mut self, _: EndpointAddress) -> bool {
            unreachable!()
        }

        async fn remote_wakeup(&mut self) -> Result<(), Unsupported> {
            unreachable!()
        }
    }

    #[test]
    fn network_connection_names_the_communication_interface() {
        // Communication interface 2 (its data interface would be 3), as in a composite device.
        let sent = RefCell::new([0; 8]);
        let (_, mut receiver) = CdcNcmClass::<TestDriver> {
            comm_if: InterfaceNumber(2),
            comm_ep: Notifications(&sent),
            read_ep: Enabled,
            write_ep: Notifications(&sent),
            _control: &ControlShared { mac_addr: [0; 6] },
            max_packet_size: 64,
        }
        .split();
        block_on(receiver.wait_connection()).unwrap();
        // NetworkConnection, connected: wIndex is the communication interface (CDC 1.2 §6.3.1).
        assert_eq!(*sent.borrow(), [0xa1, 0x00, 0x01, 0x00, 2, 0x00, 0x00, 0x00]);
    }

    fn control<'a>(shared: &'a ControlShared) -> Control<'a> {
        Control {
            mac_addr_string: StringIndex(4),
            shared,
            mac_addr_str: [0; 12],
            comm_if: InterfaceNumber(0),
            data_if: InterfaceNumber(1),
        }
    }

    // GET_NTB_PARAMETERS to the communication interface, wLength = 28.
    const GET_NTB_PARAMETERS: [u8; 8] = [0xa1, REQ_GET_NTB_PARAMETERS, 0, 0, 0, 0, 28, 0];

    #[test]
    fn ntb_parameters_are_the_spec_structure_in_little_endian() {
        let shared = ControlShared { mac_addr: [0; 6] };
        let mut control = control(&shared);
        let mut buf = [0; 64];
        let Some(InResponse::Accepted(res)) = control.control_in(Request::parse(&GET_NTB_PARAMETERS), &mut buf) else {
            panic!("rejected");
        };
        // CDC NCM 1.0 §6.2.1: wLength, bmNtbFormatsSupported, then the IN and OUT fields.
        #[rustfmt::skip]
        assert_eq!(res, [
            28, 0, 1, 0,
            0x00, 0x08, 0, 0, 4, 0, 0, 0, 4, 0, 0, 0,
            0x00, 0x08, 0, 0, 4, 0, 0, 0, 4, 0, 1, 0,
        ]);
    }

    #[test]
    fn ntb_parameters_never_write_past_a_short_control_buffer() {
        // The application sizes the control buffer; one shorter than the 28-byte structure
        // must be refused, not written past.
        let shared = ControlShared { mac_addr: [0; 6] };
        let mut control = control(&shared);
        let mut storage = [0xaa; 64];
        let res = control.control_in(Request::parse(&GET_NTB_PARAMETERS), &mut storage[..20]);
        assert!(matches!(res, Some(InResponse::Rejected)));
        assert!(storage[20..].iter().all(|&b| b == 0xaa));
    }

    #[test]
    fn out_header_is_nth16_then_ndp16_in_little_endian() {
        let header = NtbOutHeader {
            nth_sig: SIG_NTH,
            nth_len: 0x0c,
            nth_seq: 0x0102,
            nth_total_len: 28 + 60,
            nth_first_index: 0x0c,
            ndp_sig: SIG_NDP_NO_FCS,
            ndp_len: 0x10,
            ndp_next_index: 0,
            ndp_datagram_index: 28,
            ndp_datagram_len: 60,
            ndp_term1: 0,
            ndp_term2: 0,
        };
        #[rustfmt::skip]
        assert_eq!(header.to_le_bytes(), [
            b'N', b'C', b'M', b'H', 12, 0, 0x02, 0x01, 88, 0, 12, 0,
            b'N', b'C', b'M', b'0', 16, 0, 0, 0, 28, 0, 60, 0, 0, 0, 0, 0,
        ]);
    }

    /// `read_packet` into `buf`, the host sending one NTB per datagram length.
    fn receive<const N: usize>(datagram_lens: [usize; N], buf: &mut [u8]) -> Result<usize, EndpointError> {
        let mut receiver = Receiver::<NtbDriver<N>> {
            comm_if: InterfaceNumber(0),
            comm_ep: Unused,
            read_ep: Transfers {
                ntbs: datagram_lens.map(ntb),
                next: 0,
            },
        };
        block_on(receiver.read_packet(buf))
    }

    #[test]
    fn read_packet_copies_a_datagram_that_fits() {
        let mut buf = [0; 64];
        assert_eq!(receive([64], &mut buf), Ok(64));
        assert!(buf.iter().enumerate().all(|(i, &b)| b == i as u8));
    }

    #[test]
    fn read_packet_drops_a_datagram_longer_than_the_buffer_and_returns_the_next() {
        // The host decides the datagram length: one longer than `buf` must not panic, and the
        // receiver goes on with the next NTB, as it does with the other datagrams it cannot use.
        let mut buf = [0; 64];
        assert_eq!(receive([65, 40], &mut buf), Ok(40));
        assert!(buf[..40].iter().enumerate().all(|(i, &b)| b == i as u8));
    }
}
