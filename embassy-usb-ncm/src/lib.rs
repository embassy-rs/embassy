#![no_std]

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

use core::cell::Cell;
use core::intrinsics::copy_nonoverlapping;
use core::mem::{size_of, MaybeUninit};
use embassy_usb::control::{self, ControlHandler, InResponse, OutResponse, Request};
use embassy_usb::driver::{Endpoint, EndpointError, EndpointIn, EndpointOut};
use embassy_usb::{driver::Driver, types::*, Builder};

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
//const REQ_SET_ETHERNET_PACKET_FILTER: u8 = 0x43;
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

const NTB_MAX_SIZE: usize = 2048;
const SIG_NTH: u32 = 0x484d434e;
const SIG_NDP_NO_FCS: u32 = 0x304d434e;
const SIG_NDP_WITH_FCS: u32 = 0x314d434e;

const ALTERNATE_SETTING_DISABLED: u8 = 0x00;
const ALTERNATE_SETTING_ENABLED: u8 = 0x01;

/// Simple NTB header (NTH+NDP all in one) for sending packets
#[repr(packed)]
#[allow(unused)]
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

#[repr(packed)]
#[allow(unused)]
struct NtbParameters {
    length: u16,
    formats_supported: u16,
    in_params: NtbParametersDir,
    out_params: NtbParametersDir,
}

#[repr(packed)]
#[allow(unused)]
struct NtbParametersDir {
    max_size: u32,
    divisor: u16,
    payload_remainder: u16,
    out_alignment: u16,
    max_datagram_count: u16,
}

fn byteify<T>(buf: &mut [u8], data: T) -> &[u8] {
    let len = size_of::<T>();
    unsafe { copy_nonoverlapping(&data as *const _ as *const u8, buf.as_mut_ptr(), len) }
    &buf[..len]
}

pub struct State<'a> {
    comm_control: MaybeUninit<CommControl<'a>>,
    data_control: MaybeUninit<DataControl>,
    shared: ControlShared,
}

impl<'a> State<'a> {
    pub fn new() -> Self {
        Self {
            comm_control: MaybeUninit::uninit(),
            data_control: MaybeUninit::uninit(),
            shared: Default::default(),
        }
    }
}

/// Shared data between Control and CdcAcmClass
struct ControlShared {
    mac_addr: Cell<[u8; 6]>,
}

impl Default for ControlShared {
    fn default() -> Self {
        ControlShared {
            mac_addr: Cell::new([0; 6]),
        }
    }
}

struct CommControl<'a> {
    mac_addr_string: StringIndex,
    shared: &'a ControlShared,
}

impl<'d> ControlHandler for CommControl<'d> {
    fn control_out(&mut self, req: control::Request, _data: &[u8]) -> OutResponse {
        match req.request {
            REQ_SEND_ENCAPSULATED_COMMAND => {
                // We don't actually support encapsulated commands but pretend we do for standards
                // compatibility.
                OutResponse::Accepted
            }
            REQ_SET_NTB_INPUT_SIZE => {
                // TODO
                OutResponse::Accepted
            }
            _ => OutResponse::Rejected,
        }
    }

    fn control_in<'a>(&'a mut self, req: Request, buf: &'a mut [u8]) -> InResponse<'a> {
        match req.request {
            REQ_GET_NTB_PARAMETERS => {
                let res = NtbParameters {
                    length: size_of::<NtbParameters>() as _,
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
                InResponse::Accepted(byteify(buf, res))
            }
            _ => InResponse::Rejected,
        }
    }

    fn get_string<'a>(
        &'a mut self,
        index: StringIndex,
        _lang_id: u16,
        buf: &'a mut [u8],
    ) -> Option<&'a str> {
        if index == self.mac_addr_string {
            let mac_addr = self.shared.mac_addr.get();
            for i in 0..12 {
                let n = (mac_addr[i / 2] >> ((1 - i % 2) * 4)) & 0xF;
                buf[i] = match n {
                    0x0..=0x9 => b'0' + n,
                    0xA..=0xF => b'A' + n - 0xA,
                    _ => unreachable!(),
                }
            }

            Some(unsafe { core::str::from_utf8_unchecked(&buf[..12]) })
        } else {
            warn!("unknown string index requested");
            None
        }
    }
}

struct DataControl {}

impl ControlHandler for DataControl {
    fn set_alternate_setting(&mut self, alternate_setting: u8) {
        match alternate_setting {
            ALTERNATE_SETTING_ENABLED => info!("ncm: interface enabled"),
            ALTERNATE_SETTING_DISABLED => info!("ncm: interface disabled"),
            _ => unreachable!(),
        }
    }
}

pub struct CdcNcmClass<'d, D: Driver<'d>> {
    _comm_if: InterfaceNumber,
    comm_ep: D::EndpointIn,

    data_if: InterfaceNumber,
    read_ep: D::EndpointOut,
    write_ep: D::EndpointIn,

    _control: &'d ControlShared,
}

impl<'d, D: Driver<'d>> CdcNcmClass<'d, D> {
    pub fn new(
        builder: &mut Builder<'d, D>,
        state: &'d mut State<'d>,
        mac_address: [u8; 6],
        max_packet_size: u16,
    ) -> Self {
        let control_shared = &state.shared;
        control_shared.mac_addr.set(mac_address);

        let mut func = builder.function(USB_CLASS_CDC, CDC_SUBCLASS_NCM, CDC_PROTOCOL_NONE);

        // Control interface
        let mut iface = func.interface();
        let mac_addr_string = iface.string();
        iface.handler(state.comm_control.write(CommControl {
            mac_addr_string,
            shared: &control_shared,
        }));
        let comm_if = iface.interface_number();
        let mut alt = iface.alt_setting(USB_CLASS_CDC, CDC_SUBCLASS_NCM, CDC_PROTOCOL_NONE);

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
                CDC_TYPE_NCM, // bDescriptorSubtype
                0x00,         // bcdNCMVersion
                0x01,         // |
                0,            // bmNetworkCapabilities
            ],
        );

        let comm_ep = alt.endpoint_interrupt_in(8, 255);

        // Data interface
        let mut iface = func.interface();
        iface.handler(state.data_control.write(DataControl {}));
        let data_if = iface.interface_number();
        let _alt = iface.alt_setting(USB_CLASS_CDC_DATA, 0x00, CDC_PROTOCOL_NTB);
        let mut alt = iface.alt_setting(USB_CLASS_CDC_DATA, 0x00, CDC_PROTOCOL_NTB);
        let read_ep = alt.endpoint_bulk_out(max_packet_size);
        let write_ep = alt.endpoint_bulk_in(max_packet_size);

        CdcNcmClass {
            _comm_if: comm_if,
            comm_ep,
            data_if,
            read_ep,
            write_ep,
            _control: control_shared,
        }
    }

    pub fn split(self) -> (Sender<'d, D>, Receiver<'d, D>) {
        (
            Sender {
                write_ep: self.write_ep,
                seq: 0,
            },
            Receiver {
                data_if: self.data_if,
                comm_ep: self.comm_ep,
                read_ep: self.read_ep,
            },
        )
    }
}

pub struct Sender<'d, D: Driver<'d>> {
    write_ep: D::EndpointIn,
    seq: u16,
}

impl<'d, D: Driver<'d>> Sender<'d, D> {
    pub async fn write_packet(&mut self, data: &[u8]) -> Result<(), EndpointError> {
        let seq = self.seq;
        self.seq = self.seq.wrapping_add(1);

        const MAX_PACKET_SIZE: usize = 64; // TODO unhardcode
        const OUT_HEADER_LEN: usize = 28;

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
        let mut buf = [0; MAX_PACKET_SIZE];
        let n = byteify(&mut buf, header);
        assert_eq!(n.len(), OUT_HEADER_LEN);

        if OUT_HEADER_LEN + data.len() < MAX_PACKET_SIZE {
            // First packet is not full, just send it.
            // No need to send ZLP because it's short for sure.
            buf[OUT_HEADER_LEN..][..data.len()].copy_from_slice(data);
            self.write_ep
                .write(&buf[..OUT_HEADER_LEN + data.len()])
                .await?;
        } else {
            let (d1, d2) = data.split_at(MAX_PACKET_SIZE - OUT_HEADER_LEN);

            buf[OUT_HEADER_LEN..].copy_from_slice(d1);
            self.write_ep.write(&buf).await?;

            for chunk in d2.chunks(MAX_PACKET_SIZE) {
                self.write_ep.write(&chunk).await?;
            }

            // Send ZLP if needed.
            if d2.len() % MAX_PACKET_SIZE == 0 {
                self.write_ep.write(&[]).await?;
            }
        }

        Ok(())
    }
}

pub struct Receiver<'d, D: Driver<'d>> {
    data_if: InterfaceNumber,
    comm_ep: D::EndpointIn,
    read_ep: D::EndpointOut,
}

impl<'d, D: Driver<'d>> Receiver<'d, D> {
    /// Reads a single packet from the OUT endpoint.
    pub async fn read_packet(&mut self, buf: &mut [u8]) -> Result<usize, EndpointError> {
        // Retry loop
        loop {
            // read NTB
            let mut ntb = [0u8; NTB_MAX_SIZE];
            let mut pos = 0;
            loop {
                let n = self.read_ep.read(&mut ntb[pos..]).await?;
                pos += n;
                if n < self.read_ep.info().max_packet_size as usize || pos == NTB_MAX_SIZE {
                    break;
                }
            }

            let ntb = &ntb[..pos];

            // Process NTB header (NTH)
            let nth = match ntb.get(..12) {
                Some(x) => x,
                None => {
                    warn!("Received too short NTB");
                    continue;
                }
            };
            let sig = u32::from_le_bytes(nth[0..4].try_into().unwrap());
            if sig != SIG_NTH {
                warn!("Received bad NTH sig.");
                continue;
            }
            let ndp_idx = u16::from_le_bytes(nth[10..12].try_into().unwrap()) as usize;

            // Process NTB Datagram Pointer (NDP)
            let ndp = match ntb.get(ndp_idx..ndp_idx + 12) {
                Some(x) => x,
                None => {
                    warn!("NTH has an NDP pointer out of range.");
                    continue;
                }
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
            let datagram = match ntb.get(datagram_index..datagram_index + datagram_len) {
                Some(x) => x,
                None => {
                    warn!("NDP has a datagram pointer out of range.");
                    continue;
                }
            };
            buf[..datagram_len].copy_from_slice(datagram);

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
                self.data_if.into(), // wIndex = interface
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
