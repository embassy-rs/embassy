//! Timer queue operations.
use core::cell::Cell;
use core::cmp::min;
use core::task::Waker;

use embassy_executor::raw::TaskRef;

/// A timer queue, with items integrated into tasks.
pub struct Queue {
    head: Cell<Option<TaskRef>>,
}

impl Queue {
    /// Creates a new timer queue.
    pub const fn new() -> Self {
        Self { head: Cell::new(None) }
    }

    /// Schedules a task to run at a specific time.
    ///
    /// If this function returns `true`, the called should find the next expiration time and set
    /// a new alarm for that time.
    pub fn schedule_wake(&mut self, at: u64, waker: &Waker) -> bool {
        let task = embassy_executor::raw::task_from_waker(waker);
        let item = task.timer_queue_item();
        if item.next.get().is_none() {
            // If not in the queue, add it and update.
            let prev = self.head.replace(Some(task));
            item.next.set(if prev.is_none() {
                Some(unsafe { TaskRef::dangling() })
            } else {
                prev
            });
            item.expires_at.set(at);
            true
        } else if at <= item.expires_at.get() {
            // If expiration is sooner than previously set, update.
            item.expires_at.set(at);
            true
        } else {
            // Task does not need to be updated.
            false
        }
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
            if unsafe { p == TaskRef::dangling() } {
                // prev was the last item, stop
                break;
            }
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
