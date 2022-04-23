#![no_std]
#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]

//! Implements HID functionality for a usb-device device.

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

use core::mem::MaybeUninit;
use core::ops::Range;
use core::sync::atomic::{AtomicUsize, Ordering};

use embassy::time::Duration;
use embassy_usb::driver::EndpointOut;
use embassy_usb::{
    control::{ControlHandler, InResponse, OutResponse, Request, RequestType},
    driver::{Driver, Endpoint, EndpointError, EndpointIn},
    Builder,
};

#[cfg(feature = "usbd-hid")]
use ssmarshal::serialize;
#[cfg(feature = "usbd-hid")]
use usbd_hid::descriptor::AsInputReport;

const USB_CLASS_HID: u8 = 0x03;
const USB_SUBCLASS_NONE: u8 = 0x00;
const USB_PROTOCOL_NONE: u8 = 0x00;

// HID
const HID_DESC_DESCTYPE_HID: u8 = 0x21;
const HID_DESC_DESCTYPE_HID_REPORT: u8 = 0x22;
const HID_DESC_SPEC_1_10: [u8; 2] = [0x10, 0x01];
const HID_DESC_COUNTRY_UNSPEC: u8 = 0x00;

const HID_REQ_SET_IDLE: u8 = 0x0a;
const HID_REQ_GET_IDLE: u8 = 0x02;
const HID_REQ_GET_REPORT: u8 = 0x01;
const HID_REQ_SET_REPORT: u8 = 0x09;
const HID_REQ_GET_PROTOCOL: u8 = 0x03;
const HID_REQ_SET_PROTOCOL: u8 = 0x0b;

pub struct Config<'d> {
    /// HID report descriptor.
    pub report_descriptor: &'d [u8],

    /// Handler for control requests.
    pub request_handler: Option<&'d dyn RequestHandler>,

    /// Configures how frequently the host should poll for reading/writing HID reports.
    ///
    /// A lower value means better throughput & latency, at the expense
    /// of CPU on the device & bandwidth on the bus. A value of 10 is reasonable for
    /// high performance uses, and a value of 255 is good for best-effort usecases.
    pub poll_ms: u8,

    /// Max packet size for both the IN and OUT endpoints.
    pub max_packet_size: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ReportId {
    In(u8),
    Out(u8),
    Feature(u8),
}

impl ReportId {
    fn try_from(value: u16) -> Result<Self, ()> {
        match value >> 8 {
            1 => Ok(ReportId::In(value as u8)),
            2 => Ok(ReportId::Out(value as u8)),
            3 => Ok(ReportId::Feature(value as u8)),
            _ => Err(()),
        }
    }
}

pub struct State<'d> {
    control: MaybeUninit<Control<'d>>,
    out_report_offset: AtomicUsize,
}

impl<'d> State<'d> {
    pub fn new() -> Self {
        State {
            control: MaybeUninit::uninit(),
            out_report_offset: AtomicUsize::new(0),
        }
    }
}

pub struct HidReaderWriter<'d, D: Driver<'d>, const READ_N: usize, const WRITE_N: usize> {
    reader: HidReader<'d, D, READ_N>,
    writer: HidWriter<'d, D, WRITE_N>,
}

fn build<'d, D: Driver<'d>>(
    builder: &mut Builder<'d, D>,
    state: &'d mut State<'d>,
    config: Config<'d>,
    with_out_endpoint: bool,
) -> (Option<D::EndpointOut>, D::EndpointIn, &'d AtomicUsize) {
    let control = state.control.write(Control::new(
        config.report_descriptor,
        config.request_handler,
        &state.out_report_offset,
    ));

    let len = config.report_descriptor.len();

    let mut func = builder.function(USB_CLASS_HID, USB_SUBCLASS_NONE, USB_PROTOCOL_NONE);
    let mut iface = func.interface();
    iface.handler(control);
    let mut alt = iface.alt_setting(USB_CLASS_HID, USB_SUBCLASS_NONE, USB_PROTOCOL_NONE);

    // HID descriptor
    alt.descriptor(
        HID_DESC_DESCTYPE_HID,
        &[
            // HID Class spec version
            HID_DESC_SPEC_1_10[0],
            HID_DESC_SPEC_1_10[1],
            // Country code not supported
            HID_DESC_COUNTRY_UNSPEC,
            // Number of following descriptors
            1,
            // We have a HID report descriptor the host should read
            HID_DESC_DESCTYPE_HID_REPORT,
            // HID report descriptor size,
            (len & 0xFF) as u8,
            (len >> 8 & 0xFF) as u8,
        ],
    );

    let ep_in = alt.endpoint_interrupt_in(config.max_packet_size, config.poll_ms);
    let ep_out = if with_out_endpoint {
        Some(alt.endpoint_interrupt_out(config.max_packet_size, config.poll_ms))
    } else {
        None
    };

    (ep_out, ep_in, &state.out_report_offset)
}

impl<'d, D: Driver<'d>, const READ_N: usize, const WRITE_N: usize>
    HidReaderWriter<'d, D, READ_N, WRITE_N>
{
    /// Creates a new HidReaderWriter.
    ///
    /// This will allocate one IN and one OUT endpoints. If you only need writing (sending)
    /// HID reports, consider using [`HidWriter::new`] instead, which allocates an IN endpoint only.
    ///
    pub fn new(builder: &mut Builder<'d, D>, state: &'d mut State<'d>, config: Config<'d>) -> Self {
        let (ep_out, ep_in, offset) = build(builder, state, config, true);

        Self {
            reader: HidReader {
                ep_out: ep_out.unwrap(),
                offset,
            },
            writer: HidWriter { ep_in },
        }
    }

    /// Splits into seperate readers/writers for input and output reports.
    pub fn split(self) -> (HidReader<'d, D, READ_N>, HidWriter<'d, D, WRITE_N>) {
        (self.reader, self.writer)
    }

    /// Waits for both IN and OUT endpoints to be enabled.
    pub async fn ready(&mut self) -> () {
        self.reader.ready().await;
        self.writer.ready().await;
    }

    /// Writes an input report by serializing the given report structure.
    #[cfg(feature = "usbd-hid")]
    pub async fn write_serialize<IR: AsInputReport>(
        &mut self,
        r: &IR,
    ) -> Result<(), EndpointError> {
        self.writer.write_serialize(r).await
    }

    /// Writes `report` to its interrupt endpoint.
    pub async fn write(&mut self, report: &[u8]) -> Result<(), EndpointError> {
        self.writer.write(report).await
    }

    /// Reads an output report from the Interrupt Out pipe.
    ///
    /// See [`HidReader::read`].
    pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize, ReadError> {
        self.reader.read(buf).await
    }
}

pub struct HidWriter<'d, D: Driver<'d>, const N: usize> {
    ep_in: D::EndpointIn,
}

pub struct HidReader<'d, D: Driver<'d>, const N: usize> {
    ep_out: D::EndpointOut,
    offset: &'d AtomicUsize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ReadError {
    BufferOverflow,
    Disabled,
    Sync(Range<usize>),
}

impl From<embassy_usb::driver::EndpointError> for ReadError {
    fn from(val: embassy_usb::driver::EndpointError) -> Self {
        use embassy_usb::driver::EndpointError::*;
        match val {
            BufferOverflow => ReadError::BufferOverflow,
            Disabled => ReadError::Disabled,
        }
    }
}

impl<'d, D: Driver<'d>, const N: usize> HidWriter<'d, D, N> {
    /// Creates a new HidWriter.
    ///
    /// This will allocate one IN endpoint only, so the host won't be able to send
    /// reports to us. If you need that, consider using [`HidReaderWriter::new`] instead.
    ///
    /// poll_ms configures how frequently the host should poll for reading/writing
    /// HID reports. A lower value means better throughput & latency, at the expense
    /// of CPU on the device & bandwidth on the bus. A value of 10 is reasonable for
    /// high performance uses, and a value of 255 is good for best-effort usecases.
    pub fn new(builder: &mut Builder<'d, D>, state: &'d mut State<'d>, config: Config<'d>) -> Self {
        let (ep_out, ep_in, _offset) = build(builder, state, config, false);

        assert!(ep_out.is_none());

        Self { ep_in }
    }

    /// Waits for the interrupt in endpoint to be enabled.
    pub async fn ready(&mut self) -> () {
        self.ep_in.wait_enabled().await
    }

    /// Writes an input report by serializing the given report structure.
    #[cfg(feature = "usbd-hid")]
    pub async fn write_serialize<IR: AsInputReport>(
        &mut self,
        r: &IR,
    ) -> Result<(), EndpointError> {
        let mut buf: [u8; N] = [0; N];
        let size = match serialize(&mut buf, r) {
            Ok(size) => size,
            Err(_) => return Err(EndpointError::BufferOverflow),
        };
        self.write(&buf[0..size]).await
    }

    /// Writes `report` to its interrupt endpoint.
    pub async fn write(&mut self, report: &[u8]) -> Result<(), EndpointError> {
        assert!(report.len() <= N);

        let max_packet_size = usize::from(self.ep_in.info().max_packet_size);
        let zlp_needed = report.len() < N && (report.len() % max_packet_size == 0);
        for chunk in report.chunks(max_packet_size) {
            self.ep_in.write(chunk).await?;
        }

        if zlp_needed {
            self.ep_in.write(&[]).await?;
        }

        Ok(())
    }
}

impl<'d, D: Driver<'d>, const N: usize> HidReader<'d, D, N> {
    /// Waits for the interrupt out endpoint to be enabled.
    pub async fn ready(&mut self) -> () {
        self.ep_out.wait_enabled().await
    }

    /// Delivers output reports from the Interrupt Out pipe to `handler`.
    ///
    /// If `use_report_ids` is true, the first byte of the report will be used as
    /// the `ReportId` value. Otherwise the `ReportId` value will be 0.
    pub async fn run<T: RequestHandler>(mut self, use_report_ids: bool, handler: &T) -> ! {
        let offset = self.offset.load(Ordering::Acquire);
        assert!(offset == 0);
        let mut buf = [0; N];
        loop {
            match self.read(&mut buf).await {
                Ok(len) => {
                    let id = if use_report_ids { buf[0] } else { 0 };
                    handler.set_report(ReportId::Out(id), &buf[..len]); }
                Err(ReadError::BufferOverflow) => warn!("Host sent output report larger than the configured maximum output report length ({})", N),
                Err(ReadError::Disabled) => self.ep_out.wait_enabled().await,
                Err(ReadError::Sync(_)) => unreachable!(),
            }
        }
    }

    /// Reads an output report from the Interrupt Out pipe.
    ///
    /// **Note:** Any reports sent from the host over the control pipe will be
    /// passed to [`RequestHandler::set_report()`] for handling. The application
    /// is responsible for ensuring output reports from both pipes are handled
    /// correctly.
    ///
    /// **Note:** If `N` > the maximum packet size of the endpoint (i.e. output
    /// reports may be split across multiple packets) and this method's future
    /// is dropped after some packets have been read, the next call to `read()`
    /// will return a [`ReadError::SyncError()`]. The range in the sync error
    /// indicates the portion `buf` that was filled by the current call to
    /// `read()`. If the dropped future used the same `buf`, then `buf` will
    /// contain the full report.
    pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize, ReadError> {
        assert!(N != 0);
        assert!(buf.len() >= N);

        // Read packets from the endpoint
        let max_packet_size = usize::from(self.ep_out.info().max_packet_size);
        let starting_offset = self.offset.load(Ordering::Acquire);
        let mut total = starting_offset;
        loop {
            for chunk in buf[starting_offset..N].chunks_mut(max_packet_size) {
                match self.ep_out.read(chunk).await {
                    Ok(size) => {
                        total += size;
                        if size < max_packet_size || total == N {
                            self.offset.store(0, Ordering::Release);
                            break;
                        } else {
                            self.offset.store(total, Ordering::Release);
                        }
                    }
                    Err(err) => {
                        self.offset.store(0, Ordering::Release);
                        return Err(err.into());
                    }
                }
            }

            // Some hosts may send ZLPs even when not required by the HID spec, so we'll loop as long as total == 0.
            if total > 0 {
                break;
            }
        }

        if starting_offset > 0 {
            Err(ReadError::Sync(starting_offset..total))
        } else {
            Ok(total)
        }
    }
}

pub trait RequestHandler {
    /// Reads the value of report `id` into `buf` returning the size.
    ///
    /// Returns `None` if `id` is invalid or no data is available.
    fn get_report(&self, id: ReportId, buf: &mut [u8]) -> Option<usize> {
        let _ = (id, buf);
        None
    }

    /// Sets the value of report `id` to `data`.
    fn set_report(&self, id: ReportId, data: &[u8]) -> OutResponse {
        let _ = (id, data);
        OutResponse::Rejected
    }

    /// Get the idle rate for `id`.
    ///
    /// If `id` is `None`, get the idle rate for all reports. Returning `None`
    /// will reject the control request. Any duration at or above 1.024 seconds
    /// or below 4ms will be returned as an indefinite idle rate.
    fn get_idle(&self, id: Option<ReportId>) -> Option<Duration> {
        let _ = id;
        None
    }

    /// Set the idle rate for `id` to `dur`.
    ///
    /// If `id` is `None`, set the idle rate of all input reports to `dur`. If
    /// an indefinite duration is requested, `dur` will be set to `Duration::MAX`.
    fn set_idle(&self, id: Option<ReportId>, dur: Duration) {
        let _ = (id, dur);
    }
}

struct Control<'d> {
    report_descriptor: &'d [u8],
    request_handler: Option<&'d dyn RequestHandler>,
    out_report_offset: &'d AtomicUsize,
    hid_descriptor: [u8; 9],
}

impl<'d> Control<'d> {
    fn new(
        report_descriptor: &'d [u8],
        request_handler: Option<&'d dyn RequestHandler>,
        out_report_offset: &'d AtomicUsize,
    ) -> Self {
        Control {
            report_descriptor,
            request_handler,
            out_report_offset,
            hid_descriptor: [
                // Length of buf inclusive of size prefix
                9,
                // Descriptor type
                HID_DESC_DESCTYPE_HID,
                // HID Class spec version
                HID_DESC_SPEC_1_10[0],
                HID_DESC_SPEC_1_10[1],
                // Country code not supported
                HID_DESC_COUNTRY_UNSPEC,
                // Number of following descriptors
                1,
                // We have a HID report descriptor the host should read
                HID_DESC_DESCTYPE_HID_REPORT,
                // HID report descriptor size,
                (report_descriptor.len() & 0xFF) as u8,
                (report_descriptor.len() >> 8 & 0xFF) as u8,
            ],
        }
    }
}

impl<'d> ControlHandler for Control<'d> {
    fn reset(&mut self) {
        self.out_report_offset.store(0, Ordering::Release);
    }

    fn get_descriptor<'a>(&'a mut self, req: Request, _buf: &'a mut [u8]) -> InResponse<'a> {
        match (req.value >> 8) as u8 {
            HID_DESC_DESCTYPE_HID_REPORT => InResponse::Accepted(self.report_descriptor),
            HID_DESC_DESCTYPE_HID => InResponse::Accepted(&self.hid_descriptor),
            _ => InResponse::Rejected,
        }
    }

    fn control_out(&mut self, req: embassy_usb::control::Request, data: &[u8]) -> OutResponse {
        trace!("HID control_out {:?} {=[u8]:x}", req, data);
        if let RequestType::Class = req.request_type {
            match req.request {
                HID_REQ_SET_IDLE => {
                    if let Some(handler) = self.request_handler {
                        let id = req.value as u8;
                        let id = (id != 0).then(|| ReportId::In(id));
                        let dur = u64::from(req.value >> 8);
                        let dur = if dur == 0 {
                            Duration::MAX
                        } else {
                            Duration::from_millis(4 * dur)
                        };
                        handler.set_idle(id, dur);
                    }
                    OutResponse::Accepted
                }
                HID_REQ_SET_REPORT => match (ReportId::try_from(req.value), self.request_handler) {
                    (Ok(id), Some(handler)) => handler.set_report(id, data),
                    _ => OutResponse::Rejected,
                },
                HID_REQ_SET_PROTOCOL => {
                    if req.value == 1 {
                        OutResponse::Accepted
                    } else {
                        warn!("HID Boot Protocol is unsupported.");
                        OutResponse::Rejected // UNSUPPORTED: Boot Protocol
                    }
                }
                _ => OutResponse::Rejected,
            }
        } else {
            OutResponse::Rejected // UNSUPPORTED: SET_DESCRIPTOR
        }
    }

    fn control_in<'a>(&'a mut self, req: Request, buf: &'a mut [u8]) -> InResponse<'a> {
        trace!("HID control_in {:?}", req);
        match req.request {
            HID_REQ_GET_REPORT => {
                let size = match ReportId::try_from(req.value) {
                    Ok(id) => self.request_handler.and_then(|x| x.get_report(id, buf)),
                    Err(_) => None,
                };

                if let Some(size) = size {
                    InResponse::Accepted(&buf[0..size])
                } else {
                    InResponse::Rejected
                }
            }
            HID_REQ_GET_IDLE => {
                if let Some(handler) = self.request_handler {
                    let id = req.value as u8;
                    let id = (id != 0).then(|| ReportId::In(id));
                    if let Some(dur) = handler.get_idle(id) {
                        let dur = u8::try_from(dur.as_millis() / 4).unwrap_or(0);
                        buf[0] = dur;
                        InResponse::Accepted(&buf[0..1])
                    } else {
                        InResponse::Rejected
                    }
                } else {
                    InResponse::Rejected
                }
            }
            HID_REQ_GET_PROTOCOL => {
                // UNSUPPORTED: Boot Protocol
                buf[0] = 1;
                InResponse::Accepted(&buf[0..1])
            }
            _ => InResponse::Rejected,
        }
    }
}
