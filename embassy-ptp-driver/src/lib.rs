#![no_std]
#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

/// A reading of a PTP hardware clock.
///
/// The clock has an arbitrary epoch and drifts with respect to other clocks
/// unless it is actively disciplined. Do not mix this with executor time.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Default)]
pub struct Timestamp {
    /// Whole seconds.
    seconds: u32,
    /// Fraction of a second, in units of 0.25 nanoseconds.
    quarter_nanos: u32,
}

impl Timestamp {
    /// Construct a timestamp from seconds and nanoseconds.
    #[inline]
    pub const fn from_seconds_and_nanos(seconds: u32, nanos: u32) -> Self {
        debug_assert!(nanos < 1_000_000_000);
        Self::from_seconds_and_quarter_nanos(seconds, nanos << 2)
    }

    /// Construct a timestamp from seconds and quarter nanoseconds.
    #[inline]
    pub const fn from_seconds_and_quarter_nanos(seconds: u32, quarter_nanos: u32) -> Self {
        debug_assert!(quarter_nanos < 4_000_000_000);
        Self { seconds, quarter_nanos }
    }

    /// Get the whole seconds.
    #[inline]
    pub const fn seconds(&self) -> u32 {
        self.seconds
    }

    /// Get the fractional quarter nanoseconds within the current second.
    #[inline]
    pub const fn quarter_nanos(&self) -> u32 {
        self.quarter_nanos
    }

    /// Get the whole nanoseconds within the current second.
    #[inline]
    pub const fn nanos(&self) -> u32 {
        self.quarter_nanos >> 2
    }
}

/// A signed frequency adjustment in parts per million with 16 fractional bits.
///
/// Zero selects the nominal clock frequency and positive values make the clock
/// run faster. One raw unit is `2^-16` ppm, giving a range of approximately
/// -32768 to +32768 ppm.
#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ScaledPpm(i32);

impl ScaledPpm {
    /// No frequency adjustment.
    pub const ZERO: Self = Self(0);

    /// Construct an adjustment from its signed fixed-point representation.
    pub const fn from_raw(raw: i32) -> Self {
        Self(raw)
    }

    /// Return the signed fixed-point representation.
    pub const fn raw(self) -> i32 {
        self.0
    }
}

/// A readable and adjustable PTP hardware clock.
///
/// The clock must be the same time domain that produces packet timestamps.
/// Frequency adjustments are absolute relative to the nominal frequency,
/// rather than cumulative.
pub trait Clock {
    /// Error returned when adjusting the clock.
    type Error: core::error::Error;

    /// Read the current clock time.
    fn now(&self) -> Timestamp;

    /// Step the clock and return a timestamp sampled near the change.
    ///
    /// A positive offset moves the clock forward. Steps use whole nanoseconds
    /// even when timestamp observations have subnanosecond resolution: stepping
    /// is a coarse correction, while frequency adjustment preserves fine
    /// steering.
    fn step(&mut self, offset_nanos: i64) -> Result<Timestamp, Self::Error>;

    /// Set the absolute frequency adjustment and return a timestamp sampled
    /// near the change.
    fn set_frequency(&mut self, adjustment: ScaledPpm) -> Result<Timestamp, Self::Error>;
}

impl<T: ?Sized + Clock> Clock for &mut T {
    type Error = T::Error;

    fn now(&self) -> Timestamp {
        T::now(self)
    }

    fn step(&mut self, offset_nanos: i64) -> Result<Timestamp, Self::Error> {
        T::step(self, offset_nanos)
    }

    fn set_frequency(&mut self, adjustment: ScaledPpm) -> Result<Timestamp, Self::Error> {
        T::set_frequency(self, adjustment)
    }
}

#[cfg(test)]
mod tests {
    use super::Timestamp;

    #[test]
    fn timestamp_preserves_quarter_nanoseconds() {
        let timestamp = Timestamp::from_seconds_and_quarter_nanos(12, 3_999_999_999);

        assert_eq!(timestamp.seconds(), 12);
        assert_eq!(timestamp.quarter_nanos(), 3_999_999_999);
        assert_eq!(timestamp.nanos(), 999_999_999);
    }

    #[test]
    #[should_panic]
    fn timestamp_rejects_a_full_second_of_nanoseconds() {
        Timestamp::from_seconds_and_nanos(0, 1_000_000_000);
    }

    #[test]
    #[should_panic]
    fn timestamp_rejects_a_full_second_of_quarter_nanoseconds() {
        Timestamp::from_seconds_and_quarter_nanos(0, 4_000_000_000);
    }
}
