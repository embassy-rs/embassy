//! Gamepad reports and standard USB HID gamepad descriptor.

/// 16-bit button bitmask used in [`GamepadReport`].
pub type GamepadButtons = u16;

/// Standard USB HID Gamepad report with 8-bit axes, hat switch, and 16 buttons (7 bytes).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct GamepadReport {
    /// Left stick horizontal axis (signed, positive = right).
    pub x: i8,
    /// Left stick vertical axis (signed, positive = down).
    pub y: i8,
    /// Right stick horizontal axis (signed, positive = right).
    pub z: i8,
    /// Right stick vertical axis (signed, positive = down).
    pub rz: i8,
    /// 8-direction hat switch (POV D-pad).
    ///
    /// Values 1–8 correspond to 45° increments starting from Up (0°):
    /// - 1 = Up (0°)
    /// - 2 = Up-Right (45°)
    /// - 3 = Right (90°)
    /// - 4 = Down-Right (135°)
    /// - 5 = Down (180°)
    /// - 6 = Down-Left (225°)
    /// - 7 = Left (270°)
    /// - 8 = Up-Left (315°)
    /// - 0 = Released / centered (Null state)
    pub hat: u8,
    /// Button bitmask (buttons 1–16).
    pub buttons: GamepadButtons,
}

impl GamepadReport {
    // ── Hat switch positions ──────────────────────────────────────────────────
    /// Hat switch released / centered (null state).
    pub const HAT_CENTER: u8 = 0;
    /// Hat switch Up (0°).
    pub const HAT_UP: u8 = 1;
    /// Hat switch Up-Right (45°).
    pub const HAT_UP_RIGHT: u8 = 2;
    /// Hat switch Right (90°).
    pub const HAT_RIGHT: u8 = 3;
    /// Hat switch Down-Right (135°).
    pub const HAT_DOWN_RIGHT: u8 = 4;
    /// Hat switch Down (180°).
    pub const HAT_DOWN: u8 = 5;
    /// Hat switch Down-Left (225°).
    pub const HAT_DOWN_LEFT: u8 = 6;
    /// Hat switch Left (270°).
    pub const HAT_LEFT: u8 = 7;
    /// Hat switch Up-Left (315°).
    pub const HAT_UP_LEFT: u8 = 8;

    // ── Button bit constants ──────────────────────────────────────────────────
    /// Button 1
    pub const BUTTON_1: GamepadButtons = 1 << 0;
    /// Button 2
    pub const BUTTON_2: GamepadButtons = 1 << 1;
    /// Button 3
    pub const BUTTON_3: GamepadButtons = 1 << 2;
    /// Button 4
    pub const BUTTON_4: GamepadButtons = 1 << 3;
    /// Button 5
    pub const BUTTON_5: GamepadButtons = 1 << 4;
    /// Button 6
    pub const BUTTON_6: GamepadButtons = 1 << 5;
    /// Button 7
    pub const BUTTON_7: GamepadButtons = 1 << 6;
    /// Button 8
    pub const BUTTON_8: GamepadButtons = 1 << 7;
    /// Button 9
    pub const BUTTON_9: GamepadButtons = 1 << 8;
    /// Button 10
    pub const BUTTON_10: GamepadButtons = 1 << 9;
    /// Button 11
    pub const BUTTON_11: GamepadButtons = 1 << 10;
    /// Button 12
    pub const BUTTON_12: GamepadButtons = 1 << 11;
    /// Button 13
    pub const BUTTON_13: GamepadButtons = 1 << 12;
    /// Button 14
    pub const BUTTON_14: GamepadButtons = 1 << 13;
    /// Button 15
    pub const BUTTON_15: GamepadButtons = 1 << 14;
    /// Button 16
    pub const BUTTON_16: GamepadButtons = 1 << 15;

    // ── Conventional Gamepad Aliases ──────────────────────────────────────────
    /// Primary action button (A / Cross).
    pub const BUTTON_A: GamepadButtons = Self::BUTTON_1;
    /// Secondary action button (B / Circle).
    pub const BUTTON_B: GamepadButtons = Self::BUTTON_2;
    /// Tertiary action button (X / Square).
    pub const BUTTON_X: GamepadButtons = Self::BUTTON_3;
    /// Quaternary action button (Y / Triangle).
    pub const BUTTON_Y: GamepadButtons = Self::BUTTON_4;
    /// Left bumper / shoulder button (L1 / LB).
    pub const BUTTON_TL: GamepadButtons = Self::BUTTON_5;
    /// Right bumper / shoulder button (R1 / RB).
    pub const BUTTON_TR: GamepadButtons = Self::BUTTON_6;
    /// Left trigger digital button (L2 / LT).
    pub const BUTTON_TL2: GamepadButtons = Self::BUTTON_7;
    /// Right trigger digital button (R2 / RT).
    pub const BUTTON_TR2: GamepadButtons = Self::BUTTON_8;
    /// Select / Back / Share button.
    pub const BUTTON_SELECT: GamepadButtons = Self::BUTTON_9;
    /// Start / Options button.
    pub const BUTTON_START: GamepadButtons = Self::BUTTON_10;
    /// Mode / Home / Guide button.
    pub const BUTTON_MODE: GamepadButtons = Self::BUTTON_11;
    /// Left stick click (L3 / Thumbl).
    pub const BUTTON_THUMBL: GamepadButtons = Self::BUTTON_12;
    /// Right stick click (R3 / Thumbr).
    pub const BUTTON_THUMBR: GamepadButtons = Self::BUTTON_13;

    /// Standard USB HID Gamepad report descriptor without report ID (7 bytes input report).
    pub const DESCRIPTOR: &'static [u8] = &[
        0x05, 0x01, // Usage Page (Generic Desktop)
        0x09, 0x05, // Usage (Gamepad)
        0xa1, 0x01, // Collection (Application)
        // 4 Analog Axes: X, Y, Z, Rz (-127 to 127)
        0xa1, 0x00, //   Collection (Physical)
        0x05, 0x01, //     Usage Page (Generic Desktop)
        0x09, 0x30, //     Usage (X)
        0x09, 0x31, //     Usage (Y)
        0x09, 0x32, //     Usage (Z)
        0x09, 0x35, //     Usage (Rz)
        0x15, 0x81, //     Logical Minimum (-127)
        0x25, 0x7f, //     Logical Maximum (127)
        0x75, 0x08, //     Report Size (8)
        0x95, 0x04, //     Report Count (4)
        0x81, 0x02, //     Input (Data, Variable, Absolute)
        0xc0, //   End Collection
        // 8-directional Hat Switch (POV D-pad)
        0x05, 0x01, //   Usage Page (Generic Desktop)
        0x09, 0x39, //   Usage (Hat switch)
        0x15, 0x01, //   Logical Minimum (1)
        0x25, 0x08, //   Logical Maximum (8)
        0x35, 0x00, //   Physical Minimum (0)
        0x46, 0x3b, 0x01, // Physical Maximum (315)
        0x65, 0x14, //   Unit (Eng Rot: Degrees)
        0x75, 0x04, //   Report Size (4)
        0x95, 0x01, //   Report Count (1)
        0x81, 0x42, //   Input (Data, Variable, Absolute, Null State)
        // 4 bits padding to byte alignment
        0x75, 0x04, //   Report Size (4)
        0x95, 0x01, //   Report Count (1)
        0x81, 0x01, //   Input (Constant)
        // 16 Buttons
        0x05, 0x09, //   Usage Page (Button)
        0x19, 0x01, //   Usage Minimum (1)
        0x29, 0x10, //   Usage Maximum (16)
        0x15, 0x00, //   Logical Minimum (0)
        0x25, 0x01, //   Logical Maximum (1)
        0x75, 0x01, //   Report Size (1)
        0x95, 0x10, //   Report Count (16)
        0x81, 0x02, //   Input (Data, Variable, Absolute)
        0xc0, // End Collection
    ];

    /// Parse a gamepad report from a byte slice.
    /// Returns `None` if `buf` is shorter than 7 bytes.
    pub fn parse(buf: &[u8]) -> Option<Self> {
        if buf.len() < 7 {
            return None;
        }
        let hat_raw = buf[4] & 0x0F;
        let buttons = u16::from_le_bytes([buf[5], buf[6]]);
        Some(Self {
            x: buf[0] as i8,
            y: buf[1] as i8,
            z: buf[2] as i8,
            rz: buf[3] as i8,
            hat: hat_raw,
            buttons,
        })
    }

    /// Serialize the gamepad report into its 7-byte wire format.
    pub const fn to_bytes(&self) -> [u8; 7] {
        let btn_bytes = self.buttons.to_le_bytes();
        [
            self.x as u8,
            self.y as u8,
            self.z as u8,
            self.rz as u8,
            self.hat & 0x0F,
            btn_bytes[0],
            btn_bytes[1],
        ]
    }

    /// Returns `true` if the specified button is pressed.
    pub fn is_button_pressed(&self, button: GamepadButtons) -> bool {
        self.buttons & button != 0
    }
}

#[cfg(all(test, feature = "host"))]
mod tests {
    use super::*;
    use crate::class::hid::host::hid_report::{ReportDescriptor, usage, usage_page};

    #[test]
    fn gamepad_report_round_trips() {
        let report = GamepadReport {
            x: -127,
            y: 127,
            z: -50,
            rz: 50,
            hat: GamepadReport::HAT_DOWN_RIGHT,
            buttons: GamepadReport::BUTTON_A | GamepadReport::BUTTON_START | GamepadReport::BUTTON_16,
        };
        let bytes = report.to_bytes();
        assert_eq!(bytes.len(), 7);
        assert_eq!(GamepadReport::parse(&bytes), Some(report));
    }

    #[test]
    fn gamepad_descriptor_matches_report() {
        let desc = ReportDescriptor::<16>::parse(GamepadReport::DESCRIPTOR);
        assert!(!desc.has_report_ids);

        let report = GamepadReport {
            x: -120,
            y: 110,
            z: -42,
            rz: 88,
            hat: GamepadReport::HAT_UP,
            buttons: GamepadReport::BUTTON_A | GamepadReport::BUTTON_TR,
        };
        let bytes = report.to_bytes();

        assert_eq!(
            desc.extract_i32(&bytes, 0, usage_page::GENERIC_DESKTOP, usage::X),
            Some(-120)
        );
        assert_eq!(
            desc.extract_i32(&bytes, 0, usage_page::GENERIC_DESKTOP, usage::Y),
            Some(110)
        );
        assert_eq!(
            desc.extract_i32(&bytes, 0, usage_page::GENERIC_DESKTOP, usage::Z),
            Some(-42)
        );
        assert_eq!(
            desc.extract_i32(&bytes, 0, usage_page::GENERIC_DESKTOP, usage::RZ),
            Some(88)
        );
        assert_eq!(
            desc.extract_u32(&bytes, 0, usage_page::GENERIC_DESKTOP, usage::HAT_SWITCH),
            Some(1)
        );
        assert_eq!(desc.extract_bool(&bytes, 0, usage_page::BUTTON, 1), Some(true));
        assert_eq!(desc.extract_bool(&bytes, 0, usage_page::BUTTON, 2), Some(false));
        assert_eq!(desc.extract_bool(&bytes, 0, usage_page::BUTTON, 6), Some(true));
    }
}
