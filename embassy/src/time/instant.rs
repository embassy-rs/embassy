use core::convert::TryInto;
use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};
use core::pin::Pin;
use core::ptr;
use core::sync::atomic::{AtomicPtr, Ordering};
use core::task::{Context, Poll};

use super::{now, Duration};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Instant {
    ticks: u64,
}

impl Instant {
    pub fn now() -> Instant {
        Instant { ticks: now() }
    }

    pub const fn from_ticks(ticks: u64) -> Self {
        Self { ticks }
    }

    pub const fn into_ticks(&self) -> u64 {
        self.ticks
    }

    pub fn duration_since(&self, earlier: Instant) -> Duration {
        Duration {
            ticks: (self.ticks - earlier.ticks).try_into().unwrap(),
        }
    }

    pub fn checked_duration_since(&self, earlier: Instant) -> Option<Duration> {
        if self.ticks < earlier.ticks {
            None
        } else {
            Some(Duration {
                ticks: (self.ticks - earlier.ticks).try_into().unwrap(),
            })
        }
    }

    pub fn saturating_duration_since(&self, earlier: Instant) -> Duration {
        Duration {
            ticks: if self.ticks < earlier.ticks {
                0
            } else {
                (self.ticks - earlier.ticks).try_into().unwrap()
            },
        }
    }

    pub fn elapsed(&self) -> Duration {
        Instant::now() - *self
    }

    pub fn checked_add(&self, duration: Duration) -> Option<Instant> {
        self.ticks
            .checked_add(duration.ticks.into())
            .map(|ticks| Instant { ticks })
    }
    pub fn checked_sub(&self, duration: Duration) -> Option<Instant> {
        self.ticks
            .checked_sub(duration.ticks.into())
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
