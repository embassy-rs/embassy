use core::cell::Cell;
use core::convert::TryInto;
use core::future::Future;
use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};
use core::pin::Pin;
use core::ptr;
use core::sync::atomic::{AtomicPtr, Ordering};
use core::task::{Context, Poll};

use fi::LocalTimer;
use futures_intrusive::timer as fi;

static mut CLOCK: fn() -> u64 = clock_not_set;

fn clock_not_set() -> u64 {
    panic!("No clock set. You must call embassy::time::set_clock() before trying to use the clock")
}

pub unsafe fn set_clock(clock: fn() -> u64) {
    CLOCK = clock;
}

fn now() -> u64 {
    unsafe { CLOCK() }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Instant {
    ticks: u64,
}

impl Instant {
    pub fn now() -> Instant {
        Instant { ticks: now() }
    }

    pub fn into_ticks(&self) -> u64 {
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

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Duration {
    ticks: u32,
}

impl Duration {
    pub const fn from_ticks(ticks: u32) -> Duration {
        Duration { ticks }
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
        self.ticks.checked_mul(rhs).map(|ticks| Duration { ticks })
    }

    pub fn checked_div(self, rhs: u32) -> Option<Duration> {
        self.ticks.checked_div(rhs).map(|ticks| Duration { ticks })
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

pub(crate) struct IntrusiveClock;

impl fi::Clock for IntrusiveClock {
    fn now(&self) -> u64 {
        now()
    }
}

pub(crate) type TimerService = fi::LocalTimerService<IntrusiveClock>;

static CURRENT_TIMER_SERVICE: AtomicPtr<TimerService> = AtomicPtr::new(ptr::null_mut());

pub(crate) fn with_timer_service<R>(svc: &'static TimerService, f: impl FnOnce() -> R) -> R {
    let svc = svc as *const _ as *mut _;
    let prev_svc = CURRENT_TIMER_SERVICE.swap(svc, Ordering::Relaxed);
    let r = f();
    let svc2 = CURRENT_TIMER_SERVICE.swap(prev_svc, Ordering::Relaxed);
    assert_eq!(svc, svc2);
    r
}

fn current_timer_service() -> &'static TimerService {
    unsafe {
        CURRENT_TIMER_SERVICE
            .load(Ordering::Relaxed)
            .as_ref()
            .unwrap()
    }
}

pub struct Timer {
    inner: fi::LocalTimerFuture<'static>,
}

impl Timer {
    pub fn at(when: Instant) -> Self {
        let svc: &TimerService = current_timer_service();
        Self {
            inner: svc.deadline(when.into_ticks()),
        }
    }

    pub fn after(dur: Duration) -> Self {
        Self::at(Instant::now() + dur)
    }
}

impl Future for Timer {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        unsafe { Pin::new_unchecked(&mut self.get_unchecked_mut().inner) }.poll(cx)
    }
}

/// Trait to register a callback at a given timestamp.
pub trait Alarm {
    /// Sets the callback function to be called when the alarm triggers.
    /// The callback may be called from any context (interrupt or thread mode).
    fn set_callback(&self, callback: fn());

    /// Sets an alarm at the given timestamp. When the clock reaches that
    /// timestamp, the provided callback funcion will be called.
    ///
    /// When callback is called, it is guaranteed that now() will return a value greater or equal than timestamp.
    ///
    /// Only one alarm can be active at a time. This overwrites any previously-set alarm if any.
    fn set(&self, timestamp: u64);

    /// Clears the previously-set alarm.
    /// If no alarm was set, this is a noop.
    fn clear(&self);
}
