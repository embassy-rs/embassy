//! Time units

use core::ops::{Div, Mul};

/// Hertz
#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Hertz(pub u32);

impl Hertz {
    /// Create a `Hertz` from the given hertz.
    pub const fn hz(hertz: u32) -> Self {
        Self(hertz)
    }

    /// Create a `Hertz` from the given kilohertz.
    pub const fn khz(kilohertz: u32) -> Self {
        Self(kilohertz * 1_000)
    }

    /// Create a `Hertz` from the given megahertz.
    pub const fn mhz(megahertz: u32) -> Self {
        Self(megahertz * 1_000_000)
    }
}

/// This is a convenience shortcut for [`Hertz::hz`]
pub const fn hz(hertz: u32) -> Hertz {
    Hertz::hz(hertz)
}

/// This is a convenience shortcut for [`Hertz::khz`]
pub const fn khz(kilohertz: u32) -> Hertz {
    Hertz::khz(kilohertz)
}

/// This is a convenience shortcut for [`Hertz::mhz`]
pub const fn mhz(megahertz: u32) -> Hertz {
    Hertz::mhz(megahertz)
}

impl Mul<u32> for Hertz {
    type Output = Hertz;
    fn mul(self, rhs: u32) -> Self::Output {
        Hertz(self.0 * rhs)
    }
}

impl Div<u32> for Hertz {
    type Output = Hertz;
    fn div(self, rhs: u32) -> Self::Output {
        Hertz(self.0 / rhs)
    }
}

impl Mul<u16> for Hertz {
    type Output = Hertz;
    fn mul(self, rhs: u16) -> Self::Output {
        self * (rhs as u32)
    }
}

impl Div<u16> for Hertz {
    type Output = Hertz;
    fn div(self, rhs: u16) -> Self::Output {
        self / (rhs as u32)
    }
}

impl Mul<u8> for Hertz {
    type Output = Hertz;
    fn mul(self, rhs: u8) -> Self::Output {
        self * (rhs as u32)
    }
}

impl Div<u8> for Hertz {
    type Output = Hertz;
    fn div(self, rhs: u8) -> Self::Output {
        self / (rhs as u32)
    }
}

impl Div<Hertz> for Hertz {
    type Output = u32;
    fn div(self, rhs: Hertz) -> Self::Output {
        self.0 / rhs.0
    }
}
