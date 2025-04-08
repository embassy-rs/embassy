use core::future::{poll_fn, Future};
use core::task::Poll;

#[cfg(not(target_has_atomic = "ptr"))]
compile_error!("The `drs-scheduler` feature is currently only supported on targets with atomics.");

/// A type for interacting with the deadline of the current task
///
/// Requires the `drs-scheduler` feature
pub struct Deadline {
    /// Deadline value in ticks, same time base and ticks as `embassy-time`
    pub instant_ticks: u64,
}

impl Deadline {
    /// Sentinel value representing an "unset" deadline, which has lower priority
    /// than any other set deadline value
    pub const UNSET_DEADLINE_TICKS: u64 = u64::MAX;

    /// Does the given Deadline represent an "unset" deadline?
    #[inline]
    pub fn is_unset(&self) -> bool {
        self.instant_ticks == Self::UNSET_DEADLINE_TICKS
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
            // SAFETY: A task can only modify its own deadline, while the task is being
            // polled, meaning that there cannot be concurrent access to the deadline.
            unsafe {
                task.header().deadline.set(instant_ticks);
            }
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

            // SAFETY: A task can only modify its own deadline, while the task is being
            // polled, meaning that there cannot be concurrent access to the deadline.
            unsafe {
                task.header().deadline.set(deadline);
            }
            Poll::Ready(Deadline {
                instant_ticks: deadline,
            })
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

            // SAFETY: A task can only modify its own deadline, while the task is being
            // polled, meaning that there cannot be concurrent access to the deadline.
            unsafe {
                // Get the last value
                let last = task.header().deadline.get();

                // Since ticks is a u64, saturating add is PROBABLY overly cautious, leave
                // it for now, we can probably make this wrapping_add for performance
                // reasons later.
                let deadline = last.saturating_add(increment_ticks);

                // Store the new value
                task.header().deadline.set(deadline);

                Poll::Ready(Deadline {
                    instant_ticks: deadline,
                })
            }
        })
    }

    /// Get the current task's deadline as a tick value.
    ///
    /// This method is a future in order to access the currently executing task's
    /// header which contains the deadline
    pub fn get_current_task_deadline() -> impl Future<Output = Self> {
        poll_fn(move |cx| {
            let task = super::task_from_waker(cx.waker());

            // SAFETY: A task can only modify its own deadline, while the task is being
            // polled, meaning that there cannot be concurrent access to the deadline.
            let deadline = unsafe { task.header().deadline.get() };
            Poll::Ready(Self {
                instant_ticks: deadline,
            })
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

            // SAFETY: A task can only modify its own deadline, while the task is being
            // polled, meaning that there cannot be concurrent access to the deadline.
            let deadline = unsafe {
                // get the old value
                let d = task.header().deadline.get();
                // Store the default value
                task.header().deadline.set(Self::UNSET_DEADLINE_TICKS);
                // return the old value
                d
            };

            Poll::Ready(Self {
                instant_ticks: deadline,
            })
        })
    }
}
