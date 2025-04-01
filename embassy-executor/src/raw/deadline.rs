use core::future::{poll_fn, Future};
use core::task::Poll;

/// A type for interacting with the deadline of the current task
pub struct Deadline {
    /// Deadline value in ticks, same time base and ticks as `embassy-time`
    pub instant_ticks: u64,
}

impl Deadline {
    /// Set the current task's deadline at exactly `instant_ticks`
    ///
    /// This method is a future in order to access the currently executing task's
    /// header which contains the deadline.
    ///
    /// Analogous to `Timer::at`.
    ///
    /// TODO: Should we check/panic if the deadline is in the past?
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
    /// this future is polled.
    ///
    /// This method is a future in order to access the currently executing task's
    /// header which contains the deadline
    ///
    /// Analogous to `Timer::after`
    ///
    /// TODO: Do we want to return what the deadline is?
    #[must_use = "Setting deadline must be polled to be effective"]
    pub fn set_current_task_deadline_after(duration_ticks: u64) -> impl Future<Output = ()> {
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
            Poll::Ready(())
        })
    }

    /// Set the current task's deadline `increment_ticks` from the previous deadline.
    ///
    /// Note that by default (unless otherwise set), tasks start life with the deadline
    /// u64::MAX, which means this method will have no effect.
    ///
    /// This method is a future in order to access the currently executing task's
    /// header which contains the deadline
    ///
    /// Analogous to one increment of `Ticker::every().next()`.
    ///
    /// TODO: Do we want to return what the deadline is?
    #[must_use = "Setting deadline must be polled to be effective"]
    pub fn increment_current_task_deadline(increment_ticks: u64) -> impl Future<Output = ()> {
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
            }
            Poll::Ready(())
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
}
