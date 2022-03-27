use core::mem;
use defmt::info;
use embassy_usb::class::{ControlInRequestStatus, RequestStatus, UsbClass};
use embassy_usb::control::{self, Request};
use embassy_usb::driver::{Endpoint, EndpointIn, EndpointOut, ReadError, WriteError};
use embassy_usb::{driver::Driver, types::*, UsbDeviceBuilder};

/// This should be used as `device_class` when building the `UsbDevice`.
pub const USB_CLASS_CDC: u8 = 0x02;

const USB_CLASS_CDC_DATA: u8 = 0x0a;
const CDC_SUBCLASS_ACM: u8 = 0x02;
const CDC_PROTOCOL_NONE: u8 = 0x00;

const CS_INTERFACE: u8 = 0x24;
const CDC_TYPE_HEADER: u8 = 0x00;
const CDC_TYPE_CALL_MANAGEMENT: u8 = 0x01;
const CDC_TYPE_ACM: u8 = 0x02;
const CDC_TYPE_UNION: u8 = 0x06;

const REQ_SEND_ENCAPSULATED_COMMAND: u8 = 0x00;
#[allow(unused)]
const REQ_GET_ENCAPSULATED_COMMAND: u8 = 0x01;
const REQ_SET_LINE_CODING: u8 = 0x20;
const REQ_GET_LINE_CODING: u8 = 0x21;
const REQ_SET_CONTROL_LINE_STATE: u8 = 0x22;

/// Packet level implementation of a CDC-ACM serial port.
///
/// This class can be used directly and it has the least overhead due to directly reading and
/// writing USB packets with no intermediate buffers, but it will not act like a stream-like serial
/// port. The following constraints must be followed if you use this class directly:
///
/// - `read_packet` must be called with a buffer large enough to hold max_packet_size bytes, and the
///   method will return a `WouldBlock` error if there is no packet to be read.
/// - `write_packet` must not be called with a buffer larger than max_packet_size bytes, and the
///   method will return a `WouldBlock` error if the previous packet has not been sent yet.
/// - If you write a packet that is exactly max_packet_size bytes long, it won't be processed by the
///   host operating system until a subsequent shorter packet is sent. A zero-length packet (ZLP)
///   can be sent if there is no other data to send. This is because USB bulk transactions must be
///   terminated with a short packet, even if the bulk endpoint is used for stream-like data.
pub struct CdcAcmClass<'d, D: Driver<'d>> {
    // TODO not pub
    pub comm_ep: D::EndpointIn,
    pub data_if: InterfaceNumber,
    pub read_ep: D::EndpointOut,
    pub write_ep: D::EndpointIn,
    pub control: CdcAcmControl,
}

pub struct CdcAcmControl {
    pub comm_if: InterfaceNumber,
    pub line_coding: LineCoding,
    pub dtr: bool,
    pub rts: bool,
}

impl UsbClass for CdcAcmControl {
    fn reset(&mut self) {
        self.line_coding = LineCoding::default();
        self.dtr = false;
        self.rts = false;
    }

    fn control_out(&mut self, req: control::Request, data: &[u8]) -> RequestStatus {
        if !(req.request_type == control::RequestType::Class
            && req.recipient == control::Recipient::Interface
            && req.index == u8::from(self.comm_if) as u16)
        {
            return RequestStatus::Unhandled;
        }

        match req.request {
            REQ_SEND_ENCAPSULATED_COMMAND => {
                // We don't actually support encapsulated commands but pretend we do for standards
                // compatibility.
                RequestStatus::Accepted
            }
            REQ_SET_LINE_CODING if data.len() >= 7 => {
                self.line_coding.data_rate = u32::from_le_bytes(data[0..4].try_into().unwrap());
                self.line_coding.stop_bits = data[4].into();
                self.line_coding.parity_type = data[5].into();
                self.line_coding.data_bits = data[6];
                info!("Set line coding to: {:?}", self.line_coding);

                RequestStatus::Accepted
            }
            REQ_SET_CONTROL_LINE_STATE => {
                self.dtr = (req.value & 0x0001) != 0;
                self.rts = (req.value & 0x0002) != 0;
                info!("Set dtr {}, rts {}", self.dtr, self.rts);

                RequestStatus::Accepted
            }
            _ => RequestStatus::Rejected,
        }
    }

    fn control_in<'a>(
        &mut self,
        req: Request,
        control: embassy_usb::class::ControlIn<'a>,
    ) -> ControlInRequestStatus<'a> {
        if !(req.request_type == control::RequestType::Class
            && req.recipient == control::Recipient::Interface
            && req.index == u8::from(self.comm_if) as u16)
        {
            return control.ignore();
        }

        match req.request {
            // REQ_GET_ENCAPSULATED_COMMAND is not really supported - it will be rejected below.
            REQ_GET_LINE_CODING if req.length == 7 => {
                info!("Sending line coding");
                let mut data = [0; 7];
                data[0..4].copy_from_slice(&self.line_coding.data_rate.to_le_bytes());
                data[4] = self.line_coding.stop_bits as u8;
                data[5] = self.line_coding.parity_type as u8;
                data[6] = self.line_coding.data_bits;
                control.accept(&data)
            }
            _ => control.reject(),
        }
    }
}

impl<'d, D: Driver<'d>> CdcAcmClass<'d, D> {
    /// Creates a new CdcAcmClass with the provided UsbBus and max_packet_size in bytes. For
    /// full-speed devices, max_packet_size has to be one of 8, 16, 32 or 64.
    pub fn new(builder: &mut UsbDeviceBuilder<'d, D>, max_packet_size: u16) -> Self {
        let comm_if = builder.alloc_interface();
        let comm_ep = builder.alloc_interrupt_endpoint_in(8, 255);
        let data_if = builder.alloc_interface();
        let read_ep = builder.alloc_bulk_endpoint_out(max_packet_size);
        let write_ep = builder.alloc_bulk_endpoint_in(max_packet_size);

        builder
            .config_descriptor
            .iad(
                comm_if,
                2,
                USB_CLASS_CDC,
                CDC_SUBCLASS_ACM,
                CDC_PROTOCOL_NONE,
            )
            .unwrap();

        builder
            .config_descriptor
            .interface(comm_if, USB_CLASS_CDC, CDC_SUBCLASS_ACM, CDC_PROTOCOL_NONE)
            .unwrap();

        builder
            .config_descriptor
            .write(
                CS_INTERFACE,
                &[
                    CDC_TYPE_HEADER, // bDescriptorSubtype
                    0x10,
                    0x01, // bcdCDC (1.10)
                ],
            )
            .unwrap();

        builder
            .config_descriptor
            .write(
                CS_INTERFACE,
                &[
                    CDC_TYPE_ACM, // bDescriptorSubtype
                    0x00,         // bmCapabilities
                ],
            )
            .unwrap();

        builder
            .config_descriptor
            .write(
                CS_INTERFACE,
                &[
                    CDC_TYPE_UNION, // bDescriptorSubtype
                    comm_if.into(), // bControlInterface
                    data_if.into(), // bSubordinateInterface
                ],
            )
            .unwrap();

        builder
            .config_descriptor
            .write(
                CS_INTERFACE,
                &[
                    CDC_TYPE_CALL_MANAGEMENT, // bDescriptorSubtype
                    0x00,                     // bmCapabilities
                    data_if.into(),           // bDataInterface
                ],
            )
            .unwrap();

        builder.config_descriptor.endpoint(comm_ep.info()).unwrap();

        builder
            .config_descriptor
            .interface(data_if, USB_CLASS_CDC_DATA, 0x00, 0x00)
            .unwrap();

        builder.config_descriptor.endpoint(write_ep.info()).unwrap();
        builder.config_descriptor.endpoint(read_ep.info()).unwrap();

        CdcAcmClass {
            comm_ep,
            data_if,
            read_ep,
            write_ep,
            control: CdcAcmControl {
                comm_if,
                dtr: false,
                rts: false,
                line_coding: LineCoding {
                    stop_bits: StopBits::One,
                    data_bits: 8,
                    parity_type: ParityType::None,
                    data_rate: 8_000,
                },
            },
        }
    }

    /// Gets the maximum packet size in bytes.
    pub fn max_packet_size(&self) -> u16 {
        // The size is the same for both endpoints.
        self.read_ep.info().max_packet_size
    }

    /// Gets the current line coding. The line coding contains information that's mainly relevant
    /// for USB to UART serial port emulators, and can be ignored if not relevant.
    pub fn line_coding(&self) -> &LineCoding {
        &self.control.line_coding
    }

    /// Gets the DTR (data terminal ready) state
    pub fn dtr(&self) -> bool {
        self.control.dtr
    }

    /// Gets the RTS (request to send) state
    pub fn rts(&self) -> bool {
        self.control.rts
    }

    /// Writes a single packet into the IN endpoint.
    pub async fn write_packet(&mut self, data: &[u8]) -> Result<(), WriteError> {
        self.write_ep.write(data).await
    }

    /// Reads a single packet from the OUT endpoint.
    pub async fn read_packet(&mut self, data: &mut [u8]) -> Result<usize, ReadError> {
        self.read_ep.read(data).await
    }

    /// Gets the address of the IN endpoint.
    pub(crate) fn write_ep_address(&self) -> EndpointAddress {
        self.write_ep.info().addr
    }
}

/*
impl<B: UsbBus> UsbClass<B> for CdcAcmClass<'_, B> {
    fn get_configuration_descriptors(&self, builder.config_descriptor: &mut Descriptorbuilder.config_descriptor) -> Result<()> {

        Ok(())
    }

    fn reset(&mut self) {
        self.line_coding = LineCoding::default();
        self.dtr = false;
        self.rts = false;
    }

    fn control_in(&mut self, xfer: ControlIn<B>) {
        let req = xfer.request();

        if !(req.request_type == control::RequestType::Class
            && req.recipient == control::Recipient::Interface
            && req.index == u8::from(self.comm_if) as u16)
        {
            return;
        }

        match req.request {
            // REQ_GET_ENCAPSULATED_COMMAND is not really supported - it will be rejected below.
            REQ_GET_LINE_CODING if req.length == 7 => {
                xfer.accept(|data| {
                    data[0..4].copy_from_slice(&self.line_coding.data_rate.to_le_bytes());
                    data[4] = self.line_coding.stop_bits as u8;
                    data[5] = self.line_coding.parity_type as u8;
                    data[6] = self.line_coding.data_bits;

                    Ok(7)
                })
                .ok();
            }
            _ => {
                xfer.reject().ok();
            }
        }
    }

    fn control_out(&mut self, xfer: ControlOut<B>) {
        let req = xfer.request();

        if !(req.request_type == control::RequestType::Class
            && req.recipient == control::Recipient::Interface
            && req.index == u8::from(self.comm_if) as u16)
        {
            return;
        }

        match req.request {
            REQ_SEND_ENCAPSULATED_COMMAND => {
                // We don't actually support encapsulated commands but pretend we do for standards
                // compatibility.
                xfer.accept().ok();
            }
            REQ_SET_LINE_CODING if xfer.data().len() >= 7 => {
                self.line_coding.data_rate =
                    u32::from_le_bytes(xfer.data()[0..4].try_into().unwrap());
                self.line_coding.stop_bits = xfer.data()[4].into();
                self.line_coding.parity_type = xfer.data()[5].into();
                self.line_coding.data_bits = xfer.data()[6];

                xfer.accept().ok();
            }
            REQ_SET_CONTROL_LINE_STATE => {
                self.dtr = (req.value & 0x0001) != 0;
                self.rts = (req.value & 0x0002) != 0;

                xfer.accept().ok();
            }
            _ => {
                xfer.reject().ok();
            }
        };
    }
}

 */

/// Number of stop bits for LineCoding
#[derive(Copy, Clone, PartialEq, Eq, defmt::Format)]
pub enum StopBits {
    /// 1 stop bit
    One = 0,

    /// 1.5 stop bits
    OnePointFive = 1,

    /// 2 stop bits
    Two = 2,
}

impl From<u8> for StopBits {
    fn from(value: u8) -> Self {
        if value <= 2 {
            unsafe { mem::transmute(value) }
        } else {
            StopBits::One
        }
    }
}

/// Parity for LineCoding
#[derive(Copy, Clone, PartialEq, Eq, defmt::Format)]
pub enum ParityType {
    None = 0,
    Odd = 1,
    Event = 2,
    Mark = 3,
    Space = 4,
}

impl From<u8> for ParityType {
    fn from(value: u8) -> Self {
        if value <= 4 {
            unsafe { mem::transmute(value) }
        } else {
            ParityType::None
        }
    }
}

/// Line coding parameters
///
/// This is provided by the host for specifying the standard UART parameters such as baud rate. Can
/// be ignored if you don't plan to interface with a physical UART.
#[derive(defmt::Format)]
pub struct LineCoding {
    stop_bits: StopBits,
    data_bits: u8,
    parity_type: ParityType,
    data_rate: u32,
}

impl LineCoding {
    /// Gets the number of stop bits for UART communication.
    pub fn stop_bits(&self) -> StopBits {
        self.stop_bits
    }

    /// Gets the number of data bits for UART communication.
    pub fn data_bits(&self) -> u8 {
        self.data_bits
    }

    /// Gets the parity type for UART communication.
    pub fn parity_type(&self) -> ParityType {
        self.parity_type
    }

    /// Gets the data rate in bits per second for UART communication.
    pub fn data_rate(&self) -> u32 {
        self.data_rate
    }
}

impl Default for LineCoding {
    fn default() -> Self {
        LineCoding {
            stop_bits: StopBits::One,
            data_bits: 8,
            parity_type: ParityType::None,
            data_rate: 8_000,
        }
    }
}
