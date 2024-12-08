//! Timer queue operations.

use core::cell::Cell;

use super::TaskRef;

/// An item in the timer queue.
pub struct TimerQueueItem {
    /// The next item in the queue.
    pub next: Cell<Option<TaskRef>>,

    /// The time at which this item expires.
    pub expires_at: Cell<u64>,
}

unsafe impl Sync for TimerQueueItem {}

impl TimerQueueItem {
    pub(crate) const fn new() -> Self {
        Self {
            next: Cell::new(None),
            expires_at: Cell::new(0),
        }
    }
}
