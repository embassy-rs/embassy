use core::future::{poll_fn, Future};
use core::sync::atomic::{AtomicU32, Ordering};
use core::task::Poll;

/// A type for interacting with the deadline of the current task
///
/// Requires the `edf-scheduler` feature.
///
/// Note: Interacting with the deadline should be done locally in a task.
/// In theory you could try to set or read the deadline from another task,
/// but that will result in weird (though not unsound) behavior.
pub struct Deadline {
    instant_ticks_hi: AtomicU32,
    instant_ticks_lo: AtomicU32,
}

impl Deadline {
    pub(crate) const fn new(instant_ticks: u64) -> Self {
        Self {
            instant_ticks_hi: AtomicU32::new((instant_ticks >> 32) as u32),
            instant_ticks_lo: AtomicU32::new(instant_ticks as u32),
        }
    }

    pub(crate) const fn new_unset() -> Self {
        Self::new(Self::UNSET_DEADLINE_TICKS)
    }

    pub(crate) fn set(&self, instant_ticks: u64) {
        self.instant_ticks_hi
            .store((instant_ticks >> 32) as u32, Ordering::Relaxed);
        self.instant_ticks_lo.store(instant_ticks as u32, Ordering::Relaxed);
    }

    /// Deadline value in ticks, same time base and ticks as `embassy-time`
    pub fn instant_ticks(&self) -> u64 {
        let hi = self.instant_ticks_hi.load(Ordering::Relaxed) as u64;
        let lo = self.instant_ticks_lo.load(Ordering::Relaxed) as u64;

        (hi << 32) | lo
    }

    /// Sentinel value representing an "unset" deadline, which has lower priority
    /// than any other set deadline value
    pub const UNSET_DEADLINE_TICKS: u64 = u64::MAX;

    /// Does the given Deadline represent an "unset" deadline?
    #[inline]
    pub fn is_unset(&self) -> bool {
        self.instant_ticks() == Self::UNSET_DEADLINE_TICKS
    }

    /// Set the current task's deadline at exactly `instant_ticks`
    ///
    /// This method is a future in order to access the currently executing task's
    /// header which contains the deadline.
    ///
    /// Analogous to `Timer::at`.
    ///
    /// This method does NOT check whether the deadline has already passed.
    #[must_use = "Setting deadline must be polled to be effective"]
    pub fn set_current_task_deadline(instant_ticks: u64) -> impl Future<Output = ()> {
        poll_fn(move |cx| {
            let task = super::task_from_waker(cx.waker());
            task.header().deadline.set(instant_ticks);
            Poll::Ready(())
        })
    }

    /// Set the current task's deadline `duration_ticks` in the future from when
    /// this future is polled. This deadline is saturated to the max tick value.
    ///
    /// This method is a future in order to access the currently executing task's
    /// header which contains the deadline.
    ///
    /// Analogous to `Timer::after`.
    ///
    /// Returns the deadline that was set.
    #[must_use = "Setting deadline must be polled to be effective"]
    pub fn set_current_task_deadline_after(duration_ticks: u64) -> impl Future<Output = Deadline> {
        poll_fn(move |cx| {
            let task = super::task_from_waker(cx.waker());
            let now = embassy_time_driver::now();

            // Since ticks is a u64, saturating add is PROBABLY overly cautious, leave
            // it for now, we can probably make this wrapping_add for performance
            // reasons later.
            let deadline = now.saturating_add(duration_ticks);

            task.header().deadline.set(deadline);

            Poll::Ready(Deadline::new(deadline))
        })
    }

    /// Set the current task's deadline `increment_ticks` from the previous deadline.
    ///
    /// This deadline is saturated to the max tick value.
    ///
    /// Note that by default (unless otherwise set), tasks start life with the deadline
    /// u64::MAX, which means this method will have no effect.
    ///
    /// This method is a future in order to access the currently executing task's
    /// header which contains the deadline
    ///
    /// Analogous to one increment of `Ticker::every().next()`.
    ///
    /// Returns the deadline that was set.
    #[must_use = "Setting deadline must be polled to be effective"]
    pub fn increment_current_task_deadline(increment_ticks: u64) -> impl Future<Output = Deadline> {
        poll_fn(move |cx| {
            let task = super::task_from_waker(cx.waker());

            // Get the last value
            let last = task.header().deadline.instant_ticks();

            // Since ticks is a u64, saturating add is PROBABLY overly cautious, leave
            // it for now, we can probably make this wrapping_add for performance
            // reasons later.
            let deadline = last.saturating_add(increment_ticks);

            // Store the new value
            task.header().deadline.set(deadline);

            Poll::Ready(Deadline::new(deadline))
        })
    }

    /// Get the current task's deadline as a tick value.
    ///
    /// This method is a future in order to access the currently executing task's
    /// header which contains the deadline
    pub fn get_current_task_deadline() -> impl Future<Output = Self> {
        poll_fn(move |cx| {
            let task = super::task_from_waker(cx.waker());

            let deadline = task.header().deadline.instant_ticks();
            Poll::Ready(Self::new(deadline))
        })
    }

    /// Clear the current task's deadline, returning the previous value.
    ///
    /// This sets the deadline to the default value of `u64::MAX`, meaning all
    /// tasks with set deadlines will be scheduled BEFORE this task.
    #[must_use = "Clearing deadline must be polled to be effective"]
    pub fn clear_current_task_deadline() -> impl Future<Output = Self> {
        poll_fn(move |cx| {
            let task = super::task_from_waker(cx.waker());

            // get the old value
            let deadline = task.header().deadline.instant_ticks();
            // Store the default value
            task.header().deadline.set(Self::UNSET_DEADLINE_TICKS);

            Poll::Ready(Self::new(deadline))
        })
    }
}
