#[cfg(feature = "metadata-name")]
use core::cell::Cell;
use core::future::{poll_fn, Future};
use core::task::Poll;

#[cfg(feature = "metadata-name")]
use critical_section::Mutex;

use crate::raw;
#[cfg(feature = "scheduler-deadline")]
use crate::raw::Deadline;

/// Metadata associated with a task.
pub struct Metadata {
    #[cfg(feature = "metadata-name")]
    name: Mutex<Cell<Option<&'static str>>>,
    #[cfg(feature = "scheduler-deadline")]
    deadline: raw::Deadline,
}

impl Metadata {
    pub(crate) const fn new() -> Self {
        Self {
            #[cfg(feature = "metadata-name")]
            name: Mutex::new(Cell::new(None)),
            // NOTE: The deadline is set to zero to allow the initializer to reside in `.bss`. This
            // will be lazily initalized in `initialize_impl`
            #[cfg(feature = "scheduler-deadline")]
            deadline: raw::Deadline::new_unset(),
        }
    }

    pub(crate) fn reset(&self) {
        #[cfg(feature = "metadata-name")]
        critical_section::with(|cs| self.name.borrow(cs).set(None));
    }

    /// Get the metadata for the current task.
    ///
    /// You can use this to read or modify the current task's metadata.
    ///
    /// This function is `async` just to get access to the current async
    /// context. It returns instantly, it does not block/yield.
    pub fn for_current_task() -> impl Future<Output = &'static Self> {
        poll_fn(|cx| Poll::Ready(raw::task_from_waker(cx.waker()).metadata()))
    }

    /// Get this task's name
    ///
    /// NOTE: this takes a critical section.
    #[cfg(feature = "metadata-name")]
    pub fn name(&self) -> Option<&'static str> {
        critical_section::with(|cs| self.name.borrow(cs).get())
    }

    /// Set this task's name
    ///
    /// NOTE: this takes a critical section.
    #[cfg(feature = "metadata-name")]
    pub fn set_name(&self, name: &'static str) {
        critical_section::with(|cs| self.name.borrow(cs).set(Some(name)))
    }

    /// Get this task's deadline.
    #[cfg(feature = "scheduler-deadline")]
    pub fn deadline(&self) -> u64 {
        self.deadline.instant_ticks()
    }

    /// Set this task's deadline.
    ///
    /// This method does NOT check whether the deadline has already passed.
    #[cfg(feature = "scheduler-deadline")]
    pub fn set_deadline(&self, instant_ticks: u64) {
        self.deadline.set(instant_ticks);
    }

    /// Remove this task's deadline.
    /// This brings it back to the defaul where it's not scheduled ahead of other tasks.
    #[cfg(feature = "scheduler-deadline")]
    pub fn unset_deadline(&self) {
        self.deadline.set(Deadline::UNSET_TICKS);
    }

    /// Set this task's deadline `duration_ticks` in the future from when
    /// this future is polled. This deadline is saturated to the max tick value.
    ///
    /// Analogous to `Timer::after`.
    #[cfg(all(feature = "scheduler-deadline", feature = "embassy-time-driver"))]
    pub fn set_deadline_after(&self, duration_ticks: u64) {
        let now = embassy_time_driver::now();

        // Since ticks is a u64, saturating add is PROBABLY overly cautious, leave
        // it for now, we can probably make this wrapping_add for performance
        // reasons later.
        let deadline = now.saturating_add(duration_ticks);

        self.set_deadline(deadline);
    }

    /// Set the this task's deadline `increment_ticks` from the previous deadline.
    ///
    /// This deadline is saturated to the max tick value.
    ///
    /// Note that by default (unless otherwise set), tasks start life with the deadline
    /// not set, which means this method will have no effect.
    ///
    /// Analogous to one increment of `Ticker::every().next()`.
    ///
    /// Returns the deadline that was set.
    #[cfg(feature = "scheduler-deadline")]
    pub fn increment_deadline(&self, duration_ticks: u64) {
        let last = self.deadline();

        // Since ticks is a u64, saturating add is PROBABLY overly cautious, leave
        // it for now, we can probably make this wrapping_add for performance
        // reasons later.
        let deadline = last.saturating_add(duration_ticks);

        self.set_deadline(deadline);
    }
}
