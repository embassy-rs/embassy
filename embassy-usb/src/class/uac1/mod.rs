//! USB Audio Class 1.0 implementations for different applications.
//!
//! Contains:
//! - The `speaker` class with a single audio streaming interface (host to device)

pub mod speaker;

mod class_codes;
mod terminal_type;

/// The maximum supported audio channel index (corresponds to `Top`).
/// FIXME: Use `core::mem::variant_count(...)` when stabilized.
const MAX_AUDIO_CHANNEL_INDEX: usize = 12;

/// The maximum number of supported audio channels.
///
/// Includes all twelve channels from `Channel`, plus the Master channel.
const MAX_AUDIO_CHANNEL_COUNT: usize = MAX_AUDIO_CHANNEL_INDEX + 1;

/// USB Audio Channel
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(missing_docs)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Channel {
    LeftFront,
    RightFront,
    CenterFront,
    Lfe,
    LeftSurround,
    RightSurround,
    LeftOfCenter,
    RightOfCenter,
    Surround,
    SideLeft,
    SideRight,
    Top,
}

impl Channel {
    /// Map a `Channel` to its corresponding USB Audio `ChannelConfig`.
    fn get_channel_config(&self) -> ChannelConfig {
        match self {
            Channel::LeftFront => ChannelConfig::LeftFront,
            Channel::RightFront => ChannelConfig::RightFront,
            Channel::CenterFront => ChannelConfig::CenterFront,
            Channel::Lfe => ChannelConfig::Lfe,
            Channel::LeftSurround => ChannelConfig::LeftSurround,
            Channel::RightSurround => ChannelConfig::RightSurround,
            Channel::LeftOfCenter => ChannelConfig::LeftOfCenter,
            Channel::RightOfCenter => ChannelConfig::RightOfCenter,
            Channel::Surround => ChannelConfig::Surround,
            Channel::SideLeft => ChannelConfig::SideLeft,
            Channel::SideRight => ChannelConfig::SideRight,
            Channel::Top => ChannelConfig::Top,
        }
    }
}

/// USB Audio Channel configuration
#[repr(u16)]
#[non_exhaustive]
// #[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum ChannelConfig {
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
/// bRefresh field of the synch standard endpoint descriptor is used to report the exponent (10-P) to the Host."
///
/// This means:
/// - 512 ms (2^9 frames) to 2 ms (2^1 frames) for USB full-speed
/// - 64 ms (2^9 microframes) to 0.25 ms (2^1 microframes) for USB high-speed
#[repr(u8)]
#[allow(missing_docs)]
#[derive(Clone, Copy)]
pub enum FeedbackRefresh {
    Period2Frames = 1,
    Period4Frames = 2,
    Period8Frames = 3,
    Period16Frames = 4,
    Period32Frames = 5,
    Period64Frames = 6,
    Period128Frames = 7,
    Period256Frames = 8,
    Period512Frames = 9,
}

impl FeedbackRefresh {
    /// Gets the number of frames, after which a new feedback frame is returned.
    pub const fn frame_count(&self) -> usize {
        1 << (*self as usize)
    }
}

/// Audio sample width.
///
/// Stored in number of bytes per sample.
#[repr(u8)]
#[derive(Clone, Copy)]
pub enum SampleWidth {
    /// 16 bit audio
    Width2Byte = 2,
    /// 24 bit audio
    Width3Byte = 3,
    /// 32 bit audio
    Width4Byte = 4,
}

impl SampleWidth {
    /// Get the audio sample resolution in number of bit.
    pub const fn in_bit(self) -> usize {
        8 * self as usize
    }
}
