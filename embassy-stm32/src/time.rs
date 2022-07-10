//! Time units

/// Hertz
#[derive(PartialEq, PartialOrd, Clone, Copy, Debug, Eq)]
pub struct Hertz(pub u32);

impl Hertz {
    pub fn hz(hertz: u32) -> Self {
        Self(hertz)
    }

    pub fn khz(kilohertz: u32) -> Self {
        Self(kilohertz * 1_000)
    }

    pub fn mhz(megahertz: u32) -> Self {
        Self(megahertz * 1_000_000)
    }
}

/// This is a convenience shortcut for [`Hertz::hz`]
pub fn hz(hertz: u32) -> Hertz {
    Hertz::hz(hertz)
}

/// This is a convenience shortcut for [`Hertz::khz`]
pub fn khz(kilohertz: u32) -> Hertz {
    Hertz::khz(kilohertz)
}

/// This is a convenience shortcut for [`Hertz::mhz`]
pub fn mhz(megahertz: u32) -> Hertz {
    Hertz::mhz(megahertz)
}
