use core::fmt;
use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

use super::{GCD_1K, GCD_1M, TICKS_PER_SECOND};

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// Represents the difference between two [Instant](struct.Instant.html)s
pub struct Duration {
    pub(crate) ticks: u64,
}

impl Duration {
    /// The smallest value that can be represented by the `Duration` type.
    pub const MIN: Duration = Duration { ticks: u64::MIN };
    /// The largest value that can be represented by the `Duration` type.
    pub const MAX: Duration = Duration { ticks: u64::MAX };

    /// Tick count of the `Duration`.
    pub const fn as_ticks(&self) -> u64 {
        self.ticks
    }

    /// Convert the `Duration` to seconds, rounding down.
    pub const fn as_secs(&self) -> u64 {
        self.ticks / TICKS_PER_SECOND
    }

    /// Convert the `Duration` to milliseconds, rounding down.
    pub const fn as_millis(&self) -> u64 {
        self.ticks * (1000 / GCD_1K) / (TICKS_PER_SECOND / GCD_1K)
    }

    /// Convert the `Duration` to microseconds, rounding down.
    pub const fn as_micros(&self) -> u64 {
        self.ticks * (1_000_000 / GCD_1M) / (TICKS_PER_SECOND / GCD_1M)
    }

    /// Creates a duration from the specified number of clock ticks
    pub const fn from_ticks(ticks: u64) -> Duration {
        Duration { ticks }
    }

    /// Creates a duration from the specified number of seconds
    pub const fn from_secs(secs: u64) -> Duration {
        Duration {
            ticks: secs * TICKS_PER_SECOND,
        }
    }

    /// Creates a duration from the specified number of milliseconds
    pub const fn from_millis(millis: u64) -> Duration {
        Duration {
            ticks: millis * (TICKS_PER_SECOND / GCD_1K) / (1000 / GCD_1K),
        }
    }

    /// Creates a duration from the specified number of microseconds
    /// NOTE: Delays this small may be inaccurate.
    pub const fn from_micros(micros: u64) -> Duration {
        Duration {
            ticks: micros * (TICKS_PER_SECOND / GCD_1M) / (1_000_000 / GCD_1M),
        }
    }

    /// Adds one Duration to another, returning a new Duration or None in the event of an overflow.
    pub fn checked_add(self, rhs: Duration) -> Option<Duration> {
        self.ticks
            .checked_add(rhs.ticks)
            .map(|ticks| Duration { ticks })
    }

    /// Subtracts one Duration to another, returning a new Duration or None in the event of an overflow.
    pub fn checked_sub(self, rhs: Duration) -> Option<Duration> {
        self.ticks
            .checked_sub(rhs.ticks)
            .map(|ticks| Duration { ticks })
    }

    /// Multiplies one Duration by a scalar u32, returning a new Duration or None in the event of an overflow.
    pub fn checked_mul(self, rhs: u32) -> Option<Duration> {
        self.ticks
            .checked_mul(rhs as _)
            .map(|ticks| Duration { ticks })
    }

    /// Divides one Duration a scalar u32, returning a new Duration or None in the event of an overflow.
    pub fn checked_div(self, rhs: u32) -> Option<Duration> {
        self.ticks
            .checked_div(rhs as _)
            .map(|ticks| Duration { ticks })
    }
}

impl Add for Duration {
    type Output = Duration;

    fn add(self, rhs: Duration) -> Duration {
        self.checked_add(rhs)
            .expect("overflow when adding durations")
    }
}

impl AddAssign for Duration {
    fn add_assign(&mut self, rhs: Duration) {
        *self = *self + rhs;
    }
}

impl Sub for Duration {
    type Output = Duration;

    fn sub(self, rhs: Duration) -> Duration {
        self.checked_sub(rhs)
            .expect("overflow when subtracting durations")
    }
}

impl SubAssign for Duration {
    fn sub_assign(&mut self, rhs: Duration) {
        *self = *self - rhs;
    }
}

impl Mul<u32> for Duration {
    type Output = Duration;

    fn mul(self, rhs: u32) -> Duration {
        self.checked_mul(rhs)
            .expect("overflow when multiplying duration by scalar")
    }
}

impl Mul<Duration> for u32 {
    type Output = Duration;

    fn mul(self, rhs: Duration) -> Duration {
        rhs * self
    }
}

impl MulAssign<u32> for Duration {
    fn mul_assign(&mut self, rhs: u32) {
        *self = *self * rhs;
    }
}

impl Div<u32> for Duration {
    type Output = Duration;

    fn div(self, rhs: u32) -> Duration {
        self.checked_div(rhs)
            .expect("divide by zero error when dividing duration by scalar")
    }
}

impl DivAssign<u32> for Duration {
    fn div_assign(&mut self, rhs: u32) {
        *self = *self / rhs;
    }
}

impl<'a> fmt::Display for Duration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} ticks", self.ticks)
    }
}
