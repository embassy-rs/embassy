#![no_std]
#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

use core::cell::Cell;
use core::mem::{self, MaybeUninit};
use core::sync::atomic::{AtomicBool, Ordering};
use embassy::blocking_mutex::CriticalSectionMutex;
use embassy_usb::control::{self, ControlHandler, InResponse, OutResponse, Request};
use embassy_usb::driver::{Endpoint, EndpointError, EndpointIn, EndpointOut};
use embassy_usb::{driver::Driver, types::*, Builder};

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

pub struct State<'a> {
    control: MaybeUninit<Control<'a>>,
    shared: ControlShared,
}

impl<'a> State<'a> {
    pub fn new() -> Self {
        Self {
            control: MaybeUninit::uninit(),
            shared: Default::default(),
        }
    }
}

/// Packet level implementation of a CDC-ACM serial port.
///
/// This class can be used directly and it has the least overhead due to directly reading and
/// writing USB packets with no intermediate buffers, but it will not act like a stream-like serial
/// port. The following constraints must be followed if you use this class directly:
///
/// - `read_packet` must be called with a buffer large enough to hold max_packet_size bytes.
/// - `write_packet` must not be called with a buffer larger than max_packet_size bytes.
/// - If you write a packet that is exactly max_packet_size bytes long, it won't be processed by the
///   host operating system until a subsequent shorter packet is sent. A zero-length packet (ZLP)
///   can be sent if there is no other data to send. This is because USB bulk transactions must be
///   terminated with a short packet, even if the bulk endpoint is used for stream-like data.
pub struct CdcAcmClass<'d, D: Driver<'d>> {
    _comm_ep: D::EndpointIn,
    _data_if: InterfaceNumber,
    read_ep: D::EndpointOut,
    write_ep: D::EndpointIn,
    control: &'d ControlShared,
}

struct Control<'a> {
    shared: &'a ControlShared,
}

/// Shared data between Control and CdcAcmClass
struct ControlShared {
    line_coding: CriticalSectionMutex<Cell<LineCoding>>,
    dtr: AtomicBool,
    rts: AtomicBool,
}

impl Default for ControlShared {
    fn default() -> Self {
        ControlShared {
            dtr: AtomicBool::new(false),
            rts: AtomicBool::new(false),
            line_coding: CriticalSectionMutex::new(Cell::new(LineCoding {
                stop_bits: StopBits::One,
                data_bits: 8,
                parity_type: ParityType::None,
                data_rate: 8_000,
            })),
        }
    }
}

impl<'a> Control<'a> {
    fn shared(&mut self) -> &'a ControlShared {
        self.shared
    }
}

impl<'d> ControlHandler for Control<'d> {
    fn reset(&mut self) {
        let shared = self.shared();
        shared.line_coding.lock(|x| x.set(LineCoding::default()));
        shared.dtr.store(false, Ordering::Relaxed);
        shared.rts.store(false, Ordering::Relaxed);
    }

    fn control_out(&mut self, req: control::Request, data: &[u8]) -> OutResponse {
        match req.request {
            REQ_SEND_ENCAPSULATED_COMMAND => {
                // We don't actually support encapsulated commands but pretend we do for standards
                // compatibility.
                OutResponse::Accepted
            }
            REQ_SET_LINE_CODING if data.len() >= 7 => {
                let coding = LineCoding {
                    data_rate: u32::from_le_bytes(data[0..4].try_into().unwrap()),
                    stop_bits: data[4].into(),
                    parity_type: data[5].into(),
                    data_bits: data[6],
                };
                self.shared().line_coding.lock(|x| x.set(coding));
                debug!("Set line coding to: {:?}", coding);

                OutResponse::Accepted
            }
            REQ_SET_CONTROL_LINE_STATE => {
                let dtr = (req.value & 0x0001) != 0;
                let rts = (req.value & 0x0002) != 0;

                let shared = self.shared();
                shared.dtr.store(dtr, Ordering::Relaxed);
                shared.rts.store(rts, Ordering::Relaxed);
                debug!("Set dtr {}, rts {}", dtr, rts);

                OutResponse::Accepted
            }
            _ => OutResponse::Rejected,
        }
    }

    fn control_in<'a>(&'a mut self, req: Request, buf: &'a mut [u8]) -> InResponse<'a> {
        match req.request {
            // REQ_GET_ENCAPSULATED_COMMAND is not really supported - it will be rejected below.
            REQ_GET_LINE_CODING if req.length == 7 => {
                debug!("Sending line coding");
                let coding = self.shared().line_coding.lock(|x| x.get());
                assert!(buf.len() >= 7);
                buf[0..4].copy_from_slice(&coding.data_rate.to_le_bytes());
                buf[4] = coding.stop_bits as u8;
                buf[5] = coding.parity_type as u8;
                buf[6] = coding.data_bits;
                InResponse::Accepted(&buf[0..7])
            }
            _ => InResponse::Rejected,
        }
    }
}

impl<'d, D: Driver<'d>> CdcAcmClass<'d, D> {
    /// Creates a new CdcAcmClass with the provided UsbBus and max_packet_size in bytes. For
    /// full-speed devices, max_packet_size has to be one of 8, 16, 32 or 64.
    pub fn new(
        builder: &mut Builder<'d, D>,
        state: &'d mut State<'d>,
        max_packet_size: u16,
    ) -> Self {
        let control = state.control.write(Control {
            shared: &state.shared,
        });

        let control_shared = &state.shared;

        assert!(builder.control_buf_len() >= 7);

        let mut func = builder.function(USB_CLASS_CDC, CDC_SUBCLASS_ACM, CDC_PROTOCOL_NONE);

        // Control interface
        let mut iface = func.interface();
        iface.handler(control);
        let comm_if = iface.interface_number();
        let data_if = u8::from(comm_if) + 1;
        let mut alt = iface.alt_setting(USB_CLASS_CDC, CDC_SUBCLASS_ACM, CDC_PROTOCOL_NONE);

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
                CDC_TYPE_ACM, // bDescriptorSubtype
                0x00,         // bmCapabilities
            ],
        );
        alt.descriptor(
            CS_INTERFACE,
            &[
                CDC_TYPE_UNION, // bDescriptorSubtype
                comm_if.into(), // bControlInterface
                data_if.into(), // bSubordinateInterface
            ],
        );
        alt.descriptor(
            CS_INTERFACE,
            &[
                CDC_TYPE_CALL_MANAGEMENT, // bDescriptorSubtype
                0x00,                     // bmCapabilities
                data_if.into(),           // bDataInterface
            ],
        );

        let comm_ep = alt.endpoint_interrupt_in(8, 255);

        // Data interface
        let mut iface = func.interface();
        let data_if = iface.interface_number();
        let mut alt = iface.alt_setting(USB_CLASS_CDC_DATA, 0x00, CDC_PROTOCOL_NONE);
        let read_ep = alt.endpoint_bulk_out(max_packet_size);
        let write_ep = alt.endpoint_bulk_in(max_packet_size);

        CdcAcmClass {
            _comm_ep: comm_ep,
            _data_if: data_if,
            read_ep,
            write_ep,
            control: control_shared,
        }
    }

    /// Gets the maximum packet size in bytes.
    pub fn max_packet_size(&self) -> u16 {
        // The size is the same for both endpoints.
        self.read_ep.info().max_packet_size
    }

    /// Gets the current line coding. The line coding contains information that's mainly relevant
    /// for USB to UART serial port emulators, and can be ignored if not relevant.
    pub fn line_coding(&self) -> LineCoding {
        self.control.line_coding.lock(|x| x.get())
    }

    /// Gets the DTR (data terminal ready) state
    pub fn dtr(&self) -> bool {
        self.control.dtr.load(Ordering::Relaxed)
    }

    /// Gets the RTS (request to send) state
    pub fn rts(&self) -> bool {
        self.control.rts.load(Ordering::Relaxed)
    }

    /// Writes a single packet into the IN endpoint.
    pub async fn write_packet(&mut self, data: &[u8]) -> Result<(), EndpointError> {
        self.write_ep.write(data).await
    }

    /// Reads a single packet from the OUT endpoint.
    pub async fn read_packet(&mut self, data: &mut [u8]) -> Result<usize, EndpointError> {
        self.read_ep.read(data).await
    }

    /// Waits for the USB host to enable this interface
    pub async fn wait_connection(&mut self) {
        self.read_ep.wait_enabled().await
    }
}

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
#[derive(Clone, Copy, defmt::Format)]
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
