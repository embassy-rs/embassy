#![no_std]
#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]

//! Implements HID functionality for a usb-device device.

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

use core::mem::MaybeUninit;

use embassy::channel::signal::Signal;
use embassy::time::Duration;
use embassy_usb::driver::{EndpointOut, ReadError};
use embassy_usb::{
    control::{ControlHandler, InResponse, OutResponse, Request, RequestType},
    driver::{Driver, Endpoint, EndpointIn, WriteError},
    UsbDeviceBuilder,
};
use futures_util::future::{select, Either};
use futures_util::pin_mut;
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

pub struct State<'a, const IN_N: usize, const OUT_N: usize> {
    control: MaybeUninit<Control<'a, OUT_N>>,
    out_signal: Signal<(usize, [u8; OUT_N])>,
}

impl<'a, const IN_N: usize, const OUT_N: usize> State<'a, IN_N, OUT_N> {
    pub fn new() -> Self {
        State {
            control: MaybeUninit::uninit(),
            out_signal: Signal::new(),
        }
    }
}

pub struct HidClass<'d, D: Driver<'d>, T, const IN_N: usize> {
    input: ReportWriter<'d, D, IN_N>,
    output: T,
}

impl<'d, D: Driver<'d>, const IN_N: usize> HidClass<'d, D, (), IN_N> {
    /// Creates a new HidClass.
    ///
    /// poll_ms configures how frequently the host should poll for reading/writing
    /// HID reports. A lower value means better throughput & latency, at the expense
    /// of CPU on the device & bandwidth on the bus. A value of 10 is reasonable for
    /// high performance uses, and a value of 255 is good for best-effort usecases.
    ///
    /// This allocates an IN endpoint only.
    pub fn new(
        builder: &mut UsbDeviceBuilder<'d, D>,
        state: &'d mut State<'d, IN_N, 0>,
        report_descriptor: &'static [u8],
        request_handler: Option<&'d dyn RequestHandler>,
        poll_ms: u8,
        max_packet_size: u16,
    ) -> Self {
        let ep_in = builder.alloc_interrupt_endpoint_in(max_packet_size, poll_ms);
        let control = state
            .control
            .write(Control::new(report_descriptor, None, request_handler));

        control.build(builder, None, &ep_in);

        Self {
            input: ReportWriter { ep_in },
            output: (),
        }
    }
}

impl<'d, D: Driver<'d>, T, const IN_N: usize> HidClass<'d, D, T, IN_N> {
    /// Gets the [`ReportWriter`] for input reports.
    ///
    /// **Note:** If the `HidClass` was created with [`new_ep_out()`](Self::new_ep_out)
    /// this writer will be useless as no endpoint is availabe to send reports.
    pub fn input(&mut self) -> &mut ReportWriter<'d, D, IN_N> {
        &mut self.input
    }
}

impl<'d, D: Driver<'d>, const IN_N: usize, const OUT_N: usize>
    HidClass<'d, D, ReportReader<'d, D, OUT_N>, IN_N>
{
    /// Creates a new HidClass.
    ///
    /// poll_ms configures how frequently the host should poll for reading/writing
    /// HID reports. A lower value means better throughput & latency, at the expense
    /// of CPU on the device & bandwidth on the bus. A value of 10 is reasonable for
    /// high performance uses, and a value of 255 is good for best-effort usecases.
    ///
    /// This allocates two endpoints (IN and OUT).
    pub fn with_output_ep(
        builder: &mut UsbDeviceBuilder<'d, D>,
        state: &'d mut State<'d, IN_N, OUT_N>,
        report_descriptor: &'static [u8],
        request_handler: Option<&'d dyn RequestHandler>,
        poll_ms: u8,
        max_packet_size: u16,
    ) -> Self {
        let ep_out = Some(builder.alloc_interrupt_endpoint_out(max_packet_size, poll_ms));
        let ep_in = builder.alloc_interrupt_endpoint_in(max_packet_size, poll_ms);

        let control = state.control.write(Control::new(
            report_descriptor,
            Some(&state.out_signal),
            request_handler,
        ));

        control.build(builder, ep_out.as_ref(), &ep_in);

        Self {
            input: ReportWriter { ep_in },
            output: ReportReader {
                ep_out,
                receiver: &state.out_signal,
            },
        }
    }

    /// Gets the [`ReportReader`] for output reports.
    pub fn output(&mut self) -> &mut ReportReader<'d, D, OUT_N> {
        &mut self.output
    }

    /// Splits this `HidClass` into seperate readers/writers for input and output reports.
    pub fn split(self) -> (ReportWriter<'d, D, IN_N>, ReportReader<'d, D, OUT_N>) {
        (self.input, self.output)
    }
}

pub struct ReportWriter<'d, D: Driver<'d>, const N: usize> {
    ep_in: D::EndpointIn,
}

pub struct ReportReader<'d, D: Driver<'d>, const N: usize> {
    ep_out: Option<D::EndpointOut>,
    receiver: &'d Signal<(usize, [u8; N])>,
}

impl<'d, D: Driver<'d>, const N: usize> ReportWriter<'d, D, N> {
    /// Tries to write an input report by serializing the given report structure.
    ///
    /// Panics if no endpoint is available.
    #[cfg(feature = "usbd-hid")]
    pub async fn serialize<IR: AsInputReport>(&mut self, r: &IR) -> Result<(), WriteError> {
        let mut buf: [u8; N] = [0; N];
        let size = match serialize(&mut buf, r) {
            Ok(size) => size,
            Err(_) => return Err(WriteError::BufferOverflow),
        };
        self.write(&buf[0..size]).await
    }

    /// Writes `report` to its interrupt endpoint.
    ///
    /// Panics if no endpoint is available.
    pub async fn write(&mut self, report: &[u8]) -> Result<(), WriteError> {
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

impl<'d, D: Driver<'d>, const N: usize> ReportReader<'d, D, N> {
    pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize, ReadError> {
        assert!(buf.len() >= N);
        if let Some(ep) = &mut self.ep_out {
            let max_packet_size = usize::from(ep.info().max_packet_size);

            let mut chunks = buf.chunks_mut(max_packet_size);

            // Wait until we've received a chunk from the endpoint or a report from a SET_REPORT control request
            let (mut total, data) = {
                let chunk = unwrap!(chunks.next());
                let fut1 = ep.read(chunk);
                pin_mut!(fut1);
                match select(fut1, self.receiver.wait()).await {
                    Either::Left((Ok(size), _)) => (size, None),
                    Either::Left((Err(err), _)) => return Err(err),
                    Either::Right(((size, data), _)) => (size, Some(data)),
                }
            };

            if let Some(data) = data {
                buf[0..total].copy_from_slice(&data[0..total]);
                Ok(total)
            } else {
                for chunk in chunks {
                    let size = ep.read(chunk).await?;
                    total += size;
                    if size < max_packet_size || total == N {
                        break;
                    }
                }
                Ok(total)
            }
        } else {
            let (total, data) = self.receiver.wait().await;
            buf[0..total].copy_from_slice(&data[0..total]);
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
    ///
    /// If an output endpoint has been allocated, output reports
    /// are routed through [`HidClass::output()`]. Otherwise they
    /// are sent here, along with input and feature reports.
    fn set_report(&self, id: ReportId, data: &[u8]) -> OutResponse {
        let _ = (id, data);
        OutResponse::Rejected
    }

    /// Get the idle rate for `id`.
    ///
    /// If `id` is `None`, get the idle rate for all reports. Returning `None`
    /// will reject the control request. Any duration above 1.020 seconds or 0
    /// will be returned as an indefinite idle rate.
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

struct Control<'d, const OUT_N: usize> {
    report_descriptor: &'static [u8],
    out_signal: Option<&'d Signal<(usize, [u8; OUT_N])>>,
    request_handler: Option<&'d dyn RequestHandler>,
    hid_descriptor: [u8; 9],
}

impl<'a, const OUT_N: usize> Control<'a, OUT_N> {
    fn new(
        report_descriptor: &'static [u8],
        out_signal: Option<&'a Signal<(usize, [u8; OUT_N])>>,
        request_handler: Option<&'a dyn RequestHandler>,
    ) -> Self {
        Control {
            report_descriptor,
            out_signal,
            request_handler,
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

    fn build<'d, D: Driver<'d>>(
        &'d mut self,
        builder: &mut UsbDeviceBuilder<'d, D>,
        ep_out: Option<&D::EndpointOut>,
        ep_in: &D::EndpointIn,
    ) {
        let len = self.report_descriptor.len();
        let if_num = builder.alloc_interface_with_handler(self);

        builder.config_descriptor.interface(
            if_num,
            USB_CLASS_HID,
            USB_SUBCLASS_NONE,
            USB_PROTOCOL_NONE,
        );

        // HID descriptor
        builder.config_descriptor.write(
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

        builder.config_descriptor.endpoint(ep_in.info());
        if let Some(ep) = ep_out {
            builder.config_descriptor.endpoint(ep.info());
        }
    }
}

impl<'d, const OUT_N: usize> ControlHandler for Control<'d, OUT_N> {
    fn reset(&mut self) {}

    fn control_out(&mut self, req: embassy_usb::control::Request, data: &[u8]) -> OutResponse {
        trace!("HID control_out {:?} {=[u8]:x}", req, data);
        if let RequestType::Class = req.request_type {
            match req.request {
                HID_REQ_SET_IDLE => {
                    if let Some(handler) = self.request_handler.as_ref() {
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
                HID_REQ_SET_REPORT => match (
                    ReportId::try_from(req.value),
                    self.out_signal,
                    self.request_handler.as_ref(),
                ) {
                    (Ok(ReportId::Out(_)), Some(signal), _) => {
                        let mut buf = [0; OUT_N];
                        buf[0..data.len()].copy_from_slice(data);
                        if signal.signaled() {
                            warn!("Output report dropped before being read!");
                        }
                        signal.signal((data.len(), buf));
                        OutResponse::Accepted
                    }
                    (Ok(id), _, Some(handler)) => handler.set_report(id, data),
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
        match (req.request_type, req.request) {
            (RequestType::Standard, Request::GET_DESCRIPTOR) => match (req.value >> 8) as u8 {
                HID_DESC_DESCTYPE_HID_REPORT => InResponse::Accepted(self.report_descriptor),
                HID_DESC_DESCTYPE_HID => InResponse::Accepted(&self.hid_descriptor),
                _ => InResponse::Rejected,
            },
            (RequestType::Class, HID_REQ_GET_REPORT) => {
                let size = match ReportId::try_from(req.value) {
                    Ok(id) => self
                        .request_handler
                        .as_ref()
                        .and_then(|x| x.get_report(id, buf)),
                    Err(_) => None,
                };

                if let Some(size) = size {
                    InResponse::Accepted(&buf[0..size])
                } else {
                    InResponse::Rejected
                }
            }
            (RequestType::Class, HID_REQ_GET_IDLE) => {
                if let Some(handler) = self.request_handler.as_ref() {
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
            (RequestType::Class, HID_REQ_GET_PROTOCOL) => {
                // UNSUPPORTED: Boot Protocol
                buf[0] = 1;
                InResponse::Accepted(&buf[0..1])
            }
            _ => InResponse::Rejected,
        }
    }
}
