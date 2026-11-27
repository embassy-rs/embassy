//! Time-unit newtypes.
//!
//! Minimal [`Hertz`] wrapper so peripheral configuration expresses clock rates
//! in the type system instead of bare `u32` values.

/// Frequency in hertz.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Hertz(pub u32);

impl Hertz {
    /// Frequency in hertz as a raw value.
    pub const fn hz(self) -> u32 {
        self.0
    }

    /// Frequency in kilohertz (truncated).
    pub const fn khz(self) -> u32 {
        self.0 / 1_000
    }

    /// Frequency in megahertz (truncated).
    pub const fn mhz(self) -> u32 {
        self.0 / 1_000_000
    }
}

impl From<u32> for Hertz {
    fn from(hz: u32) -> Self {
        Self(hz)
    }
}
