use core::fmt;
use core::ops::{Add, AddAssign, Sub, SubAssign};

use super::TICKS_PER_SECOND;
use super::{now, Duration};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Instant {
    ticks: u64,
}

impl Instant {
    pub const MIN: Instant = Instant { ticks: u64::MIN };
    pub const MAX: Instant = Instant { ticks: u64::MAX };

    pub fn now() -> Instant {
        Instant { ticks: now() }
    }

    pub const fn from_ticks(ticks: u64) -> Self {
        Self { ticks }
    }

    pub const fn from_millis(millis: u64) -> Self {
        Self {
            ticks: millis * TICKS_PER_SECOND as u64 / 1000,
        }
    }

    pub const fn from_secs(seconds: u64) -> Self {
        Self {
            ticks: seconds * TICKS_PER_SECOND as u64,
        }
    }

    pub const fn as_ticks(&self) -> u64 {
        self.ticks
    }

    pub const fn as_secs(&self) -> u64 {
        self.ticks / TICKS_PER_SECOND as u64
    }

    pub const fn as_millis(&self) -> u64 {
        self.ticks * 1000 / TICKS_PER_SECOND as u64
    }

    pub fn duration_since(&self, earlier: Instant) -> Duration {
        Duration {
            ticks: self.ticks.checked_sub(earlier.ticks).unwrap(),
        }
    }

    pub fn checked_duration_since(&self, earlier: Instant) -> Option<Duration> {
        if self.ticks < earlier.ticks {
            None
        } else {
            Some(Duration {
                ticks: self.ticks - earlier.ticks,
            })
        }
    }

    pub fn saturating_duration_since(&self, earlier: Instant) -> Duration {
        Duration {
            ticks: if self.ticks < earlier.ticks {
                0
            } else {
                self.ticks - earlier.ticks
            },
        }
    }

    pub fn elapsed(&self) -> Duration {
        Instant::now() - *self
    }

    pub fn checked_add(&self, duration: Duration) -> Option<Instant> {
        self.ticks
            .checked_add(duration.ticks)
            .map(|ticks| Instant { ticks })
    }
    pub fn checked_sub(&self, duration: Duration) -> Option<Instant> {
        self.ticks
            .checked_sub(duration.ticks)
            .map(|ticks| Instant { ticks })
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
