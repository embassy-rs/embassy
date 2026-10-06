//! GUD (Generic USB Display) class implementation.
//!
//! Class driver for the Linux GUD driver.
//! See <https://github.com/torvalds/linux/tree/master/drivers/gpu/drm/gud>.
//!
//! Pixel transfer can be done in a streaming manner: [`GudClass::read_packet`]
//! returns one bulk packet and [`GudClass::read_buffer`] streams a whole update
//! through a small caller-provided chunk buffer, so total framebuffer size can
//! exceed microcontroller RAM.

use core::cell::RefCell;
use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicU8, Ordering};

use embassy_sync::blocking_mutex::CriticalSectionMutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use heapless::Vec;

use crate::control::{self, InResponse, OutResponse, Recipient, Request, RequestType};
use crate::driver::{Driver, Endpoint, EndpointError, EndpointOut};
use crate::types::InterfaceNumber;
use crate::{Builder, Handler};

/// Vendor ID to use for the device so that the Linux GUD driver binds to it.
pub const GUD_VENDOR_ID: u16 = 0x1209;
/// Product ID paired with [`GUD_VENDOR_ID`]. See `pid.codes` "GUD".
pub const GUD_PRODUCT_ID: u16 = 0x4FB3;

/// GUD protocol version reported in the display descriptor.
pub const GUD_PROTOCOL_VERSION: u8 = 1;

/// Result of the last GUD request: ok.
pub const GUD_STATUS_OK: u8 = 0x00;
/// Result of the last GUD request: device busy, host should retry.
pub const GUD_STATUS_BUSY: u8 = 0x01;
/// Result of the last GUD request: the request is not supported.
pub const GUD_STATUS_REQUEST_NOT_SUPPORTED: u8 = 0x02;
/// Result of the last GUD request: malformed request.
pub const GUD_STATUS_PROTOCOL_ERROR: u8 = 0x03;
/// Result of the last GUD request: a parameter was invalid.
pub const GUD_STATUS_INVALID_PARAMETER: u8 = 0x04;
/// Result of the last GUD request: generic error.
pub const GUD_STATUS_ERROR: u8 = 0x05;

/// Get status from the last GUD control request. Value is u8.
pub const GUD_REQ_GET_STATUS: u8 = 0x00;
/// Get display descriptor (30 bytes).
pub const GUD_REQ_GET_DESCRIPTOR: u8 = 0x01;
/// Get supported pixel formats as a byte array of `GUD_PIXEL_FORMAT_*`.
pub const GUD_REQ_GET_FORMATS: u8 = 0x40;
/// Get non-connector properties (currently only [`PROPERTY_ROTATION`]).
pub const GUD_REQ_GET_PROPERTIES: u8 = 0x41;
/// Get connector descriptors (5 bytes each); wValue is the connector index.
pub const GUD_REQ_GET_CONNECTORS: u8 = 0x50;
/// Get connector properties (10 bytes each); wValue is the connector index.
pub const GUD_REQ_GET_CONNECTOR_PROPERTIES: u8 = 0x51;
/// Get TV mode names (16 bytes each); wValue is the connector index.
pub const GUD_REQ_GET_CONNECTOR_TV_MODE_VALUES: u8 = 0x52;
/// Issued when userspace checks connector status; wValue is the connector index.
pub const GUD_REQ_SET_CONNECTOR_FORCE_DETECT: u8 = 0x53;
/// Get connector status (u8); wValue is the connector index.
pub const GUD_REQ_GET_CONNECTOR_STATUS: u8 = 0x54;
/// Get display modes (24 bytes each); wValue is the connector index.
pub const GUD_REQ_GET_CONNECTOR_MODES: u8 = 0x55;
/// Get EDID data; wValue is the connector index.
pub const GUD_REQ_GET_CONNECTOR_EDID: u8 = 0x56;
/// Set buffer transfer info (25 bytes) right before a bulk OUT transfer.
pub const GUD_REQ_SET_BUFFER: u8 = 0x60;
/// Check display configuration (`mode` + format + connector + properties).
pub const GUD_REQ_SET_STATE_CHECK: u8 = 0x61;
/// Apply the previously checked display configuration.
pub const GUD_REQ_SET_STATE_COMMIT: u8 = 0x62;
/// Enable/disable the display controller. Value is u8: 0/1.
pub const GUD_REQ_SET_CONTROLLER_ENABLE: u8 = 0x63;
/// Enable/disable display/output (DPMS). Value is u8: 0/1.
pub const GUD_REQ_SET_DISPLAY_ENABLE: u8 = 0x64;

/// Maximum number of pixel formats a device may advertise.
pub const GUD_FORMATS_MAX_NUM: usize = 32;
/// Maximum number of connectors a device may advertise.
pub const GUD_CONNECTORS_MAX_NUM: usize = 32;
/// Maximum number of properties stored from a `SET_STATE_CHECK` request.
pub const MAX_PROPERTIES: usize = 16;
/// Maximum number of connector properties a device may advertise.
pub const GUD_CONNECTOR_PROPERTIES_MAX_NUM: usize = 32;

const MAX_EVENTS: usize = 8;

/// Magic value in the display descriptor (`gud.h` `GUD_DISPLAY_MAGIC`).
pub const GUD_DISPLAY_MAGIC: u32 = 0x1d50614d;

/// LZ4 compression bit. This driver advertises no compression.
pub const GUD_COMPRESSION_LZ4: u8 = 0x01;

// Display mode flags (same numbering as `gud.h` `GUD_DISPLAY_MODE_FLAG_*`).
/// Mode flag: positive hsync polarity.
pub const DISPLAY_MODE_FLAG_PHSYNC: u32 = 1 << 0;
/// Mode flag: negative hsync polarity.
pub const DISPLAY_MODE_FLAG_NHSYNC: u32 = 1 << 1;
/// Mode flag: positive vsync polarity.
pub const DISPLAY_MODE_FLAG_PVSYNC: u32 = 1 << 2;
/// Mode flag: negative vsync polarity.
pub const DISPLAY_MODE_FLAG_NVSYNC: u32 = 1 << 3;
/// Mode flag: interlaced mode.
pub const DISPLAY_MODE_FLAG_INTERLACE: u32 = 1 << 4;
/// Mode flag: doublescan mode.
pub const DISPLAY_MODE_FLAG_DBLSCAN: u32 = 1 << 5;
/// Mode flag: composite sync.
pub const DISPLAY_MODE_FLAG_CSYNC: u32 = 1 << 6;
/// Mode flag: positive composite sync polarity.
pub const DISPLAY_MODE_FLAG_PCSYNC: u32 = 1 << 7;
/// Mode flag: negative composite sync polarity.
pub const DISPLAY_MODE_FLAG_NCSYNC: u32 = 1 << 8;
/// Mode flag: hsync skew.
pub const DISPLAY_MODE_FLAG_HSKEW: u32 = 1 << 9;
/// Mode flag: internal protocol flag marking the preferred mode.
pub const DISPLAY_MODE_FLAG_PREFERRED: u32 = 1 << 10;
/// Mode flag: double clock mode.
pub const DISPLAY_MODE_FLAG_DBLCLK: u32 = 1 << 12;
/// Mode flag: clock divided by 2.
pub const DISPLAY_MODE_FLAG_CLKDIV2: u32 = 1 << 13;

/// Mask of mode flag bits defined for user configuration.
pub const DISPLAY_MODE_FLAG_USER_MASK: u32 = DISPLAY_MODE_FLAG_PHSYNC
    | DISPLAY_MODE_FLAG_NHSYNC
    | DISPLAY_MODE_FLAG_PVSYNC
    | DISPLAY_MODE_FLAG_NVSYNC
    | DISPLAY_MODE_FLAG_INTERLACE
    | DISPLAY_MODE_FLAG_DBLSCAN
    | DISPLAY_MODE_FLAG_CSYNC
    | DISPLAY_MODE_FLAG_PCSYNC
    | DISPLAY_MODE_FLAG_NCSYNC
    | DISPLAY_MODE_FLAG_HSKEW
    | DISPLAY_MODE_FLAG_DBLCLK
    | DISPLAY_MODE_FLAG_CLKDIV2;

// Connector types (`gud.h` `GUD_CONNECTOR_TYPE_*`).
/// Connector type: laptop panel.
pub const CONNECTOR_TYPE_PANEL: u8 = 0;
/// Connector type: VGA.
pub const CONNECTOR_TYPE_VGA: u8 = 1;
/// Connector type: composite TV out.
pub const CONNECTOR_TYPE_COMPOSITE: u8 = 2;
/// Connector type: S-Video.
pub const CONNECTOR_TYPE_SVIDEO: u8 = 3;
/// Connector type: component.
pub const CONNECTOR_TYPE_COMPONENT: u8 = 4;
/// Connector type: DVI.
pub const CONNECTOR_TYPE_DVI: u8 = 5;
/// Connector type: DisplayPort.
pub const CONNECTOR_TYPE_DISPLAYPORT: u8 = 6;
/// Connector type: HDMI.
pub const CONNECTOR_TYPE_HDMI: u8 = 7;

/// Connector flag: status can change (host polls it every 10 s).
pub const CONNECTOR_FLAGS_POLL_STATUS: u32 = 1 << 0;
/// Connector flag: interlaced modes are supported.
pub const CONNECTOR_FLAGS_INTERLACE: u32 = 1 << 1;
/// Connector flag: doublescan modes are supported.
pub const CONNECTOR_FLAGS_DOUBLESCAN: u32 = 1 << 2;

/// Connector status: no display attached.
pub const CONNECTOR_STATUS_DISCONNECTED: u8 = 0x00;
/// Connector status: display attached.
pub const CONNECTOR_STATUS_CONNECTED: u8 = 0x01;
/// Connector status: attachment unknown.
pub const CONNECTOR_STATUS_UNKNOWN: u8 = 0x02;
/// Mask of the status bits reported in the status byte.
pub const CONNECTOR_STATUS_CONNECTED_MASK: u8 = 0x03;
/// Status bit set when the status changed since the last read. Internal.
pub const CONNECTOR_STATUS_CHANGED: u8 = 1 << 7;

// Connector property ids (`gud.h` `GUD_PROPERTY_*`).
/// Property: left margin in pixels for overscan, range 0-100.
pub const PROPERTY_TV_LEFT_MARGIN: u16 = 1;
/// Property: right margin in pixels for overscan, range 0-100.
pub const PROPERTY_TV_RIGHT_MARGIN: u16 = 2;
/// Property: top margin in pixels for overscan, range 0-100.
pub const PROPERTY_TV_TOP_MARGIN: u16 = 3;
/// Property: bottom margin in pixels for overscan, range 0-100.
pub const PROPERTY_TV_BOTTOM_MARGIN: u16 = 4;
/// Property: TV mode selector (index into `tv_mode_values`).
pub const PROPERTY_TV_MODE: u16 = 5;
/// Property: brightness percent, range 0-100.
pub const PROPERTY_TV_BRIGHTNESS: u16 = 6;
/// Property: contrast percent, range 0-100.
pub const PROPERTY_TV_CONTRAST: u16 = 7;
/// Property: flicker reduction percent, range 0-100.
pub const PROPERTY_TV_FLICKER_REDUCTION: u16 = 8;
/// Property: overscan percent, range 0-100.
pub const PROPERTY_TV_OVERSCAN: u16 = 9;
/// Property: saturation percent, range 0-100.
pub const PROPERTY_TV_SATURATION: u16 = 10;
/// Property: hue percent, range 0-100.
pub const PROPERTY_TV_HUE: u16 = 11;
/// Property: backlight brightness percent, range 0-100.
pub const PROPERTY_BACKLIGHT_BRIGHTNESS: u16 = 12;

/// Non-connector property id: plane rotation (`ROTATION_*` bitmask).
pub const PROPERTY_ROTATION: u16 = 50;

/// Rotation property bit: 0 degrees.
pub const ROTATION_0: u8 = 1 << 0;
/// Rotation property bit: 90 degrees.
pub const ROTATION_90: u8 = 1 << 1;
/// Rotation property bit: 180 degrees.
pub const ROTATION_180: u8 = 1 << 2;
/// Rotation property bit: 270 degrees.
pub const ROTATION_270: u8 = 1 << 3;
/// Rotation property bit: horizontal reflection.
pub const ROTATION_REFLECT_X: u8 = 1 << 4;
/// Rotation property bit: vertical reflection.
pub const ROTATION_REFLECT_Y: u8 = 1 << 5;
/// Mask of all rotation property bits.
pub const ROTATION_MASK: u8 =
    ROTATION_0 | ROTATION_90 | ROTATION_180 | ROTATION_270 | ROTATION_REFLECT_X | ROTATION_REFLECT_Y;

/// Length of one TV mode name entry.
pub const CONNECTOR_TV_MODE_NAME_LEN: usize = 16;

// Property ids constrained to the range 0-100 in `SET_STATE_CHECK`.
// `PROPERTY_TV_MODE` (5) is deliberately excluded.
const RANGE_PROPERTIES: [u16; 11] = [
    PROPERTY_TV_LEFT_MARGIN,
    PROPERTY_TV_RIGHT_MARGIN,
    PROPERTY_TV_TOP_MARGIN,
    PROPERTY_TV_BOTTOM_MARGIN,
    PROPERTY_TV_BRIGHTNESS,
    PROPERTY_TV_CONTRAST,
    PROPERTY_TV_FLICKER_REDUCTION,
    PROPERTY_TV_OVERSCAN,
    PROPERTY_TV_SATURATION,
    PROPERTY_TV_HUE,
    PROPERTY_BACKLIGHT_BRIGHTNESS,
];

const USB_CLASS_VENDOR_SPEC: u8 = 0xFF;

/// A display mode (24 bytes on the wire, `gud_display_mode_req`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct DisplayMode {
    /// Pixel clock in kHz.
    pub clock: u32,
    /// Horizontal display size.
    pub hdisplay: u16,
    /// Horizontal sync start.
    pub hsync_start: u16,
    /// Horizontal sync end.
    pub hsync_end: u16,
    /// Horizontal total size.
    pub htotal: u16,
    /// Vertical display size.
    pub vdisplay: u16,
    /// Vertical sync start.
    pub vsync_start: u16,
    /// Vertical sync end.
    pub vsync_end: u16,
    /// Vertical total size.
    pub vtotal: u16,
    /// `DISPLAY_MODE_FLAG_*` bits (user-mask bits only).
    pub flags: u32,
    /// Whether this is the preferred mode. Serialized as `DISPLAY_MODE_FLAG_PREFERRED`.
    pub preferred: bool,
}

impl DisplayMode {
    fn serialize_into(&self, buf: &mut [u8; 24]) {
        buf[0..4].copy_from_slice(&self.clock.to_le_bytes());
        buf[4..6].copy_from_slice(&self.hdisplay.to_le_bytes());
        buf[6..8].copy_from_slice(&self.hsync_start.to_le_bytes());
        buf[8..10].copy_from_slice(&self.hsync_end.to_le_bytes());
        buf[10..12].copy_from_slice(&self.htotal.to_le_bytes());
        buf[12..14].copy_from_slice(&self.vdisplay.to_le_bytes());
        buf[14..16].copy_from_slice(&self.vsync_start.to_le_bytes());
        buf[16..18].copy_from_slice(&self.vsync_end.to_le_bytes());
        buf[18..20].copy_from_slice(&self.vtotal.to_le_bytes());
        let flags =
            (self.flags & DISPLAY_MODE_FLAG_USER_MASK) | if self.preferred { DISPLAY_MODE_FLAG_PREFERRED } else { 0 };
        buf[20..24].copy_from_slice(&flags.to_le_bytes());
    }

    fn deserialize_from(data: &[u8; 24]) -> Self {
        let flags = u32::from_le_bytes(data[20..24].try_into().unwrap());
        Self {
            clock: u32::from_le_bytes(data[0..4].try_into().unwrap()),
            hdisplay: u16::from_le_bytes(data[4..6].try_into().unwrap()),
            hsync_start: u16::from_le_bytes(data[6..8].try_into().unwrap()),
            hsync_end: u16::from_le_bytes(data[8..10].try_into().unwrap()),
            htotal: u16::from_le_bytes(data[10..12].try_into().unwrap()),
            vdisplay: u16::from_le_bytes(data[12..14].try_into().unwrap()),
            vsync_start: u16::from_le_bytes(data[14..16].try_into().unwrap()),
            vsync_end: u16::from_le_bytes(data[16..18].try_into().unwrap()),
            vtotal: u16::from_le_bytes(data[18..20].try_into().unwrap()),
            flags: flags & DISPLAY_MODE_FLAG_USER_MASK,
            preferred: flags & DISPLAY_MODE_FLAG_PREFERRED != 0,
        }
    }
}

/// A property value (10 bytes on the wire, `gud_property_req`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Property {
    /// Property id (`PROPERTY_*`).
    pub prop: u16,
    /// Value. Its meaning depends on the property; `val` in advertised property
    /// lists is the initial value reported to the host.
    pub val: u64,
}

impl Property {
    fn serialize_into(&self, buf: &mut [u8; 10]) {
        buf[0..2].copy_from_slice(&self.prop.to_le_bytes());
        buf[2..10].copy_from_slice(&self.val.to_le_bytes());
    }

    fn deserialize_from(data: &[u8; 10]) -> Self {
        Self {
            prop: u16::from_le_bytes(data[0..2].try_into().unwrap()),
            val: u64::from_le_bytes(data[2..10].try_into().unwrap()),
        }
    }
}

/// A pixel format (`gud.h` `GUD_PIXEL_FORMAT_*`).
#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum PixelFormat {
    /// 1-bit monochrome.
    R1,
    /// 8-bit greyscale.
    R8,
    /// 4-bit XRGB.
    Xrgb1111,
    /// 8-bit RGB (3-3-2).
    Rgb332,
    /// 16-bit RGB (5-6-5).
    Rgb565,
    /// 24-bit RGB (8-8-8).
    Rgb888,
    /// 32-bit XRGB (8-8-8-8).
    Xrgb8888,
    /// 32-bit ARGB (8-8-8-8).
    Argb8888,
}

impl PixelFormat {
    /// Convert a `GUD_PIXEL_FORMAT_*` byte to a [`PixelFormat`].
    pub fn from_u8(v: u8) -> Option<Self> {
        Some(match v {
            0x01 => Self::R1,
            0x08 => Self::R8,
            0x20 => Self::Xrgb1111,
            0x30 => Self::Rgb332,
            0x40 => Self::Rgb565,
            0x50 => Self::Rgb888,
            0x80 => Self::Xrgb8888,
            0x81 => Self::Argb8888,
            _ => return None,
        })
    }

    /// Convert to the `GUD_PIXEL_FORMAT_*` byte.
    pub fn to_u8(self) -> u8 {
        match self {
            Self::R1 => 0x01,
            Self::R8 => 0x08,
            Self::Xrgb1111 => 0x20,
            Self::Rgb332 => 0x30,
            Self::Rgb565 => 0x40,
            Self::Rgb888 => 0x50,
            Self::Xrgb8888 => 0x80,
            Self::Argb8888 => 0x81,
        }
    }

    /// Bytes per scanline for a framebuffer `width` pixels wide.
    pub fn pitch(self, width: u32) -> u32 {
        match self {
            Self::R1 => width.div_ceil(8),
            Self::Xrgb1111 => width.div_ceil(2),
            Self::R8 | Self::Rgb332 => width,
            Self::Rgb565 => 2 * width,
            Self::Rgb888 => 3 * width,
            Self::Xrgb8888 | Self::Argb8888 => 4 * width,
        }
    }
}

/// Failure receiving a buffer or consuming its pixel data.
///
/// The variants preserve the failure source even when the callback error type
/// is also [`EndpointError`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ReadBufferError<E> {
    /// The USB bulk endpoint failed.
    Endpoint(EndpointError),
    /// The pixel callback failed.
    Callback(E),
}

impl<E: core::fmt::Display> core::fmt::Display for ReadBufferError<E> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Endpoint(error) => write!(f, "USB endpoint: {}", error),
            Self::Callback(error) => write!(f, "pixel callback: {}", error),
        }
    }
}

impl<E: core::error::Error + 'static> core::error::Error for ReadBufferError<E> {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        Some(match self {
            Self::Endpoint(error) => error,
            Self::Callback(error) => error,
        })
    }
}

/// Pixel rectangle announced by `SET_BUFFER`; the target of a streaming read.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct BufferInfo {
    /// X position of the rectangle inside the framebuffer.
    pub x: u32,
    /// Y position of the rectangle.
    pub y: u32,
    /// Pixel width of the rectangle.
    pub width: u32,
    /// Pixel height of the rectangle.
    pub height: u32,
    /// Uncompressed payload size in bytes.
    pub length: u32,
    /// `GUD_COMPRESSION_*`. This driver advertises no compression so it is 0.
    pub compression: u8,
    /// Bytes actually sent on the bulk endpoint when `compression != 0`.
    pub compressed_length: u32,
}

impl BufferInfo {
    /// Number of bytes the host will send on the bulk OUT endpoint.
    pub fn transfer_size(&self) -> u32 {
        if self.compression != 0 {
            self.compressed_length
        } else {
            self.length
        }
    }

    fn deserialize_from(data: &[u8; 25]) -> Self {
        Self {
            x: u32::from_le_bytes(data[0..4].try_into().unwrap()),
            y: u32::from_le_bytes(data[4..8].try_into().unwrap()),
            width: u32::from_le_bytes(data[8..12].try_into().unwrap()),
            height: u32::from_le_bytes(data[12..16].try_into().unwrap()),
            length: u32::from_le_bytes(data[16..20].try_into().unwrap()),
            compression: data[20],
            compressed_length: u32::from_le_bytes(data[21..25].try_into().unwrap()),
        }
    }
}

/// Display state accepted by a `SET_STATE_CHECK` request.
///
/// Read the latest committed configuration with [`GudClass::committed_state`]
/// when a [`GudEvent::StateCommit`] event arrives.
#[derive(Clone, PartialEq, Debug)]
pub struct DisplayState {
    /// Display mode chosen by the host.
    pub mode: DisplayMode,
    /// Pixel format of upcoming framebuffer updates.
    pub format: PixelFormat,
    /// Connector the mode applies to (index into [`Config::connectors`]).
    pub connector: u8,
    /// Property values supplied by the host.
    pub properties: Vec<Property, MAX_PROPERTIES>,
}

#[cfg(feature = "defmt")]
impl defmt::Format for DisplayState {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "DisplayState {{ mode: {:?}, format: {:?}, connector: {}, properties: {:?} }}",
            self.mode,
            self.format,
            self.connector,
            self.properties.as_slice(),
        )
    }
}

/// Error publishing a connector's current monitor data.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ConnectorUpdateError {
    /// Status is not disconnected, connected, or unknown.
    InvalidStatus,
    /// EDID length is not a multiple of 128 bytes.
    InvalidEdid,
    /// The mode list exceeds the caller-provided storage.
    TooManyModes,
    /// EDID exceeds the caller-provided storage.
    EdidTooLong,
}

impl core::fmt::Display for ConnectorUpdateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::InvalidStatus => "invalid connector status",
            Self::InvalidEdid => "EDID length must be a multiple of 128 bytes",
            Self::TooManyModes => "connector mode storage is too small",
            Self::EdidTooLong => "connector EDID storage is too small",
        })
    }
}

impl core::error::Error for ConnectorUpdateError {}

/// Current monitor data for one physical connector.
///
/// Keep a shared reference in the hotplug/DDC task and publish completed reads
/// with [`Self::update`], independently of the task receiving USB pixel data.
/// USB requests read the cached data; no DDC I/O runs inside a critical section.
/// Set [`CONNECTOR_FLAGS_POLL_STATUS`] on hotpluggable connectors so the host
/// polls for changes. USB reset and deconfiguration preserve this snapshot.
#[derive(Debug)]
pub struct ConnectorState<'d> {
    inner: CriticalSectionMutex<RefCell<ConnectorData<'d>>>,
}

#[derive(Debug)]
struct ConnectorData<'d> {
    modes: &'d mut [DisplayMode],
    modes_len: usize,
    edid: &'d mut [u8],
    edid_len: usize,
    status: u8,
}

impl<'d> ConnectorState<'d> {
    /// Create a disconnected connector with no advertised modes or EDID.
    pub const fn new(modes: &'d mut [DisplayMode], edid: &'d mut [u8]) -> Self {
        Self {
            inner: CriticalSectionMutex::new(RefCell::new(ConnectorData {
                modes,
                modes_len: 0,
                edid,
                edid_len: 0,
                status: CONNECTOR_STATUS_DISCONNECTED | CONNECTOR_STATUS_CHANGED,
            })),
        }
    }

    /// Atomically replace the connector's status, modes, and EDID.
    pub fn update(&self, status: u8, modes: &[DisplayMode], edid: Option<&[u8]>) -> Result<(), ConnectorUpdateError> {
        if !matches!(
            status,
            CONNECTOR_STATUS_DISCONNECTED | CONNECTOR_STATUS_CONNECTED | CONNECTOR_STATUS_UNKNOWN
        ) {
            return Err(ConnectorUpdateError::InvalidStatus);
        }
        let edid = edid.unwrap_or(&[]);
        if !edid.len().is_multiple_of(128) {
            return Err(ConnectorUpdateError::InvalidEdid);
        }
        self.inner.lock(|data| {
            let mut data = data.borrow_mut();
            if modes.len() > data.modes.len() {
                return Err(ConnectorUpdateError::TooManyModes);
            }
            if edid.len() > data.edid.len() {
                return Err(ConnectorUpdateError::EdidTooLong);
            }
            data.modes[..modes.len()].copy_from_slice(modes);
            data.edid[..edid.len()].copy_from_slice(edid);
            data.modes_len = modes.len();
            data.edid_len = edid.len();
            data.status = status | CONNECTOR_STATUS_CHANGED;
            Ok(())
        })
    }
}

/// Static connector descriptor.
#[derive(Clone, Copy, Debug)]
pub struct GudConnector<'d> {
    /// `CONNECTOR_TYPE_*`. Hosts fall back to PANEL for unsupported types.
    pub connector_type: u8,
    /// `CONNECTOR_FLAGS_*` bits.
    pub flags: u32,
    /// Current connector status, display modes and EDID.
    pub state: &'d ConnectorState<'d>,
    /// Connector properties; `val` holds the initial value reported to the host.
    pub properties: &'d [Property],
    /// NUL-terminated TV mode names (16 bytes each). Only used when a
    /// `PROPERTY_TV_MODE` property is declared.
    pub tv_mode_values: Option<&'d [[u8; CONNECTOR_TV_MODE_NAME_LEN]]>,
}

impl GudConnector<'_> {
    fn serialize_into(&self, buf: &mut [u8; 5]) {
        buf[0] = self.connector_type;
        buf[1..5].copy_from_slice(&self.flags.to_le_bytes());
    }
}

/// Events delivered to the user task in host request order.
#[non_exhaustive]
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum GudEvent {
    /// USB bus reset; drop any in-flight read state.
    Reset,
    /// USB configuration set (`true`) or cleared (`false`).
    Configured(bool),
    /// The rectangle to update. [`BufferInfo::transfer_size`] bytes of pixel data
    /// follow on the bulk OUT endpoint. Use [`GudClass::read_packet`] or
    /// [`GudClass::read_buffer`] to read.
    Buffer(BufferInfo),
    /// A state passed `SET_STATE_CHECK`; inspect [`GudClass::checked_state`].
    StateCheck,
    /// The host committed a state; inspect [`GudClass::committed_state`].
    StateCommit,
    /// Host-side userspace connector status check (connector index).
    ConnectorForceDetect(u8),
    /// Display/output (DPMS) enabled or disabled.
    DisplayEnable(bool),
    /// Display controller enabled or disabled.
    ControllerEnable(bool),
}

/// Static class driver configuration passed to [`GudClass::new`].
///
/// This configuration is fixed when the class is created.
pub struct Config<'d> {
    /// Maximum packet size for the bulk OUT endpoint: 8..=64 for full-speed,
    /// 512 for high-speed.
    pub max_packet_size: u16,
    /// Minimum framebuffer width the device handles.
    pub min_width: u32,
    /// Maximum framebuffer width.
    pub max_width: u32,
    /// Minimum framebuffer height.
    pub min_height: u32,
    /// Maximum framebuffer height.
    pub max_height: u32,
    /// Maximum size in bytes of one damage rectangle. `0` means no cap.
    /// Does not have to fit in memory, it can be streamed chunk by chunk.
    pub max_buffer_size: u32,
    /// Supported pixel formats, 1..=[`GUD_FORMATS_MAX_NUM`].
    pub formats: &'d [PixelFormat],
    /// `ROTATION_*` bitmask; `0` omits the rotation property entirely.
    pub supported_rotations: u8,
    /// Connectors, 1..=[`GUD_CONNECTORS_MAX_NUM`]. Index order is the index the
    /// host uses in `wValue` and `SET_STATE_CHECK`.
    pub connectors: &'d [GudConnector<'d>],
    /// Validate a mode for the given connector index during `SET_STATE_CHECK`.
    pub validate_mode: fn(connector: u8, mode: &DisplayMode) -> bool,
}

impl Config<'_> {
    /// Minimum control buffer length for this configuration.
    ///
    /// Takes the max over all control transfers.
    pub fn control_buf_len(&self) -> usize {
        let mut len = 30.max(self.formats.len()).max(5 * self.connectors.len());
        let rotation_properties = usize::from(self.supported_rotations != 0);
        for conn in self.connectors {
            let (modes_capacity, edid_capacity) = conn.state.inner.lock(|data| {
                let data = data.borrow();
                (data.modes.len(), data.edid.len())
            });
            len = len
                .max(24 * modes_capacity)
                .max(edid_capacity)
                .max(10 * conn.properties.len())
                .max(26 + 10 * (conn.properties.len() + rotation_properties));
            if let Some(values) = conn.tv_mode_values {
                len = len.max(CONNECTOR_TV_MODE_NAME_LEN * values.len());
            }
        }
        len
    }
}

/// Shared state between [`Control`] (control pipe) and [`GudClass`] (user task).
struct Shared {
    events: Channel<CriticalSectionRawMutex, GudEvent, MAX_EVENTS>,
    // Result of the last GUD request, used by `GET_STATUS`.
    last_status: AtomicU8,
    // Last state accepted by `SET_STATE_CHECK`.
    checked_state: CriticalSectionMutex<RefCell<Option<DisplayState>>>,
    // Last state accepted by `SET_STATE_COMMIT`.
    committed_state: CriticalSectionMutex<RefCell<Option<DisplayState>>>,
}

/// Internal state for the GUD class. Pass to [`GudClass::new`].
pub struct State<'d> {
    control: MaybeUninit<Control<'d>>,
    shared: Shared,
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
            shared: Shared {
                events: Channel::new(),
                last_status: AtomicU8::new(GUD_STATUS_OK),
                checked_state: CriticalSectionMutex::new(RefCell::new(None)),
                committed_state: CriticalSectionMutex::new(RefCell::new(None)),
            },
        }
    }
}

/// GUD USB display class.
///
/// One vendor-specific interface with a single bulk OUT endpoint carrying
/// framebuffer updates. Events (state changes, rectangle update announcements) arrive
/// on [`GudClass::next_event`]. Pixel transfer is read with [`GudClass::read_packet`]
/// or streamed through [`GudClass::read_buffer`].
///
/// The Linux driver only binds when the device VID/PID pair is in its id table.
pub struct GudClass<'d, D: Driver<'d>> {
    read_ep: D::EndpointOut,
    shared: &'d Shared,
}

struct Control<'d> {
    iface: InterfaceNumber,
    config: &'d Config<'d>,
    shared: &'d Shared,
}

impl<'d> Control<'d> {
    fn reject_in(&mut self, status: u8) -> InResponse<'static> {
        self.shared.last_status.store(status, Ordering::Relaxed);
        InResponse::Rejected
    }

    fn reject_out(&mut self, status: u8) -> OutResponse {
        self.shared.last_status.store(status, Ordering::Relaxed);
        OutResponse::Rejected
    }

    fn accepted_in<'a>(&mut self, data: &'a [u8]) -> InResponse<'a> {
        self.shared.last_status.store(GUD_STATUS_OK, Ordering::Relaxed);
        InResponse::Accepted(data)
    }

    fn accepted_out(&mut self) -> OutResponse {
        self.shared.last_status.store(GUD_STATUS_OK, Ordering::Relaxed);
        OutResponse::Accepted
    }

    // Connector index from `wValue`, or `None` if out of range.
    fn connector(&self, value: u16) -> Option<&'d GudConnector<'d>> {
        self.config.connectors.get(value as usize)
    }

    // Queue an event if capacity allows, else reject the request as -EBUSY.
    fn try_send_critical(&mut self, event: GudEvent) -> OutResponse {
        if self.shared.events.try_send(event).is_ok() {
            self.accepted_out()
        } else {
            self.reject_out(GUD_STATUS_BUSY)
        }
    }

    // Clear per-request state on reset/deconfigure.
    fn clear(&mut self) {
        self.shared.events.clear();
        self.shared.checked_state.lock(|c| c.borrow_mut().take());
        self.shared.committed_state.lock(|c| c.borrow_mut().take());
        self.shared.last_status.store(GUD_STATUS_OK, Ordering::Relaxed);
        for conn in self.config.connectors {
            conn.state
                .inner
                .lock(|data| data.borrow_mut().status |= CONNECTOR_STATUS_CHANGED);
        }
    }
}

impl<'d> Handler for Control<'d> {
    fn reset(&mut self) {
        self.clear();
        // Never fails: the channel was just cleared.
        let _ = self.shared.events.try_send(GudEvent::Reset);
    }

    fn configured(&mut self, configured: bool) {
        if !configured {
            self.clear();
        }
        // This notification cannot reject the configuration change.
        let _ = self.shared.events.try_send(GudEvent::Configured(configured));
    }

    fn control_in<'a>(&'a mut self, req: Request, buf: &'a mut [u8]) -> Option<InResponse<'a>> {
        if (req.request_type, req.recipient, req.index)
            != (RequestType::Vendor, Recipient::Interface, self.iface.0 as u16)
        {
            return None;
        }

        Some(match req.request {
            GUD_REQ_GET_STATUS => {
                buf[0] = self.shared.last_status.load(Ordering::Relaxed);
                // The host wants the status of the *previous* request.
                InResponse::Accepted(&buf[..1])
            }
            GUD_REQ_GET_DESCRIPTOR => {
                if buf.len() < 30 {
                    return Some(self.reject_in(GUD_STATUS_PROTOCOL_ERROR));
                }
                buf[0..4].copy_from_slice(&GUD_DISPLAY_MAGIC.to_le_bytes());
                buf[4] = GUD_PROTOCOL_VERSION;
                // TODO: STATUS_ON_SET and FULL_UPDATE
                buf[5..9].copy_from_slice(&0u32.to_le_bytes()); // flags: 0
                // TODO: LZ4 compression
                buf[9] = 0; // compression: none
                buf[10..14].copy_from_slice(&self.config.max_buffer_size.to_le_bytes());
                buf[14..18].copy_from_slice(&self.config.min_width.to_le_bytes());
                buf[18..22].copy_from_slice(&self.config.max_width.to_le_bytes());
                buf[22..26].copy_from_slice(&self.config.min_height.to_le_bytes());
                buf[26..30].copy_from_slice(&self.config.max_height.to_le_bytes());
                self.accepted_in(&buf[..30])
            }
            GUD_REQ_GET_FORMATS => {
                let n = self.config.formats.len().min(buf.len());
                for (i, fmt) in self.config.formats.iter().take(n).enumerate() {
                    buf[i] = fmt.to_u8();
                }
                self.accepted_in(&buf[..n])
            }
            GUD_REQ_GET_PROPERTIES => {
                if self.config.supported_rotations != 0 {
                    if buf.len() < 10 {
                        return Some(self.reject_in(GUD_STATUS_PROTOCOL_ERROR));
                    }
                    let prop = Property {
                        prop: PROPERTY_ROTATION,
                        val: self.config.supported_rotations as u64,
                    };
                    prop.serialize_into((&mut buf[..10]).try_into().unwrap());
                    self.accepted_in(&buf[..10])
                } else {
                    self.accepted_in(&buf[..0])
                }
            }
            GUD_REQ_GET_CONNECTORS => {
                if buf.len() < 5 {
                    return Some(self.reject_in(GUD_STATUS_PROTOCOL_ERROR));
                }
                let n = self.config.connectors.len().min(buf.len() / 5);
                for (i, conn) in self.config.connectors.iter().take(n).enumerate() {
                    conn.serialize_into((&mut buf[i * 5..i * 5 + 5]).try_into().unwrap());
                }
                self.accepted_in(&buf[..n * 5])
            }
            GUD_REQ_GET_CONNECTOR_PROPERTIES => {
                let Some(conn) = self.connector(req.value) else {
                    return Some(self.reject_in(GUD_STATUS_INVALID_PARAMETER));
                };
                if buf.len() < 10 {
                    return Some(self.reject_in(GUD_STATUS_PROTOCOL_ERROR));
                }
                let n = conn.properties.len().min(buf.len() / 10);
                for (i, prop) in conn.properties.iter().take(n).enumerate() {
                    prop.serialize_into((&mut buf[i * 10..i * 10 + 10]).try_into().unwrap());
                }
                self.accepted_in(&buf[..n * 10])
            }
            GUD_REQ_GET_CONNECTOR_TV_MODE_VALUES => {
                let Some(conn) = self.connector(req.value) else {
                    return Some(self.reject_in(GUD_STATUS_INVALID_PARAMETER));
                };
                let Some(values) = conn.tv_mode_values else {
                    return Some(self.reject_in(GUD_STATUS_REQUEST_NOT_SUPPORTED));
                };
                if buf.len() < CONNECTOR_TV_MODE_NAME_LEN {
                    return Some(self.reject_in(GUD_STATUS_PROTOCOL_ERROR));
                }
                let n = values.len().min(buf.len() / CONNECTOR_TV_MODE_NAME_LEN);
                for (i, name) in values.iter().take(n).enumerate() {
                    buf[i * CONNECTOR_TV_MODE_NAME_LEN..(i + 1) * CONNECTOR_TV_MODE_NAME_LEN].copy_from_slice(name);
                }
                self.accepted_in(&buf[..n * CONNECTOR_TV_MODE_NAME_LEN])
            }
            GUD_REQ_GET_CONNECTOR_STATUS => {
                let Some(conn) = self.connector(req.value) else {
                    return Some(self.reject_in(GUD_STATUS_INVALID_PARAMETER));
                };
                // Read-and-clear the CHANGED bit
                buf[0] = conn.state.inner.lock(|data| {
                    let mut data = data.borrow_mut();
                    let status = data.status;
                    data.status &= !CONNECTOR_STATUS_CHANGED;
                    status
                });
                self.accepted_in(&buf[..1])
            }
            GUD_REQ_GET_CONNECTOR_MODES => {
                let Some(conn) = self.connector(req.value) else {
                    return Some(self.reject_in(GUD_STATUS_INVALID_PARAMETER));
                };
                if buf.len() < 24 {
                    return Some(self.reject_in(GUD_STATUS_PROTOCOL_ERROR));
                }
                let n = conn.state.inner.lock(|data| {
                    let data = data.borrow();
                    let n = data.modes_len.min(buf.len() / 24);
                    for (i, mode) in data.modes[..n].iter().enumerate() {
                        mode.serialize_into((&mut buf[i * 24..i * 24 + 24]).try_into().unwrap());
                    }
                    n
                });
                self.accepted_in(&buf[..n * 24])
            }
            GUD_REQ_GET_CONNECTOR_EDID => {
                let Some(conn) = self.connector(req.value) else {
                    return Some(self.reject_in(GUD_STATUS_INVALID_PARAMETER));
                };
                let n = conn.state.inner.lock(|data| {
                    let data = data.borrow();
                    let n = data.edid_len.min(buf.len());
                    buf[..n].copy_from_slice(&data.edid[..n]);
                    n
                });
                self.accepted_in(&buf[..n])
            }
            _ => self.reject_in(GUD_STATUS_REQUEST_NOT_SUPPORTED),
        })
    }

    fn control_out(&mut self, req: control::Request, data: &[u8]) -> Option<OutResponse> {
        if (req.request_type, req.recipient, req.index)
            != (RequestType::Vendor, Recipient::Interface, self.iface.0 as u16)
        {
            return None;
        }

        Some(match req.request {
            GUD_REQ_SET_CONNECTOR_FORCE_DETECT => {
                let idx = req.value as usize;
                if idx >= self.config.connectors.len() {
                    return Some(self.reject_out(GUD_STATUS_INVALID_PARAMETER));
                }
                debug!("gud: force detect connector {}", idx);
                self.try_send_critical(GudEvent::ConnectorForceDetect(idx as u8))
            }
            GUD_REQ_SET_BUFFER => {
                let Ok(data) = data.try_into() else {
                    return Some(self.reject_out(GUD_STATUS_PROTOCOL_ERROR));
                };
                let info = BufferInfo::deserialize_from(data);
                if info.compression != 0
                    || (self.config.max_buffer_size != 0 && info.length > self.config.max_buffer_size)
                {
                    return Some(self.reject_out(GUD_STATUS_INVALID_PARAMETER));
                }
                debug!(
                    "gud: set buffer {}x{}+{}+{} len={}",
                    info.width, info.height, info.x, info.y, info.length
                );
                self.try_send_critical(GudEvent::Buffer(info))
            }
            GUD_REQ_SET_STATE_CHECK => match validate_state_check(self.config, data) {
                Ok(state) => {
                    // Leave the previous checked state intact if the event cannot be queued.
                    if self.shared.events.is_full() {
                        return Some(self.reject_out(GUD_STATUS_BUSY));
                    }
                    debug!(
                        "gud: state check ok: {}x{} fmt={:?}",
                        state.mode.hdisplay, state.mode.vdisplay, state.format
                    );
                    self.shared.checked_state.lock(|c| *c.borrow_mut() = Some(state));
                    let _ = self.shared.events.try_send(GudEvent::StateCheck);
                    self.accepted_out()
                }
                Err(status) => self.reject_out(status),
            },
            GUD_REQ_SET_STATE_COMMIT => {
                if !data.is_empty() {
                    return Some(self.reject_out(GUD_STATUS_PROTOCOL_ERROR));
                }
                if self.shared.checked_state.lock(|c| c.borrow().is_none()) {
                    return Some(self.reject_out(GUD_STATUS_PROTOCOL_ERROR));
                }
                if self.shared.events.is_full() {
                    return Some(self.reject_out(GUD_STATUS_BUSY));
                }
                let state = self.shared.checked_state.lock(|c| c.borrow().clone());
                self.shared.committed_state.lock(|c| *c.borrow_mut() = state);
                debug!("gud: state commit");
                self.try_send_critical(GudEvent::StateCommit)
            }
            GUD_REQ_SET_CONTROLLER_ENABLE | GUD_REQ_SET_DISPLAY_ENABLE => {
                if data.len() != 1 || data[0] > 1 {
                    return Some(self.reject_out(GUD_STATUS_INVALID_PARAMETER));
                }
                let enable = data[0] == 1;
                let event = if req.request == GUD_REQ_SET_CONTROLLER_ENABLE {
                    debug!("gud: controller enable {}", enable);
                    GudEvent::ControllerEnable(enable)
                } else {
                    debug!("gud: display enable {}", enable);
                    GudEvent::DisplayEnable(enable)
                };
                self.try_send_critical(event)
            }
            _ => self.reject_out(GUD_STATUS_REQUEST_NOT_SUPPORTED),
        })
    }
}

impl<'d, D: Driver<'d>> GudClass<'d, D> {
    /// Create a new GUD class. Adds one vendor-specific interface with one bulk
    /// OUT endpoint and registers the control handler.
    ///
    /// The builder's control buffer must be at least [`Config::control_buf_len`]
    /// bytes long. A smaller buffer cannot serve the advertised configuration
    /// without truncating responses or rejecting state checks.
    pub fn new(builder: &mut Builder<'d, D>, state: &'d mut State<'d>, config: &'d Config<'d>) -> Self {
        assert!(!config.formats.is_empty() && config.formats.len() <= GUD_FORMATS_MAX_NUM);
        assert!(!config.connectors.is_empty() && config.connectors.len() <= GUD_CONNECTORS_MAX_NUM);
        assert!(config.min_width > 0 && config.min_width <= config.max_width);
        assert!(config.min_height > 0 && config.min_height <= config.max_height);

        let max_pitch = config.formats.iter().map(|f| f.pitch(config.max_width)).max().unwrap();
        assert!(
            config.max_buffer_size == 0 || config.max_buffer_size >= max_pitch,
            "max_buffer_size must be 0 or >= widest scanline ({})",
            max_pitch
        );

        for conn in config.connectors {
            assert!(conn.properties.len() <= GUD_CONNECTOR_PROPERTIES_MAX_NUM);
            if let Some(values) = conn.tv_mode_values {
                for name in values {
                    assert!(name.contains(&0), "TV mode names must be NUL-terminated");
                }
            }
        }

        let control_buf_len = config.control_buf_len();
        assert!(
            builder.control_buf_len() >= control_buf_len,
            "control_buf must be >= {} bytes for this configuration",
            control_buf_len
        );

        let mut func = builder.function(USB_CLASS_VENDOR_SPEC, 0, 0);
        let mut iface = func.interface();
        let interface_number = iface.interface_number();
        let mut alt = iface.alt_setting(USB_CLASS_VENDOR_SPEC, 0, 0, None);

        let read_ep = alt.endpoint_bulk_out(None, config.max_packet_size);

        drop(func);
        builder.handler(state.control.write(Control {
            iface: interface_number,
            config,
            shared: &state.shared,
        }));

        Self {
            read_ep,
            shared: &state.shared,
        }
    }

    /// Wait until the host has configured the interface (bulk endpoint enabled).
    ///
    /// This future is cancel-safe.
    pub async fn wait_connection(&mut self) {
        self.read_ep.wait_enabled().await
    }

    /// Wait for the next host-side event, in request order.
    ///
    /// This future is cancel-safe.
    pub async fn next_event(&self) -> GudEvent {
        self.shared.events.receive().await
    }

    /// Try to receive the next host-side event without waiting.
    pub fn try_next_event(&self) -> Option<GudEvent> {
        self.shared.events.try_receive().ok()
    }

    /// Latest state set by `SET_STATE_CHECK`.
    ///
    /// This may differ from [`Self::committed_state`] until the host commits it.
    pub fn checked_state(&self) -> Option<DisplayState> {
        self.shared.checked_state.lock(|c| c.borrow().clone())
    }

    /// Latest state set by `SET_STATE_COMMIT`.
    pub fn committed_state(&self) -> Option<DisplayState> {
        self.shared.committed_state.lock(|c| c.borrow().clone())
    }

    /// Read one bulk packet, returning its byte count.
    ///
    /// `data` must hold at least `config.max_packet_size` bytes. Read
    /// [`BufferInfo::transfer_size`] bytes before awaiting the next event.
    /// Not cancel-safe.
    pub async fn read_packet(&mut self, data: &mut [u8]) -> Result<usize, EndpointError> {
        self.read_ep.read(data).await
    }

    /// Stream `info` through an async callback, returning its total consumed bytes.
    ///
    /// `chunk` must hold at least `config.max_packet_size` bytes.
    ///
    /// Not cancel-safe. After an error, drain unread data or reset before another transfer.
    pub async fn read_buffer<E, F: AsyncFnMut(&[u8]) -> Result<usize, E>>(
        &mut self,
        info: &BufferInfo,
        chunk: &mut [u8],
        mut f: F,
    ) -> Result<usize, ReadBufferError<E>> {
        let mut remaining = info.transfer_size() as usize;
        let mut consumed = 0;
        while remaining > 0 {
            let cap = remaining.min(chunk.len());
            let n = self
                .read_ep
                .read(&mut chunk[..cap])
                .await
                .map_err(ReadBufferError::Endpoint)?;
            consumed += f(&chunk[..n]).await.map_err(ReadBufferError::Callback)?;
            remaining -= n;
        }
        Ok(consumed)
    }
}

// Validate a `SET_STATE_CHECK` payload against the advertised config.
// Returns the parsed state or the `GUD_STATUS_*` code to report.
fn validate_state_check(config: &Config, data: &[u8]) -> Result<DisplayState, u8> {
    // mode(24) + format(1) + connector(1) + 10-byte properties
    if data.len() < 26 || !(data.len() - 26).is_multiple_of(10) {
        return Err(GUD_STATUS_INVALID_PARAMETER);
    }
    if (data.len() - 26) / 10 > MAX_PROPERTIES {
        return Err(GUD_STATUS_INVALID_PARAMETER);
    }

    let mode = DisplayMode::deserialize_from(data[..24].try_into().unwrap());
    let Some(format) = PixelFormat::from_u8(data[24]) else {
        return Err(GUD_STATUS_INVALID_PARAMETER);
    };
    if !config.formats.contains(&format) {
        return Err(GUD_STATUS_INVALID_PARAMETER);
    }
    let connector_idx = data[25];
    let Some(conn) = config.connectors.get(connector_idx as usize) else {
        return Err(GUD_STATUS_INVALID_PARAMETER);
    };

    let w = mode.hdisplay as u32;
    let h = mode.vdisplay as u32;
    let in_range = w >= config.min_width && w <= config.max_width && h >= config.min_height && h <= config.max_height;
    if !in_range || !(config.validate_mode)(connector_idx, &mode) {
        return Err(GUD_STATUS_INVALID_PARAMETER);
    }

    let mut properties: Vec<Property, MAX_PROPERTIES> = Vec::new();
    for prop in data[26..].as_chunks::<10>().0.iter().map(Property::deserialize_from) {
        let declared = conn.properties.iter().any(|p| p.prop == prop.prop)
            || (prop.prop == PROPERTY_ROTATION && config.supported_rotations != 0);
        if !declared {
            return Err(GUD_STATUS_INVALID_PARAMETER);
        }
        if RANGE_PROPERTIES.contains(&prop.prop) && prop.val > 100 {
            return Err(GUD_STATUS_INVALID_PARAMETER);
        }
        properties.push(prop).unwrap(); // Count checked above.
    }

    Ok(DisplayState {
        mode,
        format,
        connector: connector_idx,
        properties,
    })
}

#[cfg(test)]
mod tests {
    use core::cell::Cell;
    use core::future::{Future, poll_fn};
    use core::pin::{Pin, pin};
    use core::task::{Context, Poll, Waker};

    use super::*;
    use crate::driver::{
        Bus, ControlPipe, EndpointAddress, EndpointAllocError, EndpointIn, EndpointInfo, EndpointType, Event,
        Unsupported,
    };

    struct ScriptedEndpoint<'a> {
        packets: &'a [Result<&'a [u8], EndpointError>],
        reads: &'a Cell<usize>,
    }

    impl Endpoint for ScriptedEndpoint<'_> {
        fn info(&self) -> &EndpointInfo {
            unreachable!()
        }

        async fn wait_enabled(&mut self) {
            unreachable!()
        }
    }

    impl EndpointOut for ScriptedEndpoint<'_> {
        async fn read(&mut self, buf: &mut [u8]) -> Result<usize, EndpointError> {
            let index = self.reads.get();
            self.reads.set(index + 1);
            let packet = self.packets[index]?;
            buf[..packet.len()].copy_from_slice(packet);
            Ok(packet.len())
        }
    }

    struct ScriptedDriver;

    impl<'a> Driver<'a> for ScriptedDriver {
        type EndpointOut = ScriptedEndpoint<'a>;
        type EndpointIn = Unused;
        type ControlPipe = Unused;
        type Bus = Unused;

        fn alloc_endpoint_out(
            &mut self,
            _: EndpointType,
            _: Option<EndpointAddress>,
            _: u16,
            _: u8,
        ) -> Result<Self::EndpointOut, EndpointAllocError> {
            unreachable!()
        }

        fn alloc_endpoint_in(
            &mut self,
            _: EndpointType,
            _: Option<EndpointAddress>,
            _: u16,
            _: u8,
        ) -> Result<Self::EndpointIn, EndpointAllocError> {
            unreachable!()
        }

        fn start(self, _: u16) -> (Self::Bus, Self::ControlPipe) {
            unreachable!()
        }
    }

    struct Unused;

    impl Endpoint for Unused {
        fn info(&self) -> &EndpointInfo {
            unreachable!()
        }

        async fn wait_enabled(&mut self) {
            unreachable!()
        }
    }

    impl EndpointIn for Unused {
        async fn write(&mut self, _: &[u8]) -> Result<(), EndpointError> {
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

    fn scripted_class<'a>(
        shared: &'a Shared,
        packets: &'a [Result<&'a [u8], EndpointError>],
        reads: &'a Cell<usize>,
    ) -> GudClass<'a, ScriptedDriver> {
        GudClass {
            read_ep: ScriptedEndpoint { packets, reads },
            shared,
        }
    }

    fn buffer_info(length: u32) -> BufferInfo {
        BufferInfo {
            x: 0,
            y: 0,
            width: length,
            height: 1,
            length,
            compression: 0,
            compressed_length: 0,
        }
    }

    fn poll_once<F: Future>(future: Pin<&mut F>) -> Poll<F::Output> {
        future.poll(&mut Context::from_waker(Waker::noop()))
    }

    #[test]
    fn read_buffer_distinguishes_error_sources_and_stops_reading() {
        for first in [Err(EndpointError::Disabled), Ok(&[1, 2][..])] {
            let state = State::new();
            let reads = Cell::new(0);
            let packets = [first, Ok(&[7, 8][..])];
            let mut class = scripted_class(&state.shared, &packets, &reads);
            let info = buffer_info(4);
            let mut chunk = [0; 4];
            let mut calls = 0;
            let expected = if first.is_err() {
                ReadBufferError::Endpoint(EndpointError::Disabled)
            } else {
                ReadBufferError::Callback(EndpointError::Disabled)
            };

            let result = poll_once(pin!(class.read_buffer(&info, &mut chunk, async |packet| {
                calls += 1;
                assert_eq!(packet, &[1, 2]);
                Err::<usize, _>(EndpointError::Disabled)
            })));
            assert_eq!(result, Poll::Ready(Err(expected)));
            assert_eq!(calls, usize::from(first.is_ok()));
            assert_eq!(reads.get(), 1);
            assert_eq!(poll_once(pin!(class.read_packet(&mut chunk))), Poll::Ready(Ok(2)));
            assert_eq!(&chunk[..2], &[7, 8]);
        }
    }

    #[test]
    fn read_buffer_waits_for_borrowing_callback_and_sums_consumed_counts() {
        let state = State::new();
        let reads = Cell::new(0);
        let packets: &[Result<&[u8], EndpointError>] = &[Ok(&[1, 2, 3]), Ok(&[4, 5]), Ok(&[9])];
        let mut class = scripted_class(&state.shared, packets, &reads);
        let info = buffer_info(5);
        let mut chunk = [0; 4];
        let ready = Cell::new(false);
        let mut sum = 0;

        {
            let mut transfer = pin!(class.read_buffer(
                &info,
                &mut chunk,
                async |packet: &[u8]| -> Result<usize, EndpointError> {
                    let total = &mut sum;
                    poll_fn(|_| if ready.get() { Poll::Ready(()) } else { Poll::Pending }).await;
                    *total += packet.iter().sum::<u8>();
                    ready.set(false);
                    Ok(packet.len() - 1)
                },
            ));

            assert_eq!(poll_once(transfer.as_mut()), Poll::Pending);
            assert_eq!(reads.get(), 1);

            ready.set(true);
            assert_eq!(poll_once(transfer.as_mut()), Poll::Pending);
            assert_eq!(reads.get(), 2);

            ready.set(true);
            assert_eq!(poll_once(transfer.as_mut()), Poll::Ready(Ok(3)));
        }
        assert_eq!(sum, 15);
        assert_eq!(reads.get(), 2);
    }

    fn mode() -> DisplayMode {
        DisplayMode {
            clock: 148500,
            hdisplay: 1920,
            hsync_start: 2008,
            hsync_end: 2052,
            htotal: 2200,
            vdisplay: 1080,
            vsync_start: 1084,
            vsync_end: 1089,
            vtotal: 1125,
            flags: DISPLAY_MODE_FLAG_PHSYNC | DISPLAY_MODE_FLAG_PVSYNC,
            preferred: true,
        }
    }

    static CONN_PROPERTIES: [Property; 1] = [Property {
        prop: PROPERTY_BACKLIGHT_BRIGHTNESS,
        val: 80,
    }];

    fn connector<'d>(state: &'d ConnectorState<'d>) -> GudConnector<'d> {
        GudConnector {
            connector_type: CONNECTOR_TYPE_PANEL,
            flags: 0,
            state,
            properties: &CONN_PROPERTIES,
            tv_mode_values: None,
        }
    }

    static FORMATS: &[PixelFormat] = &[PixelFormat::Rgb565, PixelFormat::Xrgb8888];

    fn config<'d>(connectors: &'d [GudConnector<'d>]) -> Config<'d> {
        Config {
            max_packet_size: 64,
            min_width: 640,
            max_width: 1920,
            min_height: 480,
            max_height: 1080,
            max_buffer_size: 7680,
            formats: FORMATS,
            supported_rotations: ROTATION_0 | ROTATION_90,
            connectors,
            validate_mode: |_, _| true,
        }
    }

    fn with_control(f: impl FnOnce(&ConnectorState<'_>, &mut Control<'_>)) {
        let mut modes = [mode(); 2];
        let mut edid = [0; 256];
        let connector_state = ConnectorState::new(&mut modes, &mut edid);
        let connectors = [connector(&connector_state)];
        let cfg = config(&connectors);
        let state = State::new();
        f(
            &connector_state,
            &mut Control {
                iface: InterfaceNumber(0),
                config: &cfg,
                shared: &state.shared,
            },
        );
    }

    fn assert_status(control: &mut Control<'_>, status: u8) {
        assert_in(control, GUD_REQ_GET_CONNECTOR_STATUS, &[status]);
    }

    fn assert_out(control: &mut Control<'_>, request: u8, data: &[u8], response: OutResponse) {
        assert_eq!(
            control.control_out(out_request(request, data.len() as u16), data),
            Some(response)
        );
    }

    fn out_request(request: u8, length: u16) -> Request {
        Request {
            direction: crate::driver::Direction::Out,
            request_type: RequestType::Vendor,
            recipient: Recipient::Interface,
            request,
            value: 0,
            index: 0,
            length,
        }
    }

    fn assert_in(control: &mut Control<'_>, request: u8, expected: &[u8]) {
        let mut buf = [0; 512];
        let mut req = out_request(request, buf.len() as u16);
        req.direction = crate::driver::Direction::In;
        match control.control_in(req, &mut buf) {
            Some(InResponse::Accepted(data)) => assert_eq!(data, expected),
            _ => panic!("expected accepted control IN response"),
        }
    }

    fn mode_bytes(mode: &DisplayMode) -> [u8; 24] {
        let mut bytes = [0; 24];
        mode.serialize_into(&mut bytes);
        bytes
    }

    fn state_payload(mode: &DisplayMode, format: PixelFormat) -> [u8; 26] {
        let mut data = [0; 26];
        mode.serialize_into((&mut data[..24]).try_into().unwrap());
        data[24] = format.to_u8();
        data
    }

    #[test]
    fn connector_updates_preserve_snapshots() {
        with_control(|connector_state, control| {
            assert_eq!(control.config.control_buf_len(), 256);
            let mut edid = [0x11; 256];
            connector_state
                .update(CONNECTOR_STATUS_CONNECTED, &[mode()], Some(&edid))
                .unwrap();
            edid.fill(0x22);
            assert_status(control, CONNECTOR_STATUS_CONNECTED | CONNECTOR_STATUS_CHANGED);
            assert_status(control, CONNECTOR_STATUS_CONNECTED);
            assert_in(control, GUD_REQ_GET_CONNECTOR_MODES, &mode_bytes(&mode()));
            assert_in(control, GUD_REQ_GET_CONNECTOR_EDID, &[0x11; 256]);

            // Failed updates must not partially replace a published snapshot.
            assert_eq!(
                connector_state.update(CONNECTOR_STATUS_UNKNOWN, &[], Some(&[0; 384])),
                Err(ConnectorUpdateError::EdidTooLong)
            );
            assert_status(control, CONNECTOR_STATUS_CONNECTED);
            assert_in(control, GUD_REQ_GET_CONNECTOR_MODES, &mode_bytes(&mode()));
            assert_in(control, GUD_REQ_GET_CONNECTOR_EDID, &[0x11; 256]);

            connector_state
                .update(CONNECTOR_STATUS_CONNECTED, &[], Some(&edid[..128]))
                .unwrap();
            assert_status(control, CONNECTOR_STATUS_CONNECTED | CONNECTOR_STATUS_CHANGED);
            assert_in(control, GUD_REQ_GET_CONNECTOR_MODES, &[]);
            assert_in(control, GUD_REQ_GET_CONNECTOR_EDID, &[0x22; 128]);
            connector_state
                .update(CONNECTOR_STATUS_DISCONNECTED, &[], None)
                .unwrap();
            assert_status(control, CONNECTOR_STATUS_DISCONNECTED | CONNECTOR_STATUS_CHANGED);
            assert_in(control, GUD_REQ_GET_CONNECTOR_EDID, &[]);
        });
    }

    #[test]
    fn state_check_validates_firmware_policy_and_payload() {
        let mut modes = [mode()];
        let connector_state = ConnectorState::new(&mut modes, &mut []);
        connector_state
            .update(CONNECTOR_STATUS_CONNECTED, &[mode()], None)
            .unwrap();
        let connectors = [connector(&connector_state); 2];
        let cfg = Config {
            validate_mode: |connector, requested| connector == 1 && requested.clock == 148501,
            ..config(&connectors)
        };
        let custom = DisplayMode {
            clock: 148501,
            ..mode()
        };
        let mut data = [0; 36];
        data[..26].copy_from_slice(&state_payload(&custom, PixelFormat::Rgb565));
        data[25] = 1;
        data[26..28].copy_from_slice(&PROPERTY_BACKLIGHT_BRIGHTNESS.to_le_bytes());
        data[28..36].copy_from_slice(&50u64.to_le_bytes());
        let accepted = validate_state_check(&cfg, &data).unwrap();
        assert_eq!((accepted.mode, accepted.connector), (custom, 1));
        assert_eq!(
            accepted.properties.as_slice(),
            &[Property {
                prop: PROPERTY_BACKLIGHT_BRIGHTNESS,
                val: 50
            }]
        );

        for length in [25, 29] {
            assert_eq!(
                validate_state_check(&cfg, &data[..length]),
                Err(GUD_STATUS_INVALID_PARAMETER)
            );
        }
        for (offset, value) in [
            (24, PixelFormat::R8.to_u8()),
            (25, 0),
            (25, 2),
            (26, PROPERTY_TV_MODE as u8),
            (28, 101),
        ] {
            let mut invalid = data;
            invalid[offset] = value;
            assert_eq!(validate_state_check(&cfg, &invalid), Err(GUD_STATUS_INVALID_PARAMETER));
        }
        // An advertised clock can be rejected; callback approval cannot bypass bounds.
        for rejected in [
            mode(),
            DisplayMode {
                hdisplay: 1921,
                ..custom
            },
        ] {
            let mut invalid = data;
            invalid[..24].copy_from_slice(&mode_bytes(&rejected));
            assert_eq!(validate_state_check(&cfg, &invalid), Err(GUD_STATUS_INVALID_PARAMETER));
        }
        connector_state
            .update(CONNECTOR_STATUS_DISCONNECTED, &[], None)
            .unwrap();
        assert_eq!(validate_state_check(&cfg, &data).unwrap(), accepted);
    }

    #[test]
    fn state_commit_backpressure_and_disconnect() {
        for reset in [false, true] {
            with_control(|connector_state, control| {
                connector_state
                    .update(CONNECTOR_STATUS_CONNECTED, &[mode()], None)
                    .unwrap();
                assert_status(control, CONNECTOR_STATUS_CONNECTED | CONNECTOR_STATUS_CHANGED);
                let first = state_payload(&mode(), PixelFormat::Rgb565);
                let next = state_payload(&mode(), PixelFormat::Xrgb8888);
                assert_out(control, GUD_REQ_SET_STATE_COMMIT, &[], OutResponse::Rejected);
                assert_out(control, GUD_REQ_SET_STATE_CHECK, &first, OutResponse::Accepted);
                assert_out(control, GUD_REQ_SET_STATE_COMMIT, &[], OutResponse::Accepted);
                control.shared.events.clear();
                assert_out(control, GUD_REQ_SET_STATE_CHECK, &next, OutResponse::Accepted);
                for _ in 1..MAX_EVENTS {
                    control.shared.events.try_send(GudEvent::StateCheck).unwrap();
                }
                for request in [GUD_REQ_SET_STATE_COMMIT, GUD_REQ_SET_CONNECTOR_FORCE_DETECT] {
                    assert_out(control, request, &[], OutResponse::Rejected);
                    assert_in(control, GUD_REQ_GET_STATUS, &[GUD_STATUS_BUSY]);
                }
                control.shared.committed_state.lock(|s| {
                    assert_eq!(s.borrow().as_ref().unwrap().format, PixelFormat::Rgb565);
                });
                control.shared.events.clear();
                assert_out(control, GUD_REQ_SET_CONNECTOR_FORCE_DETECT, &[], OutResponse::Accepted);
                assert!(matches!(
                    control.shared.events.try_receive(),
                    Ok(GudEvent::ConnectorForceDetect(0))
                ));
                assert_out(control, GUD_REQ_SET_STATE_COMMIT, &[], OutResponse::Accepted);
                assert!(matches!(control.shared.events.try_receive(), Ok(GudEvent::StateCommit)));
                control.shared.committed_state.lock(|s| {
                    assert_eq!(s.borrow().as_ref().unwrap().format, PixelFormat::Xrgb8888);
                });
                assert_out(control, GUD_REQ_SET_STATE_CHECK, &first, OutResponse::Accepted);

                if reset {
                    control.reset();
                    assert!(matches!(control.shared.events.try_receive(), Ok(GudEvent::Reset)));
                } else {
                    control.configured(false);
                    assert!(matches!(
                        control.shared.events.try_receive(),
                        Ok(GudEvent::Configured(false))
                    ));
                }
                control.shared.committed_state.lock(|s| assert!(s.borrow().is_none()));
                control.shared.checked_state.lock(|s| assert!(s.borrow().is_none()));
                assert!(control.shared.events.try_receive().is_err());
                assert_status(control, CONNECTOR_STATUS_CONNECTED | CONNECTOR_STATUS_CHANGED);
                assert_in(control, GUD_REQ_GET_CONNECTOR_MODES, &mode_bytes(&mode()));
                assert_out(control, GUD_REQ_SET_STATE_COMMIT, &[], OutResponse::Rejected);
            });
        }
    }

    #[test]
    fn display_wire_invariants() {
        for preferred in [false, true] {
            let expected = DisplayMode { preferred, ..mode() };
            let bytes = mode_bytes(&DisplayMode {
                flags: expected.flags | !DISPLAY_MODE_FLAG_USER_MASK,
                ..expected
            });
            let flags = u32::from_le_bytes(bytes[20..24].try_into().unwrap());
            assert_eq!(flags & DISPLAY_MODE_FLAG_PREFERRED != 0, preferred);
            assert_eq!(flags & !DISPLAY_MODE_FLAG_PREFERRED, expected.flags);
            assert_eq!(DisplayMode::deserialize_from(&bytes), expected);
        }

        assert_eq!(PixelFormat::R1.pitch(8), 1);
        assert_eq!(PixelFormat::R1.pitch(9), 2);
        assert_eq!(PixelFormat::Xrgb1111.pitch(8), 4);
        assert_eq!(PixelFormat::Xrgb1111.pitch(9), 5);
    }

    #[test]
    fn buffer_request_length_and_decoding() {
        let mut data = [0; 25];
        for (field, value) in data[..20].chunks_exact_mut(4).zip([10u32, 20, 80, 48, 7680]) {
            field.copy_from_slice(&value.to_le_bytes());
        }
        with_control(|_, control| {
            for invalid in [&data[..24], &[0; 26][..]] {
                assert_out(control, GUD_REQ_SET_BUFFER, invalid, OutResponse::Rejected);
                assert_in(control, GUD_REQ_GET_STATUS, &[GUD_STATUS_PROTOCOL_ERROR]);
                assert!(control.shared.events.try_receive().is_err());
            }
            assert_out(control, GUD_REQ_SET_BUFFER, &data, OutResponse::Accepted);
            let GudEvent::Buffer(info) = control.shared.events.try_receive().unwrap() else {
                panic!("expected buffer event");
            };
            assert_eq!((info.x, info.y, info.width, info.height), (10, 20, 80, 48));
            assert_eq!((info.length, info.compression, info.transfer_size()), (7680, 0, 7680));
        });
    }
}
