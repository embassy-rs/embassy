//! Bluetooth Host Controller Interface (HCI) over USB device implementation.
//!
//! Conforms to Bluetooth Core Specification (v5.4, Vol 4, Part B: USB Transport Layer).
//!
//! A Bluetooth USB dongle or controller exposes:
//! - **Control Endpoint (EP0):** Receives HCI Commands from host via Class-Specific setup requests (`bRequest = 0x00`).
//! - **Interrupt IN Endpoint:** Sends HCI Events from controller to host.
//! - **Bulk OUT Endpoint:** Receives ACL Data packets from host.
//! - **Bulk IN Endpoint:** Sends ACL Data packets from controller to host.

use core::cell::RefCell;
use core::mem::MaybeUninit;

use embassy_sync::blocking_mutex::CriticalSectionMutex;
use embassy_sync::waitqueue::WakerRegistration;

use crate::control::{OutResponse, Recipient, Request, RequestType};
use crate::driver::{Driver, Endpoint, EndpointError, EndpointIn, EndpointOut};
use crate::types::{InterfaceNumber, StringIndex};
use crate::{Builder, Handler, InterfaceHandler};

/// USB Class code for Wireless Controller.
pub const USB_CLASS_WIRELESS_CONTROLLER: u8 = 0xE0;

/// USB Subclass code for RF Controller.
pub const USB_SUBCLASS_RF_CONTROLLER: u8 = 0x01;

/// USB Protocol code for Bluetooth Primary Controller (HCI).
pub const USB_PROTOCOL_BLUETOOTH_PRIMARY: u8 = 0x01;

/// CCID/HCI command setup request code (`bRequest = 0x00`).
pub const HCI_REQ_SEND_COMMAND: u8 = 0x00;

/// Maximum standard HCI command length (2 bytes opcode + 1 byte param length + up to 255 params).
pub const MAX_HCI_COMMAND_LEN: usize = 258;

/// Configuration for the Bluetooth HCI device class.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Config<'d> {
    /// Maximum packet size for ACL data bulk endpoints (64 for Full-Speed, 512 for High-Speed).
    pub max_packet_size_acl: u16,
    /// Maximum packet size for HCI event interrupt endpoint (typically 16 or 64).
    pub max_packet_size_event: u16,
    /// Polling interval for HCI event interrupt endpoint in milliseconds (typically 1 ms).
    pub poll_interval_ms: u8,
    /// Optional interface string description.
    pub interface_name: Option<&'d str>,
}

impl<'d> Default for Config<'d> {
    fn default() -> Self {
        Self {
            max_packet_size_acl: 64,
            max_packet_size_event: 64,
            poll_interval_ms: 1,
            interface_name: Some("Bluetooth HCI"),
        }
    }
}

struct SharedCmd {
    buf: [u8; MAX_HCI_COMMAND_LEN],
    len: usize,
    ready: bool,
    waker: WakerRegistration,
}

impl SharedCmd {
    const fn new() -> Self {
        Self {
            buf: [0u8; MAX_HCI_COMMAND_LEN],
            len: 0,
            ready: false,
            waker: WakerRegistration::new(),
        }
    }
}

/// State for the Bluetooth HCI device class.
pub struct State<'d> {
    control: MaybeUninit<Control<'d>>,
    shared: CriticalSectionMutex<RefCell<SharedCmd>>,
}

impl<'d> Default for State<'d> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'d> State<'d> {
    /// Create a new `State`.
    pub const fn new() -> Self {
        Self {
            control: MaybeUninit::uninit(),
            shared: CriticalSectionMutex::new(RefCell::new(SharedCmd::new())),
        }
    }
}

struct Control<'d> {
    iface_number: InterfaceNumber,
    iface_string: Option<StringIndex>,
    shared: &'d CriticalSectionMutex<RefCell<SharedCmd>>,
}

impl<'d> Handler for Control<'d> {
    fn get_string(&mut self, index: StringIndex, _lang_id: u16) -> Option<&str> {
        if self.iface_string == Some(index) {
            Some("Bluetooth HCI")
        } else {
            None
        }
    }

    fn control_out(&mut self, req: Request, buf: &[u8]) -> Option<OutResponse> {
        if req.request_type == RequestType::Class
            && req.recipient == Recipient::Interface
            && req.index == self.iface_number.0 as u16
            && req.request == HCI_REQ_SEND_COMMAND
        {
            self.shared.lock(|s| {
                let mut s = s.borrow_mut();
                let copy_len = core::cmp::min(buf.len(), s.buf.len());
                s.buf[..copy_len].copy_from_slice(&buf[..copy_len]);
                s.len = copy_len;
                s.ready = true;
                s.waker.wake();
            });
            Some(OutResponse::Accepted)
        } else {
            None
        }
    }
}

impl<'d> InterfaceHandler for Control<'d> {
    fn owns(&self, iface: InterfaceNumber) -> bool {
        iface == self.iface_number
    }
}

/// USB Bluetooth HCI class device.
pub struct BluetoothHci<'d, D: Driver<'d>> {
    cmd_reader: HciCommandReader<'d>,
    event_writer: HciEventWriter<'d, D>,
    acl_reader: AclReader<'d, D>,
    acl_writer: AclWriter<'d, D>,
}

impl<'d, D: Driver<'d>> BluetoothHci<'d, D> {
    /// Create a new Bluetooth HCI device class instance on the USB builder.
    pub fn new(builder: &mut Builder<'d, D>, state: &'d mut State<'d>, config: Config<'d>) -> Self {
        let iface_string = if config.interface_name.is_some() {
            Some(builder.string())
        } else {
            None
        };

        let mut func = builder.function(
            USB_CLASS_WIRELESS_CONTROLLER,
            USB_SUBCLASS_RF_CONTROLLER,
            USB_PROTOCOL_BLUETOOTH_PRIMARY,
        );

        let mut iface = func.interface();
        let iface_number = iface.interface_number();
        let mut alt = iface.alt_setting(
            USB_CLASS_WIRELESS_CONTROLLER,
            USB_SUBCLASS_RF_CONTROLLER,
            USB_PROTOCOL_BLUETOOTH_PRIMARY,
            iface_string,
        );

        // Endpoint 1: Interrupt IN for HCI Events
        let event_ep = alt.endpoint_interrupt_in(None, config.max_packet_size_event, config.poll_interval_ms);
        // Endpoint 2: Bulk OUT for Host-to-Controller ACL Data
        let acl_out_ep = alt.endpoint_bulk_out(None, config.max_packet_size_acl);
        // Endpoint 3: Bulk IN for Controller-to-Host ACL Data
        let acl_in_ep = alt.endpoint_bulk_in(None, config.max_packet_size_acl);

        drop(func);

        let control = state.control.write(Control {
            iface_number,
            iface_string,
            shared: &state.shared,
        });
        builder.interface_handler(control);

        Self {
            cmd_reader: HciCommandReader { shared: &state.shared },
            event_writer: HciEventWriter { event_ep },
            acl_reader: AclReader { acl_out_ep },
            acl_writer: AclWriter { acl_in_ep },
        }
    }

    /// Split this driver into separate command, event, and ACL reader/writer handles.
    pub fn split(
        self,
    ) -> (
        HciCommandReader<'d>,
        HciEventWriter<'d, D>,
        AclReader<'d, D>,
        AclWriter<'d, D>,
    ) {
        (self.cmd_reader, self.event_writer, self.acl_reader, self.acl_writer)
    }

    /// Waits for the USB host to enable this interface.
    pub async fn wait_connection(&mut self) {
        self.acl_reader.wait_connection().await;
    }

    /// Asynchronously await and read the next HCI Command from the host (received on EP0).
    /// Returns the number of bytes copied into `buf`.
    pub async fn read_command(&mut self, buf: &mut [u8]) -> Result<usize, EndpointError> {
        self.cmd_reader.read_command(buf).await
    }

    /// Asynchronously send an HCI Event packet to the host on the Interrupt IN endpoint.
    pub async fn send_event(&mut self, event: &[u8]) -> Result<(), EndpointError> {
        self.event_writer.send_event(event).await
    }

    /// Asynchronously read an ACL Data packet from the host on the Bulk OUT endpoint.
    /// Returns the number of bytes copied into `buf`.
    pub async fn read_acl(&mut self, buf: &mut [u8]) -> Result<usize, EndpointError> {
        self.acl_reader.read_acl(buf).await
    }

    /// Asynchronously send an ACL Data packet to the host on the Bulk IN endpoint.
    pub async fn send_acl(&mut self, data: &[u8]) -> Result<(), EndpointError> {
        self.acl_writer.send_acl(data).await
    }
}

/// Receiver handle for HCI commands sent by the host over the control endpoint.
pub struct HciCommandReader<'d> {
    shared: &'d CriticalSectionMutex<RefCell<SharedCmd>>,
}

impl<'d> HciCommandReader<'d> {
    /// Await the next HCI command packet from the host.
    pub async fn read_command(&mut self, buf: &mut [u8]) -> Result<usize, EndpointError> {
        core::future::poll_fn(|cx| {
            self.shared.lock(|s| {
                let mut s = s.borrow_mut();
                if s.ready {
                    let len = core::cmp::min(buf.len(), s.len);
                    buf[..len].copy_from_slice(&s.buf[..len]);
                    s.ready = false;
                    core::task::Poll::Ready(Ok(len))
                } else {
                    s.waker.register(cx.waker());
                    core::task::Poll::Pending
                }
            })
        })
        .await
    }
}

/// Sender handle for HCI events sent to the host over the Interrupt IN endpoint.
pub struct HciEventWriter<'d, D: Driver<'d>> {
    event_ep: D::EndpointIn,
}

impl<'d, D: Driver<'d>> HciEventWriter<'d, D> {
    /// Waits for the USB host to enable the Interrupt IN endpoint.
    pub async fn wait_connection(&mut self) {
        self.event_ep.wait_enabled().await;
    }

    /// Send an HCI Event packet to the host.
    pub async fn send_event(&mut self, event: &[u8]) -> Result<(), EndpointError> {
        self.event_ep.write_transfer(event, true).await
    }
}

/// Receiver handle for ACL data packets sent by the host over the Bulk OUT endpoint.
pub struct AclReader<'d, D: Driver<'d>> {
    acl_out_ep: D::EndpointOut,
}

impl<'d, D: Driver<'d>> AclReader<'d, D> {
    /// Waits for the USB host to enable the Bulk OUT endpoint.
    pub async fn wait_connection(&mut self) {
        self.acl_out_ep.wait_enabled().await;
    }

    /// Read an ACL data packet from the host.
    pub async fn read_acl(&mut self, buf: &mut [u8]) -> Result<usize, EndpointError> {
        self.acl_out_ep.read_transfer(buf).await
    }
}

/// Sender handle for ACL data packets sent to the host over the Bulk IN endpoint.
pub struct AclWriter<'d, D: Driver<'d>> {
    acl_in_ep: D::EndpointIn,
}

impl<'d, D: Driver<'d>> AclWriter<'d, D> {
    /// Waits for the USB host to enable the Bulk IN endpoint.
    pub async fn wait_connection(&mut self) {
        self.acl_in_ep.wait_enabled().await;
    }

    /// Send an ACL data packet to the host.
    pub async fn send_acl(&mut self, data: &[u8]) -> Result<(), EndpointError> {
        self.acl_in_ep.write_transfer(data, true).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::driver::Direction;

    #[test]
    fn test_constants() {
        assert_eq!(USB_CLASS_WIRELESS_CONTROLLER, 0xE0);
        assert_eq!(USB_SUBCLASS_RF_CONTROLLER, 0x01);
        assert_eq!(USB_PROTOCOL_BLUETOOTH_PRIMARY, 0x01);
        assert_eq!(HCI_REQ_SEND_COMMAND, 0x00);
        assert_eq!(MAX_HCI_COMMAND_LEN, 258);
    }

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.max_packet_size_acl, 64);
        assert_eq!(config.max_packet_size_event, 64);
        assert_eq!(config.poll_interval_ms, 1);
        assert_eq!(config.interface_name, Some("Bluetooth HCI"));
    }

    #[test]
    fn test_control_handler_and_reader() {
        let state = State::new();
        let mut control = Control {
            iface_number: InterfaceNumber(0),
            iface_string: Some(StringIndex::new(1)),
            shared: &state.shared,
        };

        // Interface string query
        assert_eq!(control.get_string(StringIndex::new(1), 0x0409), Some("Bluetooth HCI"));
        assert_eq!(control.get_string(StringIndex::new(2), 0x0409), None);

        // Irrelevant request (wrong request type / recipient)
        let standard_req = Request {
            direction: Direction::Out,
            request_type: RequestType::Standard,
            recipient: Recipient::Device,
            request: 0x01,
            value: 0,
            index: 0,
            length: 0,
        };
        assert!(control.control_out(standard_req, &[]).is_none());

        // Irrelevant interface index
        let wrong_iface_req = Request {
            direction: Direction::Out,
            request_type: RequestType::Class,
            recipient: Recipient::Interface,
            request: HCI_REQ_SEND_COMMAND,
            value: 0,
            index: 1, // Interface 1 instead of 0
            length: 3,
        };
        assert!(control.control_out(wrong_iface_req, &[0x03, 0x0C, 0x00]).is_none());

        // Valid HCI command setup request (HCI_Reset = 0x0C03, length 0)
        let hci_reset_req = Request {
            direction: Direction::Out,
            request_type: RequestType::Class,
            recipient: Recipient::Interface,
            request: HCI_REQ_SEND_COMMAND,
            value: 0,
            index: 0,
            length: 3,
        };
        let cmd_payload = [0x03, 0x0C, 0x00];
        let res = control.control_out(hci_reset_req, &cmd_payload);
        assert_eq!(res, Some(OutResponse::Accepted));

        // Now test reading it via HciCommandReader
        let mut reader = HciCommandReader { shared: &state.shared };
        let mut read_buf = [0u8; 16];
        let n = embassy_futures::block_on(reader.read_command(&mut read_buf)).unwrap();
        assert_eq!(n, 3);
        assert_eq!(&read_buf[..3], &cmd_payload);
    }
}
