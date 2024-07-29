//! USB Audio Class 1.0 implementations for different applications.
//!
//! Contains:
//! - The `speaker` class with a single audio streaming interface (host to device)

pub mod speaker;

mod class_codes;
mod terminal_type;

/// The maximum number of supported audio channels
/// FIXME: Use `core::mem::variant_count(...)` when stabilized.
const MAX_AUDIO_CHANNEL_COUNT: usize = 12;

/// USB Audio Channel configuration
#[repr(u16)]
#[non_exhaustive]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[allow(missing_docs)]
pub enum ChannelConfig {
    None = 0x0000,
    LeftFront = 0x0001,
    RightFront = 0x0002,
    CenterFront = 0x0004,
    Lfe = 0x0008,
    LeftSurround = 0x0010,
    RightSurround = 0x0020,
    LeftOfCenter = 0x0040,
    RightOfCenter = 0x0080,
    Surround = 0x0100,
    SideLeft = 0x0200,
    SideRight = 0x0400,
    Top = 0x0800,
}

impl From<ChannelConfig> for u16 {
    fn from(t: ChannelConfig) -> u16 {
        t as u16
    }
}

/// Feedback period adjustment `bRefresh` [UAC 3.7.2.2]
///
/// From the specification: "A new Ff value is available every 2^(10 – P) frames with P ranging from 1 to 9. The
/// bRefresh field of the synch standard endpoint descriptor is used to report the exponent (10-P) to the Host.
/// It can range from 9 down to 1. (512 ms down to 2 ms)"
#[repr(u8)]
#[allow(missing_docs)]
pub enum FeedbackRefreshPeriod {
    Period2ms = 1,
    Period4ms = 2,
    Period8ms = 3,
    Period16ms = 4,
    Period32ms = 5,
    Period64ms = 6,
    Period128ms = 7,
    Period256ms = 8,
    Period512ms = 9,
}

/// Audio sample resolution.
///
/// Stored in number of bytes per sample.
#[repr(u8)]
#[allow(missing_docs)]
#[derive(Clone, Copy)]
pub enum SampleResolution {
    Resolution16Bit = 2,
    Resolution24Bit = 3,
    Resolution32Bit = 4,
}

impl SampleResolution {
    /// Get the audio sample resolution in number of bit.
    pub fn in_bit(self) -> u8 {
        8 * self as u8
    }
}
