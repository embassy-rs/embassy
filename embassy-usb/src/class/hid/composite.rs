//! Composite HID multi-report descriptor and report ID wrappers.
//!
//! Allows combining Keyboard, Mouse, Consumer Control, and Gamepad reports on
//! a single HID endpoint using distinct Report IDs.

use super::boot::{KeyboardReport, MouseReport};
use super::consumer::ConsumerReport;
use super::gamepad::GamepadReport;

/// Report ID for the Keyboard report within [`COMPOSITE_DESCRIPTOR`].
pub const REPORT_ID_KEYBOARD: u8 = 1;

/// Report ID for the Mouse report within [`COMPOSITE_DESCRIPTOR`].
pub const REPORT_ID_MOUSE: u8 = 2;

/// Report ID for the Consumer Control (media keys) report within [`COMPOSITE_DESCRIPTOR`].
pub const REPORT_ID_CONSUMER: u8 = 3;

/// Report ID for the Gamepad report within [`COMPOSITE_DESCRIPTOR`].
pub const REPORT_ID_GAMEPAD: u8 = 4;

/// Composite report for a Keyboard, prefixed with [`REPORT_ID_KEYBOARD`] (9 bytes).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct IdKeyboardReport {
    /// Report ID (always [`REPORT_ID_KEYBOARD`]).
    pub report_id: u8,
    /// Keyboard report payload.
    pub report: KeyboardReport,
}

impl IdKeyboardReport {
    /// Create a new composite keyboard report.
    pub const fn new(report: KeyboardReport) -> Self {
        Self {
            report_id: REPORT_ID_KEYBOARD,
            report,
        }
    }

    /// Serialize into a 9-byte buffer.
    pub const fn to_bytes(&self) -> [u8; 9] {
        let b = self.report.to_bytes();
        [self.report_id, b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]
    }

    /// Parse a composite keyboard report from a buffer.
    /// Returns `None` if `buf.len() < 9` or if the report ID does not match.
    pub fn parse(buf: &[u8]) -> Option<Self> {
        if buf.len() < 9 || buf[0] != REPORT_ID_KEYBOARD {
            return None;
        }
        KeyboardReport::parse(&buf[1..]).map(|report| Self {
            report_id: buf[0],
            report,
        })
    }
}

/// Composite report for a Mouse, prefixed with [`REPORT_ID_MOUSE`] (5 bytes).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct IdMouseReport {
    /// Report ID (always [`REPORT_ID_MOUSE`]).
    pub report_id: u8,
    /// Mouse report payload.
    pub report: MouseReport,
}

impl IdMouseReport {
    /// Create a new composite mouse report.
    pub const fn new(report: MouseReport) -> Self {
        Self {
            report_id: REPORT_ID_MOUSE,
            report,
        }
    }

    /// Serialize into a 5-byte buffer.
    pub const fn to_bytes(&self) -> [u8; 5] {
        let b = self.report.to_bytes();
        [self.report_id, b[0], b[1], b[2], b[3]]
    }

    /// Parse a composite mouse report from a buffer.
    /// Returns `None` if `buf.len() < 5` or if the report ID does not match.
    pub fn parse(buf: &[u8]) -> Option<Self> {
        if buf.len() < 5 || buf[0] != REPORT_ID_MOUSE {
            return None;
        }
        MouseReport::parse(&buf[1..]).map(|report| Self {
            report_id: buf[0],
            report,
        })
    }
}

/// Composite report for Consumer Control, prefixed with [`REPORT_ID_CONSUMER`] (3 bytes).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct IdConsumerReport {
    /// Report ID (always [`REPORT_ID_CONSUMER`]).
    pub report_id: u8,
    /// Consumer report payload.
    pub report: ConsumerReport,
}

impl IdConsumerReport {
    /// Create a new composite consumer report.
    pub const fn new(report: ConsumerReport) -> Self {
        Self {
            report_id: REPORT_ID_CONSUMER,
            report,
        }
    }

    /// Serialize into a 3-byte buffer.
    pub const fn to_bytes(&self) -> [u8; 3] {
        let b = self.report.to_bytes();
        [self.report_id, b[0], b[1]]
    }

    /// Parse a composite consumer report from a buffer.
    /// Returns `None` if `buf.len() < 3` or if the report ID does not match.
    pub fn parse(buf: &[u8]) -> Option<Self> {
        if buf.len() < 3 || buf[0] != REPORT_ID_CONSUMER {
            return None;
        }
        ConsumerReport::parse(&buf[1..]).map(|report| Self {
            report_id: buf[0],
            report,
        })
    }
}

/// Composite report for a Gamepad, prefixed with [`REPORT_ID_GAMEPAD`] (8 bytes).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct IdGamepadReport {
    /// Report ID (always [`REPORT_ID_GAMEPAD`]).
    pub report_id: u8,
    /// Gamepad report payload.
    pub report: GamepadReport,
}

impl IdGamepadReport {
    /// Create a new composite gamepad report.
    pub const fn new(report: GamepadReport) -> Self {
        Self {
            report_id: REPORT_ID_GAMEPAD,
            report,
        }
    }

    /// Serialize into an 8-byte buffer.
    pub const fn to_bytes(&self) -> [u8; 8] {
        let b = self.report.to_bytes();
        [self.report_id, b[0], b[1], b[2], b[3], b[4], b[5], b[6]]
    }

    /// Parse a composite gamepad report from a buffer.
    /// Returns `None` if `buf.len() < 8` or if the report ID does not match.
    pub fn parse(buf: &[u8]) -> Option<Self> {
        if buf.len() < 8 || buf[0] != REPORT_ID_GAMEPAD {
            return None;
        }
        GamepadReport::parse(&buf[1..]).map(|report| Self {
            report_id: buf[0],
            report,
        })
    }
}

/// Standard composite HID report descriptor defining Keyboard, Mouse, Consumer Control,
/// and Gamepad under a single interface using Report IDs 1, 2, 3, and 4.
pub const COMPOSITE_DESCRIPTOR: &'static [u8] = &[
    // ── Report ID 1: Keyboard ─────────────────────────────────────────────────
    0x05,
    0x01, // Usage Page (Generic Desktop)
    0x09,
    0x06, // Usage (Keyboard)
    0xa1,
    0x01, // Collection (Application)
    0x85,
    REPORT_ID_KEYBOARD, //   Report ID (1)
    // Modifier byte
    0x05,
    0x07, //   Usage Page (Keyboard/Keypad)
    0x19,
    0xe0, //   Usage Minimum (Left Control)
    0x29,
    0xe7, //   Usage Maximum (Right GUI)
    0x15,
    0x00, //   Logical Minimum (0)
    0x25,
    0x01, //   Logical Maximum (1)
    0x75,
    0x01, //   Report Size (1)
    0x95,
    0x08, //   Report Count (8)
    0x81,
    0x02, //   Input (Data, Variable, Absolute)
    // Reserved byte
    0x75,
    0x08, //   Report Size (8)
    0x95,
    0x01, //   Report Count (1)
    0x81,
    0x01, //   Input (Constant)
    // LED output report
    0x05,
    0x08, //   Usage Page (LED)
    0x19,
    0x01, //   Usage Minimum (Num Lock)
    0x29,
    0x05, //   Usage Maximum (Kana)
    0x75,
    0x01, //   Report Size (1)
    0x95,
    0x05, //   Report Count (5)
    0x91,
    0x02, //   Output (Data, Variable, Absolute)
    0x75,
    0x03, //   Report Size (3)
    0x95,
    0x01, //   Report Count (1)
    0x91,
    0x01, //   Output (Constant)
    // Keycodes (6 keys)
    0x05,
    0x07, //   Usage Page (Keyboard/Keypad)
    0x19,
    0x00, //   Usage Minimum (0)
    0x29,
    0xff, //   Usage Maximum (255)
    0x15,
    0x00, //   Logical Minimum (0)
    0x26,
    0xff,
    0x00, // Logical Maximum (255)
    0x75,
    0x08, //   Report Size (8)
    0x95,
    0x06, //   Report Count (6)
    0x81,
    0x00, //   Input (Data, Array, Absolute)
    0xc0, // End Collection
    // ── Report ID 2: Mouse ────────────────────────────────────────────────────
    0x05,
    0x01, // Usage Page (Generic Desktop)
    0x09,
    0x02, // Usage (Mouse)
    0xa1,
    0x01, // Collection (Application)
    0x85,
    REPORT_ID_MOUSE, //   Report ID (2)
    0x09,
    0x01, //   Usage (Pointer)
    0xa1,
    0x00, //   Collection (Physical)
    0x05,
    0x09, //     Usage Page (Button)
    0x19,
    0x01, //     Usage Minimum (1)
    0x29,
    0x08, //     Usage Maximum (8)
    0x15,
    0x00, //     Logical Minimum (0)
    0x25,
    0x01, //     Logical Maximum (1)
    0x75,
    0x01, //     Report Size (1)
    0x95,
    0x08, //     Report Count (8)
    0x81,
    0x02, //     Input (Data, Variable, Absolute)
    0x05,
    0x01, //     Usage Page (Generic Desktop)
    0x09,
    0x30, //     Usage (X)
    0x09,
    0x31, //     Usage (Y)
    0x09,
    0x38, //     Usage (Wheel)
    0x15,
    0x81, //     Logical Minimum (-127)
    0x25,
    0x7f, //     Logical Maximum (127)
    0x75,
    0x08, //     Report Size (8)
    0x95,
    0x03, //     Report Count (3)
    0x81,
    0x06, //     Input (Data, Variable, Relative)
    0xc0, //   End Collection
    0xc0, // End Collection
    // ── Report ID 3: Consumer Control (Media Keys) ────────────────────────────
    0x05,
    0x0c, // Usage Page (Consumer)
    0x09,
    0x01, // Usage (Consumer Control)
    0xa1,
    0x01, // Collection (Application)
    0x85,
    REPORT_ID_CONSUMER, //   Report ID (3)
    0x15,
    0x00, //   Logical Minimum (0)
    0x26,
    0xff,
    0x03, // Logical Maximum (1023)
    0x19,
    0x00, //   Usage Minimum (0)
    0x2a,
    0xff,
    0x03, // Usage Maximum (1023)
    0x75,
    0x10, //   Report Size (16)
    0x95,
    0x01, //   Report Count (1)
    0x81,
    0x00, //   Input (Data, Array, Absolute)
    0xc0, // End Collection
    // ── Report ID 4: Gamepad ──────────────────────────────────────────────────
    0x05,
    0x01, // Usage Page (Generic Desktop)
    0x09,
    0x05, // Usage (Gamepad)
    0xa1,
    0x01, // Collection (Application)
    0x85,
    REPORT_ID_GAMEPAD, //   Report ID (4)
    0xa1,
    0x00, //   Collection (Physical)
    0x05,
    0x01, //     Usage Page (Generic Desktop)
    0x09,
    0x30, //     Usage (X)
    0x09,
    0x31, //     Usage (Y)
    0x09,
    0x32, //     Usage (Z)
    0x09,
    0x35, //     Usage (Rz)
    0x15,
    0x81, //     Logical Minimum (-127)
    0x25,
    0x7f, //     Logical Maximum (127)
    0x75,
    0x08, //     Report Size (8)
    0x95,
    0x04, //     Report Count (4)
    0x81,
    0x02, //     Input (Data, Variable, Absolute)
    0xc0, //   End Collection
    // Hat switch
    0x05,
    0x01, //   Usage Page (Generic Desktop)
    0x09,
    0x39, //   Usage (Hat switch)
    0x15,
    0x01, //   Logical Minimum (1)
    0x25,
    0x08, //   Logical Maximum (8)
    0x35,
    0x00, //   Physical Minimum (0)
    0x46,
    0x3b,
    0x01, // Physical Maximum (315)
    0x65,
    0x14, //   Unit (Eng Rot: Degrees)
    0x75,
    0x04, //   Report Size (4)
    0x95,
    0x01, //   Report Count (1)
    0x81,
    0x42, //   Input (Data, Variable, Absolute, Null State)
    0x75,
    0x04, //   Report Size (4)
    0x95,
    0x01, //   Report Count (1)
    0x81,
    0x01, //   Input (Constant)
    // 16 Buttons
    0x05,
    0x09, //   Usage Page (Button)
    0x19,
    0x01, //   Usage Minimum (1)
    0x29,
    0x10, //   Usage Maximum (16)
    0x15,
    0x00, //   Logical Minimum (0)
    0x25,
    0x01, //   Logical Maximum (1)
    0x75,
    0x01, //   Report Size (1)
    0x95,
    0x10, //   Report Count (16)
    0x81,
    0x02, //   Input (Data, Variable, Absolute)
    0xc0, // End Collection
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::class::hid::host::hid_report::{ReportDescriptor, usage, usage_page};

    #[test]
    fn composite_reports_round_trip() {
        let kbd = IdKeyboardReport::new(KeyboardReport {
            modifiers: 0x02,
            keycodes: [4, 5, 0, 0, 0, 0],
        });
        let kbd_bytes = kbd.to_bytes();
        assert_eq!(kbd_bytes[0], REPORT_ID_KEYBOARD);
        assert_eq!(IdKeyboardReport::parse(&kbd_bytes), Some(kbd));

        let mouse = IdMouseReport::new(MouseReport {
            buttons: MouseReport::BUTTON_LEFT,
            x: 10,
            y: -20,
            wheel: 1,
        });
        let mouse_bytes = mouse.to_bytes();
        assert_eq!(mouse_bytes[0], REPORT_ID_MOUSE);
        assert_eq!(IdMouseReport::parse(&mouse_bytes), Some(mouse));

        let consumer = IdConsumerReport::new(ConsumerReport::from_key(ConsumerReport::MUTE));
        let consumer_bytes = consumer.to_bytes();
        assert_eq!(consumer_bytes[0], REPORT_ID_CONSUMER);
        assert_eq!(IdConsumerReport::parse(&consumer_bytes), Some(consumer));

        let gamepad = IdGamepadReport::new(GamepadReport {
            x: 50,
            y: -50,
            z: 0,
            rz: 0,
            hat: GamepadReport::HAT_UP,
            buttons: GamepadReport::BUTTON_A,
        });
        let gamepad_bytes = gamepad.to_bytes();
        assert_eq!(gamepad_bytes[0], REPORT_ID_GAMEPAD);
        assert_eq!(IdGamepadReport::parse(&gamepad_bytes), Some(gamepad));
    }

    #[test]
    fn composite_descriptor_matches_all_reports() {
        let desc = ReportDescriptor::<32>::parse(COMPOSITE_DESCRIPTOR);
        assert!(desc.has_report_ids);

        // Test Mouse Report (ID 2)
        let mouse = IdMouseReport::new(MouseReport {
            buttons: MouseReport::BUTTON_LEFT | MouseReport::BUTTON_RIGHT,
            x: -25,
            y: 40,
            wheel: -2,
        });
        let mouse_bytes = mouse.to_bytes();
        assert_eq!(
            desc.extract_bool(&mouse_bytes, REPORT_ID_MOUSE, usage_page::BUTTON, 1),
            Some(true)
        );
        assert_eq!(
            desc.extract_bool(&mouse_bytes, REPORT_ID_MOUSE, usage_page::BUTTON, 2),
            Some(true)
        );
        assert_eq!(
            desc.extract_i32(&mouse_bytes, REPORT_ID_MOUSE, usage_page::GENERIC_DESKTOP, usage::X),
            Some(-25)
        );
        assert_eq!(
            desc.extract_i32(&mouse_bytes, REPORT_ID_MOUSE, usage_page::GENERIC_DESKTOP, usage::Y),
            Some(40)
        );
        assert_eq!(
            desc.extract_i32(&mouse_bytes, REPORT_ID_MOUSE, usage_page::GENERIC_DESKTOP, usage::WHEEL),
            Some(-2)
        );

        // Test Gamepad Report (ID 4)
        let gamepad = IdGamepadReport::new(GamepadReport {
            x: -100,
            y: 100,
            z: -10,
            rz: 10,
            hat: GamepadReport::HAT_LEFT,
            buttons: GamepadReport::BUTTON_X | GamepadReport::BUTTON_TR,
        });
        let gamepad_bytes = gamepad.to_bytes();
        assert_eq!(
            desc.extract_i32(&gamepad_bytes, REPORT_ID_GAMEPAD, usage_page::GENERIC_DESKTOP, usage::X),
            Some(-100)
        );
        assert_eq!(
            desc.extract_i32(&gamepad_bytes, REPORT_ID_GAMEPAD, usage_page::GENERIC_DESKTOP, usage::Y),
            Some(100)
        );
        assert_eq!(
            desc.extract_u32(
                &gamepad_bytes,
                REPORT_ID_GAMEPAD,
                usage_page::GENERIC_DESKTOP,
                usage::HAT_SWITCH
            ),
            Some(7)
        );
        assert_eq!(
            desc.extract_bool(&gamepad_bytes, REPORT_ID_GAMEPAD, usage_page::BUTTON, 3),
            Some(true)
        );
        assert_eq!(
            desc.extract_bool(&gamepad_bytes, REPORT_ID_GAMEPAD, usage_page::BUTTON, 6),
            Some(true)
        );
    }
}
