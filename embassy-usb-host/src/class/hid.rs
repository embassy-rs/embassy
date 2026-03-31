//! HID (Human Interface Device) host class driver.
//!
//! This driver can communicate with USB HID devices (keyboards, mice, gamepads, etc.).

use embassy_usb_driver::host::{ChannelError, UsbChannel, UsbHostDriver, channel};
use embassy_usb_driver::{Direction as UsbDirection, EndpointAddress, EndpointInfo, EndpointType};

pub use super::hid_report::{ReportDescriptor, ReportField};
use crate::bytes_to_setup;
use crate::descriptor::ConfigurationDescriptor;

/// HID class code.
const USB_CLASS_HID: u8 = 0x03;
/// Interrupt transfer type.
const TRANSFER_INTERRUPT: u8 = 0x03;

/// HID class request: GET_REPORT.
const GET_REPORT: u8 = 0x01;
/// HID class request: SET_IDLE.
const SET_IDLE: u8 = 0x0A;
/// HID class request: SET_PROTOCOL.
const SET_PROTOCOL: u8 = 0x0B;

/// Boot protocol.
pub const PROTOCOL_BOOT: u8 = 0;
/// Report protocol.
pub const PROTOCOL_REPORT: u8 = 1;

// ── Boot-protocol report structs ─────────────────────────────────────────────

/// Decoded keyboard report (USB HID boot protocol, 8 bytes).
///
/// All standard USB keyboards support this layout when placed in boot protocol
/// mode via [`HidHost::set_protocol`] with [`PROTOCOL_BOOT`].
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct KeyboardReport {
    /// Modifier keys bitmask.
    ///
    /// Bit 0: Left Ctrl  | Bit 1: Left Shift  | Bit 2: Left Alt  | Bit 3: Left GUI
    /// Bit 4: Right Ctrl | Bit 5: Right Shift | Bit 6: Right Alt | Bit 7: Right GUI
    pub modifiers: u8,
    /// Up to 6 simultaneously pressed key codes (HID usage page 0x07).
    /// A value of 0x00 means "no key"; 0x01 means "rollover error".
    pub keycodes: [u8; 6],
}

impl KeyboardReport {
    /// Parse a boot-protocol keyboard report from an 8-byte buffer.
    /// Returns `None` if the buffer is shorter than 8 bytes.
    pub fn parse(buf: &[u8]) -> Option<Self> {
        if buf.len() < 8 {
            return None;
        }
        Some(Self {
            modifiers: buf[0],
            // buf[1] is reserved
            keycodes: [buf[2], buf[3], buf[4], buf[5], buf[6], buf[7]],
        })
    }

    /// Returns `true` if the given HID key code is currently pressed.
    pub fn is_pressed(&self, keycode: u8) -> bool {
        keycode != 0 && self.keycodes.contains(&keycode)
    }

    /// Returns `true` if Left Ctrl or Right Ctrl is held.
    pub fn ctrl(&self) -> bool {
        self.modifiers & 0x11 != 0
    }
    /// Returns `true` if Left Shift or Right Shift is held.
    pub fn shift(&self) -> bool {
        self.modifiers & 0x22 != 0
    }
    /// Returns `true` if Left Alt or Right Alt is held.
    pub fn alt(&self) -> bool {
        self.modifiers & 0x44 != 0
    }
    /// Returns `true` if Left GUI (Win/Cmd) or Right GUI is held.
    pub fn gui(&self) -> bool {
        self.modifiers & 0x88 != 0
    }
}

/// Mouse button bitmask used in [`MouseReport`].
///
/// Bit 0: left button | Bit 1: right button | Bit 2: middle button
pub type MouseButtons = u8;

/// Decoded mouse report (USB HID boot protocol, 4 bytes).
///
/// All standard USB mice support this layout in boot protocol mode.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct MouseReport {
    /// Button state. Use the [`MouseButtons`] constants or check bits directly.
    pub buttons: MouseButtons,
    /// Horizontal movement since last report (signed, positive = right).
    pub x: i8,
    /// Vertical movement since last report (signed, positive = down).
    pub y: i8,
    /// Scroll wheel movement (signed, positive = scroll up / away from user).
    pub wheel: i8,
}

impl MouseReport {
    /// Left mouse button.
    pub const BUTTON_LEFT: MouseButtons = 1 << 0;
    /// Right mouse button.
    pub const BUTTON_RIGHT: MouseButtons = 1 << 1;
    /// Middle mouse button (scroll wheel click).
    pub const BUTTON_MIDDLE: MouseButtons = 1 << 2;

    /// Parse a boot-protocol mouse report from a buffer (minimum 3 bytes; 4 for wheel).
    /// Returns `None` if the buffer is shorter than 3 bytes.
    pub fn parse(buf: &[u8]) -> Option<Self> {
        if buf.len() < 3 {
            return None;
        }
        Some(Self {
            buttons: buf[0],
            x: buf[1] as i8,
            y: buf[2] as i8,
            wheel: if buf.len() >= 4 { buf[3] as i8 } else { 0 },
        })
    }

    /// Returns `true` if the left button is pressed.
    pub fn left(&self) -> bool {
        self.buttons & Self::BUTTON_LEFT != 0
    }
    /// Returns `true` if the right button is pressed.
    pub fn right(&self) -> bool {
        self.buttons & Self::BUTTON_RIGHT != 0
    }
    /// Returns `true` if the middle button is pressed.
    pub fn middle(&self) -> bool {
        self.buttons & Self::BUTTON_MIDDLE != 0
    }
}

/// HID class descriptor type (appears inside the configuration descriptor).
const DESC_HID: u8 = 0x21;

/// Information about a HID interface found in a configuration descriptor.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct HidInfo {
    /// HID interface number.
    pub interface_number: u8,
    /// Interrupt IN endpoint address (raw, with direction bit).
    pub interrupt_in_ep: u8,
    /// Interrupt IN max packet size.
    pub interrupt_in_mps: u16,
    /// Length of the HID Report Descriptor in bytes (from the HID class descriptor).
    /// Pass this to [`HidHost::fetch_report_descriptor`] as the buffer size.
    pub report_descriptor_len: u16,
}

/// Find the first HID interface in a configuration descriptor.
pub fn find_hid(config_desc: &[u8]) -> Option<HidInfo> {
    let cfg = ConfigurationDescriptor::try_from_slice(config_desc).ok()?;

    for iface in cfg.iter_interface() {
        if iface.interface_class != USB_CLASS_HID {
            continue;
        }

        // Extract report descriptor length from the HID class descriptor (type 0x21).
        // Layout: bLength, bDescriptorType(0x21), bcdHID(2), bCountryCode,
        //         bNumDescriptors, bDescriptorType(0x22), wDescriptorLength(2)
        let report_desc_len = iface
            .iter_descriptors()
            .find_map(|(_, data)| {
                if data.len() >= 7 && data[1] == DESC_HID {
                    Some(u16::from_le_bytes([data[5], data[6]]))
                } else {
                    None
                }
            })
            .unwrap_or(0);

        let ep = iface
            .iter_endpoints()
            .find(|ep| ep.transfer_type() == TRANSFER_INTERRUPT && ep.is_in())?;

        return Some(HidInfo {
            interface_number: iface.interface_number,
            interrupt_in_ep: ep.endpoint_address,
            interrupt_in_mps: ep.max_packet_size,
            report_descriptor_len: report_desc_len,
        });
    }

    None
}

/// HID host class driver error.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum HidError {
    /// Transfer error.
    Transfer(ChannelError),
    /// No matching HID interface found in the device.
    NoInterface,
    /// Failed to allocate a channel.
    NoChannel,
}

impl From<ChannelError> for HidError {
    fn from(e: ChannelError) -> Self {
        Self::Transfer(e)
    }
}

impl core::fmt::Display for HidError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Transfer(_e) => write!(f, "Transfer error"),
            Self::NoInterface => write!(f, "No HID interface found"),
            Self::NoChannel => write!(f, "No free channel"),
        }
    }
}

impl core::error::Error for HidError {}

/// HID host driver.
///
/// Provides report reading and optional class request access to a USB HID device.
pub struct HidHost<D: UsbHostDriver> {
    ctrl_ch: D::Channel<channel::Control, channel::InOut>,
    in_ch: D::Channel<channel::Interrupt, channel::In>,
    interface: u8,
    report_descriptor_len: u16,
}

impl<D: UsbHostDriver> HidHost<D> {
    /// Create a new HID host driver.
    ///
    /// Parses the config descriptor to find the HID interface and its interrupt IN endpoint,
    /// then allocates the necessary channels.
    pub fn new(driver: &D, config_desc: &[u8], device_address: u8, max_packet_size_0: u16) -> Result<Self, HidError> {
        let info = find_hid(config_desc).ok_or(HidError::NoInterface)?;

        let ctrl_ep_info = EndpointInfo {
            addr: EndpointAddress::from_parts(0, UsbDirection::In),
            ep_type: EndpointType::Control,
            max_packet_size: max_packet_size_0,
            interval_ms: 0,
        };

        let in_ep_info = EndpointInfo {
            addr: EndpointAddress::from_parts((info.interrupt_in_ep & 0x0F) as usize, UsbDirection::In),
            ep_type: EndpointType::Interrupt,
            max_packet_size: info.interrupt_in_mps,
            interval_ms: 0,
        };

        let ctrl_ch = driver
            .alloc_channel::<channel::Control, channel::InOut>(device_address, &ctrl_ep_info, false)
            .map_err(|_| HidError::NoChannel)?;
        let in_ch = driver
            .alloc_channel::<channel::Interrupt, channel::In>(device_address, &in_ep_info, false)
            .map_err(|_| HidError::NoChannel)?;

        Ok(Self {
            ctrl_ch,
            in_ch,
            interface: info.interface_number,
            report_descriptor_len: info.report_descriptor_len,
        })
    }

    /// Fetch the HID Report Descriptor from the device into `buf`.
    ///
    /// Returns the descriptor bytes as a slice. Pass the result to
    /// [`ReportDescriptor::parse`] to decode it:
    ///
    /// ```ignore
    /// let mut buf = [0u8; 256];
    /// let desc = hid.fetch_report_descriptor(&mut buf).await?;
    /// let report: ReportDescriptor<32> = ReportDescriptor::parse(desc);
    /// ```
    ///
    /// `buf` should be at least `HidInfo::report_descriptor_len` bytes; any
    /// excess is unused.
    pub async fn fetch_report_descriptor<'a>(&mut self, buf: &'a mut [u8]) -> Result<&'a [u8], HidError> {
        let len = (self.report_descriptor_len as usize).min(buf.len()) as u16;
        let setup_bytes = crate::control::get_hid_report_descriptor(self.interface, len);
        let setup = bytes_to_setup(&setup_bytes);
        let n = self.ctrl_ch.control_in(&setup, &mut buf[..len as usize]).await?;
        Ok(&buf[..n])
    }

    /// Set the idle rate for a report.
    ///
    /// `report_id = 0` applies to all reports. `idle_duration = 0` disables idle repeat.
    ///
    /// Note: SET_IDLE is optional; some devices STALL this request.
    /// A STALL is treated as success per the HID specification.
    pub async fn set_idle(&mut self, report_id: u8, idle_duration: u8) -> Result<(), HidError> {
        let value = (idle_duration as u16) << 8 | report_id as u16;
        let setup_bytes = crate::control::class_interface_out(SET_IDLE, value, self.interface as u16);
        let setup = bytes_to_setup(&setup_bytes);
        match self.ctrl_ch.control_out(&setup, &[]).await {
            Ok(_) => Ok(()),
            Err(ChannelError::Stall) => Ok(()),
            Err(e) => Err(HidError::Transfer(e)),
        }
    }

    /// Set the protocol (boot or report).
    pub async fn set_protocol(&mut self, protocol: u8) -> Result<(), HidError> {
        let setup_bytes = crate::control::class_interface_out(SET_PROTOCOL, protocol as u16, self.interface as u16);
        let setup = bytes_to_setup(&setup_bytes);
        self.ctrl_ch.control_out(&setup, &[]).await?;
        Ok(())
    }

    /// Read a raw input report from the interrupt IN endpoint.
    ///
    /// Returns the number of bytes received.
    pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize, HidError> {
        let n = self.in_ch.request_in(buf).await?;
        Ok(n)
    }

    /// Read and parse a boot-protocol keyboard report.
    ///
    /// Call [`HidHost::set_protocol`] with [`PROTOCOL_BOOT`] first.
    /// Returns `None` if the report is malformed (shorter than 8 bytes).
    pub async fn read_keyboard(&mut self) -> Result<Option<KeyboardReport>, HidError> {
        let mut buf = [0u8; 8];
        self.in_ch.request_in(&mut buf).await?;
        Ok(KeyboardReport::parse(&buf))
    }

    /// Read and parse a boot-protocol mouse report.
    ///
    /// Call [`HidHost::set_protocol`] with [`PROTOCOL_BOOT`] first.
    /// Returns `None` if the report is malformed (shorter than 3 bytes).
    pub async fn read_mouse(&mut self) -> Result<Option<MouseReport>, HidError> {
        let mut buf = [0u8; 4];
        // Some mice send only 3 bytes; read up to 4.
        let n = self.in_ch.request_in(&mut buf).await?;
        Ok(MouseReport::parse(&buf[..n]))
    }

    /// Issue a GET_REPORT control request.
    ///
    /// `report_type`: 1=Input, 2=Output, 3=Feature.
    /// `report_id`: 0 if the device uses a single report.
    ///
    /// Returns the number of bytes received.
    pub async fn get_report(&mut self, report_type: u8, report_id: u8, buf: &mut [u8]) -> Result<usize, HidError> {
        let value = (report_type as u16) << 8 | report_id as u16;
        let setup_bytes =
            crate::control::class_interface_in_with_data(GET_REPORT, value, self.interface as u16, buf.len() as u16);
        let setup = bytes_to_setup(&setup_bytes);
        let n = self.ctrl_ch.control_in(&setup, buf).await?;
        Ok(n)
    }
}
