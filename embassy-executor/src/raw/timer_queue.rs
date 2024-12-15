//! Timer queue operations.

use core::cell::Cell;

use super::TaskRef;

/// An item in the timer queue.
pub struct TimerQueueItem {
    /// The next item in the queue.
    ///
    /// If this field contains `Some`, the item is in the queue. The last item in the queue has a
    /// value of `Some(dangling_pointer)`
    pub next: Cell<Option<TaskRef>>,

    /// The time at which this item expires.
    pub expires_at: Cell<u64>,
}

unsafe impl Sync for TimerQueueItem {}

impl TimerQueueItem {
    pub(crate) const fn new() -> Self {
        Self {
            next: Cell::new(None),
            expires_at: Cell::new(u64::MAX),
        }
    }
}

/// The operation to perform after `timer_enqueue` is called.
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[must_use]
pub enum TimerEnqueueOperation {
    /// Enqueue the task (or update its expiration time).
    Enqueue,
    /// The task must not be enqueued in the timer queue.
    Ignore,
}
