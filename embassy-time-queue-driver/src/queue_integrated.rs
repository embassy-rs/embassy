//! Timer queue operations.
use core::cell::Cell;
use core::cmp::min;

use embassy_executor::raw::TaskRef;

/// A timer queue, with items integrated into tasks.
pub struct TimerQueue {
    head: Cell<Option<TaskRef>>,
}

impl TimerQueue {
    /// Creates a new timer queue.
    pub const fn new() -> Self {
        Self { head: Cell::new(None) }
    }

    /// Schedules a task to run at a specific time.
    ///
    /// If this function returns `true`, the called should find the next expiration time and set
    /// a new alarm for that time.
    pub fn schedule_wake(&mut self, at: u64, p: TaskRef) -> bool {
        let item = p.timer_queue_item();
        if item.next.get().is_none() {
            // If not in the queue, add it and update.
            let prev = self.head.replace(Some(p));
            item.next.set(prev);
        } else if at <= item.expires_at.get() {
            // If expiration is sooner than previously set, update.
        } else {
            // Task does not need to be updated.
            return false;
        }

        item.expires_at.set(at);
        true
    }

    /// Dequeues expired timers and returns the next alarm time.
    ///
    /// The provided callback will be called for each expired task. Tasks that never expire
    /// will be removed, but the callback will not be called.
    pub fn next_expiration(&mut self, now: u64) -> u64 {
        let mut next_expiration = u64::MAX;

        self.retain(|p| {
            let item = p.timer_queue_item();
            let expires = item.expires_at.get();

            if expires <= now {
                // Timer expired, process task.
                embassy_executor::raw::wake_task(p);
                false
            } else {
                // Timer didn't yet expire, or never expires.
                next_expiration = min(next_expiration, expires);
                expires != u64::MAX
            }
        });

        next_expiration
    }

    fn retain(&self, mut f: impl FnMut(TaskRef) -> bool) {
        let mut prev = &self.head;
        while let Some(p) = prev.get() {
            let item = p.timer_queue_item();
            if f(p) {
                // Skip to next
                prev = &item.next;
            } else {
                // Remove it
                prev.set(item.next.get());
                item.next.set(None);
            }
        }
    }
}
