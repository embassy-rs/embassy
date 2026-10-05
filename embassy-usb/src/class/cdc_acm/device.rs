//! CDC-ACM class implementation, aka Serial over USB.

use core::cell::{Cell, RefCell};
use core::future::{Future, poll_fn};
use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicBool, Ordering};
use core::task::Poll;

use embassy_sync::blocking_mutex::CriticalSectionMutex;
use embassy_sync::waitqueue::WakerRegistration;

use super::{
    CDC_PROTOCOL_NONE, CDC_SUBCLASS_ACM, LineCoding, REQ_GET_LINE_CODING, REQ_SET_CONTROL_LINE_STATE,
    REQ_SET_LINE_CODING, USB_CLASS_CDC, USB_CLASS_CDC_DATA,
};
use crate::control::{self, InResponse, OutResponse, Recipient, Request, RequestType};
use crate::driver::{Driver, Endpoint, EndpointError, EndpointIn, EndpointOut};
use crate::types::InterfaceNumber;
use crate::{Builder, Handler};

const CS_INTERFACE: u8 = 0x24;
const CDC_TYPE_HEADER: u8 = 0x00;
const CDC_TYPE_ACM: u8 = 0x02;
const CDC_TYPE_UNION: u8 = 0x06;

const REQ_SEND_ENCAPSULATED_COMMAND: u8 = 0x00;
#[allow(unused)]
const REQ_GET_ENCAPSULATED_COMMAND: u8 = 0x01;

/// CDC ACM error.
#[derive(Clone, Debug)]
pub enum CdcAcmError {
    /// USB is not connected.
    NotConnected,
}

impl core::fmt::Display for CdcAcmError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match *self {
            Self::NotConnected => f.write_str("NotConnected"),
        }
    }
}

impl core::error::Error for CdcAcmError {}
impl embedded_io_async::Error for CdcAcmError {
    fn kind(&self) -> embedded_io_async::ErrorKind {
        match *self {
            Self::NotConnected => embedded_io_async::ErrorKind::NotConnected,
        }
    }
}

/// Internal state for CDC-ACM
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
    pub const fn new() -> Self {
        Self {
            control: MaybeUninit::uninit(),
            shared: ControlShared::new(),
        }
    }
}

/// Packet level implementation of a CDC-ACM serial port.
///
/// This class can be used directly and it has the least overhead due to directly reading and
/// writing USB packets with no intermediate buffers, but it will not act like a stream-like serial
/// port. The following constraints must be followed if you use this class directly:
///
/// - `read_packet` must be called with a buffer large enough to hold `max_packet_size` bytes.
/// - `write_packet` must not be called with a buffer larger than `max_packet_size` bytes.
/// - If you write a packet that is exactly `max_packet_size` bytes long, it won't be processed by the
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
    comm_if: InterfaceNumber,
    shared: &'a ControlShared,
}

/// Shared data between Control and CdcAcmClass
struct ControlShared {
    line_coding: CriticalSectionMutex<Cell<LineCoding>>,
    dtr: AtomicBool,
    rts: AtomicBool,

    waker: RefCell<WakerRegistration>,
    changed: AtomicBool,
}

impl Default for ControlShared {
    fn default() -> Self {
        Self::new()
    }
}

impl ControlShared {
    const fn new() -> Self {
        ControlShared {
            dtr: AtomicBool::new(false),
            rts: AtomicBool::new(false),
            line_coding: CriticalSectionMutex::new(Cell::new(LineCoding::DEFAULT)),
            waker: RefCell::new(WakerRegistration::new()),
            changed: AtomicBool::new(false),
        }
    }

    fn changed(&self) -> impl Future<Output = ()> + '_ {
        poll_fn(|cx| {
            if self.changed.load(Ordering::Relaxed) {
                self.changed.store(false, Ordering::Relaxed);
                Poll::Ready(())
            } else {
                self.waker.borrow_mut().register(cx.waker());
                Poll::Pending
            }
        })
    }
}

impl<'a> Control<'a> {
    fn shared(&mut self) -> &'a ControlShared {
        self.shared
    }
}

impl<'d> Handler for Control<'d> {
    fn reset(&mut self) {
        let shared = self.shared();
        shared.line_coding.lock(|x| x.set(LineCoding::default()));
        shared.dtr.store(false, Ordering::Relaxed);
        shared.rts.store(false, Ordering::Relaxed);

        shared.changed.store(true, Ordering::Relaxed);
        shared.waker.borrow_mut().wake();
    }

    fn control_out(&mut self, req: control::Request, data: &[u8]) -> Option<OutResponse> {
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
            REQ_SET_LINE_CODING if data.len() >= 7 => {
                let coding = LineCoding::from_bytes(data[0..7].try_into().unwrap());
                let shared = self.shared();
                shared.line_coding.lock(|x| x.set(coding));
                debug!("Set line coding to: {:?}", coding);

                shared.changed.store(true, Ordering::Relaxed);
                shared.waker.borrow_mut().wake();

                Some(OutResponse::Accepted)
            }
            REQ_SET_CONTROL_LINE_STATE => {
                let dtr = (req.value & 0x0001) != 0;
                let rts = (req.value & 0x0002) != 0;

                let shared = self.shared();
                shared.dtr.store(dtr, Ordering::Relaxed);
                shared.rts.store(rts, Ordering::Relaxed);
                debug!("Set dtr {}, rts {}", dtr, rts);

                shared.changed.store(true, Ordering::Relaxed);
                shared.waker.borrow_mut().wake();

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
            // REQ_GET_ENCAPSULATED_COMMAND is not really supported - it will be rejected below.
            REQ_GET_LINE_CODING if req.length == 7 => {
                debug!("Sending line coding");
                let coding = self.shared().line_coding.lock(Cell::get);
                assert!(buf.len() >= 7);
                buf[0..7].copy_from_slice(&coding.to_bytes());
                Some(InResponse::Accepted(&buf[0..7]))
            }
            _ => Some(InResponse::Rejected),
        }
    }
}

impl<'d, D: Driver<'d>> CdcAcmClass<'d, D> {
    /// Creates a new CdcAcmClass with the provided UsbBus and `max_packet_size` in bytes. For
    /// full-speed devices, `max_packet_size` has to be one of 8, 16, 32 or 64.
    pub fn new(builder: &mut Builder<'d, D>, state: &'d mut State<'d>, max_packet_size: u16) -> Self {
        assert!(builder.control_buf_len() >= 7);

        let mut func = builder.function(USB_CLASS_CDC, CDC_SUBCLASS_ACM, CDC_PROTOCOL_NONE);

        // Control interface
        let mut iface = func.interface();
        let comm_if = iface.interface_number();
        let data_if = u8::from(comm_if) + 1;
        let mut alt = iface.alt_setting(USB_CLASS_CDC, CDC_SUBCLASS_ACM, CDC_PROTOCOL_NONE, None);

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
                0x02,         // bmCapabilities:
                              // D1: Device supports the request combination of
                              // Set_Line_Coding, Set_Control_Line_State, Get_Line_Coding,
                              // and the Notification Serial_State.
            ],
        );
        alt.descriptor(
            CS_INTERFACE,
            &[
                CDC_TYPE_UNION, // bDescriptorSubtype
                comm_if.into(), // bControlInterface
                data_if,        // bSubordinateInterface
            ],
        );

        let comm_ep = alt.endpoint_interrupt_in(None, 8, 255);

        // Data interface
        let mut iface = func.interface();
        let data_if = iface.interface_number();
        let mut alt = iface.alt_setting(USB_CLASS_CDC_DATA, 0x00, CDC_PROTOCOL_NONE, None);
        let read_ep = alt.endpoint_bulk_out(None, max_packet_size);
        let write_ep = alt.endpoint_bulk_in(None, max_packet_size);

        drop(func);

        let control = state.control.write(Control {
            shared: &state.shared,
            comm_if,
        });
        builder.handler(control);

        let control_shared = &state.shared;

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
        self.control.line_coding.lock(Cell::get)
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
        self.read_ep.wait_enabled().await;
    }

    /// Split the class into a sender and receiver.
    ///
    /// This allows concurrently sending and receiving packets from separate tasks.
    pub fn split(self) -> (Sender<'d, D>, Receiver<'d, D>) {
        (
            Sender {
                write_ep: self.write_ep,
                control: self.control,
            },
            Receiver {
                read_ep: self.read_ep,
                control: self.control,
            },
        )
    }

    /// Split the class into sender, receiver and control
    ///
    /// Allows concurrently sending and receiving packets whilst monitoring for
    /// control changes (dtr, rts)
    pub fn split_with_control(self) -> (Sender<'d, D>, Receiver<'d, D>, ControlChanged<'d>) {
        (
            Sender {
                write_ep: self.write_ep,
                control: self.control,
            },
            Receiver {
                read_ep: self.read_ep,
                control: self.control,
            },
            ControlChanged { control: self.control },
        )
    }
}

/// CDC ACM Control status change monitor
///
/// You can obtain a `ControlChanged` with [`CdcAcmClass::split_with_control`]
pub struct ControlChanged<'d> {
    control: &'d ControlShared,
}

impl<'d> ControlChanged<'d> {
    /// Return a future for when the control settings change
    pub async fn control_changed(&self) {
        self.control.changed().await;
    }

    /// Gets the DTR (data terminal ready) state
    pub fn dtr(&self) -> bool {
        self.control.dtr.load(Ordering::Relaxed)
    }

    /// Gets the RTS (request to send) state
    pub fn rts(&self) -> bool {
        self.control.rts.load(Ordering::Relaxed)
    }

    /// Gets the current line coding (baud rate, data bits, parity, stop bits).
    pub fn line_coding(&self) -> LineCoding {
        self.control.line_coding.lock(Cell::get)
    }
}

/// CDC ACM class packet sender.
///
/// You can obtain a `Sender` with [`CdcAcmClass::split`]
pub struct Sender<'d, D: Driver<'d>> {
    write_ep: D::EndpointIn,
    control: &'d ControlShared,
}

impl<'d, D: Driver<'d>> Sender<'d, D> {
    /// Gets the maximum packet size in bytes.
    pub fn max_packet_size(&self) -> u16 {
        // The size is the same for both endpoints.
        self.write_ep.info().max_packet_size
    }

    /// Gets the current line coding. The line coding contains information that's mainly relevant
    /// for USB to UART serial port emulators, and can be ignored if not relevant.
    pub fn line_coding(&self) -> LineCoding {
        self.control.line_coding.lock(Cell::get)
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

    /// Waits for the USB host to enable this interface
    pub async fn wait_connection(&mut self) {
        self.write_ep.wait_enabled().await;
    }
}

impl<'d, D: Driver<'d>> embedded_io_async::ErrorType for Sender<'d, D> {
    type Error = CdcAcmError;
}

impl<'d, D: Driver<'d>> embedded_io_async::Write for Sender<'d, D> {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        let len = core::cmp::min(buf.len(), self.max_packet_size() as usize);
        match self.write_packet(&buf[..len]).await {
            Ok(()) => Ok(len),
            Err(EndpointError::BufferOverflow) => unreachable!(),
            Err(EndpointError::Disabled) => Err(CdcAcmError::NotConnected),
        }
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

/// CDC ACM class packet receiver.
///
/// You can obtain a `Receiver` with [`CdcAcmClass::split`]
pub struct Receiver<'d, D: Driver<'d>> {
    read_ep: D::EndpointOut,
    control: &'d ControlShared,
}

impl<'d, D: Driver<'d>> Receiver<'d, D> {
    /// Gets the maximum packet size in bytes.
    pub fn max_packet_size(&self) -> u16 {
        // The size is the same for both endpoints.
        self.read_ep.info().max_packet_size
    }

    /// Gets the current line coding. The line coding contains information that's mainly relevant
    /// for USB to UART serial port emulators, and can be ignored if not relevant.
    pub fn line_coding(&self) -> LineCoding {
        self.control.line_coding.lock(Cell::get)
    }

    /// Gets the DTR (data terminal ready) state
    pub fn dtr(&self) -> bool {
        self.control.dtr.load(Ordering::Relaxed)
    }

    /// Gets the RTS (request to send) state
    pub fn rts(&self) -> bool {
        self.control.rts.load(Ordering::Relaxed)
    }

    /// Reads a single packet from the OUT endpoint.
    /// Must be called with a buffer large enough to hold max_packet_size bytes.
    pub async fn read_packet(&mut self, data: &mut [u8]) -> Result<usize, EndpointError> {
        self.read_ep.read(data).await
    }

    /// Waits for the USB host to enable this interface
    pub async fn wait_connection(&mut self) {
        self.read_ep.wait_enabled().await;
    }

    /// Turn the `Receiver` into a [`BufferedReceiver`].
    ///
    /// The supplied buffer must be large enough to hold max_packet_size bytes.
    pub fn into_buffered(self, buf: &'d mut [u8]) -> BufferedReceiver<'d, D> {
        BufferedReceiver {
            receiver: self,
            buffer: buf,
            start: 0,
            end: 0,
        }
    }
}

/// CDC ACM class buffered receiver.
///
/// It is a requirement of the [`embedded_io_async::Read`] trait that arbitrarily small lengths of
/// data can be read from the stream. The [`Receiver`] can only read full packets at a time. The
/// `BufferedReceiver` instead buffers a single packet if the caller does not read all of the data,
/// so that the remaining data can be returned in subsequent calls.
///
/// If you have no requirement to use the [`embedded_io_async::Read`] trait or to read a data length
/// less than the packet length, then it is more efficient to use the [`Receiver`] directly.
///
/// You can obtain a `BufferedReceiver` with [`Receiver::into_buffered`].
///
/// [`embedded_io_async::Read`]: https://docs.rs/embedded-io-async/latest/embedded_io_async/trait.Read.html
pub struct BufferedReceiver<'d, D: Driver<'d>> {
    receiver: Receiver<'d, D>,
    buffer: &'d mut [u8],
    start: usize,
    end: usize,
}

impl<'d, D: Driver<'d>> BufferedReceiver<'d, D> {
    fn read_from_buffer(&mut self, buf: &mut [u8]) -> usize {
        let available = &self.buffer[self.start..self.end];
        let len = core::cmp::min(available.len(), buf.len());
        buf[..len].copy_from_slice(&available[..len]);
        self.start += len;
        len
    }

    /// Gets the current line coding. The line coding contains information that's mainly relevant
    /// for USB to UART serial port emulators, and can be ignored if not relevant.
    pub fn line_coding(&self) -> LineCoding {
        self.receiver.line_coding()
    }

    /// Gets the DTR (data terminal ready) state
    pub fn dtr(&self) -> bool {
        self.receiver.dtr()
    }

    /// Gets the RTS (request to send) state
    pub fn rts(&self) -> bool {
        self.receiver.rts()
    }

    /// Waits for the USB host to enable this interface
    pub async fn wait_connection(&mut self) {
        self.receiver.wait_connection().await;
    }
}

impl<'d, D: Driver<'d>> embedded_io_async::ErrorType for BufferedReceiver<'d, D> {
    type Error = CdcAcmError;
}

impl<'d, D: Driver<'d>> embedded_io_async::Read for BufferedReceiver<'d, D> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        // If there is a buffered packet, return data from that first
        if self.start != self.end {
            return Ok(self.read_from_buffer(buf));
        }

        // If the caller's buffer is large enough to contain an entire packet, read directly into
        // that instead of buffering the packet internally.
        if buf.len() > self.receiver.max_packet_size() as usize {
            return match self.receiver.read_packet(buf).await {
                Ok(n) => Ok(n),
                Err(EndpointError::BufferOverflow) => unreachable!(),
                Err(EndpointError::Disabled) => Err(CdcAcmError::NotConnected),
            };
        }

        // Otherwise read a packet into the internal buffer, and return some of it to the caller.
        //
        // It's important that `start` and `end` be updated in this order so they're left in a
        // consistent state if the `read` future is dropped mid-execution, e.g. from a timeout.
        match self.receiver.read_packet(self.buffer).await {
            Ok(n) => self.end = n,
            Err(EndpointError::BufferOverflow) => unreachable!(),
            Err(EndpointError::Disabled) => return Err(CdcAcmError::NotConnected),
        }
        self.start = 0;
        Ok(self.read_from_buffer(buf))
    }
}
