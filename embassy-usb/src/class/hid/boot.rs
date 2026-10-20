//! Keyboard and mouse reports in the layout of the HID boot protocol (HID 1.11 Appendix B).
//!
//! A device using [`KeyboardReport::DESCRIPTOR`] or [`MouseReport::DESCRIPTOR`] sends the same
//! bytes in both boot and report protocol, so it doesn't need to track the protocol mode.

/// Keyboard report in the boot protocol layout (8 bytes).
///
/// All standard USB keyboards support this layout when placed in boot protocol
/// mode via [`HidHost::set_protocol`](super::host::HidHost::set_protocol) with
/// [`HidProtocolMode::Boot`](super::HidProtocolMode::Boot).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
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
    /// Report descriptor for a keyboard sending this report.
    ///
    /// It also declares a 1-byte LED output report, which the host sends with SET_REPORT
    /// or on the interrupt OUT endpoint. Bit 0: Num Lock | Bit 1: Caps Lock |
    /// Bit 2: Scroll Lock | Bit 3: Compose | Bit 4: Kana
    pub const DESCRIPTOR: &'static [u8] = &[
        0x05, 0x01, // Usage Page (Generic Desktop)
        0x09, 0x06, // Usage (Keyboard)
        0xa1, 0x01, // Collection (Application)
        // Modifier byte.
        0x05, 0x07, //   Usage Page (Keyboard/Keypad)
        0x19, 0xe0, //   Usage Minimum (Left Control)
        0x29, 0xe7, //   Usage Maximum (Right GUI)
        0x15, 0x00, //   Logical Minimum (0)
        0x25, 0x01, //   Logical Maximum (1)
        0x75, 0x01, //   Report Size (1)
        0x95, 0x08, //   Report Count (8)
        0x81, 0x02, //   Input (Data, Variable, Absolute)
        // Reserved byte.
        0x75, 0x08, //   Report Size (8)
        0x95, 0x01, //   Report Count (1)
        0x81, 0x01, //   Input (Constant)
        // LED output report: 5 LEDs and 3 bits of padding.
        0x05, 0x08, //   Usage Page (LED)
        0x19, 0x01, //   Usage Minimum (Num Lock)
        0x29, 0x05, //   Usage Maximum (Kana)
        0x75, 0x01, //   Report Size (1)
        0x95, 0x05, //   Report Count (5)
        0x91, 0x02, //   Output (Data, Variable, Absolute)
        0x75, 0x03, //   Report Size (3)
        0x95, 0x01, //   Report Count (1)
        0x91, 0x01, //   Output (Constant)
        // Key codes.
        0x05, 0x07, //   Usage Page (Keyboard/Keypad)
        0x19, 0x00, //   Usage Minimum (0)
        0x29, 0xff, //   Usage Maximum (255)
        0x15, 0x00, //   Logical Minimum (0)
        0x26, 0xff, 0x00, //   Logical Maximum (255)
        0x75, 0x08, //   Report Size (8)
        0x95, 0x06, //   Report Count (6)
        0x81, 0x00, //   Input (Data, Array, Absolute)
        0xc0, // End Collection
    ];

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

    /// Serialize the report for sending it to the host.
    pub const fn to_bytes(&self) -> [u8; 8] {
        let k = self.keycodes;
        [self.modifiers, 0, k[0], k[1], k[2], k[3], k[4], k[5]]
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

/// Mouse report in the boot protocol layout, with a scroll wheel (4 bytes).
///
/// All standard USB mice support the first 3 bytes in boot protocol mode.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
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

    /// Report descriptor for a mouse sending this report.
    ///
    /// It declares 8 buttons, one per bit of [`MouseReport::buttons`].
    pub const DESCRIPTOR: &'static [u8] = &[
        0x05, 0x01, // Usage Page (Generic Desktop)
        0x09, 0x02, // Usage (Mouse)
        0xa1, 0x01, // Collection (Application)
        0x09, 0x01, //   Usage (Pointer)
        0xa1, 0x00, //   Collection (Physical)
        0x05, 0x09, //     Usage Page (Button)
        0x19, 0x01, //     Usage Minimum (1)
        0x29, 0x08, //     Usage Maximum (8)
        0x15, 0x00, //     Logical Minimum (0)
        0x25, 0x01, //     Logical Maximum (1)
        0x75, 0x01, //     Report Size (1)
        0x95, 0x08, //     Report Count (8)
        0x81, 0x02, //     Input (Data, Variable, Absolute)
        0x05, 0x01, //     Usage Page (Generic Desktop)
        0x09, 0x30, //     Usage (X)
        0x09, 0x31, //     Usage (Y)
        0x09, 0x38, //     Usage (Wheel)
        0x15, 0x81, //     Logical Minimum (-127)
        0x25, 0x7f, //     Logical Maximum (127)
        0x75, 0x08, //     Report Size (8)
        0x95, 0x03, //     Report Count (3)
        0x81, 0x06, //     Input (Data, Variable, Relative)
        0xc0, //   End Collection
        0xc0, // End Collection
    ];

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

    /// Serialize the report for sending it to the host.
    pub const fn to_bytes(&self) -> [u8; 4] {
        [self.buttons, self.x as u8, self.y as u8, self.wheel as u8]
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

#[cfg(test)]
mod tests {
    use super::{KeyboardReport, MouseReport};
    use crate::class::hid::host::hid_report::{ReportDescriptor, usage, usage_page};

    /// Size in bytes of the input report a descriptor declares.
    fn input_report_len<const N: usize>(desc: &ReportDescriptor<N>) -> u32 {
        desc.fields()
            .map(|f| f.bit_offset + f.bit_size as u32 * f.count as u32)
            .max()
            .unwrap()
            .div_ceil(8)
    }

    #[test]
    fn keyboard_report_round_trips() {
        let report = KeyboardReport {
            modifiers: 0x22,
            keycodes: [4, 5, 6, 7, 8, 9],
        };
        let bytes = report.to_bytes();
        assert_eq!(bytes, [0x22, 0, 4, 5, 6, 7, 8, 9]);
        assert_eq!(KeyboardReport::parse(&bytes), Some(report));
    }

    #[test]
    fn keyboard_descriptor_matches_report() {
        let desc = ReportDescriptor::<8>::parse(KeyboardReport::DESCRIPTOR);
        assert!(!desc.has_report_ids);
        assert_eq!(input_report_len(&desc), 8);

        let bytes = KeyboardReport {
            modifiers: 0x02, // Left Shift
            keycodes: [4, 0, 0, 0, 0, 0xe7],
        }
        .to_bytes();
        assert_eq!(desc.extract_bool(&bytes, 0, usage_page::KEYBOARD, 0xe1), Some(true));
        assert_eq!(desc.extract_bool(&bytes, 0, usage_page::KEYBOARD, 0xe0), Some(false));

        let keys = desc
            .fields()
            .find(|f| f.usage_page == usage_page::KEYBOARD && !f.is_constant() && !f.is_variable())
            .unwrap();
        assert_eq!((keys.count, keys.bit_size, keys.logical_max), (6, 8, 0xff));
        assert_eq!(keys.extract_u32(&bytes, 0), Some(4));
        assert_eq!(keys.extract_u32(&bytes, 5), Some(0xe7));
    }

    #[test]
    fn mouse_report_round_trips() {
        let report = MouseReport {
            buttons: MouseReport::BUTTON_RIGHT,
            x: -3,
            y: 5,
            wheel: -1,
        };
        let bytes = report.to_bytes();
        assert_eq!(bytes, [0x02, 0xfd, 0x05, 0xff]);
        assert_eq!(MouseReport::parse(&bytes), Some(report));
    }

    #[test]
    fn mouse_descriptor_matches_report() {
        let desc = ReportDescriptor::<8>::parse(MouseReport::DESCRIPTOR);
        assert!(!desc.has_report_ids);
        assert_eq!(input_report_len(&desc), 4);

        let bytes = MouseReport {
            buttons: MouseReport::BUTTON_LEFT | MouseReport::BUTTON_MIDDLE,
            x: -127,
            y: 127,
            wheel: -1,
        }
        .to_bytes();
        assert_eq!(desc.extract_bool(&bytes, 0, usage_page::BUTTON, 1), Some(true));
        assert_eq!(desc.extract_bool(&bytes, 0, usage_page::BUTTON, 2), Some(false));
        assert_eq!(desc.extract_bool(&bytes, 0, usage_page::BUTTON, 3), Some(true));
        assert_eq!(
            desc.extract_i32(&bytes, 0, usage_page::GENERIC_DESKTOP, usage::X),
            Some(-127)
        );
        assert_eq!(
            desc.extract_i32(&bytes, 0, usage_page::GENERIC_DESKTOP, usage::Y),
            Some(127)
        );
        assert_eq!(
            desc.extract_i32(&bytes, 0, usage_page::GENERIC_DESKTOP, usage::WHEEL),
            Some(-1)
        );
    }
}
