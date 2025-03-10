use core::fmt;
use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

use super::{GCD_1K, GCD_1M, TICK_HZ};
use crate::GCD_1G;

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
        self.ticks / TICK_HZ
    }

    /// Convert the `Duration` to milliseconds, rounding down.
    pub const fn as_millis(&self) -> u64 {
        self.ticks * (1000 / GCD_1K) / (TICK_HZ / GCD_1K)
    }

    /// Convert the `Duration` to microseconds, rounding down.
    pub const fn as_micros(&self) -> u64 {
        self.ticks * (1_000_000 / GCD_1M) / (TICK_HZ / GCD_1M)
    }

    /// Creates a duration from the specified number of clock ticks
    pub const fn from_ticks(ticks: u64) -> Duration {
        Duration { ticks }
    }

    /// Creates a duration from the specified number of seconds, rounding up.
    pub const fn from_secs(secs: u64) -> Duration {
        Duration { ticks: secs * TICK_HZ }
    }

    /// Creates a duration from the specified number of milliseconds, rounding up.
    pub const fn from_millis(millis: u64) -> Duration {
        Duration {
            ticks: div_ceil(millis * (TICK_HZ / GCD_1K), 1000 / GCD_1K),
        }
    }

    /// Creates a duration from the specified number of microseconds, rounding up.
    /// NOTE: Delays this small may be inaccurate.
    pub const fn from_micros(micros: u64) -> Duration {
        Duration {
            ticks: div_ceil(micros * (TICK_HZ / GCD_1M), 1_000_000 / GCD_1M),
        }
    }

    /// Creates a duration from the specified number of nanoseconds, rounding up.
    /// NOTE: Delays this small may be inaccurate.
    pub const fn from_nanos(nanoseconds: u64) -> Duration {
        Duration {
            ticks: div_ceil(nanoseconds * (TICK_HZ / GCD_1G), 1_000_000_000 / GCD_1G),
        }
    }

    /// Creates a duration from the specified number of seconds, rounding down.
    pub const fn from_secs_floor(secs: u64) -> Duration {
        Duration { ticks: secs * TICK_HZ }
    }

    /// Creates a duration from the specified number of milliseconds, rounding down.
    pub const fn from_millis_floor(millis: u64) -> Duration {
        Duration {
            ticks: millis * (TICK_HZ / GCD_1K) / (1000 / GCD_1K),
        }
    }

    /// Creates a duration from the specified number of microseconds, rounding down.
    /// NOTE: Delays this small may be inaccurate.
    pub const fn from_micros_floor(micros: u64) -> Duration {
        Duration {
            ticks: micros * (TICK_HZ / GCD_1M) / (1_000_000 / GCD_1M),
        }
    }

    /// Try to create a duration from the specified number of seconds, rounding up.
    /// Fails if the number of seconds is too large.
    pub const fn try_from_secs(secs: u64) -> Option<Duration> {
        let Some(ticks) = secs.checked_mul(TICK_HZ) else {
            return None;
        };
        Some(Duration { ticks })
    }

    /// Try to create a duration from the specified number of milliseconds, rounding up.
    /// Fails if the number of milliseconds is too large.
    pub const fn try_from_millis(millis: u64) -> Option<Duration> {
        let Some(value) = millis.checked_mul(TICK_HZ / GCD_1K) else {
            return None;
        };
        Some(Duration {
            ticks: div_ceil(value, 1000 / GCD_1K),
        })
    }

    /// Try to create a duration from the specified number of microseconds, rounding up.
    /// Fails if the number of microseconds is too large.
    /// NOTE: Delays this small may be inaccurate.
    pub const fn try_from_micros(micros: u64) -> Option<Duration> {
        let Some(value) = micros.checked_mul(TICK_HZ / GCD_1M) else {
            return None;
        };
        Some(Duration {
            ticks: div_ceil(value, 1_000_000 / GCD_1M),
        })
    }

    /// Try to create a duration from the specified number of nanoseconds, rounding up.
    /// Fails if the number of nanoseconds is too large.
    /// NOTE: Delays this small may be inaccurate.
    pub const fn try_from_nanos(nanoseconds: u64) -> Option<Duration> {
        let Some(value) = nanoseconds.checked_mul(TICK_HZ / GCD_1G) else {
            return None;
        };
        Some(Duration {
            ticks: div_ceil(value, 1_000_000_000 / GCD_1G),
        })
    }

    /// Try to create a duration from the specified number of seconds, rounding down.
    /// Fails if the number of seconds is too large.
    pub const fn try_from_secs_floor(secs: u64) -> Option<Duration> {
        let Some(ticks) = secs.checked_mul(TICK_HZ) else {
            return None;
        };
        Some(Duration { ticks })
    }

    /// Try to create a duration from the specified number of milliseconds, rounding down.
    /// Fails if the number of milliseconds is too large.
    pub const fn try_from_millis_floor(millis: u64) -> Option<Duration> {
        let Some(value) = millis.checked_mul(TICK_HZ / GCD_1K) else {
            return None;
        };
        Some(Duration {
            ticks: value / (1000 / GCD_1K),
        })
    }

    /// Try to create a duration from the specified number of microseconds, rounding down.
    /// Fails if the number of microseconds is too large.
    /// NOTE: Delays this small may be inaccurate.
    pub const fn try_from_micros_floor(micros: u64) -> Option<Duration> {
        let Some(value) = micros.checked_mul(TICK_HZ / GCD_1M) else {
            return None;
        };
        Some(Duration {
            ticks: value / (1_000_000 / GCD_1M),
        })
    }

    /// Creates a duration corresponding to the specified Hz.
    /// NOTE: Giving this function a hz >= the TICK_HZ of your platform will clamp the Duration to 1
    /// tick. Doing so will not deadlock, but will certainly not produce the desired output.
    pub const fn from_hz(hz: u64) -> Duration {
        let ticks = {
            if hz >= TICK_HZ {
                1
            } else {
                (TICK_HZ + hz / 2) / hz
            }
        };
        Duration { ticks }
    }

    /// Adds one Duration to another, returning a new Duration or None in the event of an overflow.
    pub fn checked_add(self, rhs: Duration) -> Option<Duration> {
        self.ticks.checked_add(rhs.ticks).map(|ticks| Duration { ticks })
    }

    /// Subtracts one Duration to another, returning a new Duration or None in the event of an overflow.
    pub fn checked_sub(self, rhs: Duration) -> Option<Duration> {
        self.ticks.checked_sub(rhs.ticks).map(|ticks| Duration { ticks })
    }

    /// Multiplies one Duration by a scalar u32, returning a new Duration or None in the event of an overflow.
    pub fn checked_mul(self, rhs: u32) -> Option<Duration> {
        self.ticks.checked_mul(rhs as _).map(|ticks| Duration { ticks })
    }

    /// Divides one Duration a scalar u32, returning a new Duration or None in the event of an overflow.
    pub fn checked_div(self, rhs: u32) -> Option<Duration> {
        self.ticks.checked_div(rhs as _).map(|ticks| Duration { ticks })
    }
}

impl Add for Duration {
    type Output = Duration;

    fn add(self, rhs: Duration) -> Duration {
        self.checked_add(rhs).expect("overflow when adding durations")
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
        self.checked_sub(rhs).expect("overflow when subtracting durations")
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

#[inline]
const fn div_ceil(num: u64, den: u64) -> u64 {
    (num + den - 1) / den
}

impl TryFrom<core::time::Duration> for Duration {
    type Error = <u64 as TryFrom<u128>>::Error;

    /// Converts using [`Duration::from_micros`]. Fails if value can not be represented as u64.
    fn try_from(value: core::time::Duration) -> Result<Self, Self::Error> {
        Ok(Self::from_micros(value.as_micros().try_into()?))
    }
}

impl From<Duration> for core::time::Duration {
    /// Converts using [`Duration::as_micros`].
    fn from(value: Duration) -> Self {
        core::time::Duration::from_micros(value.as_micros())
    }
}
