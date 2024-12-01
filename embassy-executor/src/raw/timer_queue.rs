//! Timer queue.
use core::cmp::min;

use super::util::SyncUnsafeCell;
use super::TaskRef;

pub(crate) struct TimerQueueItem {
    next: SyncUnsafeCell<Option<TaskRef>>,
}

impl TimerQueueItem {
    pub const fn new() -> Self {
        Self {
            next: SyncUnsafeCell::new(None),
        }
    }
}

/// A timer queue, with items integrated into tasks.
pub struct TimerQueue {
    head: SyncUnsafeCell<Option<TaskRef>>,
}

impl TimerQueue {
    /// Creates a new timer queue.
    pub const fn new() -> Self {
        Self {
            head: SyncUnsafeCell::new(None),
        }
    }

    /// Schedules a task to run at a specific time.
    ///
    /// Returns whether the task was scheduled. `false` means the task was already
    /// scheduled to wake at an earlier time.
    pub unsafe fn schedule_wake(&self, p: TaskRef, at: u64) -> bool {
        let task = p.header();
        if task.state.timer_enqueue() {
            // If not in the queue, add it and update.
            let prev = self.head.replace(Some(p));
            task.timer_queue_item.next.set(prev);
        } else if at <= task.expires_at.get() {
            // If expiration is sooner than previously set, update.
        } else {
            // Task does not need to be updated.
            return false;
        }

        task.expires_at.set(at);
        true
    }

    /// Dequeues expired timers and returns the next alarm time.
    ///
    /// The provided callback will be called for each expired task. Tasks that never expire
    /// will be removed, but the callback will not be called.
    pub unsafe fn next_expiration(&self, now: u64, on_task: fn(TaskRef)) -> u64 {
        let mut next_expiration = u64::MAX;

        self.retain(|p| {
            let task = p.header();
            let expires = task.expires_at.get();

            if expires <= now {
                // Timer expired, process task.
                on_task(p);
                false
            } else {
                // Timer didn't yet expire, or never expires.
                next_expiration = min(next_expiration, expires);
                expires != u64::MAX
            }
        });

        next_expiration
    }

    unsafe fn retain(&self, mut f: impl FnMut(TaskRef) -> bool) {
        let mut prev = &self.head;
        while let Some(p) = prev.get() {
            let task = p.header();
            if f(p) {
                prev = &task.timer_queue_item.next;
            } else {
                prev.set(task.timer_queue_item.next.get());
                task.state.timer_dequeue();
            }
        }
    }
}
