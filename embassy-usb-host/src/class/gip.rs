//! GIP (Gaming Input Protocol) host class driver.
//!
//! Supports Xbox One, Xbox Series X|S, and compatible controllers
//! using Microsoft's Gaming Input Protocol ([MS-GIPUSB]) over USB.
//!
//! # Architecture
//!
//! [`GipHost`] handles the common GIP protocol framing — headers, ACKs,
//! sequence numbers, Hello handshake, and channel management.
//! Device-specific behavior (init sequences, input parsing quirks) is
//! delegated to a [`GipDevice`] trait implementation.
//!
//! A built-in [`XboxOneSGamepad`] covers the Xbox One S controller and
//! is broadly compatible with other standard GIP gamepads.
//!
//! # GIP lifecycle
//!
//! After USB enumeration, [`GipHost::try_register`] performs the full GIP
//! handshake:
//!
//! 1. The controller enters the **Arrival** state and sends Hello (0x02)
//!    messages every ~500 ms.
//! 2. The driver responds with the device-specific init packets from
//!    [`GipDevice::init_packets`] (e.g., power-on).
//! 3. Remaining handshake traffic (ACKs, status) is drained until the
//!    controller transitions to the **Active** state.
//!
//! Once `try_register` returns successfully, the driver is ready for
//! [`GipHost::poll`].
//!
//! # Example
//!
//! Rumble while the A button is held:
//!
//! ```ignore
//! use embassy_usb_host::class::gip::{GipHost, XboxOneSGamepad, GipEvent, RumbleCommand};
//!
//! let mut gip = GipHost::<_, XboxOneSGamepad>::try_register(
//!     host.driver(),
//!     &config_buf[..config_len],
//!     addr,
//!     dev_desc.vendor_id,
//!     dev_desc.product_id,
//! ).await?;
//!
//! let mut rumbling = false;
//! loop {
//!     match gip.poll().await? {
//!         GipEvent::Input(report) => {
//!             if report.a && !rumbling {
//!                 gip.set_rumble(&RumbleCommand {
//!                     strong: 128,
//!                     weak: 64,
//!                     ..Default::default()
//!                 }).await?;
//!                 rumbling = true;
//!             } else if !report.a && rumbling {
//!                 gip.stop_rumble().await?;
//!                 rumbling = false;
//!             }
//!         }
//!         _ => {}
//!     }
//! }
//! ```
//!
//! [MS-GIPUSB]: https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-gipusb/

use embassy_usb_driver::host::{PipeError, UsbHostDriver, UsbPipe, pipe};
use embassy_usb_driver::{Direction as UsbDirection, EndpointAddress, EndpointInfo, EndpointType};

use crate::descriptor::ConfigurationDescriptor;
use crate::handler::EnumerationInfo;

// ── GIP USB interface identifiers ────────────────────────────────────────────

const GIP_IFACE_CLASS: u8 = 0xFF; // Vendor-specific
const GIP_IFACE_SUBCLASS: u8 = 0x47; // 71 decimal
const GIP_IFACE_PROTOCOL: u8 = 0xD0; // 208 decimal
const GIP_DATA_INTERFACE: u8 = 0;
const TRANSFER_TYPE_INTERRUPT: u8 = 0x03;

// ── GIP message types (MS-GIPUSB §2.2) ──────────────────────────────────────

const GIP_CMD_ACK: u8 = 0x01;
const GIP_CMD_HELLO: u8 = 0x02;
const GIP_CMD_VIRTUAL_KEY: u8 = 0x07;
const GIP_CMD_RUMBLE: u8 = 0x09;
const GIP_CMD_INPUT: u8 = 0x20;

// ── GIP flags ────────────────────────────────────────────────────────────────

const GIP_FLAG_ACK: u8 = 0x10;
const GIP_FLAG_INTERNAL: u8 = 0x20;

/// Maximum GIP packet size (matches the Command/Low-Latency MTU).
pub const GIP_MAX_PACKET: usize = 64;

/// Minimum GIP header length.
const GIP_HEADER_LEN: usize = 4;

// ── Public types ─────────────────────────────────────────────────────────────

/// Parsed gamepad input report from a GIP controller.
///
/// Axes use the native ranges reported by the hardware:
/// triggers are 0–1023, sticks are −32768–32767.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct GamepadReport {
    /// D-pad up.
    pub dpad_up: bool,
    /// D-pad down.
    pub dpad_down: bool,
    /// D-pad left.
    pub dpad_left: bool,
    /// D-pad right.
    pub dpad_right: bool,
    /// A button.
    pub a: bool,
    /// B button.
    pub b: bool,
    /// X button.
    pub x: bool,
    /// Y button.
    pub y: bool,
    /// Left bumper (LB).
    pub left_bumper: bool,
    /// Right bumper (RB).
    pub right_bumper: bool,
    /// Left stick click.
    pub left_stick_press: bool,
    /// Right stick click.
    pub right_stick_press: bool,
    /// Menu button (≡).
    pub menu: bool,
    /// View button (⧉).
    pub view: bool,
    /// Left trigger analog value (0–1023).
    pub left_trigger: u16,
    /// Right trigger analog value (0–1023).
    pub right_trigger: u16,
    /// Left stick X axis (−32768–32767, positive = right).
    pub left_stick_x: i16,
    /// Left stick Y axis (−32768–32767, positive = up).
    pub left_stick_y: i16,
    /// Right stick X axis (−32768–32767, positive = right).
    pub right_stick_x: i16,
    /// Right stick Y axis (−32768–32767, positive = up).
    pub right_stick_y: i16,
}

/// Rumble motor intensities.
///
/// All values are 0–255 where 0 is off and 255 is maximum.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct RumbleCommand {
    /// Left trigger impulse motor.
    pub left_trigger: u8,
    /// Right trigger impulse motor.
    pub right_trigger: u8,
    /// Strong (left) rumble motor.
    pub strong: u8,
    /// Weak (right) rumble motor.
    pub weak: u8,
}

/// Event produced by [`GipHost::poll`].
#[derive(Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum GipEvent {
    /// Standard gamepad input report (message type 0x20).
    Input(GamepadReport),
    /// Xbox / Guide button state changed (message type 0x07).
    GuideButton(bool),
}

/// GIP host class driver error.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum GipError {
    /// USB transfer error.
    Transfer(PipeError),
    /// No GIP interface found in the configuration descriptor.
    NoInterface,
    /// No free USB pipe available.
    NoPipe,
    /// The [`GipDevice`] implementation rejected this VID/PID.
    UnsupportedDevice,
}

impl From<PipeError> for GipError {
    fn from(e: PipeError) -> Self {
        Self::Transfer(e)
    }
}

impl core::fmt::Display for GipError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Transfer(_e) => write!(f, "Transfer error"),
            Self::NoInterface => write!(f, "No GIP interface found"),
            Self::NoPipe => write!(f, "No free pipe"),
            Self::UnsupportedDevice => write!(f, "Unsupported GIP device"),
        }
    }
}

impl core::error::Error for GipError {}

// ── Descriptor discovery ─────────────────────────────────────────────────────

/// Information about a GIP data interface found in a configuration descriptor.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct GipInterfaceInfo {
    /// Interface number.
    pub interface_number: u8,
    /// Interrupt IN endpoint address (raw, with direction bit).
    pub interrupt_in_ep: u8,
    /// Interrupt IN max packet size.
    pub interrupt_in_mps: u16,
    /// Interrupt IN polling interval (from endpoint descriptor).
    pub interrupt_in_interval: u8,
    /// Interrupt OUT endpoint address (raw).
    pub interrupt_out_ep: u8,
    /// Interrupt OUT max packet size.
    pub interrupt_out_mps: u16,
    /// Interrupt OUT polling interval (from endpoint descriptor).
    pub interrupt_out_interval: u8,
}

/// Locate the GIP data interface in a configuration descriptor.
///
/// Matches vendor-specific class (0xFF), GIP subclass (0x47),
/// GIP protocol (0xD0), interface number 0.
pub fn find_gip(config_desc: &[u8]) -> Option<GipInterfaceInfo> {
    let cfg = ConfigurationDescriptor::try_from_slice(config_desc).ok()?;

    for iface in cfg.iter_interface() {
        if iface.interface_class != GIP_IFACE_CLASS
            || iface.interface_subclass != GIP_IFACE_SUBCLASS
            || iface.interface_protocol != GIP_IFACE_PROTOCOL
        {
            continue;
        }

        if iface.interface_number != GIP_DATA_INTERFACE {
            continue;
        }

        let mut in_ep: Option<(u8, u16, u8)> = None;
        let mut out_ep: Option<(u8, u16, u8)> = None;

        for ep in iface.iter_endpoints() {
            if ep.transfer_type() == TRANSFER_TYPE_INTERRUPT {
                if ep.is_in() {
                    in_ep = Some((ep.endpoint_address, ep.max_packet_size, ep.interval));
                } else {
                    out_ep = Some((ep.endpoint_address, ep.max_packet_size, ep.interval));
                }
            }
        }

        if let (Some((in_addr, in_mps, in_interval)), Some((out_addr, out_mps, out_interval))) = (in_ep, out_ep) {
            return Some(GipInterfaceInfo {
                interface_number: iface.interface_number,
                interrupt_in_ep: in_addr,
                interrupt_in_mps: in_mps,
                interrupt_in_interval: in_interval,
                interrupt_out_ep: out_addr,
                interrupt_out_mps: out_mps,
                interrupt_out_interval: out_interval,
            });
        }
    }

    None
}

// ── Standard GIP parsing helpers ─────────────────────────────────────────────

/// Parse a standard GIP gamepad input report (message type 0x20).
///
/// Expects the full packet including the 4-byte GIP header.
/// Layout (offsets from packet start):
///
/// | Offset | Content                             |
/// |--------|-------------------------------------|
/// | 0      | Message type (0x20)                 |
/// | 1      | Flags                               |
/// | 2      | Sequence                            |
/// | 3      | Payload length (0x0E = 14)          |
/// | 4      | Buttons 0: menu, view, A, B, X, Y   |
/// | 5      | Buttons 1: dpad, LB, RB, LS, RS     |
/// | 6–7    | Left trigger (u16 LE, 0–1023)       |
/// | 8–9    | Right trigger (u16 LE, 0–1023)      |
/// | 10–11  | Left stick X (i16 LE)               |
/// | 12–13  | Left stick Y (i16 LE)               |
/// | 14–15  | Right stick X (i16 LE)              |
/// | 16–17  | Right stick Y (i16 LE)              |
pub fn parse_standard_input(data: &[u8]) -> Option<GamepadReport> {
    if data.len() < 18 {
        return None;
    }

    Some(GamepadReport {
        menu: data[4] & (1 << 2) != 0,
        view: data[4] & (1 << 3) != 0,
        a: data[4] & (1 << 4) != 0,
        b: data[4] & (1 << 5) != 0,
        x: data[4] & (1 << 6) != 0,
        y: data[4] & (1 << 7) != 0,

        dpad_up: data[5] & (1 << 0) != 0,
        dpad_down: data[5] & (1 << 1) != 0,
        dpad_left: data[5] & (1 << 2) != 0,
        dpad_right: data[5] & (1 << 3) != 0,
        left_bumper: data[5] & (1 << 4) != 0,
        right_bumper: data[5] & (1 << 5) != 0,
        left_stick_press: data[5] & (1 << 6) != 0,
        right_stick_press: data[5] & (1 << 7) != 0,

        left_trigger: u16::from_le_bytes([data[6], data[7]]),
        right_trigger: u16::from_le_bytes([data[8], data[9]]),

        left_stick_x: i16::from_le_bytes([data[10], data[11]]),
        left_stick_y: i16::from_le_bytes([data[12], data[13]]),
        right_stick_x: i16::from_le_bytes([data[14], data[15]]),
        right_stick_y: i16::from_le_bytes([data[16], data[17]]),
    })
}

/// Build a standard GIP rumble packet (message type 0x09, 9-byte payload).
///
/// Returns the total packet length (13 bytes).
pub fn build_standard_rumble(buf: &mut [u8; GIP_MAX_PACKET], seq: u8, cmd: &RumbleCommand) -> usize {
    const MOTOR_ALL: u8 = 0x0F;
    buf[0] = GIP_CMD_RUMBLE;
    buf[1] = 0x00;
    buf[2] = seq;
    buf[3] = 0x09; // payload length
    buf[4] = 0x00; // sub-command
    buf[5] = MOTOR_ALL;
    buf[6] = cmd.left_trigger;
    buf[7] = cmd.right_trigger;
    buf[8] = cmd.strong;
    buf[9] = cmd.weak;
    buf[10] = 0xFF; // on period
    buf[11] = 0x00; // off period
    buf[12] = 0xFF; // repeat count
    13
}

// ── GipDevice trait ──────────────────────────────────────────────────────────

/// Trait for GIP device-specific behavior.
///
/// Different Xbox One–family controllers require different initialization
/// sequences and may have variant input report formats (e.g., Elite paddle
/// data, Share button location). Implement this trait to add support for a
/// specific controller or family.
///
/// The built-in [`XboxOneSGamepad`] handles the Xbox One S and is broadly
/// compatible with most standard GIP gamepads.
pub trait GipDevice: Sized {
    /// Attempt to create a device handler for the given USB device.
    ///
    /// Returns `None` if this implementation does not support the device.
    /// Implementations may capture VID/PID to vary behavior at runtime
    /// (e.g., selecting init packets based on product ID).
    fn try_new(vendor_id: u16, product_id: u16) -> Option<Self>;

    /// GIP init packets sent in response to the device's Hello message.
    ///
    /// Each entry is a complete GIP message (header + payload).
    /// The driver patches byte 2 (sequence number) with an incrementing
    /// value before transmission.
    fn init_packets(&self) -> &'static [&'static [u8]];

    /// Parse a GIP input report (message type 0x20) into a [`GamepadReport`].
    ///
    /// `data` is the full GIP packet including the 4-byte header.
    /// Return `None` to silently drop the message.
    ///
    /// The default delegates to [`parse_standard_input`].
    fn parse_input(&self, data: &[u8]) -> Option<GamepadReport> {
        parse_standard_input(data)
    }

    /// Parse a virtual key report (message type 0x07) into a guide button state.
    ///
    /// The default reads bits 0–1 of byte 4.
    fn parse_guide_button(&self, data: &[u8]) -> Option<bool> {
        if data.len() >= 5 {
            Some(data[4] & 0x03 != 0)
        } else {
            None
        }
    }

    /// Build a rumble command packet into `buf`.
    ///
    /// Returns the total number of bytes written.
    /// The default delegates to [`build_standard_rumble`].
    fn build_rumble(&self, buf: &mut [u8; GIP_MAX_PACKET], seq: u8, cmd: &RumbleCommand) -> usize {
        build_standard_rumble(buf, seq, cmd)
    }
}

// ── Xbox One S implementation ────────────────────────────────────────────────

/// GIP device implementation for the Xbox One S controller.
///
/// Handles the Microsoft Xbox One S pad (`045e:02ea`) and is broadly
/// compatible with most standard GIP gamepads including:
///
/// - Xbox One original (`045e:02d1`)
/// - Xbox One S (`045e:02ea`)
/// - Xbox One (2015 firmware) (`045e:02dd`)
/// - Xbox Series X|S (`045e:0b12`)
/// - Most third-party GIP controllers
///
/// For controllers with non-standard features (Elite paddles, custom
/// firmware packets), implement [`GipDevice`] directly.
pub struct XboxOneSGamepad {
    extended_init: bool,
}

/// Power-on packet sent to all Xbox One controllers.
static POWER_ON: &[u8] = &[0x05, 0x20, 0x00, 0x01, 0x00];

/// Additional init packet for Xbox One S and Elite Series 2 controllers.
/// Needed when the controller was previously used in Bluetooth mode.
static S_INIT: &[u8] = &[0x05, 0x20, 0x00, 0x0f, 0x06];

static INIT_STANDARD: &[&[u8]] = &[POWER_ON];
static INIT_EXTENDED: &[&[u8]] = &[POWER_ON, S_INIT];

impl GipDevice for XboxOneSGamepad {
    fn try_new(vendor_id: u16, product_id: u16) -> Option<Self> {
        let extended_init = vendor_id == 0x045e
            && matches!(
                product_id,
                0x02ea  // Xbox One S
                | 0x0b00 // Xbox One Elite Series 2
                | 0x0b12 // Xbox Series X|S
            );

        Some(XboxOneSGamepad { extended_init })
    }

    fn init_packets(&self) -> &'static [&'static [u8]] {
        if self.extended_init {
            INIT_EXTENDED
        } else {
            INIT_STANDARD
        }
    }
}

// ── GipHost driver ───────────────────────────────────────────────────────────

/// GIP host class driver.
///
/// Generic over the USB host driver `D` and a [`GipDevice`] implementation
/// that customizes init sequences and input parsing.
///
/// # Lifecycle
///
/// 1. Register with [`GipHost::try_register`] after USB enumeration.
/// 2. Poll for events with [`GipHost::poll`].
/// 3. Optionally send rumble with [`GipHost::set_rumble`].
pub struct GipHost<'d, D: UsbHostDriver<'d>, DEV: GipDevice> {
    in_ch: D::Pipe<pipe::Interrupt, pipe::In>,
    out_ch: D::Pipe<pipe::Interrupt, pipe::Out>,
    seq: u8,
    device: DEV,
    _phantom: core::marker::PhantomData<&'d ()>,
}

impl<'d, D: UsbHostDriver<'d>, DEV: GipDevice> GipHost<'d, D, DEV> {
    /// Create and initialize a GIP host driver.
    ///
    /// Performs the full setup sequence:
    /// 1. Validates the device via [`GipDevice::try_new`].
    /// 2. Locates the GIP data interface in the configuration descriptor.
    /// 3. Allocates interrupt IN and OUT pipes.
    /// 4. Completes the GIP handshake: waits for the device's Hello
    ///    message, responds with [`GipDevice::init_packets`], and drains
    ///    remaining handshake traffic until the controller is active.
    ///
    /// On success the driver is ready for [`poll`](Self::poll).
    ///
    /// # Errors
    ///
    /// - [`GipError::UnsupportedDevice`] if [`GipDevice::try_new`] returns `None`.
    /// - [`GipError::NoInterface`] if no GIP interface is found.
    /// - [`GipError::NoPipe`] if pipes cannot be allocated.
    /// - [`GipError::Transfer`] if a handshake transfer fails.
    pub async fn try_register(driver: &D, config_desc: &[u8], enum_info: &EnumerationInfo) -> Result<Self, GipError> {
        let vendor_id = enum_info.device_desc.vendor_id;
        let product_id = enum_info.device_desc.product_id;
        let device = DEV::try_new(vendor_id, product_id).ok_or(GipError::UnsupportedDevice)?;

        let info = find_gip(config_desc).ok_or(GipError::NoInterface)?;

        let in_ep_info = EndpointInfo {
            addr: EndpointAddress::from_parts((info.interrupt_in_ep & 0x0F) as usize, UsbDirection::In),
            ep_type: EndpointType::Interrupt,
            max_packet_size: info.interrupt_in_mps,
            interval_ms: info.interrupt_in_interval,
        };

        let out_ep_info = EndpointInfo {
            addr: EndpointAddress::from_parts((info.interrupt_out_ep & 0x0F) as usize, UsbDirection::Out),
            ep_type: EndpointType::Interrupt,
            max_packet_size: info.interrupt_out_mps,
            interval_ms: info.interrupt_out_interval,
        };

        let device_address = enum_info.device_address;
        let split = enum_info.split();

        let in_ch = driver
            .alloc_pipe::<pipe::Interrupt, pipe::In>(device_address, &in_ep_info, split)
            .map_err(|_| GipError::NoPipe)?;
        let out_ch = driver
            .alloc_pipe::<pipe::Interrupt, pipe::Out>(device_address, &out_ep_info, split)
            .map_err(|_| GipError::NoPipe)?;

        let mut host = Self {
            in_ch,
            out_ch,
            seq: 1,
            device,
            _phantom: core::marker::PhantomData,
        };

        host.init_device().await?;

        Ok(host)
    }

    async fn send_init_packets(&mut self) -> Result<(), GipError> {
        for packet in self.device.init_packets() {
            let mut buf = [0u8; GIP_MAX_PACKET];
            let len = packet.len().min(GIP_MAX_PACKET);
            buf[..len].copy_from_slice(&packet[..len]);
            buf[2] = self.next_seq();
            self.out_ch.request_out(&buf[..len], true).await?;
        }
        Ok(())
    }

    /// Complete the GIP handshake with the controller.
    ///
    /// After USB enumeration the controller enters the Arrival state and
    /// sends Hello (0x02) messages every ~500 ms until the host responds.
    /// This method waits for those Hello messages, responds with the
    /// device-specific init packets, and drains remaining handshake
    /// traffic (ACKs, status) until the controller transitions to the
    /// Active state or a maximum number of reads is reached.
    async fn init_device(&mut self) -> Result<(), GipError> {
        const MAX_DRAIN: usize = 16;
        let mut buf = [0u8; GIP_MAX_PACKET];

        for _ in 0..MAX_DRAIN {
            let n = self.in_ch.request_in(&mut buf).await?;
            if n < GIP_HEADER_LEN {
                debug!("GIP init drain: received short packet ({}B), ignoring", n);
                continue;
            }

            let msg_type = buf[0];
            let flags = buf[1];

            trace!(
                "GIP init drain: type={:02x} flags={:02x} seq={:02x} len={:02x} ({}B)",
                msg_type, flags, buf[2], buf[3], n
            );

            if flags & GIP_FLAG_ACK != 0 {
                self.send_ack(msg_type, flags, buf[2], buf[3]).await?;
            }

            match msg_type {
                GIP_CMD_HELLO => {
                    debug!("GIP device hello, sending init sequence");
                    self.send_init_packets().await?;
                }
                GIP_CMD_ACK => {}
                _ => {
                    trace!("GIP init handshake complete (saw type 0x{:02x})", msg_type);
                    return Ok(());
                }
            }
        }

        warn!("GIP init drain: hit max iterations without seeing non-hello traffic");
        Ok(())
    }

    /// Poll for the next GIP event.
    ///
    /// Reads from the interrupt IN endpoint, parses the GIP header, and
    /// dispatches to the [`GipDevice`] for interpretation.
    ///
    /// Any incoming message with the ACK flag (0x10) set is automatically
    /// acknowledged before being processed. This is required by the GIP
    /// protocol — controllers stall and stop sending input if the host
    /// fails to acknowledge flagged messages.
    ///
    /// This method blocks until a meaningful event is available. Protocol
    /// messages that don't produce user-facing events are acknowledged
    /// and consumed internally.
    pub async fn poll(&mut self) -> Result<GipEvent, GipError> {
        let mut buf = [0u8; GIP_MAX_PACKET];
        loop {
            let n = self.in_ch.request_in(&mut buf).await?;
            if n < GIP_HEADER_LEN {
                debug!("GIP poll: received short packet ({}B), ignoring", n);
                continue;
            }

            let msg_type = buf[0];
            let flags = buf[1];

            trace!(
                "GIP rx: type={:02x} flags={:02x} seq={:02x} len={:02x} ({}B)",
                msg_type, flags, buf[2], buf[3], n
            );

            if flags & GIP_FLAG_ACK != 0 {
                self.send_ack(msg_type, flags, buf[2], buf[3]).await?;
            }

            match msg_type {
                GIP_CMD_INPUT => {
                    if let Some(report) = self.device.parse_input(&buf[..n]) {
                        return Ok(GipEvent::Input(report));
                    }
                }
                GIP_CMD_VIRTUAL_KEY => {
                    if let Some(pressed) = self.device.parse_guide_button(&buf[..n]) {
                        return Ok(GipEvent::GuideButton(pressed));
                    }
                }
                _ => {
                    trace!("GIP message type 0x{:02x} (consumed)", msg_type);
                }
            }
        }
    }

    /// Read a raw GIP packet from the device.
    ///
    /// Returns the number of bytes received. No parsing or ACK handling
    /// is performed; use [`poll`](Self::poll) for the high-level interface.
    pub async fn read_raw(&mut self, buf: &mut [u8]) -> Result<usize, GipError> {
        let n = self.in_ch.request_in(buf).await?;
        Ok(n)
    }

    /// Send a rumble command to the controller.
    pub async fn set_rumble(&mut self, cmd: &RumbleCommand) -> Result<(), GipError> {
        let mut buf = [0u8; GIP_MAX_PACKET];
        let seq = self.next_seq();
        let len = self.device.build_rumble(&mut buf, seq, cmd);
        self.out_ch.request_out(&buf[..len], true).await?;
        Ok(())
    }

    /// Stop all rumble motors.
    pub async fn stop_rumble(&mut self) -> Result<(), GipError> {
        self.set_rumble(&RumbleCommand::default()).await
    }

    /// Send a raw GIP packet.
    ///
    /// The sequence byte (offset 2) is **not** patched; the caller
    /// is responsible for the full packet contents.
    pub async fn write_raw(&mut self, data: &[u8]) -> Result<(), GipError> {
        self.out_ch.request_out(data, true).await?;
        Ok(())
    }

    /// Get a reference to the underlying [`GipDevice`].
    pub fn device(&self) -> &DEV {
        &self.device
    }

    fn next_seq(&mut self) -> u8 {
        let s = self.seq;
        self.seq = self.seq.wrapping_add(1);
        if self.seq == 0 {
            self.seq = 1;
        }
        s
    }

    /// Send a GIP ACK packet mirroring the incoming message's header fields.
    ///
    /// The ACK format (13 bytes = 4-byte header + 9-byte payload):
    /// ```text
    /// [0x01] [0x20] [seq] [0x09]  [0x00] [orig_type] [orig_flags] [orig_len] [0x00 ×5]
    ///  ACK   INTERNAL seq   plen    ---    echoed from incoming message        padding
    /// ```
    async fn send_ack(
        &mut self,
        orig_type: u8,
        orig_flags: u8,
        orig_seq: u8,
        orig_payload_len: u8,
    ) -> Result<(), GipError> {
        let ack: [u8; 13] = [
            GIP_CMD_ACK,
            GIP_FLAG_INTERNAL,
            orig_seq,
            0x09, // ACK payload is always 9 bytes
            0x00,
            orig_type,
            orig_flags,
            orig_payload_len,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
        ];
        self.out_ch.request_out(&ack, true).await?;
        Ok(())
    }
}
