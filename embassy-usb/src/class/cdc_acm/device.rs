//! CDC-ACM class implementation, aka Serial over USB.

use core::cell::{Cell, RefCell};
use core::future::{Future, poll_fn};
use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicBool, Ordering};
use core::task::Poll;

use embassy_sync::blocking_mutex::CriticalSectionMutex;
use embassy_sync::waitqueue::WakerRegistration;

use super::{
    CDC_PROTOCOL_NONE, CDC_SUBCLASS_ACM, LineCoding, REQ_GET_LINE_CODING, REQ_SEND_BREAK, REQ_SET_CONTROL_LINE_STATE,
    REQ_SET_LINE_CODING, USB_CLASS_CDC, USB_CLASS_CDC_DATA,
};
use crate::control::{self, InResponse, OutResponse, Recipient, Request, RequestType};
use crate::driver::{Driver, Endpoint, EndpointDisabledError, EndpointError, EndpointIn, EndpointOut};
use crate::types::InterfaceNumber;
use crate::{Builder, EndpointOptions, Handler, InterfaceHandler};

macro_rules! bitflags {
    ($($tt:tt)*) => {
        #[cfg(feature = "defmt")]
        defmt::bitflags! { $($tt)* }
        #[cfg(not(feature = "defmt"))]
        bitflags::bitflags! { #[derive(Debug, Clone, Copy, PartialEq, Eq)] $($tt)* }
    };
}

const CS_INTERFACE: u8 = 0x24;
const CDC_TYPE_HEADER: u8 = 0x00;
const CDC_TYPE_ACM: u8 = 0x02;
const CDC_TYPE_UNION: u8 = 0x06;

/// `bmCapabilities` bit D1: line coding, control line state and SERIAL_STATE.
const ACM_CAP_LINE: u8 = 1 << 1;
/// `bmCapabilities` bit D2: SEND_BREAK.
const ACM_CAP_SEND_BREAK: u8 = 1 << 2;

// SERIAL_STATE notification and the line bits of its UART state (CDC PSTN 1.2 §6.5.4, Table 31).
const NOTIFY_SERIAL_STATE: u8 = 0x20;
const SERIAL_STATE_DCD: u16 = 1 << 0;
const SERIAL_STATE_DSR: u16 = 1 << 1;

const REQ_SEND_ENCAPSULATED_COMMAND: u8 = 0x00;
#[allow(unused)]
const REQ_GET_ENCAPSULATED_COMMAND: u8 = 0x01;

/// CDC ACM error.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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

/// Configuration for [`CdcAcmClass`].
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub struct Config {
    /// Options for the data OUT endpoint, which receives the data the host sends.
    pub out_endpoint_options: EndpointOptions,
    /// Options for the data IN endpoint, which sends data to the host.
    pub in_endpoint_options: EndpointOptions,
    /// Accept `SEND_BREAK` requests from the host and advertise them in the descriptor. Off by
    /// default.
    ///
    /// Requests are reported by [`ControlChanged::take_break`] and [`CdcAcmClass::take_break`].
    /// Only enable this if the application handles them: when it is off, Linux tells programs that
    /// the port can't send a break, instead of accepting breaks that are never handled.
    pub send_break: bool,
}

impl Config {
    /// Create a configuration with default endpoint options and break support turned off.
    pub const fn new() -> Self {
        Self {
            out_endpoint_options: EndpointOptions::new(),
            in_endpoint_options: EndpointOptions::new(),
            send_break: false,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

/// A break request from the host.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Break {
    /// Stop the break (the host sent a duration of 0).
    ///
    /// This is also reported after a bus reset or when the device is disabled, since the host
    /// can no longer end the break in those cases.
    Stop,
    /// Keep the break going until a [`Break::Stop`] (sent as a duration of 0xFFFF).
    UntilStopped,
    /// Send a break for this many milliseconds.
    Millis(u16),
}

/// The `bmCapabilities` byte of the ACM functional descriptor.
const fn acm_capabilities(config: &Config) -> u8 {
    if config.send_break {
        ACM_CAP_LINE | ACM_CAP_SEND_BREAK
    } else {
        ACM_CAP_LINE
    }
}

bitflags! {
    /// UART events to report to the host with [`Notifier::report`] (CDC PSTN 1.2 Table 31).
    pub struct SerialEvents: u16 {
        /// Break received.
        const BREAK   = 1 << 2;
        /// Ring signal detected.
        const RING    = 1 << 3;
        /// Framing error.
        const FRAMING = 1 << 4;
        /// Parity error.
        const PARITY  = 1 << 5;
        /// Overrun, the UART lost received data.
        const OVERRUN = 1 << 6;
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
    notifier: SerialStateWriter<D::EndpointIn>,
    _data_if: InterfaceNumber,
    read_ep: D::EndpointOut,
    write_ep: D::EndpointIn,
    control: &'d ControlShared,
}

struct Control<'a> {
    comm_if: InterfaceNumber,
    data_if: InterfaceNumber,
    send_break: bool,
    shared: &'a ControlShared,
}

/// Shared data between Control and CdcAcmClass
struct ControlShared {
    line_coding: CriticalSectionMutex<Cell<Option<LineCoding>>>,
    dtr: AtomicBool,
    rts: AtomicBool,
    pending_break: CriticalSectionMutex<Cell<Option<Break>>>,

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
            line_coding: CriticalSectionMutex::new(Cell::new(None)),
            pending_break: CriticalSectionMutex::new(Cell::new(None)),
            waker: RefCell::new(WakerRegistration::new()),
            changed: AtomicBool::new(false),
        }
    }

    fn take_break(&self) -> Option<Break> {
        self.pending_break.lock(|c| c.take())
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

    /// Ends a break the host started but can no longer stop, e.g. after a bus reset.
    fn end_break(&mut self) {
        if self.send_break {
            self.shared().pending_break.lock(|x| x.set(Some(Break::Stop)));
        }
    }
}

impl<'d> Handler for Control<'d> {
    fn enabled(&mut self, enabled: bool) {
        if !enabled && self.send_break {
            self.end_break();

            let shared = self.shared();
            shared.changed.store(true, Ordering::Relaxed);
            shared.waker.borrow_mut().wake();
        }
    }

    fn reset(&mut self) {
        let shared = self.shared();
        shared.line_coding.lock(|x| x.set(None));
        shared.dtr.store(false, Ordering::Relaxed);
        shared.rts.store(false, Ordering::Relaxed);
        self.end_break();

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
                shared.line_coding.lock(|x| x.set(Some(coding)));
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
            REQ_SEND_BREAK if self.send_break => {
                let request = match req.value {
                    0 => Break::Stop,
                    0xFFFF => Break::UntilStopped,
                    ms => Break::Millis(ms),
                };

                let shared = self.shared();
                shared.pending_break.lock(|x| x.set(Some(request)));
                debug!("Send break: {:?}", request);

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
                let coding = self.shared().line_coding.lock(|c| c.get()).unwrap_or_default();
                assert!(buf.len() >= 7);
                buf[0..7].copy_from_slice(&coding.to_bytes());
                Some(InResponse::Accepted(&buf[0..7]))
            }
            _ => Some(InResponse::Rejected),
        }
    }
}

impl<'d> InterfaceHandler for Control<'d> {
    fn owns(&self, iface: InterfaceNumber) -> bool {
        iface == self.comm_if || iface == self.data_if
    }
}

impl<'d, D: Driver<'d>> CdcAcmClass<'d, D> {
    /// Creates a new CdcAcmClass with the provided configuration.
    pub fn new(builder: &mut Builder<'d, D>, state: &'d mut State<'d>, config: Config) -> Self {
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
                CDC_TYPE_ACM,              // bDescriptorSubtype
                acm_capabilities(&config), // bmCapabilities
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

        // 16 bytes, so that the 10-byte SERIAL_STATE notification fits in one packet.
        let comm_ep = alt.endpoint_interrupt_in(None, 16, 255);

        // Data interface
        let mut iface = func.interface();
        let data_if = iface.interface_number();
        let mut alt = iface.alt_setting(USB_CLASS_CDC_DATA, 0x00, CDC_PROTOCOL_NONE, None);
        let read_ep = alt.endpoint_bulk_out(None, config.out_endpoint_options);
        let write_ep = alt.endpoint_bulk_in(None, config.in_endpoint_options);

        drop(func);

        let control = state.control.write(Control {
            shared: &state.shared,
            comm_if,
            data_if,
            send_break: config.send_break,
        });
        builder.interface_handler(control);

        let control_shared = &state.shared;

        CdcAcmClass {
            notifier: SerialStateWriter::new(comm_ep, comm_if),
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

    /// Gets the line coding the host has set. The line coding contains information that's mainly
    /// relevant for USB to UART serial port emulators, and can be ignored if not relevant.
    ///
    /// Returns `None` until the host sends `SET_LINE_CODING`, and again after a bus reset.
    pub fn line_coding(&self) -> Option<LineCoding> {
        self.control.line_coding.lock(|c| c.get())
    }

    /// Gets the DTR (data terminal ready) state
    pub fn dtr(&self) -> bool {
        self.control.dtr.load(Ordering::Relaxed)
    }

    /// Gets the RTS (request to send) state
    pub fn rts(&self) -> bool {
        self.control.rts.load(Ordering::Relaxed)
    }

    /// Returns the host's latest `SEND_BREAK` request, if it hasn't been taken yet.
    ///
    /// Only the newest request is kept, so a [`Break::Stop`] received before the previous
    /// request is taken replaces a [`Break::UntilStopped`]. After a bus reset or when the device
    /// is disabled, this returns [`Break::Stop`]. Always returns `None` unless
    /// [`Config::send_break`] is set.
    ///
    /// To be notified of new requests, use [`split_with_control()`](Self::split_with_control).
    pub fn take_break(&self) -> Option<Break> {
        self.control.take_break()
    }

    /// Sets the DCD and DSR lines and notifies the host.
    ///
    /// The new levels are kept in place even if sending failed. Every notification, including
    /// the ones from [`report()`](Self::report), contains them. They are not sent automatically
    /// after a bus reset, so they need to be set again once the host has reconnected. Linux
    /// hangs up the port when DCD turns off, unless the port is in `CLOCAL` mode.
    ///
    /// This call waits until the host has read the previous notification. The host only polls
    /// for notifications during the time the port is open, up to 255 ms apart. `Ok` means the
    /// endpoint has accepted the notification, but does not mean the host has read it already.
    pub async fn set_lines(&mut self, dcd: bool, dsr: bool) -> Result<(), EndpointDisabledError> {
        self.notifier.set_lines(dcd, dsr).await
    }

    /// Reports UART events to the host in a single notification, along with the current lines.
    ///
    /// This notification contains the event bits, use this if they are required. Like
    /// [`set_lines()`](Self::set_lines), this may have to wait for the host, so events that
    /// occur in the meantime need to be combined into one call.
    pub async fn report(&mut self, events: SerialEvents) -> Result<(), EndpointDisabledError> {
        self.notifier.report(events).await
    }

    /// Gets the DCD (data carrier detect) level set with [`set_lines()`](Self::set_lines).
    pub fn dcd(&self) -> bool {
        self.notifier.dcd()
    }

    /// Gets the DSR (data set ready) level set with [`set_lines()`](Self::set_lines).
    pub fn dsr(&self) -> bool {
        self.notifier.dsr()
    }

    /// Writes a single packet into the IN endpoint.
    pub async fn write_packet(&mut self, data: &[u8]) -> Result<(), EndpointError> {
        crate::class::write_packet(&mut self.write_ep, data).await
    }

    /// Reads a single packet from the OUT endpoint.
    pub async fn read_packet(&mut self, data: &mut [u8]) -> Result<usize, EndpointError> {
        crate::class::read_packet(&mut self.read_ep, data).await
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
                needs_zlp: false,
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
    /// control changes (dtr, rts, line coding, break requests)
    pub fn split_with_control(self) -> (Sender<'d, D>, Receiver<'d, D>, ControlChanged<'d>) {
        (
            Sender {
                write_ep: self.write_ep,
                control: self.control,
                needs_zlp: false,
            },
            Receiver {
                read_ep: self.read_ep,
                control: self.control,
            },
            ControlChanged { control: self.control },
        )
    }

    /// Split the class into sender, receiver, control and notifier
    ///
    /// Like [`split_with_control()`](Self::split_with_control), with an additional [`Notifier`]
    /// to send `SERIAL_STATE` notifications concurrently.
    pub fn split_with_notifier(self) -> (Sender<'d, D>, Receiver<'d, D>, ControlChanged<'d>, Notifier<'d, D>) {
        (
            Sender {
                write_ep: self.write_ep,
                control: self.control,
                needs_zlp: false,
            },
            Receiver {
                read_ep: self.read_ep,
                control: self.control,
            },
            ControlChanged { control: self.control },
            Notifier { writer: self.notifier },
        )
    }
}

/// CDC ACM `SERIAL_STATE` notification sender
///
/// Tells the host the state of the DCD and DSR lines and reports UART events such as breaks and
/// overruns.
///
/// A `Notifier` can be obtained with [`CdcAcmClass::split_with_notifier`]
pub struct Notifier<'d, D: Driver<'d>> {
    writer: SerialStateWriter<D::EndpointIn>,
}

impl<'d, D: Driver<'d>> Notifier<'d, D> {
    /// Sets the DCD and DSR lines and notifies the host.
    ///
    /// The new levels are kept in place even if sending failed. Every notification, including
    /// the ones from [`report()`](Self::report), contains them. They are not sent automatically
    /// after a bus reset, so they need to be set again once the host has reconnected. Linux
    /// hangs up the port when DCD turns off, unless the port is in `CLOCAL` mode.
    ///
    /// This call waits until the host has read the previous notification. The host only polls
    /// for notifications during the time the port is open, up to 255 ms apart. `Ok` means the
    /// endpoint has accepted the notification, but does not mean the host has read it already.
    pub async fn set_lines(&mut self, dcd: bool, dsr: bool) -> Result<(), EndpointDisabledError> {
        self.writer.set_lines(dcd, dsr).await
    }

    /// Reports UART events to the host in a single notification, along with the current lines.
    ///
    /// This notification contains the event bits, use this if they are required. Like
    /// [`set_lines()`](Self::set_lines), this may have to wait for the host, so events that
    /// occur in the meantime need to be combined into one call.
    pub async fn report(&mut self, events: SerialEvents) -> Result<(), EndpointDisabledError> {
        self.writer.report(events).await
    }

    /// Gets the DCD (data carrier detect) level set with [`set_lines()`](Self::set_lines).
    pub fn dcd(&self) -> bool {
        self.writer.dcd()
    }

    /// Gets the DSR (data set ready) level set with [`set_lines()`](Self::set_lines).
    pub fn dsr(&self) -> bool {
        self.writer.dsr()
    }

    /// Waits for the USB host to enable this interface
    pub async fn wait_connection(&mut self) {
        self.writer.ep.wait_enabled().await;
    }
}

/// Remembers the DCD and DSR levels and writes `SERIAL_STATE` notifications to the interrupt endpoint.
struct SerialStateWriter<E> {
    ep: E,
    comm_if: InterfaceNumber,
    lines: u16,
}

impl<E: EndpointIn> SerialStateWriter<E> {
    fn new(ep: E, comm_if: InterfaceNumber) -> Self {
        Self { ep, comm_if, lines: 0 }
    }

    async fn set_lines(&mut self, dcd: bool, dsr: bool) -> Result<(), EndpointDisabledError> {
        self.lines = 0;
        if dcd {
            self.lines |= SERIAL_STATE_DCD;
        }
        if dsr {
            self.lines |= SERIAL_STATE_DSR;
        }
        self.send(self.lines).await
    }

    async fn report(&mut self, events: SerialEvents) -> Result<(), EndpointDisabledError> {
        self.send(self.lines | events.bits()).await
    }

    fn dcd(&self) -> bool {
        self.lines & SERIAL_STATE_DCD != 0
    }

    fn dsr(&self) -> bool {
        self.lines & SERIAL_STATE_DSR != 0
    }

    async fn send(&mut self, state: u16) -> Result<(), EndpointDisabledError> {
        let [lo, hi] = state.to_le_bytes();
        let notification = [
            0xA1,                // bmRequestType
            NOTIFY_SERIAL_STATE, // bNotification
            0x00,                // wValue
            0x00,                // |
            self.comm_if.0,      // wIndex = interface
            0x00,                // |
            0x02,                // wLength
            0x00,                // |
            lo,                  // UART state bitmap
            hi,                  // |
        ];
        self.ep.write_transfer(&notification, false).await
    }
}

/// CDC ACM Control status change monitor
///
/// A `ControlChanged` can be obtained with [`CdcAcmClass::split_with_control`] or
/// [`CdcAcmClass::split_with_notifier`]
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

    /// Gets the line coding (baud rate, data bits, parity, stop bits) the host has set.
    ///
    /// Returns `None` until the host sends `SET_LINE_CODING`, and again after a bus reset.
    pub fn line_coding(&self) -> Option<LineCoding> {
        self.control.line_coding.lock(|c| c.get())
    }

    /// Returns the host's latest `SEND_BREAK` request, if it hasn't been taken yet.
    ///
    /// Only the newest request is kept, so a [`Break::Stop`] received before the previous
    /// request is taken replaces a [`Break::UntilStopped`]. After a bus reset or when the device
    /// is disabled, this returns [`Break::Stop`]. Always returns `None` unless
    /// [`Config::send_break`] is set.
    ///
    /// [`control_changed()`](Self::control_changed) returns whenever the host sends a new request.
    pub fn take_break(&self) -> Option<Break> {
        self.control.take_break()
    }
}

/// CDC ACM class packet sender.
///
/// You can obtain a `Sender` with [`CdcAcmClass::split`]
pub struct Sender<'d, D: Driver<'d>> {
    write_ep: D::EndpointIn,
    control: &'d ControlShared,
    needs_zlp: bool,
}

impl<'d, D: Driver<'d>> Sender<'d, D> {
    /// Gets the maximum packet size in bytes.
    pub fn max_packet_size(&self) -> u16 {
        // The size is the same for both endpoints.
        self.write_ep.info().max_packet_size
    }

    /// Gets the line coding the host has set. The line coding contains information that's mainly
    /// relevant for USB to UART serial port emulators, and can be ignored if not relevant.
    ///
    /// Returns `None` until the host sends `SET_LINE_CODING`, and again after a bus reset.
    pub fn line_coding(&self) -> Option<LineCoding> {
        self.control.line_coding.lock(|c| c.get())
    }

    /// Gets the DTR (data terminal ready) state
    pub fn dtr(&self) -> bool {
        self.control.dtr.load(Ordering::Relaxed)
    }

    /// Gets the RTS (request to send) state
    pub fn rts(&self) -> bool {
        self.control.rts.load(Ordering::Relaxed)
    }

    /// Writes data to the IN endpoint.
    ///
    /// The transfer can be larger than max_packet_size; the driver handles packetization.
    pub async fn write_transfer(&mut self, data: &[u8]) -> Result<(), EndpointDisabledError> {
        let _ = self.write_ep.write_transfer(data, false).await?;
        let max_packet_size = self.max_packet_size() as usize;
        self.needs_zlp = !data.is_empty() && (data.len() % max_packet_size == 0);
        Ok(())
    }

    /// Writes a single packet into the IN endpoint.
    pub async fn write_packet(&mut self, data: &[u8]) -> Result<(), EndpointError> {
        crate::class::write_packet(&mut self.write_ep, data).await?;
        self.needs_zlp = data.len() == self.max_packet_size() as usize;
        Ok(())
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
        match self.write_ep.write_transfer(buf, false).await {
            Ok(()) => {
                let max_packet_size = self.max_packet_size() as usize;
                self.needs_zlp = !buf.is_empty() && (buf.len() % max_packet_size == 0);
                Ok(buf.len())
            }
            Err(EndpointDisabledError) => Err(CdcAcmError::NotConnected),
        }
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        if self.needs_zlp {
            self.write_ep
                .write_transfer(&[], false)
                .await
                .map_err(|EndpointDisabledError| CdcAcmError::NotConnected)?;
            self.needs_zlp = false;
        }
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

    /// Gets the line coding the host has set. The line coding contains information that's mainly
    /// relevant for USB to UART serial port emulators, and can be ignored if not relevant.
    ///
    /// Returns `None` until the host sends `SET_LINE_CODING`, and again after a bus reset.
    pub fn line_coding(&self) -> Option<LineCoding> {
        self.control.line_coding.lock(|c| c.get())
    }

    /// Gets the DTR (data terminal ready) state
    pub fn dtr(&self) -> bool {
        self.control.dtr.load(Ordering::Relaxed)
    }

    /// Gets the RTS (request to send) state
    pub fn rts(&self) -> bool {
        self.control.rts.load(Ordering::Relaxed)
    }

    /// Reads data from the OUT endpoint.
    ///
    /// Reads multi-packet transfers into `data` until the buffer is filled or a short packet is received.
    pub async fn read_transfer(&mut self, data: &mut [u8]) -> Result<usize, EndpointError> {
        self.read_ep.read_transfer(data).await
    }

    /// Reads a single packet from the OUT endpoint.
    /// Must be called with a buffer large enough to hold max_packet_size bytes.
    pub async fn read_packet(&mut self, data: &mut [u8]) -> Result<usize, EndpointError> {
        crate::class::read_packet(&mut self.read_ep, data).await
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

    /// Gets the line coding the host has set. The line coding contains information that's mainly
    /// relevant for USB to UART serial port emulators, and can be ignored if not relevant.
    ///
    /// Returns `None` until the host sends `SET_LINE_CODING`, and again after a bus reset.
    pub fn line_coding(&self) -> Option<LineCoding> {
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

        // If the caller's buffer is large enough to contain at least one full packet, read
        // packet-aligned bytes directly into it instead of buffering internally.
        let max_packet_size = self.receiver.max_packet_size() as usize;
        if buf.len() >= max_packet_size {
            let aligned = buf.len() - (buf.len() % max_packet_size);
            return match self.receiver.read_transfer(&mut buf[..aligned]).await {
                Ok(n) => Ok(n),
                Err(EndpointError::BufferOverflow) => unreachable!(),
                Err(_) => Err(CdcAcmError::NotConnected),
            };
        }

        // Otherwise read a packet into the internal buffer, and return some of it to the caller.
        //
        // It's important that `start` and `end` be updated in this order so they're left in a
        // consistent state if the `read` future is dropped mid-execution, e.g. from a timeout.
        match self.receiver.read_packet(self.buffer).await {
            Ok(n) => self.end = n,
            Err(EndpointError::BufferOverflow) => unreachable!(),
            Err(_) => return Err(CdcAcmError::NotConnected),
        }
        self.start = 0;
        Ok(self.read_from_buffer(buf))
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use core::pin::pin;
    use core::task::{Context, Waker};
    use std::sync::Arc;
    use std::task::Wake;
    use std::vec::Vec;

    use embassy_futures::block_on;

    use super::*;
    use crate::class::cdc_acm::{ParityType, StopBits};
    use crate::device::test_utils::{TestEndpoint, TestIo};
    use crate::driver::EndpointType;

    const COMM_IF: InterfaceNumber = InterfaceNumber(0);

    fn control(shared: &ControlShared, send_break: bool) -> Control<'_> {
        Control {
            comm_if: COMM_IF,
            data_if: InterfaceNumber(1),
            send_break,
            shared,
        }
    }

    fn set_line_coding(control: &mut Control<'_>, coding: LineCoding) -> Option<OutResponse> {
        let req = Request::parse(&[0x21, REQ_SET_LINE_CODING, 0, 0, COMM_IF.0, 0, 7, 0]);
        control.control_out(req, &coding.to_bytes())
    }

    fn get_line_coding<'a>(control: &'a mut Control<'_>, buf: &'a mut [u8; 7]) -> Option<InResponse<'a>> {
        let req = Request::parse(&[0xA1, REQ_GET_LINE_CODING, 0, 0, COMM_IF.0, 0, 7, 0]);
        control.control_in(req, buf)
    }

    #[test]
    fn line_coding_is_none_until_host_sets_it() {
        let shared = ControlShared::new();

        assert_eq!(ControlChanged { control: &shared }.line_coding(), None);
    }

    #[test]
    fn line_coding_reports_what_host_set() {
        let shared = ControlShared::new();
        let coding = LineCoding::new(115_200, StopBits::One, ParityType::Even, 8);

        assert_eq!(
            set_line_coding(&mut control(&shared, false), coding),
            Some(OutResponse::Accepted)
        );
        assert_eq!(ControlChanged { control: &shared }.line_coding(), Some(coding));
    }

    #[test]
    fn reset_clears_line_coding() {
        let shared = ControlShared::new();
        let mut control = control(&shared, false);
        let coding = LineCoding::new(9600, StopBits::One, ParityType::None, 8);
        set_line_coding(&mut control, coding);
        assert_eq!(ControlChanged { control: &shared }.line_coding(), Some(coding));

        Handler::reset(&mut control);

        assert_eq!(ControlChanged { control: &shared }.line_coding(), None);
    }

    #[test]
    fn get_line_coding_answers_default_until_host_sets_it() {
        let shared = ControlShared::new();
        let mut buf = [0; 7];

        // 8000 baud, 1 stop bit, no parity, 8 data bits.
        assert_eq!(
            get_line_coding(&mut control(&shared, false), &mut buf),
            Some(InResponse::Accepted(&[0x40, 0x1f, 0x00, 0x00, 0, 0, 8][..]))
        );
    }

    #[test]
    fn get_line_coding_answers_what_host_set() {
        let shared = ControlShared::new();
        let mut control = control(&shared, false);
        set_line_coding(
            &mut control,
            LineCoding::new(115_200, StopBits::Two, ParityType::Odd, 7),
        );
        let mut buf = [0; 7];

        assert_eq!(
            get_line_coding(&mut control, &mut buf),
            Some(InResponse::Accepted(&[0x00, 0xc2, 0x01, 0x00, 2, 1, 7][..]))
        );
    }

    fn request_break(control: &mut Control<'_>, duration: u16) -> Option<OutResponse> {
        let [lo, hi] = duration.to_le_bytes();
        let req = Request::parse(&[0x21, REQ_SEND_BREAK, lo, hi, COMM_IF.0, 0, 0, 0]);
        control.control_out(req, &[])
    }

    /// Remembers whether it was woken.
    struct Woken(AtomicBool);

    impl Wake for Woken {
        fn wake(self: Arc<Self>) {
            self.0.store(true, Ordering::Relaxed);
        }
    }

    /// Checks that `f` wakes up a waiting `control_changed()`.
    fn assert_wakes_control_changed(shared: &ControlShared, f: impl FnOnce()) {
        let changed = ControlChanged { control: shared };
        let woken = Arc::new(Woken(AtomicBool::new(false)));
        let waker = Waker::from(woken.clone());
        let mut cx = Context::from_waker(&waker);
        let mut fut = pin!(changed.control_changed());
        assert!(fut.as_mut().poll(&mut cx).is_pending());

        f();

        assert!(woken.0.load(Ordering::Relaxed));
        assert!(fut.as_mut().poll(&mut cx).is_ready());
    }

    #[test]
    fn send_break_is_rejected_unless_enabled() {
        let shared = ControlShared::new();

        assert_eq!(
            request_break(&mut control(&shared, false), 0xFFFF),
            Some(OutResponse::Rejected)
        );
        assert_eq!(ControlChanged { control: &shared }.take_break(), None);
    }

    #[test]
    fn send_break_reports_each_request_once() {
        let shared = ControlShared::new();
        let changed = ControlChanged { control: &shared };

        assert_eq!(
            request_break(&mut control(&shared, true), 0xFFFF),
            Some(OutResponse::Accepted)
        );
        assert_eq!(changed.take_break(), Some(Break::UntilStopped));
        assert_eq!(changed.take_break(), None);
    }

    #[test]
    fn send_break_decodes_duration() {
        let shared = ControlShared::new();
        let mut control = control(&shared, true);
        let changed = ControlChanged { control: &shared };

        for (duration, expected) in [
            (0, Break::Stop),
            (1, Break::Millis(1)),
            (250, Break::Millis(250)),
            (0xFFFE, Break::Millis(0xFFFE)),
            (0xFFFF, Break::UntilStopped),
        ] {
            assert_eq!(request_break(&mut control, duration), Some(OutResponse::Accepted));
            assert_eq!(changed.take_break(), Some(expected));
        }
    }

    #[test]
    fn later_break_request_replaces_untaken_one() {
        let shared = ControlShared::new();
        let mut control = control(&shared, true);
        let changed = ControlChanged { control: &shared };

        request_break(&mut control, 0xFFFF);
        request_break(&mut control, 0);

        assert_eq!(changed.take_break(), Some(Break::Stop));
        assert_eq!(changed.take_break(), None);
    }

    #[test]
    fn send_break_wakes_control_changed() {
        let shared = ControlShared::new();

        assert_wakes_control_changed(&shared, || {
            request_break(&mut control(&shared, true), 250);
        });
    }

    #[test]
    fn reset_reports_break_stop() {
        let shared = ControlShared::new();
        let mut control = control(&shared, true);
        request_break(&mut control, 0xFFFF);

        Handler::reset(&mut control);

        assert_eq!(ControlChanged { control: &shared }.take_break(), Some(Break::Stop));
    }

    #[test]
    fn disabling_reports_break_stop_and_wakes_control_changed() {
        let shared = ControlShared::new();
        let mut control = control(&shared, true);

        assert_wakes_control_changed(&shared, || Handler::enabled(&mut control, false));

        assert_eq!(ControlChanged { control: &shared }.take_break(), Some(Break::Stop));
    }

    #[test]
    fn reset_and_disable_report_nothing_unless_enabled() {
        let shared = ControlShared::new();
        let mut control = control(&shared, false);

        Handler::reset(&mut control);
        Handler::enabled(&mut control, false);

        assert_eq!(ControlChanged { control: &shared }.take_break(), None);
    }

    #[test]
    fn capabilities_advertise_send_break_only_if_enabled() {
        let mut config = Config::new();
        assert_eq!(acm_capabilities(&config), 0x02);

        config.send_break = true;
        assert_eq!(acm_capabilities(&config), 0x06);
    }

    /// A writer on interface 2, as in a composite device. On interface 0 the tests could not tell
    /// `wIndex` from `wValue`, since both would be 0.
    fn writer(sent: &TestIo) -> SerialStateWriter<TestEndpoint> {
        let ep = TestEndpoint::new(sent, 0x81, EndpointType::Interrupt, 16);
        SerialStateWriter::new(ep, InterfaceNumber(2))
    }

    #[test]
    fn set_lines_sends_dcd_and_dsr() {
        let sent = TestIo::default();
        let mut writer = writer(&sent);

        block_on(writer.set_lines(true, false)).unwrap();
        assert!(writer.dcd() && !writer.dsr());
        block_on(writer.set_lines(true, true)).unwrap();
        block_on(writer.set_lines(false, false)).unwrap();

        // Header: SERIAL_STATE for interface 2, wLength 2. Then the two bytes of UART state.
        assert_eq!(
            sent.data(0x81),
            [
                [0xA1, 0x20, 0, 0, 2, 0, 2, 0, 0x01, 0x00],
                [0xA1, 0x20, 0, 0, 2, 0, 2, 0, 0x03, 0x00],
                [0xA1, 0x20, 0, 0, 2, 0, 2, 0, 0x00, 0x00],
            ]
        );
    }

    #[test]
    fn report_sends_events_once_with_the_lines() {
        let sent = TestIo::default();
        let mut writer = writer(&sent);
        block_on(writer.set_lines(false, true)).unwrap();

        block_on(writer.report(SerialEvents::OVERRUN)).unwrap();
        block_on(writer.set_lines(true, true)).unwrap();

        let sent = sent.data(0x81);
        assert_eq!(sent[1][8..], [0x42, 0x00]);
        assert_eq!(sent[2][8..], [0x03, 0x00]);
    }

    #[test]
    fn report_does_not_repeat_the_previous_events() {
        let sent = TestIo::default();
        let mut writer = writer(&sent);

        block_on(writer.report(SerialEvents::PARITY)).unwrap();
        block_on(writer.report(SerialEvents::BREAK)).unwrap();

        assert_eq!(sent.data(0x81)[1][8..], [0x04, 0x00]);
    }

    #[test]
    fn report_sends_several_events_in_one_notification() {
        let sent = TestIo::default();
        let mut writer = writer(&sent);
        block_on(writer.set_lines(true, false)).unwrap();

        block_on(writer.report(SerialEvents::BREAK | SerialEvents::FRAMING | SerialEvents::OVERRUN)).unwrap();

        let sent = sent.data(0x81);
        assert_eq!(sent.len(), 2);
        assert_eq!(sent[1][8..], [0x55, 0x00]);
    }

    #[test]
    fn set_lines_keeps_the_levels_when_sending_fails() {
        let sent = TestIo::default();
        let mut writer = writer(&sent);
        sent.set_enabled(0x81, false);

        assert!(block_on(writer.set_lines(true, false)).is_err());

        sent.set_enabled(0x81, true);
        block_on(writer.report(SerialEvents::BREAK)).unwrap();
        assert_eq!(sent.data(0x81), [[0xA1, 0x20, 0, 0, 2, 0, 2, 0, 0x05, 0x00]]);
    }

    #[test]
    fn report_uses_the_event_bits_from_the_spec() {
        let sent = TestIo::default();
        let mut writer = writer(&sent);

        for events in [
            SerialEvents::BREAK,
            SerialEvents::RING,
            SerialEvents::FRAMING,
            SerialEvents::PARITY,
            SerialEvents::OVERRUN,
        ] {
            block_on(writer.report(events)).unwrap();
        }

        let bits: Vec<u8> = sent.data(0x81).iter().map(|n| n[8]).collect();
        assert_eq!(bits, [0x04, 0x08, 0x10, 0x20, 0x40]);
    }
}
