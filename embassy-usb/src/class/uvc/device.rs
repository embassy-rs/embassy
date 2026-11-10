//! USB Video Class 1.1 - camera device (device to host).
//!
//! A single video stream over an isochronous IN endpoint. The camera offers any number of
//! [`Format`]s (MJPEG, YUY2 or NV12), each with one or more [`Frame`] sizes and frame intervals.
//! The host picks one through the probe/commit handshake; [`Camera::settings`] reports the result
//! and [`Camera::write_frame`] sends a frame, adding the payload headers.
//!
//! The camera exposes a camera input terminal wired straight to the USB output terminal, so it has
//! no controls (brightness, exposure, ...) and no still image capture.

use core::cell::Cell;

use embassy_sync::blocking_mutex::CriticalSectionMutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use heapless::Vec;

use crate::control::{self, InResponse, OutResponse, Recipient, Request, RequestType};
use crate::descriptor::{SynchronizationType, UsageType};
use crate::driver::{Driver, Endpoint, EndpointError, EndpointIn, EndpointType};
use crate::types::InterfaceNumber;
use crate::{Builder, Handler, InterfaceHandler, UsbDeviceSpeed};

const USB_CLASS_VIDEO: u8 = 0x0E;
const SC_VIDEOCONTROL: u8 = 0x01;
const SC_VIDEOSTREAMING: u8 = 0x02;
const SC_VIDEO_INTERFACE_COLLECTION: u8 = 0x03;
const PC_PROTOCOL_UNDEFINED: u8 = 0x00;

const CS_INTERFACE: u8 = 0x24;

// Video control interface descriptor subtypes.
const VC_HEADER: u8 = 0x01;
const VC_INPUT_TERMINAL: u8 = 0x02;
const VC_OUTPUT_TERMINAL: u8 = 0x03;

// Video streaming interface descriptor subtypes.
const VS_INPUT_HEADER: u8 = 0x01;
const VS_FORMAT_UNCOMPRESSED: u8 = 0x04;
const VS_FRAME_UNCOMPRESSED: u8 = 0x05;
const VS_FORMAT_MJPEG: u8 = 0x06;
const VS_FRAME_MJPEG: u8 = 0x07;

const ITT_CAMERA: u16 = 0x0201;
const TT_STREAMING: u16 = 0x0101;

// Requests.
const SET_CUR: u8 = 0x01;
const GET_CUR: u8 = 0x81;
const GET_MIN: u8 = 0x82;
const GET_MAX: u8 = 0x83;
const GET_RES: u8 = 0x84;
const GET_LEN: u8 = 0x85;
const GET_INFO: u8 = 0x86;
const GET_DEF: u8 = 0x87;

// Video streaming interface control selectors.
const VS_PROBE_CONTROL: u8 = 0x01;
const VS_COMMIT_CONTROL: u8 = 0x02;

const INPUT_TERMINAL_ID: u8 = 1;
const OUTPUT_TERMINAL_ID: u8 = 2;

/// `dwClockFrequency` of the video control header: unused, but the field must be set.
const CLOCK_FREQUENCY_HZ: u32 = 48_000_000;

/// Size of the UVC 1.1 probe/commit control.
const STREAM_CONTROL_LEN: usize = 34;
/// Size of the UVC 1.0 probe/commit control, which hosts may still send.
const STREAM_CONTROL_LEN_V10: usize = 26;

/// Every payload starts with a header: its length and the `BFH` bit field.
const PAYLOAD_HEADER_LEN: usize = 2;
const HEADER_FID: u8 = 1 << 0;
const HEADER_EOF: u8 = 1 << 1;

/// The input header lists one control byte per format, within the 255 byte descriptor limit.
const MAX_FORMATS: usize = 16;

/// Frame descriptors are at most 255 bytes: 26 plus four per frame interval.
const MAX_FRAME_INTERVALS: usize = (255 - 26) / 4;

/// Largest packet an isochronous endpoint takes without extra transactions per microframe.
const MAX_PACKET_SIZE_FULL_SPEED: u16 = 1023;
const MAX_PACKET_SIZE_HIGH_SPEED: u16 = 1024;

/// The pixel format of a [`Format`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum PixelFormat {
    /// Motion JPEG: each frame is a complete JPEG image.
    Mjpeg,
    /// Uncompressed packed YUV 4:2:2, 16 bits per pixel (`Y0 U Y1 V`).
    Yuy2,
    /// Uncompressed planar YUV 4:2:0, 12 bits per pixel: the Y plane, then interleaved `U V`.
    Nv12,
}

impl PixelFormat {
    /// Bits per pixel, as far as it is fixed.
    const fn bits_per_pixel(self) -> u32 {
        match self {
            PixelFormat::Mjpeg | PixelFormat::Yuy2 => 16,
            PixelFormat::Nv12 => 12,
        }
    }

    /// The `guidFormat` of an uncompressed format.
    const fn guid(self) -> Option<[u8; 16]> {
        // The fourcc, then the GUID tail every UVC format shares.
        const fn guid(fourcc: [u8; 4]) -> [u8; 16] {
            [
                fourcc[0], fourcc[1], fourcc[2], fourcc[3], 0x00, 0x00, 0x10, 0x00, 0x80, 0x00, 0x00, 0xAA, 0x00, 0x38,
                0x9B, 0x71,
            ]
        }
        match self {
            PixelFormat::Mjpeg => None,
            PixelFormat::Yuy2 => Some(guid(*b"YUY2")),
            PixelFormat::Nv12 => Some(guid(*b"NV12")),
        }
    }
}

/// A frame size of a [`Format`], with the frame intervals it supports.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Frame<'d> {
    /// Width in pixels.
    pub width: u16,
    /// Height in pixels.
    pub height: u16,
    /// The supported frame intervals in 100 ns units, e.g. `333_333` for 30 fps. At least one, at
    /// most 56. The first is the default.
    pub intervals: &'d [u32],
    /// Upper bound of a frame's size in bytes. `None` means the size of an uncompressed 4:2:2
    /// image, which no JPEG exceeds in practice.
    pub max_frame_size: Option<u32>,
}

impl<'d> Frame<'d> {
    /// Creates a frame size.
    pub const fn new(width: u16, height: u16, intervals: &'d [u32]) -> Self {
        Self {
            width,
            height,
            intervals,
            max_frame_size: None,
        }
    }

    /// Frame interval in 100 ns units for a frame rate.
    pub const fn interval_for_fps(fps: u32) -> u32 {
        10_000_000 / fps
    }

    fn frame_size(&self, pixel_format: PixelFormat) -> u32 {
        let pixels = self.width as u32 * self.height as u32;
        match self.max_frame_size {
            Some(size) => size,
            None => match pixel_format {
                PixelFormat::Nv12 => pixels * 3 / 2,
                PixelFormat::Mjpeg | PixelFormat::Yuy2 => pixels * 2,
            },
        }
    }

    fn assert_valid(&self) {
        assert!(self.width != 0 && self.height != 0, "frame size must not be zero");
        assert!(
            (1..=MAX_FRAME_INTERVALS).contains(&self.intervals.len()),
            "between one and {} frame intervals per frame",
            MAX_FRAME_INTERVALS
        );
        assert!(
            self.intervals.iter().all(|&i| i != 0),
            "frame intervals must not be zero"
        );
    }
}

/// A video format the camera can stream, with its frame sizes.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Format<'d> {
    /// The pixel format.
    pub pixel_format: PixelFormat,
    /// The frame sizes, at least one. The first is the default.
    pub frames: &'d [Frame<'d>],
}

impl<'d> Format<'d> {
    /// Creates a format.
    pub const fn new(pixel_format: PixelFormat, frames: &'d [Frame<'d>]) -> Self {
        Self { pixel_format, frames }
    }
}

/// Everything the camera advertises to the host.
#[derive(Clone)]
pub struct Config<'d> {
    /// The formats the camera streams, at least one and at most 16. The host numbers them from
    /// one, in this order. The first is the default.
    pub formats: &'d [Format<'d>],
    /// Packet size of the isochronous endpoint in bytes, including the 2-byte payload header.
    /// Bounds the data rate: 1023 at full speed, 1024 at high speed, per (micro)frame.
    pub max_packet_size: u16,
}

impl<'d> Config<'d> {
    /// Creates a camera configuration. The packet size is the largest the speed allows.
    pub const fn new(formats: &'d [Format<'d>], speed: UsbDeviceSpeed) -> Self {
        Self {
            formats,
            max_packet_size: match speed {
                UsbDeviceSpeed::Full => MAX_PACKET_SIZE_FULL_SPEED,
                UsbDeviceSpeed::High => MAX_PACKET_SIZE_HIGH_SPEED,
            },
        }
    }
}

/// What the host selected for the stream.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct StreamSettings<'d> {
    /// The selected format.
    pub format: &'d Format<'d>,
    /// The selected frame size of `format`.
    pub frame: &'d Frame<'d>,
    /// The frame interval in 100 ns units.
    pub frame_interval: u32,
}

/// The probe/commit control [UVC 1.1 4.3.1.1].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct StreamControl {
    hint: u16,
    format_index: u8,
    frame_index: u8,
    frame_interval: u32,
    key_frame_rate: u16,
    p_frame_rate: u16,
    comp_quality: u16,
    comp_window_size: u16,
    delay: u16,
    max_video_frame_size: u32,
    max_payload_transfer_size: u32,
    clock_frequency: u32,
    framing_info: u8,
    preferred_version: u8,
    min_version: u8,
    max_version: u8,
}

impl StreamControl {
    /// Parses a control the host sent. The UVC 1.0 layout is a prefix of the 1.1 one.
    fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < STREAM_CONTROL_LEN_V10 {
            return None;
        }
        let u16_at = |i: usize| u16::from_le_bytes([data[i], data[i + 1]]);
        let u32_at = |i: usize| u32::from_le_bytes([data[i], data[i + 1], data[i + 2], data[i + 3]]);
        let v11 = data.len() >= STREAM_CONTROL_LEN;
        Some(Self {
            hint: u16_at(0),
            format_index: data[2],
            frame_index: data[3],
            frame_interval: u32_at(4),
            key_frame_rate: u16_at(8),
            p_frame_rate: u16_at(10),
            comp_quality: u16_at(12),
            comp_window_size: u16_at(14),
            delay: u16_at(16),
            max_video_frame_size: u32_at(18),
            max_payload_transfer_size: u32_at(22),
            clock_frequency: if v11 { u32_at(26) } else { 0 },
            framing_info: if v11 { data[30] } else { 0 },
            preferred_version: if v11 { data[31] } else { 0 },
            min_version: if v11 { data[32] } else { 0 },
            max_version: if v11 { data[33] } else { 0 },
        })
    }

    fn to_bytes(self) -> [u8; STREAM_CONTROL_LEN] {
        let mut b = [0; STREAM_CONTROL_LEN];
        b[0..2].copy_from_slice(&self.hint.to_le_bytes());
        b[2] = self.format_index;
        b[3] = self.frame_index;
        b[4..8].copy_from_slice(&self.frame_interval.to_le_bytes());
        b[8..10].copy_from_slice(&self.key_frame_rate.to_le_bytes());
        b[10..12].copy_from_slice(&self.p_frame_rate.to_le_bytes());
        b[12..14].copy_from_slice(&self.comp_quality.to_le_bytes());
        b[14..16].copy_from_slice(&self.comp_window_size.to_le_bytes());
        b[16..18].copy_from_slice(&self.delay.to_le_bytes());
        b[18..22].copy_from_slice(&self.max_video_frame_size.to_le_bytes());
        b[22..26].copy_from_slice(&self.max_payload_transfer_size.to_le_bytes());
        b[26..30].copy_from_slice(&self.clock_frequency.to_le_bytes());
        b[30] = self.framing_info;
        b[31] = self.preferred_version;
        b[32] = self.min_version;
        b[33] = self.max_version;
        b
    }
}

/// The formats a camera offers, and what it takes to stream one.
#[derive(Clone, Copy)]
struct Catalog<'d> {
    formats: &'d [Format<'d>],
    max_packet_size: u16,
}

impl<'d> Catalog<'d> {
    /// The control for a frame: what the host gets back as the probe result.
    fn control_for(&self, format_index: u8, frame_index: u8, frame_interval: u32) -> StreamControl {
        let format = &self.formats[format_index as usize - 1];
        let frame = &format.frames[frame_index as usize - 1];
        StreamControl {
            hint: 0,
            format_index,
            frame_index,
            frame_interval,
            max_video_frame_size: frame.frame_size(format.pixel_format),
            max_payload_transfer_size: self.max_packet_size as u32,
            clock_frequency: CLOCK_FREQUENCY_HZ,
            // Every payload has the frame ID and end-of-frame bits.
            framing_info: 0b11,
            ..StreamControl::default()
        }
    }

    /// The default format, frame and interval.
    fn default_control(&self) -> StreamControl {
        self.control_for(1, 1, self.formats[0].frames[0].intervals[0])
    }

    /// The last format and frame, at the shortest interval.
    fn max_control(&self) -> StreamControl {
        let format_index = self.formats.len() as u8;
        let frames = self.formats[format_index as usize - 1].frames;
        let frame_index = frames.len() as u8;
        let interval = frames[frame_index as usize - 1]
            .intervals
            .iter()
            .min()
            .copied()
            .unwrap();
        self.control_for(format_index, frame_index, interval)
    }

    /// Adjusts what the host proposes to what the camera supports: indices out of range fall back
    /// to the first, and the interval to the closest supported one [UVC 1.1 4.3.1.1].
    fn negotiate(&self, proposal: &StreamControl) -> StreamControl {
        let format_index = match proposal.format_index {
            i if (1..=self.formats.len() as u8).contains(&i) => i,
            _ => 1,
        };
        let frames = self.formats[format_index as usize - 1].frames;
        let frame_index = match proposal.frame_index {
            i if (1..=frames.len() as u8).contains(&i) => i,
            _ => 1,
        };
        let intervals = frames[frame_index as usize - 1].intervals;
        let interval = intervals
            .iter()
            .copied()
            .min_by_key(|&i| i.abs_diff(proposal.frame_interval))
            .unwrap();
        self.control_for(format_index, frame_index, interval)
    }
}

/// Mutable state shared by the control handler and the [`Camera`].
#[derive(Clone, Copy)]
struct Inner {
    /// What the next `GET_CUR` on the probe control returns.
    probe: StreamControl,
    /// What the host committed. Starts out as the default, so the camera can stream without a
    /// commit.
    commit: StreamControl,
    /// The streaming interface has its endpoint selected.
    streaming: bool,
}

/// Shared between the control handler and the camera.
struct Shared<'d> {
    catalog: Catalog<'d>,
    inner: CriticalSectionMutex<Cell<Inner>>,
    /// Signalled when streaming starts or stops: single consumer, cross-executor safe.
    changed: Signal<CriticalSectionRawMutex, ()>,
}

impl<'d> Shared<'d> {
    fn update(&self, f: impl FnOnce(&mut Inner)) {
        self.inner.lock(|cell| {
            let mut inner = cell.get();
            f(&mut inner);
            cell.set(inner);
        });
    }

    fn get(&self) -> Inner {
        self.inner.lock(|cell| cell.get())
    }

    fn restore_defaults(&self) {
        let default = self.catalog.default_control();
        self.inner.lock(|cell| {
            cell.set(Inner {
                probe: default,
                commit: default,
                streaming: false,
            })
        });
        self.changed.signal(());
    }
}

/// Internal state for a USB Video Class camera.
pub struct State<'d> {
    control: Option<Control<'d>>,
    shared: Option<Shared<'d>>,
}

impl<'d> State<'d> {
    /// Creates a new `State`.
    pub const fn new() -> Self {
        Self {
            control: None,
            shared: None,
        }
    }
}

impl<'d> Default for State<'d> {
    fn default() -> Self {
        Self::new()
    }
}

struct Control<'d> {
    shared: &'d Shared<'d>,
    video_control: InterfaceNumber,
    video_streaming: InterfaceNumber,
}

impl<'d> Control<'d> {
    fn streaming_get<'a>(&mut self, req: Request, buf: &'a mut [u8]) -> Option<InResponse<'a>> {
        let selector = (req.value >> 8) as u8;
        if selector != VS_PROBE_CONTROL && selector != VS_COMMIT_CONTROL {
            return Some(InResponse::Rejected);
        }
        let probe = selector == VS_PROBE_CONTROL;
        let catalog = &self.shared.catalog;
        let inner = self.shared.get();

        let control = match req.request {
            GET_CUR => Some(if probe { inner.probe } else { inner.commit }),
            // The commit control has no range.
            GET_MIN | GET_DEF if probe => Some(catalog.default_control()),
            GET_MAX if probe => Some(catalog.max_control()),
            GET_RES if probe => Some(StreamControl::default()),
            _ => None,
        };
        if let Some(control) = control {
            let bytes = control.to_bytes();
            let len = bytes.len().min(buf.len());
            buf[..len].copy_from_slice(&bytes[..len]);
            return Some(InResponse::Accepted(&buf[..len]));
        }

        match req.request {
            GET_LEN => {
                buf[..2].copy_from_slice(&(STREAM_CONTROL_LEN as u16).to_le_bytes());
                Some(InResponse::Accepted(&buf[..2]))
            }
            // Supports GET and SET.
            GET_INFO => {
                buf[0] = 0b11;
                Some(InResponse::Accepted(&buf[..1]))
            }
            _ => Some(InResponse::Rejected),
        }
    }

    fn streaming_set(&mut self, req: Request, data: &[u8]) -> Option<OutResponse> {
        if req.request != SET_CUR {
            return Some(OutResponse::Rejected);
        }
        let Some(proposal) = StreamControl::parse(data) else {
            return Some(OutResponse::Rejected);
        };
        let negotiated = self.shared.catalog.negotiate(&proposal);
        match (req.value >> 8) as u8 {
            VS_PROBE_CONTROL => self.shared.update(|inner| inner.probe = negotiated),
            VS_COMMIT_CONTROL => self.shared.update(|inner| inner.commit = negotiated),
            _ => return Some(OutResponse::Rejected),
        }
        Some(OutResponse::Accepted)
    }

    fn is_streaming_request(&self, req: &Request) -> bool {
        req.request_type == RequestType::Class
            && req.recipient == Recipient::Interface
            && req.index as u8 == self.video_streaming.0
    }
}

impl<'d> Handler for Control<'d> {
    fn reset(&mut self) {
        self.shared.restore_defaults();
    }

    fn configured(&mut self, configured: bool) {
        if !configured {
            self.shared.restore_defaults();
        }
    }

    fn set_alternate_setting(&mut self, iface: InterfaceNumber, alternate_setting: u8) {
        if iface == self.video_streaming {
            self.shared.update(|inner| inner.streaming = alternate_setting != 0);
            self.shared.changed.signal(());
        }
    }

    fn control_out(&mut self, req: control::Request, data: &[u8]) -> Option<OutResponse> {
        if self.is_streaming_request(&req) {
            self.streaming_set(req, data)
        } else {
            None
        }
    }

    fn control_in<'a>(&'a mut self, req: Request, buf: &'a mut [u8]) -> Option<InResponse<'a>> {
        if self.is_streaming_request(&req) {
            self.streaming_get(req, buf)
        } else {
            None
        }
    }
}

impl<'d> InterfaceHandler for Control<'d> {
    fn owns(&self, iface: InterfaceNumber) -> bool {
        iface == self.video_control || iface == self.video_streaming
    }
}

/// Implementation of the USB video class 1.1, camera side.
pub struct Camera<'d, D: Driver<'d>> {
    endpoint: D::EndpointIn,
    shared: &'d Shared<'d>,
    max_packet_size: usize,
    /// The frame ID bit of the frame being sent.
    frame_id: bool,
}

impl<'d, D: Driver<'d>> Camera<'d, D> {
    /// Creates a new [`Camera`] device, and registers its control handler.
    ///
    /// # Panics
    ///
    /// If `config` has no format or more than 16, a format without a frame or with more than
    /// 255, a frame of zero size or without a frame interval or with more than 56, a zero frame
    /// interval, a packet size below 3 or above the speed's limit, or the `control_buf` of the
    /// builder is smaller than 34 bytes.
    pub fn new(builder: &mut Builder<'d, D>, state: &'d mut State<'d>, config: Config<'d>) -> Self {
        let Config {
            formats,
            max_packet_size,
        } = config;

        assert!(
            (1..=MAX_FORMATS).contains(&formats.len()),
            "between one and {} formats",
            MAX_FORMATS
        );
        for format in formats {
            assert!(
                (1..=255).contains(&format.frames.len()),
                "between one and 255 frames per format"
            );
            format.frames.iter().for_each(Frame::assert_valid);
        }
        let speed_limit = match builder.max_speed() {
            UsbDeviceSpeed::Full => MAX_PACKET_SIZE_FULL_SPEED,
            UsbDeviceSpeed::High => MAX_PACKET_SIZE_HIGH_SPEED,
        };
        assert!(
            (PAYLOAD_HEADER_LEN as u16 + 1..=speed_limit).contains(&max_packet_size),
            "the packet size must hold the payload header and at most {} bytes at this speed",
            speed_limit
        );
        assert!(
            builder.control_buf_len() >= STREAM_CONTROL_LEN,
            "the control buffer must hold a {} byte stream control",
            STREAM_CONTROL_LEN
        );

        let catalog = Catalog {
            formats,
            max_packet_size,
        };

        let mut func = builder.function(USB_CLASS_VIDEO, SC_VIDEO_INTERFACE_COLLECTION, PC_PROTOCOL_UNDEFINED);

        // Video control interface: a camera feeding the USB stream.
        let mut vc = func.interface();
        let vc_number = vc.interface_number();
        let vs_number = InterfaceNumber::new(vc_number.0 + 1);
        let mut alt = vc.alt_setting(USB_CLASS_VIDEO, SC_VIDEOCONTROL, PC_PROTOCOL_UNDEFINED, None);
        alt.descriptors_then_patch(
            |alt| {
                alt.descriptor(CS_INTERFACE, &vc_header(vs_number));
                alt.descriptor(CS_INTERFACE, &camera_input_terminal());
                alt.descriptor(CS_INTERFACE, &streaming_output_terminal());
            },
            |written| {
                // wTotalLength follows bLength, bDescriptorType, bDescriptorSubtype, bcdUVC.
                let len = (written.len() as u16).to_le_bytes();
                written[5..7].copy_from_slice(&len);
            },
        );

        // Video streaming interface. Alternate setting 0 has no endpoint: no bandwidth is claimed
        // until the host starts the stream by selecting alternate setting 1.
        let mut vs = func.interface();
        assert_eq!(vs.interface_number(), vs_number);
        let mut alt0 = vs.alt_setting(USB_CLASS_VIDEO, SC_VIDEOSTREAMING, PC_PROTOCOL_UNDEFINED, None);
        // The input header names the endpoint, which is only described in alternate setting 1.
        let endpoint = alt0.alloc_endpoint_in(EndpointType::Isochronous, None, max_packet_size, 0);
        let endpoint_address = endpoint.info().addr.into();
        alt0.descriptors_then_patch(
            |alt| {
                alt.descriptor(CS_INTERFACE, &vs_input_header(formats.len() as u8, endpoint_address));
                for (i, format) in formats.iter().enumerate() {
                    alt.descriptor(CS_INTERFACE, &format_descriptor(format, i as u8 + 1));
                    for (j, frame) in format.frames.iter().enumerate() {
                        alt.descriptor(CS_INTERFACE, &frame_descriptor(format, frame, j as u8 + 1));
                    }
                }
            },
            |written| {
                // wTotalLength follows bLength, bDescriptorType, bDescriptorSubtype, bNumFormats.
                let len = (written.len() as u16).to_le_bytes();
                written[4..6].copy_from_slice(&len);
            },
        );
        let mut alt1 = vs.alt_setting(USB_CLASS_VIDEO, SC_VIDEOSTREAMING, PC_PROTOCOL_UNDEFINED, None);
        alt1.endpoint_descriptor(
            endpoint.info(),
            SynchronizationType::Asynchronous,
            UsageType::DataEndpoint,
            &[],
        );
        drop(func);

        let State { control, shared } = state;
        let shared: &'d Shared<'d> = shared.insert(Shared {
            catalog,
            inner: CriticalSectionMutex::new(Cell::new(Inner {
                probe: catalog.default_control(),
                commit: catalog.default_control(),
                streaming: false,
            })),
            changed: Signal::new(),
        });
        builder.interface_handler(control.insert(Control {
            shared,
            video_control: vc_number,
            video_streaming: vs_number,
        }));

        Self {
            endpoint,
            shared,
            max_packet_size: max_packet_size as usize,
            frame_id: false,
        }
    }

    /// What the host selected for the stream, or the defaults before it selects anything.
    pub fn settings(&self) -> StreamSettings<'d> {
        let control = self.shared.get().commit;
        let format = &self.shared.catalog.formats[control.format_index as usize - 1];
        StreamSettings {
            format,
            frame: &format.frames[control.frame_index as usize - 1],
            frame_interval: control.frame_interval,
        }
    }

    /// Waits until the host starts the stream by enabling the streaming interface, then returns
    /// what it selected.
    ///
    /// Frames written before that fail with [`EndpointError::Disabled`].
    pub async fn wait_streaming(&mut self) -> StreamSettings<'d> {
        loop {
            if self.shared.get().streaming {
                return self.settings();
            }
            self.shared.changed.wait().await;
        }
    }

    /// Sends one frame: the image in the selected format, split into packets with a payload
    /// header each. The last packet is marked as the end of the frame.
    ///
    /// Each packet waits for the host to poll the endpoint, so the call takes about a frame
    /// interval for a frame that fills the bandwidth. If the host stops the stream meanwhile this
    /// fails with [`EndpointError::Disabled`] and the frame is cut short, which the host drops.
    pub async fn write_frame(&mut self, frame: &[u8]) -> Result<(), EndpointError> {
        self.frame_id = !self.frame_id;
        let payload_len = self.max_packet_size - PAYLOAD_HEADER_LEN;
        let mut packet = [0u8; MAX_PACKET_SIZE_HIGH_SPEED as usize];

        // An empty frame still sends one packet, to end the frame.
        let mut offset = 0;
        loop {
            let len = (frame.len() - offset).min(payload_len);
            let last = offset + len == frame.len();
            let mut header = if self.frame_id { HEADER_FID } else { 0 };
            if last {
                header |= HEADER_EOF;
            }
            packet[0] = PAYLOAD_HEADER_LEN as u8;
            packet[1] = header;
            packet[PAYLOAD_HEADER_LEN..PAYLOAD_HEADER_LEN + len].copy_from_slice(&frame[offset..offset + len]);
            self.endpoint.write(&packet[..PAYLOAD_HEADER_LEN + len]).await?;
            offset += len;
            if last {
                return Ok(());
            }
        }
    }
}

fn vc_header(video_streaming: InterfaceNumber) -> [u8; 11] {
    let mut d = [0; 11];
    d[0] = VC_HEADER;
    d[1..3].copy_from_slice(&0x0110u16.to_le_bytes()); // bcdUVC 1.10
    // d[3..5]: wTotalLength, patched.
    d[5..9].copy_from_slice(&CLOCK_FREQUENCY_HZ.to_le_bytes());
    d[9] = 1; // bInCollection
    d[10] = video_streaming.0;
    d
}

fn camera_input_terminal() -> [u8; 16] {
    let mut d = [0; 16];
    d[0] = VC_INPUT_TERMINAL;
    d[1] = INPUT_TERMINAL_ID;
    d[2..4].copy_from_slice(&ITT_CAMERA.to_le_bytes());
    // bAssocTerminal, iTerminal and the focal lengths: none.
    d[12] = 3; // bControlSize; bmControls: none.
    d
}

fn streaming_output_terminal() -> [u8; 7] {
    let mut d = [0; 7];
    d[0] = VC_OUTPUT_TERMINAL;
    d[1] = OUTPUT_TERMINAL_ID;
    d[2..4].copy_from_slice(&TT_STREAMING.to_le_bytes());
    d[5] = INPUT_TERMINAL_ID; // bSourceID
    d
}

fn vs_input_header(num_formats: u8, endpoint_address: u8) -> Vec<u8, { 10 + MAX_FORMATS }> {
    let mut d = Vec::new();
    d.push(VS_INPUT_HEADER).unwrap();
    d.push(num_formats).unwrap();
    d.extend_from_slice(&[0, 0]).unwrap(); // wTotalLength, patched.
    d.push(endpoint_address).unwrap();
    d.push(0).unwrap(); // bmInfo: no dynamic format change.
    d.push(OUTPUT_TERMINAL_ID).unwrap(); // bTerminalLink
    d.push(0).unwrap(); // bStillCaptureMethod
    d.push(0).unwrap(); // bTriggerSupport
    d.push(0).unwrap(); // bTriggerUsage
    d.push(1).unwrap(); // bControlSize
    // bmaControls: no format-specific controls.
    d.resize(d.len() + num_formats as usize, 0).unwrap();
    d
}

fn format_descriptor(format: &Format, index: u8) -> Vec<u8, 25> {
    let mut d = Vec::new();
    d.push(match format.pixel_format {
        PixelFormat::Mjpeg => VS_FORMAT_MJPEG,
        _ => VS_FORMAT_UNCOMPRESSED,
    })
    .unwrap();
    d.push(index).unwrap();
    d.push(format.frames.len() as u8).unwrap();
    match format.pixel_format.guid() {
        // Uncompressed: the GUID and bits per pixel precede the common tail.
        Some(guid) => {
            d.extend_from_slice(&guid).unwrap();
            d.push(format.pixel_format.bits_per_pixel() as u8).unwrap();
        }
        // MJPEG: bmFlags, where bit 0 says every sample has the same size.
        None => d.push(0).unwrap(),
    }
    d.push(1).unwrap(); // bDefaultFrameIndex
    d.push(0).unwrap(); // bAspectRatioX
    d.push(0).unwrap(); // bAspectRatioY
    d.push(0).unwrap(); // bmInterlaceFlags
    d.push(0).unwrap(); // bCopyProtect
    d
}

fn frame_descriptor(format: &Format, frame: &Frame, index: u8) -> Vec<u8, 255> {
    let max_frame_size = frame.frame_size(format.pixel_format);
    // The bit rate is the frame size times the frame rate. The longest interval is the slowest.
    let bit_rate =
        |interval: u32| (max_frame_size as u64 * 8 * 10_000_000 / interval as u64).min(u32::MAX as u64) as u32;
    let min_interval = frame.intervals.iter().min().copied().unwrap();
    let max_interval = frame.intervals.iter().max().copied().unwrap();

    let mut d = Vec::new();
    d.push(match format.pixel_format {
        PixelFormat::Mjpeg => VS_FRAME_MJPEG,
        _ => VS_FRAME_UNCOMPRESSED,
    })
    .unwrap();
    d.push(index).unwrap();
    d.push(0).unwrap(); // bmCapabilities: no still image, fixed frame rate
    d.extend_from_slice(&frame.width.to_le_bytes()).unwrap();
    d.extend_from_slice(&frame.height.to_le_bytes()).unwrap();
    d.extend_from_slice(&bit_rate(max_interval).to_le_bytes()).unwrap();
    d.extend_from_slice(&bit_rate(min_interval).to_le_bytes()).unwrap();
    d.extend_from_slice(&max_frame_size.to_le_bytes()).unwrap();
    d.extend_from_slice(&frame.intervals[0].to_le_bytes()).unwrap(); // dwDefaultFrameInterval
    d.push(frame.intervals.len() as u8).unwrap(); // bFrameIntervalType: discrete
    for interval in frame.intervals {
        d.extend_from_slice(&interval.to_le_bytes()).unwrap();
    }
    d
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;

    const INTERVALS: [u32; 2] = [333_333, 666_666];
    const FRAMES: [Frame; 2] = [Frame::new(160, 120, &INTERVALS), Frame::new(320, 240, &INTERVALS)];
    const FORMATS: [Format; 2] = [
        Format::new(PixelFormat::Yuy2, &FRAMES),
        Format::new(PixelFormat::Mjpeg, &FRAMES),
    ];
    const CATALOG: Catalog = Catalog {
        formats: &FORMATS,
        max_packet_size: 1023,
    };

    #[test]
    fn unit_descriptors_match_spec_lengths() {
        // bLength adds the length and type bytes, which the builder writes.
        assert_eq!(2 + vc_header(InterfaceNumber(1)).len(), 13);
        assert_eq!(2 + camera_input_terminal().len(), 18);
        assert_eq!(2 + streaming_output_terminal().len(), 9);
        assert_eq!(2 + vs_input_header(2, 0x81).len(), 13 + 2);
    }

    #[test]
    fn format_descriptors_match_spec_lengths() {
        assert_eq!(2 + format_descriptor(&FORMATS[0], 1).len(), 27);
        assert_eq!(2 + format_descriptor(&FORMATS[1], 2).len(), 11);
        // 26 plus four per interval.
        assert_eq!(2 + frame_descriptor(&FORMATS[0], &FRAMES[0], 1).len(), 26 + 4 * 2);
    }

    #[test]
    fn uncompressed_format_carries_guid_and_bpp() {
        let d = format_descriptor(&FORMATS[0], 1);
        assert_eq!(d[0], VS_FORMAT_UNCOMPRESSED);
        assert_eq!(&d[3..7], b"YUY2");
        assert_eq!(&d[7..19], &[0, 0, 0x10, 0, 0x80, 0, 0, 0xAA, 0, 0x38, 0x9B, 0x71]);
        assert_eq!(d[19], 16);
    }

    #[test]
    fn frame_descriptor_fields() {
        let d = frame_descriptor(&FORMATS[0], &FRAMES[0], 1);
        assert_eq!(d[0], VS_FRAME_UNCOMPRESSED);
        assert_eq!(u16::from_le_bytes([d[3], d[4]]), 160);
        assert_eq!(u16::from_le_bytes([d[5], d[6]]), 120);
        let max_size = u32::from_le_bytes([d[15], d[16], d[17], d[18]]);
        assert_eq!(max_size, 160 * 120 * 2);
        assert_eq!(d[23], 2); // bFrameIntervalType
        assert_eq!(u32::from_le_bytes([d[24], d[25], d[26], d[27]]), 333_333);
        // Max bit rate is at the shortest interval.
        let max_bit_rate = u32::from_le_bytes([d[11], d[12], d[13], d[14]]);
        assert_eq!(max_bit_rate as u64, 160 * 120 * 2 * 8 * 10_000_000 / 333_333);
    }

    #[test]
    fn stream_control_round_trips() {
        let control = CATALOG.control_for(2, 1, 666_666);
        let bytes = control.to_bytes();
        assert_eq!(bytes.len(), STREAM_CONTROL_LEN);
        assert_eq!(StreamControl::parse(&bytes), Some(control));
        // A UVC 1.0 host sends the first 26 bytes.
        let v10 = StreamControl::parse(&bytes[..STREAM_CONTROL_LEN_V10]).unwrap();
        assert_eq!(v10.max_payload_transfer_size, 1023);
        assert_eq!(v10.clock_frequency, 0);
        assert_eq!(StreamControl::parse(&bytes[..10]), None);
    }

    #[test]
    fn negotiate_adjusts_to_supported() {
        let proposal = StreamControl {
            format_index: 9,
            frame_index: 0,
            frame_interval: 1,
            ..StreamControl::default()
        };
        let n = CATALOG.negotiate(&proposal);
        assert_eq!((n.format_index, n.frame_index, n.frame_interval), (1, 1, 333_333));
        assert_eq!(n.max_video_frame_size, 160 * 120 * 2);
        assert_eq!(n.max_payload_transfer_size, 1023);

        let proposal = StreamControl {
            format_index: 2,
            frame_index: 2,
            frame_interval: 600_000,
            ..StreamControl::default()
        };
        let n = CATALOG.negotiate(&proposal);
        assert_eq!((n.format_index, n.frame_index, n.frame_interval), (2, 2, 666_666));
        assert_eq!(n.max_video_frame_size, 320 * 240 * 2);
    }

    mod bus {
        extern crate std;

        use core::cell::RefCell;
        use std::rc::Rc;
        use std::vec::Vec;

        use crate::driver::*;

        pub type Log = Rc<RefCell<Vec<Vec<u8>>>>;

        pub struct MockDriver(pub Log);

        pub struct MockEndpoint(EndpointInfo, Log);

        pub struct Unused;

        impl<'a> Driver<'a> for MockDriver {
            type EndpointOut = Unused;
            type EndpointIn = MockEndpoint;
            type ControlPipe = Unused;
            type Bus = Unused;

            fn alloc_endpoint_out(
                &mut self,
                _: EndpointType,
                _: Option<EndpointAddress>,
                _: u16,
                _: u8,
            ) -> Result<Unused, EndpointAllocError> {
                unreachable!()
            }

            fn alloc_endpoint_in(
                &mut self,
                ep_type: EndpointType,
                _: Option<EndpointAddress>,
                max_packet_size: u16,
                interval_ms: u8,
            ) -> Result<MockEndpoint, EndpointAllocError> {
                let info = EndpointInfo {
                    addr: EndpointAddress::from_parts(3, Direction::In),
                    ep_type,
                    max_packet_size,
                    interval_ms,
                };
                Ok(MockEndpoint(info, self.0.clone()))
            }

            fn start(self, _: u16) -> (Unused, Unused) {
                (Unused, Unused)
            }
        }

        impl Endpoint for MockEndpoint {
            fn info(&self) -> &EndpointInfo {
                &self.0
            }
            async fn wait_enabled(&mut self) {}
        }

        impl EndpointIn for MockEndpoint {
            async fn write(&mut self, buf: &[u8]) -> Result<(), EndpointError> {
                self.1.borrow_mut().push(buf.to_vec());
                Ok(())
            }
        }

        impl Endpoint for Unused {
            fn info(&self) -> &EndpointInfo {
                unreachable!()
            }
            async fn wait_enabled(&mut self) {
                unreachable!()
            }
        }

        impl EndpointOut for Unused {
            async fn read(&mut self, _: &mut [u8]) -> Result<usize, EndpointError> {
                unreachable!()
            }
        }

        impl Bus for Unused {
            async fn enable(&mut self) {
                unreachable!()
            }
            async fn disable(&mut self) {
                unreachable!()
            }
            async fn poll(&mut self) -> Event {
                unreachable!()
            }
            fn endpoint_set_enabled(&mut self, _: EndpointAddress, _: bool) {
                unreachable!()
            }
            fn endpoint_set_stalled(&mut self, _: EndpointAddress, _: bool) {
                unreachable!()
            }
            fn endpoint_is_stalled(&mut self, _: EndpointAddress) -> bool {
                unreachable!()
            }
            async fn remote_wakeup(&mut self) -> Result<(), Unsupported> {
                unreachable!()
            }
        }

        impl ControlPipe for Unused {
            fn max_packet_size(&self) -> usize {
                unreachable!()
            }
            async fn setup(&mut self) -> [u8; 8] {
                unreachable!()
            }
            async fn data_out(&mut self, _: &mut [u8], _: bool, _: bool) -> Result<usize, EndpointError> {
                unreachable!()
            }
            async fn data_in(&mut self, _: &[u8], _: bool, _: bool) -> Result<(), EndpointError> {
                unreachable!()
            }
            async fn accept(&mut self) {
                unreachable!()
            }
            async fn reject(&mut self) {
                unreachable!()
            }
            async fn accept_set_address(&mut self, _: u8) {
                unreachable!()
            }
        }
    }

    /// Splits a configuration descriptor into its descriptors.
    fn descriptors(buf: &[u8]) -> std::vec::Vec<&[u8]> {
        let mut out = std::vec::Vec::new();
        let mut rest = buf;
        // The buffer is zero-padded after the last descriptor.
        while !rest.is_empty() && rest[0] != 0 {
            let (d, tail) = rest.split_at(rest[0] as usize);
            out.push(d);
            rest = tail;
        }
        out
    }

    #[test]
    fn config_descriptor_assembly() {
        let log = bus::Log::default();
        let mut config_buf = [0u8; 512];
        let mut control_buf = [0u8; 64];
        let mut state = State::new();
        {
            let mut builder = Builder::new(
                bus::MockDriver(log),
                crate::Config::new(0x1234, 0x5678),
                &mut config_buf,
                &mut [],
                &mut [],
                &mut control_buf,
            );
            let _camera = Camera::new(&mut builder, &mut state, Config::new(&FORMATS, UsbDeviceSpeed::Full));
            let _ = builder.build();
        }

        let all = descriptors(&config_buf);
        let all: std::vec::Vec<&[u8]> = all.into_iter().take_while(|d| d[0] != 0).collect();
        // Everything is accounted for in the configuration's total length.
        let total: usize = all.iter().map(|d| d.len()).sum();
        assert_eq!(u16::from_le_bytes([all[0][2], all[0][3]]) as usize, total);

        // Configuration, IAD for the two interfaces, then the video control interface.
        assert_eq!(all[1][1], 0x0B);
        assert_eq!((all[1][2], all[1][3], all[1][4], all[1][5]), (0, 2, 0x0E, 0x03));
        assert_eq!((all[2][1], all[2][5], all[2][6]), (0x04, 0x0E, 0x01));
        let vc: std::vec::Vec<_> = all[3..].iter().take_while(|d| d[1] == CS_INTERFACE).collect();
        assert_eq!(vc.len(), 3);
        let vc_len: usize = vc.iter().map(|d| d.len()).sum();
        assert_eq!(u16::from_le_bytes([vc[0][5], vc[0][6]]) as usize, vc_len);
        assert_eq!(vc_len, 13 + 18 + 9);
        assert_eq!(vc[0][12], 1); // the video streaming interface

        // Video streaming alternate setting 0: no endpoint, the input header, formats and frames.
        let vs0 = &all[3 + vc.len()..];
        assert_eq!(
            (vs0[0][1], vs0[0][2], vs0[0][4], vs0[0][5], vs0[0][6]),
            (0x04, 1, 0, 0x0E, 0x02)
        );
        let cs: std::vec::Vec<_> = vs0[1..].iter().take_while(|d| d[1] == CS_INTERFACE).collect();
        // Header, then per format its descriptor and two frames.
        assert_eq!(cs.len(), 1 + 2 * 3);
        let cs_len: usize = cs.iter().map(|d| d.len()).sum();
        assert_eq!(u16::from_le_bytes([cs[0][4], cs[0][5]]) as usize, cs_len);
        let endpoint_address = cs[0][6];

        // Alternate setting 1 carries the isochronous endpoint the header names.
        let vs1 = &vs0[1 + cs.len()..];
        assert_eq!((vs1[0][1], vs1[0][2], vs1[0][3], vs1[0][4]), (0x04, 1, 1, 1));
        assert_eq!(vs1[1][1], 0x05);
        assert_eq!(vs1[1][2], endpoint_address);
        assert_eq!(vs1[1][3] & 0b11, 0b01); // isochronous
        assert_eq!(u16::from_le_bytes([vs1[1][4], vs1[1][5]]), 1023);
        assert_eq!(vs1.len(), 2);
    }

    #[test]
    fn write_frame_splits_into_payloads() {
        let log = bus::Log::default();
        let mut config_buf = [0u8; 512];
        let mut control_buf = [0u8; 64];
        let mut state = State::new();
        let mut builder = Builder::new(
            bus::MockDriver(log.clone()),
            crate::Config::new(0x1234, 0x5678),
            &mut config_buf,
            &mut [],
            &mut [],
            &mut control_buf,
        );
        let mut config = Config::new(&FORMATS, UsbDeviceSpeed::Full);
        config.max_packet_size = 8; // 6 payload bytes per packet
        let mut camera = Camera::new(&mut builder, &mut state, config);

        let frame = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13];
        embassy_futures::block_on(camera.write_frame(&frame)).unwrap();
        embassy_futures::block_on(camera.write_frame(&frame[..6])).unwrap();
        embassy_futures::block_on(camera.write_frame(&[])).unwrap();

        let log = log.borrow();
        let packets: std::vec::Vec<&[u8]> = log.iter().map(|p| p.as_slice()).collect();
        assert_eq!(
            packets,
            [
                // First frame: FID 1, EOF on the last of three packets.
                &[2, 1, 1, 2, 3, 4, 5, 6][..],
                &[2, 1, 7, 8, 9, 10, 11, 12],
                &[2, 3, 13],
                // A frame that exactly fills a packet is one packet, and the FID toggled.
                &[2, 2, 1, 2, 3, 4, 5, 6],
                // An empty frame is a lone header.
                &[2, 3],
            ]
        );
    }

    #[test]
    fn host_negotiation_through_control_requests() {
        let shared = Shared {
            catalog: CATALOG,
            inner: CriticalSectionMutex::new(Cell::new(Inner {
                probe: CATALOG.default_control(),
                commit: CATALOG.default_control(),
                streaming: false,
            })),
            changed: Signal::new(),
        };
        let mut control = Control {
            shared: &shared,
            video_control: InterfaceNumber(0),
            video_streaming: InterfaceNumber(1),
        };
        let request = |direction, request, selector: u8, length| Request {
            direction,
            request_type: RequestType::Class,
            recipient: Recipient::Interface,
            request,
            value: (selector as u16) << 8,
            index: 1,
            length,
        };
        use crate::driver::Direction;

        // Other interfaces are not ours.
        let mut other = request(Direction::In, GET_CUR, VS_PROBE_CONTROL, 34);
        other.index = 0;
        assert!(control.control_in(other, &mut [0; 64]).is_none());

        // Probe: the host proposes the second format, the camera answers with its limits.
        let proposal = StreamControl {
            format_index: 2,
            frame_index: 2,
            frame_interval: 666_666,
            ..StreamControl::default()
        };
        let out = request(Direction::Out, SET_CUR, VS_PROBE_CONTROL, 34);
        assert!(matches!(
            control.control_out(out, &proposal.to_bytes()),
            Some(OutResponse::Accepted)
        ));
        let mut buf = [0; 64];
        let Some(InResponse::Accepted(data)) =
            control.control_in(request(Direction::In, GET_CUR, VS_PROBE_CONTROL, 34), &mut buf)
        else {
            panic!("probe GET_CUR rejected");
        };
        let probed = StreamControl::parse(data).unwrap();
        assert_eq!((probed.format_index, probed.frame_index), (2, 2));
        assert_eq!(probed.max_payload_transfer_size, 1023);

        // Commit, then start streaming by selecting alternate setting 1.
        let out = request(Direction::Out, SET_CUR, VS_COMMIT_CONTROL, 34);
        control.control_out(out, &probed.to_bytes());
        control.set_alternate_setting(InterfaceNumber(1), 1);
        let inner = shared.get();
        assert!(inner.streaming);
        assert_eq!(inner.commit.format_index, 2);

        // A bus reset puts everything back.
        control.reset();
        let inner = shared.get();
        assert!(!inner.streaming);
        assert_eq!(inner.commit, CATALOG.default_control());

        // GET_LEN, and requests the camera doesn't implement.
        let Some(InResponse::Accepted(len)) =
            control.control_in(request(Direction::In, GET_LEN, VS_PROBE_CONTROL, 2), &mut buf)
        else {
            panic!("GET_LEN rejected");
        };
        assert_eq!(len, &[34, 0]);
        assert!(matches!(
            control.control_in(request(Direction::In, GET_MIN, VS_COMMIT_CONTROL, 34), &mut buf),
            Some(InResponse::Rejected)
        ));
    }

    #[test]
    fn probe_range() {
        assert_eq!(CATALOG.default_control().format_index, 1);
        let max = CATALOG.max_control();
        assert_eq!((max.format_index, max.frame_index, max.frame_interval), (2, 2, 333_333));
    }
}
