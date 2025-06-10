use core::fmt;
use core::ops::{Add, AddAssign, Sub, SubAssign};

use super::{Duration, GCD_1K, GCD_1M, TICK_HZ};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// An Instant in time, based on the MCU's clock ticks since startup.
pub struct Instant {
    ticks: u64,
}

impl Instant {
    /// The smallest (earliest) value that can be represented by the `Instant` type.
    pub const MIN: Instant = Instant { ticks: u64::MIN };
    /// The largest (latest) value that can be represented by the `Instant` type.
    pub const MAX: Instant = Instant { ticks: u64::MAX };

    /// Returns an Instant representing the current time.
    #[inline]
    pub fn now() -> Instant {
        Instant {
            ticks: embassy_time_driver::now(),
        }
    }

    /// Create an Instant from a tick count since system boot.
    pub const fn from_ticks(ticks: u64) -> Self {
        Self { ticks }
    }

    /// Create an Instant from a microsecond count since system boot.
    pub const fn from_micros(micros: u64) -> Self {
        Self {
            ticks: micros * (TICK_HZ / GCD_1M) / (1_000_000 / GCD_1M),
        }
    }

    /// Create an Instant from a millisecond count since system boot.
    pub const fn from_millis(millis: u64) -> Self {
        Self {
            ticks: millis * (TICK_HZ / GCD_1K) / (1000 / GCD_1K),
        }
    }

    /// Create an Instant from a second count since system boot.
    pub const fn from_secs(seconds: u64) -> Self {
        Self {
            ticks: seconds * TICK_HZ,
        }
    }

    /// Try to create an Instant from a microsecond count since system boot.
    /// Fails if the number of microseconds is too large.
    pub const fn try_from_micros(micros: u64) -> Option<Self> {
        let Some(value) = micros.checked_mul(TICK_HZ / GCD_1M) else {
            return None;
        };
        Some(Self {
            ticks: value / (1_000_000 / GCD_1M),
        })
    }

    /// Try to create an Instant from a millisecond count since system boot.
    /// Fails if the number of milliseconds is too large.
    pub const fn try_from_millis(millis: u64) -> Option<Self> {
        let Some(value) = millis.checked_mul(TICK_HZ / GCD_1K) else {
            return None;
        };
        Some(Self {
            ticks: value / (1000 / GCD_1K),
        })
    }

    /// Try to create an Instant from a second count since system boot.
    /// Fails if the number of seconds is too large.
    pub const fn try_from_secs(seconds: u64) -> Option<Self> {
        let Some(ticks) = seconds.checked_mul(TICK_HZ) else {
            return None;
        };
        Some(Self { ticks })
    }

    /// Tick count since system boot.
    pub const fn as_ticks(&self) -> u64 {
        self.ticks
    }

    /// Seconds since system boot.
    pub const fn as_secs(&self) -> u64 {
        self.ticks / TICK_HZ
    }

    /// Milliseconds since system boot.
    pub const fn as_millis(&self) -> u64 {
        self.ticks * (1000 / GCD_1K) / (TICK_HZ / GCD_1K)
    }

    /// Microseconds since system boot.
    pub const fn as_micros(&self) -> u64 {
        self.ticks * (1_000_000 / GCD_1M) / (TICK_HZ / GCD_1M)
    }

    /// Duration between this Instant and another Instant
    /// Panics on over/underflow.
    pub fn duration_since(&self, earlier: Instant) -> Duration {
        Duration {
            ticks: unwrap!(self.ticks.checked_sub(earlier.ticks)),
        }
    }

    /// Duration between this Instant and another Instant
    pub fn checked_duration_since(&self, earlier: Instant) -> Option<Duration> {
        if self.ticks < earlier.ticks {
            None
        } else {
            Some(Duration {
                ticks: self.ticks - earlier.ticks,
            })
        }
    }

    /// Returns the duration since the "earlier" Instant.
    /// If the "earlier" instant is in the future, the duration is set to zero.
    pub fn saturating_duration_since(&self, earlier: Instant) -> Duration {
        Duration {
            ticks: if self.ticks < earlier.ticks {
                0
            } else {
                self.ticks - earlier.ticks
            },
        }
    }

    /// Duration elapsed since this Instant.
    pub fn elapsed(&self) -> Duration {
        Instant::now() - *self
    }

    /// Adds one Duration to self, returning a new `Instant` or None in the event of an overflow.
    pub fn checked_add(&self, duration: Duration) -> Option<Instant> {
        self.ticks.checked_add(duration.ticks).map(|ticks| Instant { ticks })
    }

    /// Subtracts one Duration to self, returning a new `Instant` or None in the event of an overflow.
    pub fn checked_sub(&self, duration: Duration) -> Option<Instant> {
        self.ticks.checked_sub(duration.ticks).map(|ticks| Instant { ticks })
    }

    /// Adds a Duration to self. In case of overflow, the maximum value is returned.
    pub fn saturating_add(mut self, duration: Duration) -> Self {
        self.ticks = self.ticks.saturating_add(duration.ticks);
        self
    }

    /// Subtracts a Duration from self. In case of overflow, the minimum value is returned.
    pub fn saturating_sub(mut self, duration: Duration) -> Self {
        self.ticks = self.ticks.saturating_sub(duration.ticks);
        self
    }
}

impl Add<Duration> for Instant {
    type Output = Instant;

    fn add(self, other: Duration) -> Instant {
        self.checked_add(other)
            .expect("overflow when adding duration to instant")
    }
}

impl AddAssign<Duration> for Instant {
    fn add_assign(&mut self, other: Duration) {
        *self = *self + other;
    }
}

impl Sub<Duration> for Instant {
    type Output = Instant;

    fn sub(self, other: Duration) -> Instant {
        self.checked_sub(other)
            .expect("overflow when subtracting duration from instant")
    }
}

impl SubAssign<Duration> for Instant {
    fn sub_assign(&mut self, other: Duration) {
        *self = *self - other;
    }
}

impl Sub<Instant> for Instant {
    type Output = Duration;

    fn sub(self, other: Instant) -> Duration {
        self.duration_since(other)
    }
}

impl<'a> fmt::Display for Instant {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} ticks", self.ticks)
    }
}
