//! USB HID (Human Interface Device) class implementation.

use core::mem::MaybeUninit;
use core::ops::Range;
use core::sync::atomic::{AtomicUsize, Ordering};

#[cfg(feature = "usbd-hid")]
use ssmarshal::serialize;
#[cfg(feature = "usbd-hid")]
use usbd_hid::descriptor::AsInputReport;

use crate::control::{InResponse, OutResponse, Recipient, Request, RequestType};
use crate::driver::{Driver, Endpoint, EndpointError, EndpointIn, EndpointOut};
use crate::types::InterfaceNumber;
use crate::{Builder, Handler};

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

/// Configuration for the HID class.
pub struct Config<'d> {
    /// HID report descriptor.
    pub report_descriptor: &'d [u8],

    /// Handler for control requests.
    pub request_handler: Option<&'d mut dyn RequestHandler>,

    /// Configures how frequently the host should poll for reading/writing HID reports.
    ///
    /// A lower value means better throughput & latency, at the expense
    /// of CPU on the device & bandwidth on the bus. A value of 10 is reasonable for
    /// high performance uses, and a value of 255 is good for best-effort usecases.
    pub poll_ms: u8,

    /// Max packet size for both the IN and OUT endpoints.
    pub max_packet_size: u16,
}

/// Report ID
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ReportId {
    /// IN report
    In(u8),
    /// OUT report
    Out(u8),
    /// Feature report
    Feature(u8),
}

impl ReportId {
    const fn try_from(value: u16) -> Result<Self, ()> {
        match value >> 8 {
            1 => Ok(ReportId::In(value as u8)),
            2 => Ok(ReportId::Out(value as u8)),
            3 => Ok(ReportId::Feature(value as u8)),
            _ => Err(()),
        }
    }
}

/// Internal state for USB HID.
pub struct State<'d> {
    control: MaybeUninit<Control<'d>>,
    out_report_offset: AtomicUsize,
}

impl<'d> Default for State<'d> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'d> State<'d> {
    /// Create a new `State`.
    pub const fn new() -> Self {
        State {
            control: MaybeUninit::uninit(),
            out_report_offset: AtomicUsize::new(0),
        }
    }
}

/// USB HID reader/writer.
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
    let len = config.report_descriptor.len();

    let mut func = builder.function(USB_CLASS_HID, USB_SUBCLASS_NONE, USB_PROTOCOL_NONE);
    let mut iface = func.interface();
    let if_num = iface.interface_number();
    let mut alt = iface.alt_setting(USB_CLASS_HID, USB_SUBCLASS_NONE, USB_PROTOCOL_NONE, None);

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

    drop(func);

    let control = state.control.write(Control::new(
        if_num,
        config.report_descriptor,
        config.request_handler,
        &state.out_report_offset,
    ));
    builder.handler(control);

    (ep_out, ep_in, &state.out_report_offset)
}

impl<'d, D: Driver<'d>, const READ_N: usize, const WRITE_N: usize> HidReaderWriter<'d, D, READ_N, WRITE_N> {
    /// Creates a new `HidReaderWriter`.
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

    /// Splits into separate readers/writers for input and output reports.
    pub fn split(self) -> (HidReader<'d, D, READ_N>, HidWriter<'d, D, WRITE_N>) {
        (self.reader, self.writer)
    }

    /// Waits for both IN and OUT endpoints to be enabled.
    pub async fn ready(&mut self) {
        self.reader.ready().await;
        self.writer.ready().await;
    }

    /// Writes an input report by serializing the given report structure.
    #[cfg(feature = "usbd-hid")]
    pub async fn write_serialize<IR: AsInputReport>(&mut self, r: &IR) -> Result<(), EndpointError> {
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

/// USB HID writer.
///
/// You can obtain a `HidWriter` using [`HidReaderWriter::split`].
pub struct HidWriter<'d, D: Driver<'d>, const N: usize> {
    ep_in: D::EndpointIn,
}

/// USB HID reader.
///
/// You can obtain a `HidReader` using [`HidReaderWriter::split`].
pub struct HidReader<'d, D: Driver<'d>, const N: usize> {
    ep_out: D::EndpointOut,
    offset: &'d AtomicUsize,
}

/// Error when reading a HID report.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ReadError {
    /// The given buffer was too small to read the received report.
    BufferOverflow,
    /// The endpoint is disabled.
    Disabled,
    /// The report was only partially read. See [`HidReader::read`] for details.
    Sync(Range<usize>),
}

impl From<EndpointError> for ReadError {
    fn from(val: EndpointError) -> Self {
        use EndpointError::{BufferOverflow, Disabled};
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
    pub async fn ready(&mut self) {
        self.ep_in.wait_enabled().await;
    }

    /// Writes an input report by serializing the given report structure.
    #[cfg(feature = "usbd-hid")]
    pub async fn write_serialize<IR: AsInputReport>(&mut self, r: &IR) -> Result<(), EndpointError> {
        let mut buf: [u8; N] = [0; N];
        let Ok(size) = serialize(&mut buf, r) else {
            return Err(EndpointError::BufferOverflow);
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
    pub async fn ready(&mut self) {
        self.ep_out.wait_enabled().await;
    }

    /// Delivers output reports from the Interrupt Out pipe to `handler`.
    ///
    /// If `use_report_ids` is true, the first byte of the report will be used as
    /// the `ReportId` value. Otherwise the `ReportId` value will be 0.
    pub async fn run<T: RequestHandler>(mut self, use_report_ids: bool, handler: &mut T) -> ! {
        let offset = self.offset.load(Ordering::Acquire);
        assert!(offset == 0);
        let mut buf = [0; N];
        loop {
            match self.read(&mut buf).await {
                Ok(len) => {
                    let id = if use_report_ids { buf[0] } else { 0 };
                    handler.set_report(ReportId::Out(id), &buf[..len]);
                }
                Err(ReadError::BufferOverflow) => warn!(
                    "Host sent output report larger than the configured maximum output report length ({})",
                    N
                ),
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
    /// will return a [`ReadError::Sync`]. The range in the sync error
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
                        }
                        self.offset.store(total, Ordering::Release);
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

/// Handler for HID-related control requests.
pub trait RequestHandler {
    /// Reads the value of report `id` into `buf` returning the size.
    ///
    /// Returns `None` if `id` is invalid or no data is available.
    fn get_report(&mut self, id: ReportId, buf: &mut [u8]) -> Option<usize> {
        let _ = (id, buf);
        None
    }

    /// Sets the value of report `id` to `data`.
    fn set_report(&mut self, id: ReportId, data: &[u8]) -> OutResponse {
        let _ = (id, data);
        OutResponse::Rejected
    }

    /// Get the idle rate for `id`.
    ///
    /// If `id` is `None`, get the idle rate for all reports. Returning `None`
    /// will reject the control request. Any duration at or above 1.024 seconds
    /// or below 4ms will be returned as an indefinite idle rate.
    fn get_idle_ms(&mut self, id: Option<ReportId>) -> Option<u32> {
        let _ = id;
        None
    }

    /// Set the idle rate for `id` to `dur`.
    ///
    /// If `id` is `None`, set the idle rate of all input reports to `dur`. If
    /// an indefinite duration is requested, `dur` will be set to `u32::MAX`.
    fn set_idle_ms(&mut self, id: Option<ReportId>, duration_ms: u32) {
        let _ = (id, duration_ms);
    }
}

struct Control<'d> {
    if_num: InterfaceNumber,
    report_descriptor: &'d [u8],
    request_handler: Option<&'d mut dyn RequestHandler>,
    out_report_offset: &'d AtomicUsize,
    hid_descriptor: [u8; 9],
}

impl<'d> Control<'d> {
    fn new(
        if_num: InterfaceNumber,
        report_descriptor: &'d [u8],
        request_handler: Option<&'d mut dyn RequestHandler>,
        out_report_offset: &'d AtomicUsize,
    ) -> Self {
        Control {
            if_num,
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

impl<'d> Handler for Control<'d> {
    fn reset(&mut self) {
        self.out_report_offset.store(0, Ordering::Release);
    }

    fn control_out(&mut self, req: Request, data: &[u8]) -> Option<OutResponse> {
        if (req.request_type, req.recipient, req.index)
            != (RequestType::Class, Recipient::Interface, self.if_num.0 as u16)
        {
            return None;
        }

        // This uses a defmt-specific formatter that causes use of the `log`
        // feature to fail to build, so leave it defmt-specific for now.
        #[cfg(feature = "defmt")]
        trace!("HID control_out {:?} {=[u8]:x}", req, data);
        match req.request {
            HID_REQ_SET_IDLE => {
                if let Some(handler) = self.request_handler.as_mut() {
                    let id = req.value as u8;
                    let id = (id != 0).then_some(ReportId::In(id));
                    let dur = u32::from(req.value >> 8);
                    let dur = if dur == 0 { u32::MAX } else { 4 * dur };
                    handler.set_idle_ms(id, dur);
                }
                Some(OutResponse::Accepted)
            }
            HID_REQ_SET_REPORT => match (ReportId::try_from(req.value), self.request_handler.as_mut()) {
                (Ok(id), Some(handler)) => Some(handler.set_report(id, data)),
                _ => Some(OutResponse::Rejected),
            },
            HID_REQ_SET_PROTOCOL => {
                if req.value == 1 {
                    Some(OutResponse::Accepted)
                } else {
                    warn!("HID Boot Protocol is unsupported.");
                    Some(OutResponse::Rejected) // UNSUPPORTED: Boot Protocol
                }
            }
            _ => Some(OutResponse::Rejected),
        }
    }

    fn control_in<'a>(&'a mut self, req: Request, buf: &'a mut [u8]) -> Option<InResponse<'a>> {
        if req.index != self.if_num.0 as u16 {
            return None;
        }

        match (req.request_type, req.recipient) {
            (RequestType::Standard, Recipient::Interface) => match req.request {
                Request::GET_DESCRIPTOR => match (req.value >> 8) as u8 {
                    HID_DESC_DESCTYPE_HID_REPORT => Some(InResponse::Accepted(self.report_descriptor)),
                    HID_DESC_DESCTYPE_HID => Some(InResponse::Accepted(&self.hid_descriptor)),
                    _ => Some(InResponse::Rejected),
                },

                _ => Some(InResponse::Rejected),
            },
            (RequestType::Class, Recipient::Interface) => {
                trace!("HID control_in {:?}", req);
                match req.request {
                    HID_REQ_GET_REPORT => {
                        let size = match ReportId::try_from(req.value) {
                            Ok(id) => self.request_handler.as_mut().and_then(|x| x.get_report(id, buf)),
                            Err(_) => None,
                        };

                        if let Some(size) = size {
                            Some(InResponse::Accepted(&buf[0..size]))
                        } else {
                            Some(InResponse::Rejected)
                        }
                    }
                    HID_REQ_GET_IDLE => {
                        if let Some(handler) = self.request_handler.as_mut() {
                            let id = req.value as u8;
                            let id = (id != 0).then_some(ReportId::In(id));
                            if let Some(dur) = handler.get_idle_ms(id) {
                                let dur = u8::try_from(dur / 4).unwrap_or(0);
                                buf[0] = dur;
                                Some(InResponse::Accepted(&buf[0..1]))
                            } else {
                                Some(InResponse::Rejected)
                            }
                        } else {
                            Some(InResponse::Rejected)
                        }
                    }
                    HID_REQ_GET_PROTOCOL => {
                        // UNSUPPORTED: Boot Protocol
                        buf[0] = 1;
                        Some(InResponse::Accepted(&buf[0..1]))
                    }
                    _ => Some(InResponse::Rejected),
                }
            }
            _ => None,
        }
    }
}
