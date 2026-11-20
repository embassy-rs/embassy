//! Consumer Control (multimedia / media keys) reports and descriptors.

/// Consumer Control report carrying a 16-bit usage code (2 bytes).
///
/// When no media key is pressed, `usage_id` is set to `0`.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ConsumerReport {
    /// 16-bit Consumer Page Usage ID.
    pub usage_id: u16,
}

impl ConsumerReport {
    // ── Media Playback ────────────────────────────────────────────────────────
    /// Play/Pause toggle.
    pub const PLAY_PAUSE: u16 = 0x00CD;
    /// Scan next track.
    pub const SCAN_NEXT_TRACK: u16 = 0x00B5;
    /// Scan previous track.
    pub const SCAN_PREV_TRACK: u16 = 0x00B6;
    /// Stop playback.
    pub const STOP: u16 = 0x00B7;
    /// Record.
    pub const RECORD: u16 = 0x00B2;
    /// Fast forward.
    pub const FAST_FORWARD: u16 = 0x00B3;
    /// Rewind.
    pub const REWIND: u16 = 0x00B4;
    /// Eject media.
    pub const EJECT: u16 = 0x00B8;

    // ── Volume & Audio Controls ───────────────────────────────────────────────
    /// Volume increment (Volume Up).
    pub const VOLUME_INCREMENT: u16 = 0x00E9;
    /// Volume decrement (Volume Down).
    pub const VOLUME_DECREMENT: u16 = 0x00EA;
    /// Mute audio.
    pub const MUTE: u16 = 0x00E2;

    // ── Display & Brightness Controls ─────────────────────────────────────────
    /// Increase display brightness.
    pub const BRIGHTNESS_INCREMENT: u16 = 0x006F;
    /// Decrease display brightness.
    pub const BRIGHTNESS_DECREMENT: u16 = 0x0070;

    // ── Application Launchers (AL) ────────────────────────────────────────────
    /// Launch Calculator.
    pub const AL_CALCULATOR: u16 = 0x0192;
    /// Launch Default Email Client.
    pub const AL_EMAIL: u16 = 0x018A;
    /// Launch Internet Browser.
    pub const AL_BROWSER: u16 = 0x0196;
    /// Launch File Browser / Local Machine Browser.
    pub const AL_FILE_BROWSER: u16 = 0x0194;

    // ── Application Controls (AC) ─────────────────────────────────────────────
    /// Contextual Search.
    pub const AC_SEARCH: u16 = 0x0221;
    /// Browser / App Home.
    pub const AC_HOME: u16 = 0x0223;
    /// Browser / App Back.
    pub const AC_BACK: u16 = 0x0224;
    /// Browser / App Forward.
    pub const AC_FORWARD: u16 = 0x0225;
    /// Browser / App Stop.
    pub const AC_STOP: u16 = 0x0226;
    /// Browser / App Refresh.
    pub const AC_REFRESH: u16 = 0x0227;
    /// Browser Bookmarks.
    pub const AC_BOOKMARKS: u16 = 0x022A;

    /// Standard USB HID Consumer Control report descriptor without report ID (2 bytes input report).
    pub const DESCRIPTOR: &'static [u8] = &[
        0x05, 0x0c, // Usage Page (Consumer)
        0x09, 0x01, // Usage (Consumer Control)
        0xa1, 0x01, // Collection (Application)
        0x15, 0x00, //   Logical Minimum (0)
        0x26, 0xff, 0x03, // Logical Maximum (1023)
        0x19, 0x00, //   Usage Minimum (0)
        0x2a, 0xff, 0x03, // Usage Maximum (1023)
        0x75, 0x10, //   Report Size (16)
        0x95, 0x01, //   Report Count (1)
        0x81, 0x00, //   Input (Data, Array, Absolute)
        0xc0, // End Collection
    ];

    /// Parse a consumer report from a byte slice.
    /// Returns `None` if `buf` is shorter than 2 bytes.
    pub fn parse(buf: &[u8]) -> Option<Self> {
        if buf.len() < 2 {
            return None;
        }
        Some(Self {
            usage_id: u16::from_le_bytes([buf[0], buf[1]]),
        })
    }

    /// Serialize the consumer report into 2 little-endian bytes.
    pub const fn to_bytes(&self) -> [u8; 2] {
        self.usage_id.to_le_bytes()
    }

    /// Creates a report with no keys pressed (`usage_id = 0`).
    pub const fn release() -> Self {
        Self { usage_id: 0 }
    }

    /// Creates a report with the specified media key.
    pub const fn from_key(usage_id: u16) -> Self {
        Self { usage_id }
    }
}

#[cfg(all(test, feature = "host"))]
mod tests {
    use super::*;
    use crate::class::hid::host::hid_report::{ReportDescriptor, usage_page};

    #[test]
    fn consumer_report_round_trips() {
        let report = ConsumerReport::from_key(ConsumerReport::VOLUME_INCREMENT);
        let bytes = report.to_bytes();
        assert_eq!(bytes, [0xE9, 0x00]);
        assert_eq!(ConsumerReport::parse(&bytes), Some(report));

        let release = ConsumerReport::release();
        assert_eq!(release.to_bytes(), [0x00, 0x00]);
        assert_eq!(ConsumerReport::parse(&release.to_bytes()), Some(release));
    }

    #[test]
    fn consumer_descriptor_matches_report() {
        let desc = ReportDescriptor::<8>::parse(ConsumerReport::DESCRIPTOR);
        assert!(!desc.has_report_ids);

        let report = ConsumerReport::from_key(ConsumerReport::PLAY_PAUSE);
        let bytes = report.to_bytes();

        let field = desc
            .fields()
            .find(|f| f.usage_page == usage_page::CONSUMER)
            .expect("Consumer field should be present");
        assert_eq!(field.bit_size, 16);
        assert_eq!(field.extract_u32(&bytes, 0), Some(ConsumerReport::PLAY_PAUSE as u32));
    }
}
