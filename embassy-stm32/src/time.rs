//! Time units

use core::fmt::Display;
use core::ops::{Div, Mul};

/// Hertz
#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Debug)]
pub struct Hertz(pub u32);

impl Display for Hertz {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{} Hz", self.0)
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for Hertz {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "{=u32} Hz", self.0)
    }
}

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

#[repr(C)]
#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Debug, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// A variant on [Hertz] that acts as an `Option<Hertz>` that is smaller and repr C.
///
/// An `Option<Hertz>` can be `.into()`'d into this type and back.
/// The only restriction is that that [Hertz] cannot have the value 0 since that's
/// seen as the `None` variant.
pub struct MaybeHertz(u32);

impl MaybeHertz {
    /// Same as calling the `.into()` function, but without type inference.
    pub fn to_hertz(self) -> Option<Hertz> {
        self.into()
    }
}

impl From<Option<Hertz>> for MaybeHertz {
    fn from(value: Option<Hertz>) -> Self {
        match value {
            Some(Hertz(0)) => panic!("Hertz cannot be 0"),
            Some(Hertz(val)) => Self(val),
            None => Self(0),
        }
    }
}

impl From<MaybeHertz> for Option<Hertz> {
    fn from(value: MaybeHertz) -> Self {
        match value {
            MaybeHertz(0) => None,
            MaybeHertz(val) => Some(Hertz(val)),
        }
    }
}
