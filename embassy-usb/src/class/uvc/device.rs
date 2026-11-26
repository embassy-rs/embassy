//! USB Video Class 1.5 - camera device (device to host).
//!
//! Streams video over an isochronous or bulk IN endpoint. The camera offers any number of
//! [`Format`]s (MJPEG, YUY2, NV12, I420, H.264, or custom frame-based formats), each with one
//! or more [`Frame`] sizes and frame intervals. The host picks one through the 48-byte UVC 1.5
//! probe/commit handshake; [`Camera::settings`] reports the result and [`Camera::write_frame`]
//! (or [`Camera::start_frame`] for chunked streaming) sends frames with UVC payload headers.
//!
//! Optional UVC 1.5 features include:
//! - Processing Unit controls ([`ProcessingControls`]: brightness, contrast, hue, saturation,
//!   sharpness, gamma, gain, white balance, backlight, power line frequency)
//! - Camera Terminal controls ([`CameraControls`]: auto-exposure mode, exposure time, focus,
//!   auto-focus, zoom)
//! - Method 2 still image capture ([`StillConfig`], [`Camera::take_still_trigger`],
//!   [`Camera::write_still_frame`])
//! - Bulk streaming transport ([`Config::bulk`])

use core::cell::Cell;

use embassy_sync::blocking_mutex::CriticalSectionMutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_usb_driver::EndpointInterval;
use heapless::Vec;

use crate::control::{self, InResponse, OutResponse, Recipient, Request, RequestType};
use crate::descriptor::{SynchronizationType, UsageType};
use crate::driver::{
    Driver, Endpoint, EndpointError, EndpointIn, EndpointInfo, IsoEndpointIn, IsoInSlot, IsoWriteError,
};
use crate::types::InterfaceNumber;
use crate::{Builder, EndpointOptions, Handler, InterfaceHandler, UsbDeviceSpeed};

const USB_CLASS_VIDEO: u8 = 0x0E;
const SC_VIDEOCONTROL: u8 = 0x01;
const SC_VIDEOSTREAMING: u8 = 0x02;
const SC_VIDEO_INTERFACE_COLLECTION: u8 = 0x03;
const PC_PROTOCOL_15: u8 = 0x01;
const BCD_UVC_15: u16 = 0x0150;

const CS_INTERFACE: u8 = 0x24;

// Video control interface descriptor subtypes.
const VC_HEADER: u8 = 0x01;
const VC_INPUT_TERMINAL: u8 = 0x02;
const VC_OUTPUT_TERMINAL: u8 = 0x03;
const VC_PROCESSING_UNIT: u8 = 0x05;

// Video streaming interface descriptor subtypes.
const VS_INPUT_HEADER: u8 = 0x01;
const VS_STILL_IMAGE_FRAME: u8 = 0x03;
const VS_FORMAT_UNCOMPRESSED: u8 = 0x04;
const VS_FRAME_UNCOMPRESSED: u8 = 0x05;
const VS_FORMAT_MJPEG: u8 = 0x06;
const VS_FRAME_MJPEG: u8 = 0x07;
const VS_FORMAT_FRAME_BASED: u8 = 0x10;
const VS_FRAME_FRAME_BASED: u8 = 0x11;

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

// VideoControl interface control selectors.
const VC_REQUEST_ERROR_CODE_CONTROL: u8 = 0x02;

// Camera Terminal control selectors.
const CT_AE_MODE_CONTROL: u8 = 0x02;
const CT_EXPOSURE_TIME_ABSOLUTE_CONTROL: u8 = 0x04;
const CT_FOCUS_ABSOLUTE_CONTROL: u8 = 0x06;
const CT_FOCUS_AUTO_CONTROL: u8 = 0x08;
const CT_ZOOM_ABSOLUTE_CONTROL: u8 = 0x0B;

// Processing Unit control selectors.
const PU_BACKLIGHT_COMPENSATION_CONTROL: u8 = 0x01;
const PU_BRIGHTNESS_CONTROL: u8 = 0x02;
const PU_CONTRAST_CONTROL: u8 = 0x03;
const PU_GAIN_CONTROL: u8 = 0x04;
const PU_POWER_LINE_FREQUENCY_CONTROL: u8 = 0x05;
const PU_HUE_CONTROL: u8 = 0x06;
const PU_SATURATION_CONTROL: u8 = 0x07;
const PU_SHARPNESS_CONTROL: u8 = 0x08;
const PU_GAMMA_CONTROL: u8 = 0x09;
const PU_WHITE_BALANCE_TEMPERATURE_CONTROL: u8 = 0x0A;
const PU_WHITE_BALANCE_TEMPERATURE_AUTO_CONTROL: u8 = 0x0B;
const PU_HUE_AUTO_CONTROL: u8 = 0x10;

// Video streaming interface control selectors.
const VS_PROBE_CONTROL: u8 = 0x01;
const VS_COMMIT_CONTROL: u8 = 0x02;
const VS_STILL_PROBE_CONTROL: u8 = 0x03;
const VS_STILL_COMMIT_CONTROL: u8 = 0x04;
const VS_STILL_IMAGE_TRIGGER_CONTROL: u8 = 0x05;

// VC Request Error Codes [UVC 1.5 4.2.1.2].
const ERR_NO_ERROR: u8 = 0x00;
const ERR_OUT_OF_RANGE: u8 = 0x04;
const ERR_INVALID_UNIT: u8 = 0x05;
const ERR_INVALID_CONTROL: u8 = 0x06;
const ERR_INVALID_REQUEST: u8 = 0x07;

const INPUT_TERMINAL_ID: u8 = 1;
const PROCESSING_UNIT_ID: u8 = 2;

/// `dwClockFrequency` reported in stream control and VC header.
const CLOCK_FREQUENCY_HZ: u32 = 48_000_000;

/// Minimum stream control size accepted from older UVC 1.0 hosts.
const STREAM_CONTROL_LEN_V10: usize = 26;
/// Size of the UVC 1.1 probe/commit control.
const STREAM_CONTROL_LEN_V11: usize = 34;
/// Size of the UVC 1.5 probe/commit control.
const STREAM_CONTROL_LEN: usize = 48;
/// Size of the UVC still image probe/commit control.
const STILL_CONTROL_LEN: usize = 11;

/// Payload header lengths and bit flags [UVC 1.5 2.4.3.3].
const PAYLOAD_HEADER_LEN_MIN: usize = 2;
const PAYLOAD_HEADER_LEN_PTS: usize = 6;
const PAYLOAD_HEADER_LEN_PTS_SCR: usize = 12;

const HEADER_FID: u8 = 1 << 0;
const HEADER_EOF: u8 = 1 << 1;
const HEADER_PTS: u8 = 1 << 2;
const HEADER_SCR: u8 = 1 << 3;
const HEADER_STI: u8 = 1 << 5;
const HEADER_ERR: u8 = 1 << 6;
const HEADER_EOH: u8 = 1 << 7;

/// The input header lists one control byte per format, within the 255 byte descriptor limit.
const MAX_FORMATS: usize = 16;

/// Frame descriptors are at most 255 bytes: 30 (frame-based) plus four per frame interval.
const MAX_FRAME_INTERVALS: usize = (255 - 30) / 4;

/// Largest packet an isochronous endpoint takes without extra transactions per microframe.
const MAX_PACKET_SIZE_FULL_SPEED: u16 = 1023;
const MAX_PACKET_SIZE_HIGH_SPEED: u16 = 1024;
const MAX_BULK_PACKET_SIZE_FULL_SPEED: u16 = 64;
const MAX_BULK_PACKET_SIZE_HIGH_SPEED: u16 = 512;

/// Transport endpoint type used for the VideoStreaming interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum StreamTransport {
    /// Isochronous IN endpoint on alternate setting 1 (zero-bandwidth alternate setting 0).
    #[default]
    Isochronous,
    /// Bulk IN endpoint on alternate setting 0 (streaming active once committed).
    Bulk,
}

/// Builds a standard UVC 16-byte GUID from a 4-character FourCC code.
pub const fn fourcc_guid(fourcc: [u8; 4]) -> [u8; 16] {
    [
        fourcc[0], fourcc[1], fourcc[2], fourcc[3], 0x00, 0x00, 0x10, 0x00, 0x80, 0x00, 0x00, 0xAA, 0x00, 0x38, 0x9B,
        0x71,
    ]
}

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
    /// Uncompressed planar YUV 4:2:0, 12 bits per pixel: Y, then U, then V plane.
    I420,
    /// H.264 / AVC elementary stream using `VS_FORMAT_FRAME_BASED` (`H264` GUID).
    H264,
    /// Arbitrary frame-based payload format using `VS_FORMAT_FRAME_BASED`.
    FrameBased {
        /// Format GUID (see [`fourcc_guid`]).
        guid: [u8; 16],
        /// Nominal bits per pixel (e.g. 16).
        bits_per_pixel: u8,
        /// `true` for compressed / variable-size frames (`dwBytesPerLine = 0`).
        variable_size: bool,
    },
}

impl PixelFormat {
    const fn bits_per_pixel(self) -> u8 {
        match self {
            PixelFormat::Mjpeg | PixelFormat::Yuy2 | PixelFormat::H264 => 16,
            PixelFormat::Nv12 | PixelFormat::I420 => 12,
            PixelFormat::FrameBased { bits_per_pixel, .. } => bits_per_pixel,
        }
    }

    const fn is_frame_based(self) -> bool {
        matches!(self, PixelFormat::H264 | PixelFormat::FrameBased { .. })
    }

    const fn variable_size(self) -> bool {
        match self {
            PixelFormat::H264 => true,
            PixelFormat::FrameBased { variable_size, .. } => variable_size,
            _ => false,
        }
    }

    const fn guid(self) -> Option<[u8; 16]> {
        match self {
            PixelFormat::Mjpeg => None,
            PixelFormat::Yuy2 => Some(fourcc_guid(*b"YUY2")),
            PixelFormat::Nv12 => Some(fourcc_guid(*b"NV12")),
            PixelFormat::I420 => Some(fourcc_guid(*b"I420")),
            PixelFormat::H264 => Some(fourcc_guid(*b"H264")),
            PixelFormat::FrameBased { guid, .. } => Some(guid),
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
    /// Supported frame intervals in 100 ns units, e.g. `333_333` for 30 fps. At least one, at
    /// most 56. The first is the default.
    pub intervals: &'d [u32],
    /// Upper bound of a frame's size in bytes. `None` defaults to the uncompressed frame size.
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

    /// Overrides the maximum frame size advertised in descriptors and probe/commit.
    pub const fn with_max_frame_size(mut self, max_frame_size: u32) -> Self {
        self.max_frame_size = Some(max_frame_size);
        self
    }

    /// Frame interval in 100 ns units for a frame rate.
    pub const fn interval_for_fps(fps: u32) -> u32 {
        10_000_000 / fps
    }

    fn frame_size(&self, pixel_format: PixelFormat) -> u32 {
        let pixels = self.width as u32 * self.height as u32;
        match self.max_frame_size {
            Some(size) => size,
            None => pixels * pixel_format.bits_per_pixel() as u32 / 8,
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

/// Range and default value for a numeric UVC Processing Unit or Camera Terminal control.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ControlRange<T> {
    /// Minimum allowed value (`GET_MIN`).
    pub min: T,
    /// Maximum allowed value (`GET_MAX`).
    pub max: T,
    /// Step resolution (`GET_RES`).
    pub res: T,
    /// Default value (`GET_DEF` and initial `GET_CUR`).
    pub def: T,
}

impl<T> ControlRange<T> {
    /// Creates a control range with explicit min, max, resolution, and default.
    pub const fn new(min: T, max: T, res: T, def: T) -> Self {
        Self { min, max, res, def }
    }
}

/// Supported Processing Unit (`VC_PROCESSING_UNIT`) controls.
#[derive(Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ProcessingControls {
    /// Backlight compensation (`PU_BACKLIGHT_COMPENSATION_CONTROL`).
    pub backlight_compensation: Option<ControlRange<u16>>,
    /// Brightness (`PU_BRIGHTNESS_CONTROL`).
    pub brightness: Option<ControlRange<i16>>,
    /// Contrast (`PU_CONTRAST_CONTROL`).
    pub contrast: Option<ControlRange<u16>>,
    /// Gain (`PU_GAIN_CONTROL`).
    pub gain: Option<ControlRange<u16>>,
    /// Power line frequency anti-flicker default (`0` = disabled, `1` = 50 Hz, `2` = 60 Hz, `3` = Auto).
    pub power_line_frequency: Option<u8>,
    /// Hue (`PU_HUE_CONTROL`).
    pub hue: Option<ControlRange<i16>>,
    /// Saturation (`PU_SATURATION_CONTROL`).
    pub saturation: Option<ControlRange<u16>>,
    /// Sharpness (`PU_SHARPNESS_CONTROL`).
    pub sharpness: Option<ControlRange<u16>>,
    /// Gamma (`PU_GAMMA_CONTROL`).
    pub gamma: Option<ControlRange<u16>>,
    /// White balance color temperature in Kelvin (`PU_WHITE_BALANCE_TEMPERATURE_CONTROL`).
    pub white_balance_temp: Option<ControlRange<u16>>,
    /// Auto white balance temperature default (`PU_WHITE_BALANCE_TEMPERATURE_AUTO_CONTROL`).
    pub white_balance_auto: Option<bool>,
    /// Auto hue default (`PU_HUE_AUTO_CONTROL`).
    pub hue_auto: Option<bool>,
}

impl ProcessingControls {
    /// No Processing Unit controls (omits the `VC_PROCESSING_UNIT` descriptor).
    pub const fn none() -> Self {
        Self {
            backlight_compensation: None,
            brightness: None,
            contrast: None,
            gain: None,
            power_line_frequency: None,
            hue: None,
            saturation: None,
            sharpness: None,
            gamma: None,
            white_balance_temp: None,
            white_balance_auto: None,
            hue_auto: None,
        }
    }

    const fn is_any_enabled(&self) -> bool {
        self.backlight_compensation.is_some()
            || self.brightness.is_some()
            || self.contrast.is_some()
            || self.gain.is_some()
            || self.power_line_frequency.is_some()
            || self.hue.is_some()
            || self.saturation.is_some()
            || self.sharpness.is_some()
            || self.gamma.is_some()
            || self.white_balance_temp.is_some()
            || self.white_balance_auto.is_some()
            || self.hue_auto.is_some()
    }

    const fn bm_controls(&self) -> [u8; 3] {
        let mut mask: u32 = 0;
        if self.brightness.is_some() {
            mask |= 1 << 0;
        }
        if self.contrast.is_some() {
            mask |= 1 << 1;
        }
        if self.hue.is_some() {
            mask |= 1 << 2;
        }
        if self.saturation.is_some() {
            mask |= 1 << 3;
        }
        if self.sharpness.is_some() {
            mask |= 1 << 4;
        }
        if self.gamma.is_some() {
            mask |= 1 << 5;
        }
        if self.white_balance_temp.is_some() {
            mask |= 1 << 6;
        }
        if self.backlight_compensation.is_some() {
            mask |= 1 << 8;
        }
        if self.gain.is_some() {
            mask |= 1 << 9;
        }
        if self.power_line_frequency.is_some() {
            mask |= 1 << 10;
        }
        if self.hue_auto.is_some() {
            mask |= 1 << 11;
        }
        if self.white_balance_auto.is_some() {
            mask |= 1 << 12;
        }
        let le = mask.to_le_bytes();
        [le[0], le[1], le[2]]
    }
}

/// Supported Camera Terminal (`ITT_CAMERA`) controls.
#[derive(Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CameraControls {
    /// Auto-exposure mode default bitmap (`1`=Manual, `2`=Auto, `4`=Shutter Priority, `8`=Aperture Priority).
    pub ae_mode: Option<u8>,
    /// Absolute exposure time in 100 µs units (`CT_EXPOSURE_TIME_ABSOLUTE_CONTROL`).
    pub exposure_time_abs: Option<ControlRange<u32>>,
    /// Absolute focus (`CT_FOCUS_ABSOLUTE_CONTROL`).
    pub focus_abs: Option<ControlRange<u16>>,
    /// Auto-focus default (`CT_FOCUS_AUTO_CONTROL`).
    pub focus_auto: Option<bool>,
    /// Absolute zoom (`CT_ZOOM_ABSOLUTE_CONTROL`).
    pub zoom_abs: Option<ControlRange<u16>>,
}

impl CameraControls {
    /// No Camera Terminal controls.
    pub const fn none() -> Self {
        Self {
            ae_mode: None,
            exposure_time_abs: None,
            focus_abs: None,
            focus_auto: None,
            zoom_abs: None,
        }
    }

    const fn bm_controls(&self) -> [u8; 3] {
        let mut mask: u32 = 0;
        if self.ae_mode.is_some() {
            mask |= 1 << 1;
        }
        if self.exposure_time_abs.is_some() {
            mask |= 1 << 3;
        }
        if self.focus_abs.is_some() {
            mask |= 1 << 5;
        }
        if self.zoom_abs.is_some() {
            mask |= 1 << 9;
        }
        if self.focus_auto.is_some() {
            mask |= 1 << 16;
        }
        let le = mask.to_le_bytes();
        [le[0], le[1], le[2]]
    }
}

/// Current values of all VideoControl Processing Unit and Camera Terminal controls.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ControlValues {
    /// Backlight compensation.
    pub backlight_compensation: u16,
    /// Brightness.
    pub brightness: i16,
    /// Contrast.
    pub contrast: u16,
    /// Gain.
    pub gain: u16,
    /// Power line frequency (`0`..=`3`).
    pub power_line_frequency: u8,
    /// Hue.
    pub hue: i16,
    /// Saturation.
    pub saturation: u16,
    /// Sharpness.
    pub sharpness: u16,
    /// Gamma.
    pub gamma: u16,
    /// White balance temperature in Kelvin.
    pub white_balance_temp: u16,
    /// Auto white balance enabled.
    pub white_balance_auto: bool,
    /// Auto hue enabled.
    pub hue_auto: bool,
    /// Auto-exposure mode bitmap.
    pub ae_mode: u8,
    /// Absolute exposure time in 100 µs units.
    pub exposure_time_abs: u32,
    /// Absolute focus position.
    pub focus_abs: u16,
    /// Auto-focus enabled.
    pub focus_auto: bool,
    /// Absolute zoom position.
    pub zoom_abs: u16,
}

impl ControlValues {
    const fn from_config(pu: &ProcessingControls, ct: &CameraControls) -> Self {
        Self {
            backlight_compensation: match pu.backlight_compensation {
                Some(r) => r.def,
                None => 0,
            },
            brightness: match pu.brightness {
                Some(r) => r.def,
                None => 0,
            },
            contrast: match pu.contrast {
                Some(r) => r.def,
                None => 0,
            },
            gain: match pu.gain {
                Some(r) => r.def,
                None => 0,
            },
            power_line_frequency: match pu.power_line_frequency {
                Some(v) => v,
                None => 0,
            },
            hue: match pu.hue {
                Some(r) => r.def,
                None => 0,
            },
            saturation: match pu.saturation {
                Some(r) => r.def,
                None => 0,
            },
            sharpness: match pu.sharpness {
                Some(r) => r.def,
                None => 0,
            },
            gamma: match pu.gamma {
                Some(r) => r.def,
                None => 0,
            },
            white_balance_temp: match pu.white_balance_temp {
                Some(r) => r.def,
                None => 0,
            },
            white_balance_auto: match pu.white_balance_auto {
                Some(v) => v,
                None => false,
            },
            hue_auto: match pu.hue_auto {
                Some(v) => v,
                None => false,
            },
            ae_mode: match ct.ae_mode {
                Some(v) => v,
                None => 0,
            },
            exposure_time_abs: match ct.exposure_time_abs {
                Some(r) => r.def,
                None => 0,
            },
            focus_abs: match ct.focus_abs {
                Some(r) => r.def,
                None => 0,
            },
            focus_auto: match ct.focus_auto {
                Some(v) => v,
                None => false,
            },
            zoom_abs: match ct.zoom_abs {
                Some(r) => r.def,
                None => 0,
            },
        }
    }
}

/// Resolution offered for UVC Method 2 still image capture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct StillFrame {
    /// Still image width in pixels.
    pub width: u16,
    /// Still image height in pixels.
    pub height: u16,
}

impl StillFrame {
    /// Creates a still image size.
    pub const fn new(width: u16, height: u16) -> Self {
        Self { width, height }
    }
}

/// Method 2 still image capture configuration (`VS_STILL_IMAGE_FRAME`).
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct StillConfig<'d> {
    /// Supported still image sizes (`1..=255`).
    pub frames: &'d [StillFrame],
    /// Supported compression quality levels (`0..=255`).
    pub compression: &'d [u8],
    /// Optional explicit upper bound on still image byte size.
    pub max_frame_size: Option<u32>,
}

impl<'d> StillConfig<'d> {
    /// Creates a Method 2 still image capture configuration.
    pub const fn new(frames: &'d [StillFrame], compression: &'d [u8]) -> Self {
        Self {
            frames,
            compression,
            max_frame_size: None,
        }
    }

    /// Overrides the maximum still image size in bytes.
    pub const fn with_max_frame_size(mut self, max_frame_size: u32) -> Self {
        self.max_frame_size = Some(max_frame_size);
        self
    }

    fn frame_size(&self, frame_index: u8, pixel_format: PixelFormat) -> u32 {
        if let Some(size) = self.max_frame_size {
            return size;
        }
        let frame = &self.frames[frame_index as usize - 1];
        frame.width as u32 * frame.height as u32 * pixel_format.bits_per_pixel() as u32 / 8
    }
}

/// Committed still image capture parameters when triggered by the host.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct StillSettings<'d> {
    /// Selected video format.
    pub format: &'d Format<'d>,
    /// Selected still image dimensions.
    pub frame: StillFrame,
    /// Selected compression quality value, if compression levels were configured.
    pub compression: Option<u8>,
    /// Maximum still image size in bytes.
    pub max_frame_size: u32,
}

/// Optional UVC payload timestamps for [`Camera::write_frame_with_timestamp`] and
/// [`Camera::start_frame_with_timestamp`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PayloadTimestamp {
    /// Presentation Time Stamp (`PTS`) in native device clock units (`dwClockFrequency`).
    pub pts: u32,
    /// Optional Source Clock Reference: `(STC, SOF)` where `STC` is the 32-bit device clock
    /// counter and `SOF` is the 16-bit USB (micro)frame number.
    pub scr_stc: Option<(u32, u16)>,
}

/// Everything the camera advertises to the host.
#[derive(Clone)]
pub struct Config<'d> {
    /// The formats the camera streams, at least one and at most 16. The host numbers them from
    /// one, in this order. The first is the default.
    pub formats: &'d [Format<'d>],
    /// Packet size of the VideoStreaming endpoint in bytes, including the payload header.
    pub max_packet_size: u16,
    /// VideoStreaming transport (`Isochronous` by default).
    pub transport: StreamTransport,
    /// Optional Processing Unit (`VC_PROCESSING_UNIT`) controls.
    pub processing_controls: ProcessingControls,
    /// Optional Camera Terminal (`ITT_CAMERA`) controls.
    pub camera_controls: CameraControls,
    /// Optional Method 2 still image capture configuration.
    pub still: Option<StillConfig<'d>>,
}

impl<'d> Config<'d> {
    /// Creates an isochronous UVC 1.5 camera configuration with the largest packet size for `speed`.
    pub const fn new(formats: &'d [Format<'d>], speed: UsbDeviceSpeed) -> Self {
        Self::with_transport(formats, StreamTransport::Isochronous, speed)
    }

    /// Creates a bulk UVC 1.5 camera configuration with the largest bulk packet size for `speed`.
    pub const fn bulk(formats: &'d [Format<'d>], speed: UsbDeviceSpeed) -> Self {
        Self::with_transport(formats, StreamTransport::Bulk, speed)
    }

    /// Creates a UVC 1.5 camera configuration for the given `transport` and `speed`.
    pub const fn with_transport(formats: &'d [Format<'d>], transport: StreamTransport, speed: UsbDeviceSpeed) -> Self {
        Self {
            formats,
            max_packet_size: match (transport, speed) {
                (StreamTransport::Isochronous, UsbDeviceSpeed::Full) => MAX_PACKET_SIZE_FULL_SPEED,
                (StreamTransport::Isochronous, _) => MAX_PACKET_SIZE_HIGH_SPEED,
                (StreamTransport::Bulk, UsbDeviceSpeed::Full) => MAX_BULK_PACKET_SIZE_FULL_SPEED,
                (StreamTransport::Bulk, _) => MAX_BULK_PACKET_SIZE_HIGH_SPEED,
            },
            transport,
            processing_controls: ProcessingControls::none(),
            camera_controls: CameraControls::none(),
            still: None,
        }
    }

    /// Configures Processing Unit (`VC_PROCESSING_UNIT`) controls.
    pub const fn with_processing_controls(mut self, controls: ProcessingControls) -> Self {
        self.processing_controls = controls;
        self
    }

    /// Configures Camera Terminal (`ITT_CAMERA`) controls.
    pub const fn with_camera_controls(mut self, controls: CameraControls) -> Self {
        self.camera_controls = controls;
        self
    }

    /// Enables Method 2 still image capture.
    pub const fn with_still(mut self, still: StillConfig<'d>) -> Self {
        self.still = Some(still);
        self
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

/// The UVC 1.5 probe/commit control (48 bytes) [UVC 1.5 4.3.1.1].
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
    usage: u8,
    bit_depth_luma: u8,
    settings: u8,
    max_number_of_ref_frames_plus1: u8,
    rate_control_modes: u16,
    layout_per_stream: u64,
}

impl StreamControl {
    /// Parses a probe/commit control sent by the host (48 bytes for UVC 1.5, also tolerating
    /// shorter 26/34-byte proposals).
    fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < STREAM_CONTROL_LEN_V10 {
            return None;
        }
        let u16_at = |i: usize| u16::from_le_bytes([data[i], data[i + 1]]);
        let u32_at = |i: usize| u32::from_le_bytes([data[i], data[i + 1], data[i + 2], data[i + 3]]);
        let v11 = data.len() >= STREAM_CONTROL_LEN_V11;
        let v15 = data.len() >= STREAM_CONTROL_LEN;
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
            usage: if v15 { data[34] } else { 0 },
            bit_depth_luma: if v15 { data[35] } else { 0 },
            settings: if v15 { data[36] } else { 0 },
            max_number_of_ref_frames_plus1: if v15 { data[37] } else { 0 },
            rate_control_modes: if v15 { u16_at(38) } else { 0 },
            layout_per_stream: if v15 {
                u64::from_le_bytes(data[40..48].try_into().unwrap())
            } else {
                0
            },
        })
    }

    fn to_bytes(self) -> [u8; STREAM_CONTROL_LEN] {
        let mut b = [0u8; STREAM_CONTROL_LEN];
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
        b[34] = self.usage;
        b[35] = self.bit_depth_luma;
        b[36] = self.settings;
        b[37] = self.max_number_of_ref_frames_plus1;
        b[38..40].copy_from_slice(&self.rate_control_modes.to_le_bytes());
        b[40..48].copy_from_slice(&self.layout_per_stream.to_le_bytes());
        b
    }
}

/// 11-byte Still Image Probe/Commit control [UVC 1.5 4.3.1.2].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct StillControl {
    format_index: u8,
    frame_index: u8,
    compression_index: u8,
    max_video_frame_size: u32,
    max_payload_transfer_size: u32,
}

impl StillControl {
    fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < STILL_CONTROL_LEN {
            return None;
        }
        Some(Self {
            format_index: data[0],
            frame_index: data[1],
            compression_index: data[2],
            max_video_frame_size: u32::from_le_bytes([data[3], data[4], data[5], data[6]]),
            max_payload_transfer_size: u32::from_le_bytes([data[7], data[8], data[9], data[10]]),
        })
    }

    fn to_bytes(self) -> [u8; STILL_CONTROL_LEN] {
        let mut b = [0u8; STILL_CONTROL_LEN];
        b[0] = self.format_index;
        b[1] = self.frame_index;
        b[2] = self.compression_index;
        b[3..7].copy_from_slice(&self.max_video_frame_size.to_le_bytes());
        b[7..11].copy_from_slice(&self.max_payload_transfer_size.to_le_bytes());
        b
    }
}

/// The formats and capabilities a camera offers.
#[derive(Clone, Copy)]
struct Catalog<'d> {
    formats: &'d [Format<'d>],
    max_packet_size: u16,
    transport: StreamTransport,
    processing_controls: ProcessingControls,
    camera_controls: CameraControls,
    still: Option<StillConfig<'d>>,
}

impl<'d> Catalog<'d> {
    fn control_for(&self, format_index: u8, frame_index: u8, frame_interval: u32) -> StreamControl {
        let format = &self.formats[format_index as usize - 1];
        let frame = &format.frames[frame_index as usize - 1];
        StreamControl {
            format_index,
            frame_index,
            frame_interval,
            max_video_frame_size: frame.frame_size(format.pixel_format),
            max_payload_transfer_size: self.max_packet_size as u32,
            clock_frequency: CLOCK_FREQUENCY_HZ,
            // Every payload has the frame ID and end-of-frame bits.
            framing_info: 0b11,
            preferred_version: 0x01,
            min_version: 0x01,
            max_version: 0x01,
            ..StreamControl::default()
        }
    }

    fn default_control(&self) -> StreamControl {
        self.control_for(1, 1, self.formats[0].frames[0].intervals[0])
    }

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
        let mut negotiated = self.control_for(format_index, frame_index, interval);
        negotiated.usage = proposal.usage;
        negotiated.bit_depth_luma = proposal.bit_depth_luma;
        negotiated.settings = proposal.settings;
        negotiated.max_number_of_ref_frames_plus1 = proposal.max_number_of_ref_frames_plus1;
        negotiated.rate_control_modes = proposal.rate_control_modes;
        negotiated.layout_per_stream = proposal.layout_per_stream;
        negotiated
    }

    fn still_control_for(&self, format_index: u8, frame_index: u8, compression_index: u8) -> StillControl {
        let Some(still) = self.still else {
            return StillControl::default();
        };
        StillControl {
            format_index,
            frame_index,
            compression_index,
            max_video_frame_size: still.frame_size(frame_index, self.formats[format_index as usize - 1].pixel_format),
            max_payload_transfer_size: self.max_packet_size as u32,
        }
    }

    fn default_still_control(&self) -> StillControl {
        let Some(still) = self.still else {
            return StillControl::default();
        };
        self.still_control_for(1, 1, if still.compression.is_empty() { 0 } else { 1 })
    }

    fn max_still_control(&self) -> StillControl {
        let Some(still) = self.still else {
            return StillControl::default();
        };
        self.still_control_for(
            self.formats.len() as u8,
            still.frames.len() as u8,
            still.compression.len() as u8,
        )
    }

    fn negotiate_still(&self, proposal: &StillControl) -> StillControl {
        let Some(still) = self.still else {
            return StillControl::default();
        };
        let format_index = if (1..=self.formats.len() as u8).contains(&proposal.format_index) {
            proposal.format_index
        } else {
            1
        };
        let frame_index = if (1..=still.frames.len() as u8).contains(&proposal.frame_index) {
            proposal.frame_index
        } else {
            1
        };
        let compression_index = if still.compression.is_empty() {
            0
        } else if (1..=still.compression.len() as u8).contains(&proposal.compression_index) {
            proposal.compression_index
        } else {
            1
        };
        self.still_control_for(format_index, frame_index, compression_index)
    }

    const fn output_terminal_id(&self) -> u8 {
        if self.processing_controls.is_any_enabled() {
            3
        } else {
            2
        }
    }

    const fn output_source_id(&self) -> u8 {
        if self.processing_controls.is_any_enabled() {
            PROCESSING_UNIT_ID
        } else {
            INPUT_TERMINAL_ID
        }
    }
}

/// Mutable state shared by the control handler and the [`Camera`].
#[derive(Clone, Copy)]
struct Inner {
    probe: StreamControl,
    commit: StreamControl,
    still_probe: StillControl,
    still_commit: StillControl,
    still_trigger_pending: bool,
    streaming: bool,
    controls: ControlValues,
    controls_changed: bool,
    vc_error_code: u8,
}

struct Shared<'d> {
    catalog: Catalog<'d>,
    inner: CriticalSectionMutex<Cell<Inner>>,
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
        let default_still = self.catalog.default_still_control();
        let default_controls =
            ControlValues::from_config(&self.catalog.processing_controls, &self.catalog.camera_controls);
        self.inner.lock(|cell| {
            cell.set(Inner {
                probe: default,
                commit: default,
                still_probe: default_still,
                still_commit: default_still,
                still_trigger_pending: false,
                streaming: false,
                controls: default_controls,
                controls_changed: false,
                vc_error_code: ERR_NO_ERROR,
            })
        });
        self.changed.signal(());
    }
}

/// Internal state for a USB Video Class 1.5 camera.
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

enum ControlQuery {
    Value { len: usize, raw: u32 },
    UnsupportedRequest,
    UnsupportedControl,
}

impl ControlQuery {
    fn u16(req: u8, cur: u16, range: Option<ControlRange<u16>>) -> Self {
        let Some(r) = range else {
            return Self::UnsupportedControl;
        };
        match req {
            GET_CUR => Self::Value {
                len: 2,
                raw: u32::from(cur),
            },
            GET_MIN => Self::Value {
                len: 2,
                raw: u32::from(r.min),
            },
            GET_MAX => Self::Value {
                len: 2,
                raw: u32::from(r.max),
            },
            GET_RES => Self::Value {
                len: 2,
                raw: u32::from(r.res),
            },
            GET_DEF => Self::Value {
                len: 2,
                raw: u32::from(r.def),
            },
            GET_INFO => Self::Value { len: 1, raw: 0b11 },
            _ => Self::UnsupportedRequest,
        }
    }

    fn i16(req: u8, cur: i16, range: Option<ControlRange<i16>>) -> Self {
        let Some(r) = range else {
            return Self::UnsupportedControl;
        };
        match req {
            GET_CUR => Self::Value {
                len: 2,
                raw: cur as u16 as u32,
            },
            GET_MIN => Self::Value {
                len: 2,
                raw: r.min as u16 as u32,
            },
            GET_MAX => Self::Value {
                len: 2,
                raw: r.max as u16 as u32,
            },
            GET_RES => Self::Value {
                len: 2,
                raw: r.res as u16 as u32,
            },
            GET_DEF => Self::Value {
                len: 2,
                raw: r.def as u16 as u32,
            },
            GET_INFO => Self::Value { len: 1, raw: 0b11 },
            _ => Self::UnsupportedRequest,
        }
    }

    fn u32(req: u8, cur: u32, range: Option<ControlRange<u32>>) -> Self {
        let Some(r) = range else {
            return Self::UnsupportedControl;
        };
        match req {
            GET_CUR => Self::Value { len: 4, raw: cur },
            GET_MIN => Self::Value { len: 4, raw: r.min },
            GET_MAX => Self::Value { len: 4, raw: r.max },
            GET_RES => Self::Value { len: 4, raw: r.res },
            GET_DEF => Self::Value { len: 4, raw: r.def },
            GET_INFO => Self::Value { len: 1, raw: 0b11 },
            _ => Self::UnsupportedRequest,
        }
    }

    fn bool(req: u8, cur: bool, def: Option<bool>) -> Self {
        let Some(d) = def else {
            return Self::UnsupportedControl;
        };
        match req {
            GET_CUR => Self::Value {
                len: 1,
                raw: u32::from(cur),
            },
            GET_DEF => Self::Value {
                len: 1,
                raw: u32::from(d),
            },
            GET_INFO => Self::Value { len: 1, raw: 0b11 },
            _ => Self::UnsupportedRequest,
        }
    }
}

impl<'d> Control<'d> {
    fn set_vc_error(&self, code: u8) {
        self.shared.update(|inner| inner.vc_error_code = code);
    }

    fn accept_bytes<'a>(&self, buf: &'a mut [u8], src: &[u8]) -> InResponse<'a> {
        let n = src.len().min(buf.len());
        buf[..n].copy_from_slice(&src[..n]);
        InResponse::Accepted(&buf[..n])
    }

    fn streaming_get<'a>(&mut self, req: Request, buf: &'a mut [u8]) -> Option<InResponse<'a>> {
        let selector = (req.value >> 8) as u8;
        match selector {
            VS_PROBE_CONTROL | VS_COMMIT_CONTROL => {
                let probe = selector == VS_PROBE_CONTROL;
                let catalog = &self.shared.catalog;
                let inner = self.shared.get();
                let control = match req.request {
                    GET_CUR => Some(if probe { inner.probe } else { inner.commit }),
                    GET_MIN | GET_DEF if probe => Some(catalog.default_control()),
                    GET_MAX if probe => Some(catalog.max_control()),
                    GET_RES if probe => Some(StreamControl::default()),
                    _ => None,
                };
                if let Some(control) = control {
                    return Some(self.accept_bytes(buf, &control.to_bytes()));
                }
                match req.request {
                    GET_LEN => Some(self.accept_bytes(buf, &(STREAM_CONTROL_LEN as u16).to_le_bytes())),
                    GET_INFO => Some(self.accept_bytes(buf, &[0b11])),
                    _ => Some(InResponse::Rejected),
                }
            }
            VS_STILL_PROBE_CONTROL | VS_STILL_COMMIT_CONTROL if self.shared.catalog.still.is_some() => {
                let probe = selector == VS_STILL_PROBE_CONTROL;
                let catalog = &self.shared.catalog;
                let inner = self.shared.get();
                let control = match req.request {
                    GET_CUR => Some(if probe { inner.still_probe } else { inner.still_commit }),
                    GET_MIN | GET_DEF if probe => Some(catalog.default_still_control()),
                    GET_MAX if probe => Some(catalog.max_still_control()),
                    GET_RES if probe => Some(StillControl::default()),
                    _ => None,
                };
                if let Some(control) = control {
                    return Some(self.accept_bytes(buf, &control.to_bytes()));
                }
                match req.request {
                    GET_LEN => Some(self.accept_bytes(buf, &(STILL_CONTROL_LEN as u16).to_le_bytes())),
                    GET_INFO => Some(self.accept_bytes(buf, &[0b11])),
                    _ => Some(InResponse::Rejected),
                }
            }
            VS_STILL_IMAGE_TRIGGER_CONTROL if self.shared.catalog.still.is_some() => match req.request {
                GET_CUR => Some(self.accept_bytes(buf, &[u8::from(self.shared.get().still_trigger_pending)])),
                GET_INFO => Some(self.accept_bytes(buf, &[0b11])),
                _ => Some(InResponse::Rejected),
            },
            _ => Some(InResponse::Rejected),
        }
    }

    fn streaming_set(&mut self, req: Request, data: &[u8]) -> Option<OutResponse> {
        if req.request != SET_CUR {
            return Some(OutResponse::Rejected);
        }
        match (req.value >> 8) as u8 {
            VS_PROBE_CONTROL => {
                let Some(proposal) = StreamControl::parse(data) else {
                    return Some(OutResponse::Rejected);
                };
                let negotiated = self.shared.catalog.negotiate(&proposal);
                self.shared.update(|inner| inner.probe = negotiated);
                Some(OutResponse::Accepted)
            }
            VS_COMMIT_CONTROL => {
                let Some(proposal) = StreamControl::parse(data) else {
                    return Some(OutResponse::Rejected);
                };
                let negotiated = self.shared.catalog.negotiate(&proposal);
                let is_bulk = self.shared.catalog.transport == StreamTransport::Bulk;
                self.shared.update(|inner| {
                    inner.commit = negotiated;
                    if is_bulk {
                        inner.streaming = true;
                    }
                });
                if is_bulk {
                    self.shared.changed.signal(());
                }
                Some(OutResponse::Accepted)
            }
            VS_STILL_PROBE_CONTROL if self.shared.catalog.still.is_some() => {
                let Some(proposal) = StillControl::parse(data) else {
                    return Some(OutResponse::Rejected);
                };
                let negotiated = self.shared.catalog.negotiate_still(&proposal);
                self.shared.update(|inner| inner.still_probe = negotiated);
                Some(OutResponse::Accepted)
            }
            VS_STILL_COMMIT_CONTROL if self.shared.catalog.still.is_some() => {
                let Some(proposal) = StillControl::parse(data) else {
                    return Some(OutResponse::Rejected);
                };
                let negotiated = self.shared.catalog.negotiate_still(&proposal);
                self.shared.update(|inner| inner.still_commit = negotiated);
                Some(OutResponse::Accepted)
            }
            VS_STILL_IMAGE_TRIGGER_CONTROL if self.shared.catalog.still.is_some() => {
                if data.len() != 1 {
                    return Some(OutResponse::Rejected);
                }
                match data[0] {
                    0 => self.shared.update(|inner| inner.still_trigger_pending = false),
                    1 => self.shared.update(|inner| inner.still_trigger_pending = true),
                    _ => return Some(OutResponse::Rejected),
                }
                Some(OutResponse::Accepted)
            }
            _ => Some(OutResponse::Rejected),
        }
    }

    fn handle_vc_get<'a>(&mut self, req: Request, buf: &'a mut [u8]) -> Option<InResponse<'a>> {
        let entity_id = (req.index >> 8) as u8;
        let selector = (req.value >> 8) as u8;
        let inner = self.shared.get();

        if entity_id == 0 {
            if selector == VC_REQUEST_ERROR_CODE_CONTROL && req.request == GET_CUR {
                return Some(self.accept_bytes(buf, &[inner.vc_error_code]));
            }
            self.set_vc_error(ERR_INVALID_CONTROL);
            return Some(InResponse::Rejected);
        }

        let query = if entity_id == INPUT_TERMINAL_ID {
            let ct = self.shared.catalog.camera_controls;
            match selector {
                CT_AE_MODE_CONTROL => match ct.ae_mode {
                    None => ControlQuery::UnsupportedControl,
                    Some(def) => match req.request {
                        GET_CUR => ControlQuery::Value {
                            len: 1,
                            raw: u32::from(inner.controls.ae_mode),
                        },
                        GET_DEF => ControlQuery::Value {
                            len: 1,
                            raw: u32::from(def),
                        },
                        GET_RES => ControlQuery::Value { len: 1, raw: 0x0F },
                        GET_INFO => ControlQuery::Value { len: 1, raw: 0b11 },
                        _ => ControlQuery::UnsupportedRequest,
                    },
                },
                CT_EXPOSURE_TIME_ABSOLUTE_CONTROL => {
                    ControlQuery::u32(req.request, inner.controls.exposure_time_abs, ct.exposure_time_abs)
                }
                CT_FOCUS_ABSOLUTE_CONTROL => ControlQuery::u16(req.request, inner.controls.focus_abs, ct.focus_abs),
                CT_FOCUS_AUTO_CONTROL => ControlQuery::bool(req.request, inner.controls.focus_auto, ct.focus_auto),
                CT_ZOOM_ABSOLUTE_CONTROL => ControlQuery::u16(req.request, inner.controls.zoom_abs, ct.zoom_abs),
                _ => ControlQuery::UnsupportedControl,
            }
        } else if entity_id == PROCESSING_UNIT_ID && self.shared.catalog.processing_controls.is_any_enabled() {
            let pu = self.shared.catalog.processing_controls;
            match selector {
                PU_BACKLIGHT_COMPENSATION_CONTROL => ControlQuery::u16(
                    req.request,
                    inner.controls.backlight_compensation,
                    pu.backlight_compensation,
                ),
                PU_BRIGHTNESS_CONTROL => ControlQuery::i16(req.request, inner.controls.brightness, pu.brightness),
                PU_CONTRAST_CONTROL => ControlQuery::u16(req.request, inner.controls.contrast, pu.contrast),
                PU_GAIN_CONTROL => ControlQuery::u16(req.request, inner.controls.gain, pu.gain),
                PU_POWER_LINE_FREQUENCY_CONTROL => match pu.power_line_frequency {
                    None => ControlQuery::UnsupportedControl,
                    Some(def) => match req.request {
                        GET_CUR => ControlQuery::Value {
                            len: 1,
                            raw: u32::from(inner.controls.power_line_frequency),
                        },
                        GET_MIN => ControlQuery::Value { len: 1, raw: 0 },
                        GET_MAX => ControlQuery::Value { len: 1, raw: 3 },
                        GET_RES => ControlQuery::Value { len: 1, raw: 1 },
                        GET_DEF => ControlQuery::Value {
                            len: 1,
                            raw: u32::from(def),
                        },
                        GET_INFO => ControlQuery::Value { len: 1, raw: 0b11 },
                        _ => ControlQuery::UnsupportedRequest,
                    },
                },
                PU_HUE_CONTROL => ControlQuery::i16(req.request, inner.controls.hue, pu.hue),
                PU_SATURATION_CONTROL => ControlQuery::u16(req.request, inner.controls.saturation, pu.saturation),
                PU_SHARPNESS_CONTROL => ControlQuery::u16(req.request, inner.controls.sharpness, pu.sharpness),
                PU_GAMMA_CONTROL => ControlQuery::u16(req.request, inner.controls.gamma, pu.gamma),
                PU_WHITE_BALANCE_TEMPERATURE_CONTROL => {
                    ControlQuery::u16(req.request, inner.controls.white_balance_temp, pu.white_balance_temp)
                }
                PU_WHITE_BALANCE_TEMPERATURE_AUTO_CONTROL => {
                    ControlQuery::bool(req.request, inner.controls.white_balance_auto, pu.white_balance_auto)
                }
                PU_HUE_AUTO_CONTROL => ControlQuery::bool(req.request, inner.controls.hue_auto, pu.hue_auto),
                _ => ControlQuery::UnsupportedControl,
            }
        } else {
            self.set_vc_error(ERR_INVALID_UNIT);
            return Some(InResponse::Rejected);
        };

        match query {
            ControlQuery::Value { len, raw } => {
                self.set_vc_error(ERR_NO_ERROR);
                Some(self.accept_bytes(buf, &raw.to_le_bytes()[..len]))
            }
            ControlQuery::UnsupportedRequest => {
                self.set_vc_error(ERR_INVALID_REQUEST);
                Some(InResponse::Rejected)
            }
            ControlQuery::UnsupportedControl => {
                self.set_vc_error(ERR_INVALID_CONTROL);
                Some(InResponse::Rejected)
            }
        }
    }

    fn apply_vc_update(&self, apply: impl FnOnce(&mut ControlValues)) -> OutResponse {
        self.shared.update(|inner| {
            apply(&mut inner.controls);
            inner.controls_changed = true;
            inner.vc_error_code = ERR_NO_ERROR;
        });
        OutResponse::Accepted
    }

    fn set_ranged_u16(
        &self,
        range: Option<ControlRange<u16>>,
        data: &[u8],
        apply: impl FnOnce(&mut ControlValues, u16),
    ) -> OutResponse {
        let Some(r) = range else {
            self.set_vc_error(ERR_INVALID_CONTROL);
            return OutResponse::Rejected;
        };
        if data.len() != 2 {
            self.set_vc_error(ERR_INVALID_REQUEST);
            return OutResponse::Rejected;
        }
        let v = u16::from_le_bytes([data[0], data[1]]);
        if v < r.min || v > r.max {
            self.set_vc_error(ERR_OUT_OF_RANGE);
            return OutResponse::Rejected;
        }
        self.apply_vc_update(|c| apply(c, v))
    }

    fn set_ranged_i16(
        &self,
        range: Option<ControlRange<i16>>,
        data: &[u8],
        apply: impl FnOnce(&mut ControlValues, i16),
    ) -> OutResponse {
        let Some(r) = range else {
            self.set_vc_error(ERR_INVALID_CONTROL);
            return OutResponse::Rejected;
        };
        if data.len() != 2 {
            self.set_vc_error(ERR_INVALID_REQUEST);
            return OutResponse::Rejected;
        }
        let v = i16::from_le_bytes([data[0], data[1]]);
        if v < r.min || v > r.max {
            self.set_vc_error(ERR_OUT_OF_RANGE);
            return OutResponse::Rejected;
        }
        self.apply_vc_update(|c| apply(c, v))
    }

    fn set_bool_ctrl(&self, enabled: bool, data: &[u8], apply: impl FnOnce(&mut ControlValues, bool)) -> OutResponse {
        if !enabled {
            self.set_vc_error(ERR_INVALID_CONTROL);
            return OutResponse::Rejected;
        }
        if data.len() != 1 {
            self.set_vc_error(ERR_INVALID_REQUEST);
            return OutResponse::Rejected;
        }
        let v = data[0] != 0;
        self.apply_vc_update(|c| apply(c, v))
    }

    fn handle_vc_set(&mut self, req: Request, data: &[u8]) -> Option<OutResponse> {
        if req.request != SET_CUR {
            self.set_vc_error(ERR_INVALID_REQUEST);
            return Some(OutResponse::Rejected);
        }
        let entity_id = (req.index >> 8) as u8;
        let selector = (req.value >> 8) as u8;

        if entity_id == INPUT_TERMINAL_ID {
            let ct = self.shared.catalog.camera_controls;
            let resp = match selector {
                CT_AE_MODE_CONTROL if ct.ae_mode.is_some() && data.len() == 1 => {
                    let v = data[0];
                    if v == 0 || (v & !0x0F) != 0 {
                        self.set_vc_error(ERR_OUT_OF_RANGE);
                        OutResponse::Rejected
                    } else {
                        self.apply_vc_update(|c| c.ae_mode = v)
                    }
                }
                CT_EXPOSURE_TIME_ABSOLUTE_CONTROL if ct.exposure_time_abs.is_some() && data.len() == 4 => {
                    let r = ct.exposure_time_abs.unwrap();
                    let v = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
                    if v < r.min || v > r.max {
                        self.set_vc_error(ERR_OUT_OF_RANGE);
                        OutResponse::Rejected
                    } else {
                        self.apply_vc_update(|c| c.exposure_time_abs = v)
                    }
                }
                CT_FOCUS_ABSOLUTE_CONTROL => self.set_ranged_u16(ct.focus_abs, data, |c, v| c.focus_abs = v),
                CT_FOCUS_AUTO_CONTROL => self.set_bool_ctrl(ct.focus_auto.is_some(), data, |c, v| c.focus_auto = v),
                CT_ZOOM_ABSOLUTE_CONTROL => self.set_ranged_u16(ct.zoom_abs, data, |c, v| c.zoom_abs = v),
                _ => {
                    self.set_vc_error(ERR_INVALID_CONTROL);
                    OutResponse::Rejected
                }
            };
            return Some(resp);
        }

        if entity_id == PROCESSING_UNIT_ID && self.shared.catalog.processing_controls.is_any_enabled() {
            let pu = self.shared.catalog.processing_controls;
            let resp = match selector {
                PU_BACKLIGHT_COMPENSATION_CONTROL => {
                    self.set_ranged_u16(pu.backlight_compensation, data, |c, v| c.backlight_compensation = v)
                }
                PU_BRIGHTNESS_CONTROL => self.set_ranged_i16(pu.brightness, data, |c, v| c.brightness = v),
                PU_CONTRAST_CONTROL => self.set_ranged_u16(pu.contrast, data, |c, v| c.contrast = v),
                PU_GAIN_CONTROL => self.set_ranged_u16(pu.gain, data, |c, v| c.gain = v),
                PU_POWER_LINE_FREQUENCY_CONTROL if pu.power_line_frequency.is_some() && data.len() == 1 => {
                    let v = data[0];
                    if v > 3 {
                        self.set_vc_error(ERR_OUT_OF_RANGE);
                        OutResponse::Rejected
                    } else {
                        self.apply_vc_update(|c| c.power_line_frequency = v)
                    }
                }
                PU_HUE_CONTROL => self.set_ranged_i16(pu.hue, data, |c, v| c.hue = v),
                PU_SATURATION_CONTROL => self.set_ranged_u16(pu.saturation, data, |c, v| c.saturation = v),
                PU_SHARPNESS_CONTROL => self.set_ranged_u16(pu.sharpness, data, |c, v| c.sharpness = v),
                PU_GAMMA_CONTROL => self.set_ranged_u16(pu.gamma, data, |c, v| c.gamma = v),
                PU_WHITE_BALANCE_TEMPERATURE_CONTROL => {
                    self.set_ranged_u16(pu.white_balance_temp, data, |c, v| c.white_balance_temp = v)
                }
                PU_WHITE_BALANCE_TEMPERATURE_AUTO_CONTROL => {
                    self.set_bool_ctrl(pu.white_balance_auto.is_some(), data, |c, v| c.white_balance_auto = v)
                }
                PU_HUE_AUTO_CONTROL => self.set_bool_ctrl(pu.hue_auto.is_some(), data, |c, v| c.hue_auto = v),
                _ => {
                    self.set_vc_error(ERR_INVALID_CONTROL);
                    OutResponse::Rejected
                }
            };
            return Some(resp);
        }

        self.set_vc_error(ERR_INVALID_UNIT);
        Some(OutResponse::Rejected)
    }

    fn is_streaming_request(&self, req: &Request) -> bool {
        req.request_type == RequestType::Class
            && req.recipient == Recipient::Interface
            && req.index as u8 == self.video_streaming.0
    }

    fn is_vc_request(&self, req: &Request) -> bool {
        req.request_type == RequestType::Class
            && req.recipient == Recipient::Interface
            && req.index as u8 == self.video_control.0
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
        if iface == self.video_streaming && self.shared.catalog.transport == StreamTransport::Isochronous {
            self.shared.update(|inner| inner.streaming = alternate_setting != 0);
            self.shared.changed.signal(());
        }
    }

    fn control_out(&mut self, req: control::Request, data: &[u8]) -> Option<OutResponse> {
        if self.is_streaming_request(&req) {
            self.streaming_set(req, data)
        } else if self.is_vc_request(&req) {
            self.handle_vc_set(req, data)
        } else {
            None
        }
    }

    fn control_in<'a>(&'a mut self, req: Request, buf: &'a mut [u8]) -> Option<InResponse<'a>> {
        if self.is_streaming_request(&req) {
            self.streaming_get(req, buf)
        } else if self.is_vc_request(&req) {
            self.handle_vc_get(req, buf)
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

enum StreamEndpoint<'d, D: Driver<'d>> {
    Iso(D::IsoEndpointIn),
    Bulk(D::EndpointIn),
}

impl<'d, D: Driver<'d>> StreamEndpoint<'d, D> {
    fn info(&self) -> EndpointInfo {
        match self {
            Self::Iso(ep) => ep.info(),
            Self::Bulk(ep) => ep.info(),
        }
    }

    fn is_enabled(&self) -> bool {
        match self {
            Self::Iso(ep) => ep.is_enabled(),
            Self::Bulk(ep) => ep.is_enabled(),
        }
    }

    async fn wait_enabled(&mut self) {
        match self {
            Self::Iso(ep) => ep.wait_enabled().await,
            Self::Bulk(ep) => ep.wait_enabled().await,
        }
    }

    async fn send(&mut self, packet: &[u8], last: bool) -> Result<(), EndpointError> {
        match self {
            Self::Iso(ep) => {
                let (_skipped, slot) = ep.ready().await?;
                match slot.write(packet) {
                    // Late packets are dropped.
                    Ok(()) | Err(IsoWriteError::TooLate) => Ok(()),
                    Err(_) => Err(EndpointError::BufferOverflow),
                }
            }
            Self::Bulk(ep) => {
                ep.write_transfer(packet, last).await?;
                Ok(())
            }
        }
    }
}

/// Implementation of the USB Video Class 1.5, camera side.
pub struct Camera<'d, D: Driver<'d>> {
    endpoint: StreamEndpoint<'d, D>,
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
    /// interval, invalid still image configuration, a packet size below 13 or above the speed's
    /// limit, or the `control_buf` of the builder is smaller than 48 bytes.
    pub fn new(builder: &mut Builder<'d, D>, state: &'d mut State<'d>, config: Config<'d>) -> Self {
        let Config {
            formats,
            max_packet_size,
            transport,
            processing_controls,
            camera_controls,
            still,
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
        if let Some(still_cfg) = still {
            assert!(
                (1..=255).contains(&still_cfg.frames.len()),
                "between one and 255 still frames"
            );
            assert!(still_cfg.compression.len() <= 255, "at most 255 compression patterns");
            for f in still_cfg.frames {
                assert!(f.width != 0 && f.height != 0, "still frame size must not be zero");
            }
        }
        let speed_limit = match (transport, builder.max_speed()) {
            (StreamTransport::Isochronous, UsbDeviceSpeed::Full) => MAX_PACKET_SIZE_FULL_SPEED,
            (StreamTransport::Isochronous, _) => MAX_PACKET_SIZE_HIGH_SPEED,
            (StreamTransport::Bulk, UsbDeviceSpeed::Full) => MAX_BULK_PACKET_SIZE_FULL_SPEED,
            (StreamTransport::Bulk, _) => MAX_BULK_PACKET_SIZE_HIGH_SPEED,
        };
        assert!(
            (PAYLOAD_HEADER_LEN_PTS_SCR as u16 + 1..=speed_limit).contains(&max_packet_size),
            "the packet size must hold the 12-byte payload header and at most {} bytes at this speed",
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
            transport,
            processing_controls,
            camera_controls,
            still,
        };

        let mut func = builder.function(USB_CLASS_VIDEO, SC_VIDEO_INTERFACE_COLLECTION, PC_PROTOCOL_15);

        // Video control interface: Camera Input Terminal -> [Processing Unit] -> Streaming Output Terminal.
        let mut vc = func.interface();
        let vc_number = vc.interface_number();
        let vs_number = InterfaceNumber::new(vc_number.0 + 1);
        let mut alt = vc.alt_setting(USB_CLASS_VIDEO, SC_VIDEOCONTROL, PC_PROTOCOL_15, None);
        alt.descriptors_then_patch(
            |alt| {
                alt.descriptor(CS_INTERFACE, &vc_header(vs_number));
                alt.descriptor(CS_INTERFACE, &camera_input_terminal(&camera_controls));
                if processing_controls.is_any_enabled() {
                    alt.descriptor(CS_INTERFACE, &processing_unit(&processing_controls));
                }
                alt.descriptor(
                    CS_INTERFACE,
                    &streaming_output_terminal(catalog.output_terminal_id(), catalog.output_source_id()),
                );
            },
            |written| {
                // wTotalLength follows bLength, bDescriptorType, bDescriptorSubtype, bcdUVC.
                let len = (written.len() as u16).to_le_bytes();
                written[5..7].copy_from_slice(&len);
            },
        );

        // Video streaming interface.
        let mut vs = func.interface();
        assert_eq!(vs.interface_number(), vs_number);
        let mut alt0 = vs.alt_setting(USB_CLASS_VIDEO, SC_VIDEOSTREAMING, PC_PROTOCOL_15, None);
        let endpoint = match transport {
            StreamTransport::Isochronous => StreamEndpoint::Iso(alt0.alloc_iso_endpoint_in(
                None,
                max_packet_size,
                EndpointInterval::NONE,
                EndpointOptions::new(),
            )),
            StreamTransport::Bulk => StreamEndpoint::Bulk(alt0.endpoint_bulk_in(None, EndpointOptions::new())),
        };
        let endpoint_address = endpoint.info().addr.into();
        alt0.descriptors_then_patch(
            |alt| {
                alt.descriptor(
                    CS_INTERFACE,
                    &vs_input_header(
                        formats.len() as u8,
                        endpoint_address,
                        catalog.output_terminal_id(),
                        still.is_some(),
                    ),
                );
                for (i, format) in formats.iter().enumerate() {
                    alt.descriptor(CS_INTERFACE, &format_descriptor(format, i as u8 + 1));
                    for (j, frame) in format.frames.iter().enumerate() {
                        alt.descriptor(CS_INTERFACE, &frame_descriptor(format, frame, j as u8 + 1));
                    }
                }
                if let Some(still_cfg) = still {
                    alt.descriptor(CS_INTERFACE, &still_image_frame_descriptor(&still_cfg));
                }
            },
            |written| {
                // wTotalLength follows bLength, bDescriptorType, bDescriptorSubtype, bNumFormats.
                let len = (written.len() as u16).to_le_bytes();
                written[4..6].copy_from_slice(&len);
            },
        );

        if transport == StreamTransport::Isochronous {
            let mut alt1 = vs.alt_setting(USB_CLASS_VIDEO, SC_VIDEOSTREAMING, PC_PROTOCOL_15, None);
            alt1.endpoint_descriptor(
                &endpoint.info(),
                SynchronizationType::Asynchronous,
                UsageType::DataEndpoint,
                &[],
            );
        } else {
            alt0.endpoint_descriptor(
                &endpoint.info(),
                SynchronizationType::NoSynchronization,
                UsageType::DataEndpoint,
                &[],
            );
        }
        drop(func);

        let State { control, shared } = state;
        let shared: &'d Shared<'d> = shared.insert(Shared {
            catalog,
            inner: CriticalSectionMutex::new(Cell::new(Inner {
                probe: catalog.default_control(),
                commit: catalog.default_control(),
                still_probe: catalog.default_still_control(),
                still_commit: catalog.default_still_control(),
                still_trigger_pending: false,
                streaming: false,
                controls: ControlValues::from_config(&processing_controls, &camera_controls),
                controls_changed: false,
                vc_error_code: ERR_NO_ERROR,
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

    /// Returns the current Processing Unit and Camera Terminal control values.
    pub fn control_values(&self) -> ControlValues {
        self.shared.get().controls
    }

    /// Returns updated control values if the host modified any since the last call.
    pub fn take_controls_changed(&self) -> Option<ControlValues> {
        let mut updated = None;
        self.shared.update(|inner| {
            if inner.controls_changed {
                inner.controls_changed = false;
                updated = Some(inner.controls);
            }
        });
        updated
    }

    /// Returns and clears the committed [`StillSettings`] if the host triggered a Method 2
    /// still image capture via `VS_STILL_IMAGE_TRIGGER_CONTROL`.
    pub fn take_still_trigger(&self) -> Option<StillSettings<'d>> {
        let still = self.shared.catalog.still?;
        let mut result = None;
        self.shared.update(|inner| {
            if inner.still_trigger_pending {
                inner.still_trigger_pending = false;
                let c = inner.still_commit;
                let compression = if c.compression_index > 0 {
                    still.compression.get(c.compression_index as usize - 1).copied()
                } else {
                    None
                };
                result = Some(StillSettings {
                    format: &self.shared.catalog.formats[c.format_index as usize - 1],
                    frame: still.frames[c.frame_index as usize - 1],
                    compression,
                    max_frame_size: c.max_video_frame_size,
                });
            }
        });
        result
    }

    /// Waits until the host starts the stream (selects alternate setting 1 for isochronous,
    /// or commits the stream for bulk), then returns what it selected.
    pub async fn wait_streaming(&mut self) -> StreamSettings<'d> {
        loop {
            if self.shared.get().streaming && self.endpoint.is_enabled() {
                return self.settings();
            }
            if self.shared.get().streaming {
                self.endpoint.wait_enabled().await;
                if self.shared.get().streaming {
                    return self.settings();
                }
            }
            self.shared.changed.wait().await;
        }
    }

    /// Starts streaming a video frame chunk by chunk without holding the full frame in RAM.
    pub fn start_frame(&mut self) -> FrameWriter<'_, 'd, D> {
        self.start_payload(false, None)
    }

    /// Starts streaming a video frame with `PTS` (and optional `SCR`) timestamps.
    pub fn start_frame_with_timestamp(&mut self, ts: PayloadTimestamp) -> FrameWriter<'_, 'd, D> {
        self.start_payload(false, Some(ts))
    }

    /// Starts streaming a Method 2 still image frame chunk by chunk.
    pub fn start_still_frame(&mut self) -> FrameWriter<'_, 'd, D> {
        self.start_payload(true, None)
    }

    /// Sends one complete video frame with the minimal 2-byte payload header.
    pub async fn write_frame(&mut self, frame: &[u8]) -> Result<(), EndpointError> {
        let mut writer = self.start_frame();
        writer.write(frame).await?;
        writer.finish().await
    }

    /// Sends one complete video frame with UVC `PTS` (and optional `SCR`) timestamps.
    pub async fn write_frame_with_timestamp(
        &mut self,
        frame: &[u8],
        ts: PayloadTimestamp,
    ) -> Result<(), EndpointError> {
        let mut writer = self.start_frame_with_timestamp(ts);
        writer.write(frame).await?;
        writer.finish().await
    }

    /// Sends one complete Method 2 still image frame (sets the `STI` still-image bit).
    pub async fn write_still_frame(&mut self, frame: &[u8]) -> Result<(), EndpointError> {
        let mut writer = self.start_still_frame();
        writer.write(frame).await?;
        writer.finish().await
    }

    fn start_payload(&mut self, still: bool, timestamp: Option<PayloadTimestamp>) -> FrameWriter<'_, 'd, D> {
        self.frame_id = !self.frame_id;
        let header_len = match timestamp {
            None => PAYLOAD_HEADER_LEN_MIN,
            Some(PayloadTimestamp { scr_stc: None, .. }) => PAYLOAD_HEADER_LEN_PTS,
            Some(PayloadTimestamp { scr_stc: Some(_), .. }) => PAYLOAD_HEADER_LEN_PTS_SCR,
        };
        let mut packet = [0u8; MAX_PACKET_SIZE_HIGH_SPEED as usize];
        let mut flags = HEADER_EOH | if self.frame_id { HEADER_FID } else { 0 };
        if still {
            flags |= HEADER_STI;
        }
        if let Some(ts) = timestamp {
            flags |= HEADER_PTS;
            packet[2..6].copy_from_slice(&ts.pts.to_le_bytes());
            if let Some((stc, sof)) = ts.scr_stc {
                flags |= HEADER_SCR;
                packet[6..10].copy_from_slice(&stc.to_le_bytes());
                packet[10..12].copy_from_slice(&sof.to_le_bytes());
            }
        }
        packet[0] = header_len as u8;
        packet[1] = flags;

        FrameWriter {
            endpoint: &mut self.endpoint,
            packet,
            header_len,
            filled: header_len,
            max_packet_size: self.max_packet_size,
        }
    }
}

/// Streaming writer for a single UVC video or still image frame.
pub struct FrameWriter<'a, 'd, D: Driver<'d>> {
    endpoint: &'a mut StreamEndpoint<'d, D>,
    packet: [u8; MAX_PACKET_SIZE_HIGH_SPEED as usize],
    header_len: usize,
    filled: usize,
    max_packet_size: usize,
}

impl<'a, 'd, D: Driver<'d>> FrameWriter<'a, 'd, D> {
    /// Writes a chunk of frame payload bytes, flushing full packets as needed.
    pub async fn write(&mut self, mut chunk: &[u8]) -> Result<(), EndpointError> {
        while !chunk.is_empty() {
            let space = self.max_packet_size - self.filled;
            let take = chunk.len().min(space);
            self.packet[self.filled..self.filled + take].copy_from_slice(&chunk[..take]);
            self.filled += take;
            chunk = &chunk[take..];

            // Only flush when more bytes remain so `finish` can attach EOF to the final packet.
            if self.filled == self.max_packet_size && !chunk.is_empty() {
                self.endpoint.send(&self.packet[..self.max_packet_size], false).await?;
                self.filled = self.header_len;
            }
        }
        Ok(())
    }

    /// Finishes the frame by sending the final packet with `EOF` set (and a ZLP for Bulk if needed).
    pub async fn finish(mut self) -> Result<(), EndpointError> {
        self.packet[1] |= HEADER_EOF;
        self.endpoint.send(&self.packet[..self.filled], true).await?;
        Ok(())
    }

    /// Aborts the frame by sending the final packet with `ERR | EOF` set so the host discards it.
    pub async fn abort(mut self) -> Result<(), EndpointError> {
        self.packet[1] |= HEADER_ERR | HEADER_EOF;
        self.endpoint.send(&self.packet[..self.filled], true).await?;
        Ok(())
    }
}

fn vc_header(video_streaming: InterfaceNumber) -> [u8; 11] {
    let mut d = [0; 11];
    d[0] = VC_HEADER;
    d[1..3].copy_from_slice(&BCD_UVC_15.to_le_bytes());
    // d[3..5]: wTotalLength, patched.
    d[5..9].copy_from_slice(&CLOCK_FREQUENCY_HZ.to_le_bytes());
    d[9] = 1; // bInCollection
    d[10] = video_streaming.0;
    d
}

fn camera_input_terminal(controls: &CameraControls) -> [u8; 16] {
    let mut d = [0; 16];
    d[0] = VC_INPUT_TERMINAL;
    d[1] = INPUT_TERMINAL_ID;
    d[2..4].copy_from_slice(&ITT_CAMERA.to_le_bytes());
    // bAssocTerminal, iTerminal and the focal lengths: none.
    d[12] = 3; // bControlSize
    d[13..16].copy_from_slice(&controls.bm_controls());
    d
}

fn processing_unit(controls: &ProcessingControls) -> [u8; 11] {
    let mut d = [0; 11];
    d[0] = VC_PROCESSING_UNIT;
    d[1] = PROCESSING_UNIT_ID;
    d[2] = INPUT_TERMINAL_ID; // bSourceID
    // d[3..5]: wMaxMultiplier = 0
    d[5] = 3; // bControlSize
    d[6..9].copy_from_slice(&controls.bm_controls());
    d
}

fn streaming_output_terminal(terminal_id: u8, source_id: u8) -> [u8; 7] {
    let mut d = [0; 7];
    d[0] = VC_OUTPUT_TERMINAL;
    d[1] = terminal_id;
    d[2..4].copy_from_slice(&TT_STREAMING.to_le_bytes());
    d[5] = source_id;
    d
}

fn vs_input_header(
    num_formats: u8,
    endpoint_address: u8,
    terminal_link: u8,
    still_capture: bool,
) -> Vec<u8, { 10 + MAX_FORMATS }> {
    let mut d = Vec::new();
    d.push(VS_INPUT_HEADER).unwrap();
    d.push(num_formats).unwrap();
    d.extend_from_slice(&[0, 0]).unwrap(); // wTotalLength, patched.
    d.push(endpoint_address).unwrap();
    d.push(0).unwrap(); // bmInfo: no dynamic format change.
    d.push(terminal_link).unwrap(); // bTerminalLink
    d.push(if still_capture { 2 } else { 0 }).unwrap(); // bStillCaptureMethod
    d.push(0).unwrap(); // bTriggerSupport
    d.push(0).unwrap(); // bTriggerUsage
    d.push(1).unwrap(); // bControlSize
    d.resize(d.len() + num_formats as usize, 0).unwrap();
    d
}

fn format_descriptor(format: &Format, index: u8) -> Vec<u8, 26> {
    let mut d = Vec::new();
    let subtype = if format.pixel_format.is_frame_based() {
        VS_FORMAT_FRAME_BASED
    } else if format.pixel_format == PixelFormat::Mjpeg {
        VS_FORMAT_MJPEG
    } else {
        VS_FORMAT_UNCOMPRESSED
    };
    d.push(subtype).unwrap();
    d.push(index).unwrap();
    d.push(format.frames.len() as u8).unwrap();
    match format.pixel_format.guid() {
        Some(guid) => {
            d.extend_from_slice(&guid).unwrap();
            d.push(format.pixel_format.bits_per_pixel()).unwrap();
        }
        None => d.push(0).unwrap(),
    }
    d.push(1).unwrap(); // bDefaultFrameIndex
    d.push(0).unwrap(); // bAspectRatioX
    d.push(0).unwrap(); // bAspectRatioY
    d.push(0).unwrap(); // bmInterlaceFlags
    d.push(0).unwrap(); // bCopyProtect
    if format.pixel_format.is_frame_based() {
        d.push(u8::from(format.pixel_format.variable_size())).unwrap();
    }
    d
}

fn frame_descriptor(format: &Format, frame: &Frame, index: u8) -> Vec<u8, 255> {
    let max_frame_size = frame.frame_size(format.pixel_format);
    let bit_rate =
        |interval: u32| (max_frame_size as u64 * 8 * 10_000_000 / interval as u64).min(u32::MAX as u64) as u32;
    let min_interval = frame.intervals.iter().min().copied().unwrap();
    let max_interval = frame.intervals.iter().max().copied().unwrap();

    let mut d = Vec::new();
    let subtype = if format.pixel_format.is_frame_based() {
        VS_FRAME_FRAME_BASED
    } else if format.pixel_format == PixelFormat::Mjpeg {
        VS_FRAME_MJPEG
    } else {
        VS_FRAME_UNCOMPRESSED
    };
    d.push(subtype).unwrap();
    d.push(index).unwrap();
    d.push(0).unwrap(); // bmCapabilities: fixed frame rate
    d.extend_from_slice(&frame.width.to_le_bytes()).unwrap();
    d.extend_from_slice(&frame.height.to_le_bytes()).unwrap();
    d.extend_from_slice(&bit_rate(max_interval).to_le_bytes()).unwrap();
    d.extend_from_slice(&bit_rate(min_interval).to_le_bytes()).unwrap();
    d.extend_from_slice(&max_frame_size.to_le_bytes()).unwrap();
    d.extend_from_slice(&frame.intervals[0].to_le_bytes()).unwrap();
    d.push(frame.intervals.len() as u8).unwrap();
    if format.pixel_format.is_frame_based() {
        let bytes_per_line: u32 = if format.pixel_format.variable_size() {
            0
        } else {
            u32::from(frame.width) * u32::from(format.pixel_format.bits_per_pixel()) / 8
        };
        d.extend_from_slice(&bytes_per_line.to_le_bytes()).unwrap();
    }
    for interval in frame.intervals {
        d.extend_from_slice(&interval.to_le_bytes()).unwrap();
    }
    d
}

fn still_image_frame_descriptor(still: &StillConfig) -> Vec<u8, 255> {
    let mut d = Vec::new();
    d.push(VS_STILL_IMAGE_FRAME).unwrap();
    d.push(0).unwrap(); // bEndpointAddress = 0 for Method 2
    d.push(still.frames.len() as u8).unwrap();
    for frame in still.frames {
        d.extend_from_slice(&frame.width.to_le_bytes()).unwrap();
        d.extend_from_slice(&frame.height.to_le_bytes()).unwrap();
    }
    d.push(still.compression.len() as u8).unwrap();
    for &c in still.compression {
        d.push(c).unwrap();
    }
    d
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;
    use crate::device::test_utils::TestDriver;

    const INTERVALS: [u32; 2] = [333_333, 666_666];
    const FRAMES: [Frame; 2] = [Frame::new(160, 120, &INTERVALS), Frame::new(320, 240, &INTERVALS)];
    const FORMATS: [Format; 2] = [
        Format::new(PixelFormat::Yuy2, &FRAMES),
        Format::new(PixelFormat::Mjpeg, &FRAMES),
    ];
    const CATALOG: Catalog = Catalog {
        formats: &FORMATS,
        max_packet_size: 1023,
        transport: StreamTransport::Isochronous,
        processing_controls: ProcessingControls::none(),
        camera_controls: CameraControls::none(),
        still: None,
    };

    #[test]
    fn unit_descriptors_match_spec_lengths() {
        assert_eq!(2 + vc_header(InterfaceNumber(1)).len(), 13);
        assert_eq!(2 + camera_input_terminal(&CameraControls::none()).len(), 18);
        assert_eq!(2 + processing_unit(&ProcessingControls::none()).len(), 13);
        assert_eq!(2 + streaming_output_terminal(2, 1).len(), 9);
        assert_eq!(2 + vs_input_header(2, 0x81, 2, false).len(), 13 + 2);
    }

    #[test]
    fn format_descriptors_match_spec_lengths() {
        assert_eq!(2 + format_descriptor(&FORMATS[0], 1).len(), 27);
        assert_eq!(2 + format_descriptor(&FORMATS[1], 2).len(), 11);
        assert_eq!(2 + frame_descriptor(&FORMATS[0], &FRAMES[0], 1).len(), 26 + 4 * 2);

        let h264_fmt = Format::new(PixelFormat::H264, &FRAMES);
        assert_eq!(2 + format_descriptor(&h264_fmt, 1).len(), 28);
        assert_eq!(2 + frame_descriptor(&h264_fmt, &FRAMES[0], 1).len(), 30 + 4 * 2);
    }

    #[test]
    fn uncompressed_and_h264_format_guids() {
        let d = format_descriptor(&FORMATS[0], 1);
        assert_eq!(d[0], VS_FORMAT_UNCOMPRESSED);
        assert_eq!(&d[3..7], b"YUY2");
        assert_eq!(&d[7..19], &[0, 0, 0x10, 0, 0x80, 0, 0, 0xAA, 0, 0x38, 0x9B, 0x71]);
        assert_eq!(d[19], 16);

        let h264 = Format::new(PixelFormat::H264, &FRAMES);
        let hd = format_descriptor(&h264, 1);
        assert_eq!(hd[0], VS_FORMAT_FRAME_BASED);
        assert_eq!(&hd[3..7], b"H264");
        assert_eq!(hd[25], 1);
    }

    #[test]
    fn frame_descriptor_fields() {
        let d = frame_descriptor(&FORMATS[0], &FRAMES[0], 1);
        assert_eq!(d[0], VS_FRAME_UNCOMPRESSED);
        assert_eq!(u16::from_le_bytes([d[3], d[4]]), 160);
        assert_eq!(u16::from_le_bytes([d[5], d[6]]), 120);
        let max_size = u32::from_le_bytes([d[15], d[16], d[17], d[18]]);
        assert_eq!(max_size, 160 * 120 * 2);
        assert_eq!(d[23], 2);
        assert_eq!(u32::from_le_bytes([d[24], d[25], d[26], d[27]]), 333_333);
        let max_bit_rate = u32::from_le_bytes([d[11], d[12], d[13], d[14]]);
        assert_eq!(max_bit_rate as u64, 160 * 120 * 2 * 8 * 10_000_000 / 333_333);
    }

    #[test]
    fn stream_control_round_trips() {
        let mut control = CATALOG.control_for(2, 1, 666_666);
        control.usage = 0x01;
        control.bit_depth_luma = 8;
        control.rate_control_modes = 0x0002;
        control.layout_per_stream = 0x1234_5678;

        let bytes = control.to_bytes();
        assert_eq!(bytes.len(), STREAM_CONTROL_LEN);
        assert_eq!(StreamControl::parse(&bytes), Some(control));

        // Still tolerates 34-byte / 26-byte proposals from legacy hosts.
        let v11 = StreamControl::parse(&bytes[..STREAM_CONTROL_LEN_V11]).unwrap();
        assert_eq!(v11.clock_frequency, CLOCK_FREQUENCY_HZ);
        assert_eq!(v11.usage, 0);
        let v10 = StreamControl::parse(&bytes[..STREAM_CONTROL_LEN_V10]).unwrap();
        assert_eq!(v10.max_payload_transfer_size, 1023);
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
            rate_control_modes: 0x0001,
            ..StreamControl::default()
        };
        let n = CATALOG.negotiate(&proposal);
        assert_eq!((n.format_index, n.frame_index, n.frame_interval), (2, 2, 666_666));
        assert_eq!(n.max_video_frame_size, 320 * 240 * 2);
        assert_eq!(n.rate_control_modes, 0x0001);
    }

    fn descriptors(buf: &[u8]) -> std::vec::Vec<&[u8]> {
        let mut out = std::vec::Vec::new();
        let mut rest = buf;
        while !rest.is_empty() && rest[0] != 0 {
            let (d, tail) = rest.split_at(rest[0] as usize);
            out.push(d);
            rest = tail;
        }
        out
    }

    #[test]
    fn config_descriptor_assembly_uvc15_with_controls_and_still() {
        let driver = TestDriver::default().reserve(0x81).reserve(0x82);
        let mut config_buf = [0u8; 512];
        let mut control_buf = [0u8; 64];
        let mut state = State::new();
        const STILL_SIZES: [StillFrame; 1] = [StillFrame::new(640, 480)];
        const STILL_COMP: [u8; 2] = [1, 4];

        let cfg = Config::new(&FORMATS, UsbDeviceSpeed::Full)
            .with_processing_controls(ProcessingControls {
                brightness: Some(ControlRange::new(-64, 64, 1, 0)),
                ..ProcessingControls::none()
            })
            .with_camera_controls(CameraControls {
                exposure_time_abs: Some(ControlRange::new(10, 1000, 1, 100)),
                ..CameraControls::none()
            })
            .with_still(StillConfig::new(&STILL_SIZES, &STILL_COMP));

        {
            let mut builder = Builder::new(
                driver,
                crate::Config::new(0x1234, 0x5678),
                &mut config_buf,
                &mut [],
                &mut [],
                &mut control_buf,
            );
            let _camera = Camera::new(&mut builder, &mut state, cfg);
            let _ = builder.build();
        }

        let all = descriptors(&config_buf);
        let total: usize = all.iter().map(|d| d.len()).sum();
        assert_eq!(u16::from_le_bytes([all[0][2], all[0][3]]) as usize, total);

        assert_eq!(all[1][1], 0x0B);
        assert_eq!((all[1][4], all[1][5], all[1][6]), (0x0E, 0x03, 0x01));

        let vc: std::vec::Vec<_> = all[3..].iter().take_while(|d| d[1] == CS_INTERFACE).collect();
        assert_eq!(vc.len(), 4);
        assert_eq!(u16::from_le_bytes([vc[0][3], vc[0][4]]), BCD_UVC_15);
        assert_eq!(vc[2][2], VC_PROCESSING_UNIT);
        assert_eq!(vc[3][2], VC_OUTPUT_TERMINAL);
        assert_eq!(vc[3][7], PROCESSING_UNIT_ID);

        let vs0 = &all[3 + vc.len()..];
        let cs: std::vec::Vec<_> = vs0[1..].iter().take_while(|d| d[1] == CS_INTERFACE).collect();
        assert_eq!(cs.len(), 1 + 2 * 3 + 1);
        assert_eq!(cs[0][9], 2);
        assert_eq!(cs.last().unwrap()[2], VS_STILL_IMAGE_FRAME);
    }

    #[test]
    fn write_frame_streaming_timestamps_still_and_abort() {
        let driver = TestDriver::default().reserve(0x81).reserve(0x82);
        let mock = driver.io();
        let mut config_buf = [0u8; 512];
        let mut control_buf = [0u8; 64];
        let mut state = State::new();
        let mut builder = Builder::new(
            driver,
            crate::Config::new(0x1234, 0x5678),
            &mut config_buf,
            &mut [],
            &mut [],
            &mut control_buf,
        );
        let mut config = Config::new(&FORMATS, UsbDeviceSpeed::Full);
        config.max_packet_size = 16;
        let mut camera = Camera::new(&mut builder, &mut state, config);
        // Streaming alternate setting selected.
        mock.set_enabled(0x83, true);

        let frame = [1, 2, 3, 4, 5];
        embassy_futures::block_on(camera.write_frame(&frame)).unwrap();
        embassy_futures::block_on(camera.write_frame_with_timestamp(
            &frame,
            PayloadTimestamp {
                pts: 0x1122_3344,
                scr_stc: Some((0xAABB_CCDD, 0x0102)),
            },
        ))
        .unwrap();
        embassy_futures::block_on(camera.write_still_frame(&[9, 8])).unwrap();
        embassy_futures::block_on(async {
            let mut w = camera.start_frame();
            w.write(&[0xAA, 0xBB]).await.unwrap();
            w.abort().await.unwrap();
        });

        let log = mock.writes(0x83);
        assert_eq!(log[0].0, &[2, 0x83, 1, 2, 3, 4, 5]);
        assert_eq!(
            log[1].0,
            &[
                12, 0x8C, 0x44, 0x33, 0x22, 0x11, 0xDD, 0xCC, 0xBB, 0xAA, 0x02, 0x01, 1, 2, 3, 4
            ]
        );
        assert_eq!(
            log[2].0,
            &[12, 0x8E, 0x44, 0x33, 0x22, 0x11, 0xDD, 0xCC, 0xBB, 0xAA, 0x02, 0x01, 5]
        );
        assert_eq!(log[3].0, &[2, 0xA3, 9, 8]);
        assert_eq!(log[4].0, &[2, 0xC2, 0xAA, 0xBB]);
    }

    #[test]
    fn host_negotiation_and_vc_controls_and_still() {
        const STILL_SIZES: [StillFrame; 1] = [StillFrame::new(640, 480)];
        const STILL_COMP: [u8; 2] = [1, 4];
        let catalog = Catalog {
            formats: &FORMATS,
            max_packet_size: 1023,
            transport: StreamTransport::Isochronous,
            processing_controls: ProcessingControls {
                brightness: Some(ControlRange::new(-64, 64, 1, 10)),
                ..ProcessingControls::none()
            },
            camera_controls: CameraControls {
                exposure_time_abs: Some(ControlRange::new(10, 1000, 1, 100)),
                ..CameraControls::none()
            },
            still: Some(StillConfig::new(&STILL_SIZES, &STILL_COMP)),
        };
        let shared = Shared {
            catalog,
            inner: CriticalSectionMutex::new(Cell::new(Inner {
                probe: catalog.default_control(),
                commit: catalog.default_control(),
                still_probe: catalog.default_still_control(),
                still_commit: catalog.default_still_control(),
                still_trigger_pending: false,
                streaming: false,
                controls: ControlValues::from_config(&catalog.processing_controls, &catalog.camera_controls),
                controls_changed: false,
                vc_error_code: ERR_NO_ERROR,
            })),
            changed: Signal::new(),
        };
        let mut control = Control {
            shared: &shared,
            video_control: InterfaceNumber(0),
            video_streaming: InterfaceNumber(1),
        };
        use crate::driver::Direction;
        let vs_req = |direction, request, selector: u8, length| Request {
            direction,
            request_type: RequestType::Class,
            recipient: Recipient::Interface,
            request,
            value: (selector as u16) << 8,
            index: 1,
            length,
        };
        let vc_req = |direction, request, entity: u8, selector: u8, length| Request {
            direction,
            request_type: RequestType::Class,
            recipient: Recipient::Interface,
            request,
            value: (selector as u16) << 8,
            index: ((entity as u16) << 8) | 0,
            length,
        };

        let mut buf = [0u8; 64];
        let Some(InResponse::Accepted(len)) =
            control.control_in(vs_req(Direction::In, GET_LEN, VS_PROBE_CONTROL, 2), &mut buf)
        else {
            panic!("GET_LEN rejected");
        };
        assert_eq!(len, &[48, 0]);

        let set_br = vc_req(Direction::Out, SET_CUR, PROCESSING_UNIT_ID, PU_BRIGHTNESS_CONTROL, 2);
        assert!(matches!(
            control.control_out(set_br, &(-12i16).to_le_bytes()),
            Some(OutResponse::Accepted)
        ));
        assert_eq!(shared.get().controls.brightness, -12);

        let Some(InResponse::Accepted(br)) = control.control_in(
            vc_req(Direction::In, GET_CUR, PROCESSING_UNIT_ID, PU_BRIGHTNESS_CONTROL, 2),
            &mut buf,
        ) else {
            panic!("brightness GET_CUR rejected");
        };
        assert_eq!(br, &(-12i16).to_le_bytes());

        assert!(matches!(
            control.control_out(set_br, &(200i16).to_le_bytes()),
            Some(OutResponse::Rejected)
        ));
        let Some(InResponse::Accepted(err)) = control.control_in(
            vc_req(Direction::In, GET_CUR, 0, VC_REQUEST_ERROR_CODE_CONTROL, 1),
            &mut buf,
        ) else {
            panic!("error code rejected");
        };
        assert_eq!(err, &[ERR_OUT_OF_RANGE]);

        let set_exp = vc_req(
            Direction::Out,
            SET_CUR,
            INPUT_TERMINAL_ID,
            CT_EXPOSURE_TIME_ABSOLUTE_CONTROL,
            4,
        );
        assert!(matches!(
            control.control_out(set_exp, &250u32.to_le_bytes()),
            Some(OutResponse::Accepted)
        ));
        assert_eq!(shared.get().controls.exposure_time_abs, 250);

        let still_ctl = StillControl {
            format_index: 2,
            frame_index: 1,
            compression_index: 2,
            max_video_frame_size: 0,
            max_payload_transfer_size: 0,
        };
        assert!(matches!(
            control.control_out(
                vs_req(Direction::Out, SET_CUR, VS_STILL_COMMIT_CONTROL, 11),
                &still_ctl.to_bytes()
            ),
            Some(OutResponse::Accepted)
        ));
        assert!(matches!(
            control.control_out(vs_req(Direction::Out, SET_CUR, VS_STILL_IMAGE_TRIGGER_CONTROL, 1), &[1]),
            Some(OutResponse::Accepted)
        ));
        assert!(shared.get().still_trigger_pending);
    }

    #[test]
    fn probe_range() {
        assert_eq!(CATALOG.default_control().format_index, 1);
        let max = CATALOG.max_control();
        assert_eq!((max.format_index, max.frame_index, max.frame_interval), (2, 2, 333_333));
    }
}
