//! USB Audio Channel configuration

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
