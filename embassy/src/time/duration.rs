use core::fmt;
use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

use super::TICKS_PER_SECOND;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, defmt::Format)]
pub struct Duration {
    pub(crate) ticks: u64,
}

impl Duration {
    pub const fn as_ticks(&self) -> u64 {
        self.ticks
    }

    pub const fn as_secs(&self) -> u64 {
        self.ticks / TICKS_PER_SECOND
    }

    pub const fn as_millis(&self) -> u64 {
        self.ticks * 1000 / TICKS_PER_SECOND
    }

    pub const fn from_ticks(ticks: u64) -> Duration {
        Duration { ticks }
    }

    pub const fn from_secs(secs: u64) -> Duration {
        Duration {
            ticks: secs * TICKS_PER_SECOND,
        }
    }

    pub const fn from_millis(millis: u64) -> Duration {
        Duration {
            ticks: millis * TICKS_PER_SECOND / 1000,
        }
    }

    pub fn checked_add(self, rhs: Duration) -> Option<Duration> {
        self.ticks
            .checked_add(rhs.ticks)
            .map(|ticks| Duration { ticks })
    }

    pub fn checked_sub(self, rhs: Duration) -> Option<Duration> {
        self.ticks
            .checked_sub(rhs.ticks)
            .map(|ticks| Duration { ticks })
    }

    pub fn checked_mul(self, rhs: u32) -> Option<Duration> {
        self.ticks
            .checked_mul(rhs as _)
            .map(|ticks| Duration { ticks })
    }

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
