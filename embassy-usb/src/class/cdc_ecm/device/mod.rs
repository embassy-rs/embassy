//! CDC-ECM (Ethernet Control Model) class implementation, aka raw Ethernet over USB.
//!
//! Unlike CDC-NCM which batches datagrams into complex NTBs, CDC-ECM transfers raw Ethernet
//! frames directly over USB bulk endpoints (one Ethernet frame per USB transfer).
//!
//! # Compatibility
//!
//! Well-supported out of the box on Linux, macOS, Android, and Unix/embedded systems.

use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicU8, AtomicU16, Ordering};

use embassy_usb_driver::{Driver, Endpoint, EndpointError, EndpointIn, EndpointOut};

use crate::control::{self, InResponse, OutResponse, Recipient, Request, RequestType};
use crate::types::{InterfaceNumber, StringIndex};
use crate::{Builder, Handler, InterfaceHandler};

pub mod embassy_net;

/// USB CDC base class.
pub const USB_CLASS_CDC: u8 = 0x02;
/// USB CDC-Data class.
pub const USB_CLASS_CDC_DATA: u8 = 0x0A;
/// CDC ECM subclass.
pub const CDC_SUBCLASS_ECM: u8 = 0x06;
/// Default protocol.
pub const CDC_PROTOCOL_NONE: u8 = 0x00;

const CS_INTERFACE: u8 = 0x24;
const CDC_TYPE_HEADER: u8 = 0x00;
const CDC_TYPE_UNION: u8 = 0x06;
const CDC_TYPE_ETHERNET: u8 = 0x0F;

const REQ_SEND_ENCAPSULATED_COMMAND: u8 = 0x00;
const REQ_SET_ETHERNET_MULTICAST_FILTERS: u8 = 0x40;
const REQ_SET_ETHERNET_PACKET_FILTER: u8 = 0x43;

const ALTERNATE_SETTING_DISABLED: u8 = 0x00;
const ALTERNATE_SETTING_ENABLED: u8 = 0x01;

/// State for the CDC-ECM class.
pub struct State<'d> {
    control: MaybeUninit<Control<'d>>,
    shared: ControlShared,
}

impl<'d> State<'d> {
    /// Create a new `State`.
    pub const fn new() -> Self {
        Self {
            control: MaybeUninit::uninit(),
            shared: ControlShared {
                mac_addr: [0; 6],
                alt_setting: AtomicU8::new(0),
                packet_filter: AtomicU16::new(0),
            },
        }
    }
}

impl<'d> Default for State<'d> {
    fn default() -> Self {
        Self::new()
    }
}

struct ControlShared {
    mac_addr: [u8; 6],
    alt_setting: AtomicU8,
    packet_filter: AtomicU16,
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
            ALTERNATE_SETTING_ENABLED => {
                self.shared
                    .alt_setting
                    .store(ALTERNATE_SETTING_ENABLED, Ordering::Relaxed);
            }
            ALTERNATE_SETTING_DISABLED => {
                self.shared
                    .alt_setting
                    .store(ALTERNATE_SETTING_DISABLED, Ordering::Relaxed);
            }
            _ => {}
        }
    }

    fn control_out(&mut self, req: control::Request, _data: &[u8]) -> Option<OutResponse> {
        if (req.request_type, req.recipient, req.index)
            != (RequestType::Class, Recipient::Interface, self.comm_if.0 as u16)
        {
            return None;
        }

        match req.request {
            REQ_SEND_ENCAPSULATED_COMMAND => Some(OutResponse::Accepted),
            REQ_SET_ETHERNET_PACKET_FILTER => {
                if req.length != 0 {
                    return Some(OutResponse::Rejected);
                }
                self.shared.packet_filter.store(req.value, Ordering::Relaxed);
                Some(OutResponse::Accepted)
            }
            REQ_SET_ETHERNET_MULTICAST_FILTERS => Some(OutResponse::Accepted),
            _ => Some(OutResponse::Rejected),
        }
    }

    fn control_in<'a>(&'a mut self, req: Request, _buf: &'a mut [u8]) -> Option<InResponse<'a>> {
        if (req.request_type, req.recipient, req.index)
            != (RequestType::Class, Recipient::Interface, self.comm_if.0 as u16)
        {
            return None;
        }

        Some(InResponse::Rejected)
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
                };
            }

            Some(unsafe { core::str::from_utf8_unchecked(s) })
        } else {
            None
        }
    }
}

impl<'d> InterfaceHandler for Control<'d> {
    fn owns(&self, iface: InterfaceNumber) -> bool {
        iface == self.comm_if || iface == self.data_if
    }
}

/// CDC-ECM class instance.
pub struct CdcEcmClass<'d, D: Driver<'d>> {
    _comm_if: InterfaceNumber,
    comm_ep: D::EndpointIn,
    data_if: InterfaceNumber,
    read_ep: D::EndpointOut,
    write_ep: D::EndpointIn,
    _control: &'d ControlShared,
}

impl<'d, D: Driver<'d>> CdcEcmClass<'d, D> {
    /// Create a new CDC-ECM class instance.
    pub fn new(
        builder: &mut Builder<'d, D>,
        state: &'d mut State<'d>,
        mac_address: [u8; 6],
        max_packet_size: u16,
    ) -> Self {
        state.shared.mac_addr = mac_address;

        let mut func = builder.function(USB_CLASS_CDC, CDC_SUBCLASS_ECM, CDC_PROTOCOL_NONE);

        // Control interface
        let mut iface = func.interface();
        let mac_addr_string = iface.string();
        let comm_if = iface.interface_number();
        let mut alt = iface.alt_setting(USB_CLASS_CDC, CDC_SUBCLASS_ECM, CDC_PROTOCOL_NONE, None);

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
                0,
                0,
                0,
                0,    // bmEthernetStatistics
                0xea, // wMaxSegmentSize = 1514
                0x05, // |
                0,
                0, // wNumberMCFilters
                0, // bNumberPowerFilters
            ],
        );

        let comm_ep = alt.endpoint_interrupt_in(None, 16, 16);

        // Data interface
        let mut iface = func.interface();
        let data_if = iface.interface_number();
        let _alt0 = iface.alt_setting(USB_CLASS_CDC_DATA, 0x00, 0x00, None);
        let mut alt1 = iface.alt_setting(USB_CLASS_CDC_DATA, 0x00, 0x00, None);
        let read_ep = alt1.endpoint_bulk_out(None, max_packet_size);
        let write_ep = alt1.endpoint_bulk_in(None, max_packet_size);

        drop(func);

        let control = state.control.write(Control {
            mac_addr_string,
            shared: &state.shared,
            mac_addr_str: [0; 12],
            comm_if,
            data_if,
        });
        builder.interface_handler(control);

        CdcEcmClass {
            _comm_if: comm_if,
            comm_ep,
            data_if,
            read_ep,
            write_ep,
            _control: &state.shared,
        }
    }

    /// Split the class into separate sender and receiver halves.
    pub fn split(self) -> (Sender<'d, D>, Receiver<'d, D>) {
        (
            Sender {
                write_ep: self.write_ep,
            },
            Receiver {
                data_if: self.data_if,
                comm_ep: self.comm_ep,
                read_ep: self.read_ep,
            },
        )
    }

    /// Write an Ethernet frame.
    pub async fn write_packet(&mut self, data: &[u8]) -> Result<(), EndpointError> {
        self.write_ep.write_transfer(data, true).await
    }

    /// Read an Ethernet frame.
    pub async fn read_packet(&mut self, buf: &mut [u8]) -> Result<usize, EndpointError> {
        self.read_ep.read_transfer(buf).await
    }

    /// Waits for the USB host to activate the data interface.
    pub async fn wait_connection(&mut self) -> Result<(), EndpointError> {
        loop {
            self.read_ep.wait_enabled().await;
            self.comm_ep.wait_enabled().await;

            let buf = [
                0xA1,                // bmRequestType
                0x00,                // bNotificationType = NETWORK_CONNECTION
                0x01,                // wValue = connected
                0x00,                // |
                self.data_if.into(), // wIndex = interface
                0x00,                // |
                0x00,                // wLength
                0x00,                // |
            ];
            match self.comm_ep.write(&buf).await {
                Ok(()) => break,
                Err(EndpointError::Disabled) => {}
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }
}

/// Sender half for the CDC-ECM class.
pub struct Sender<'d, D: Driver<'d>> {
    write_ep: D::EndpointIn,
}

impl<'d, D: Driver<'d>> Sender<'d, D> {
    /// Write an Ethernet frame to the host.
    pub async fn write_packet(&mut self, data: &[u8]) -> Result<(), EndpointError> {
        self.write_ep.write_transfer(data, true).await
    }
}

/// Receiver half for the CDC-ECM class.
pub struct Receiver<'d, D: Driver<'d>> {
    data_if: InterfaceNumber,
    comm_ep: D::EndpointIn,
    read_ep: D::EndpointOut,
}

impl<'d, D: Driver<'d>> Receiver<'d, D> {
    /// Read an Ethernet frame from the host.
    pub async fn read_packet(&mut self, buf: &mut [u8]) -> Result<usize, EndpointError> {
        self.read_ep.read_transfer(buf).await
    }

    /// Waits for the USB host to activate the data interface.
    pub async fn wait_connection(&mut self) -> Result<(), EndpointError> {
        loop {
            self.read_ep.wait_enabled().await;
            self.comm_ep.wait_enabled().await;

            let buf = [
                0xA1,                // bmRequestType
                0x00,                // bNotificationType = NETWORK_CONNECTION
                0x01,                // wValue = connected
                0x00,                // |
                self.data_if.into(), // wIndex = interface
                0x00,                // |
                0x00,                // wLength
                0x00,                // |
            ];
            match self.comm_ep.write(&buf).await {
                Ok(()) => break,
                Err(EndpointError::Disabled) => {}
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_mac_string() {
        let mac: [u8; 6] = [0x02, 0x1A, 0x2B, 0x3C, 0x4D, 0x5E];
        let mut s = [0u8; 12];
        for i in 0..12 {
            let n = (mac[i / 2] >> ((1 - i % 2) * 4)) & 0xF;
            s[i] = match n {
                0x0..=0x9 => b'0' + n,
                0xA..=0xF => b'A' + n - 0xA,
                _ => unreachable!(),
            };
        }
        assert_eq!(&s, b"021A2B3C4D5E");
    }
}
