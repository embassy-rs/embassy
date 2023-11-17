use core::cmp::min;

use embassy_time::Instant;

use super::TaskRef;
use crate::raw::util::SyncUnsafeCell;

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

pub(crate) struct TimerQueue {
    head: SyncUnsafeCell<Option<TaskRef>>,
}

impl TimerQueue {
    pub const fn new() -> Self {
        Self {
            head: SyncUnsafeCell::new(None),
        }
    }

    pub(crate) unsafe fn update(&self, p: TaskRef) {
        let task = p.header();
        if task.expires_at.get() != Instant::MAX {
            if task.state.timer_enqueue() {
                task.timer_queue_item.next.set(self.head.get());
                self.head.set(Some(p));
            }
        }
    }

    pub(crate) unsafe fn next_expiration(&self) -> Instant {
        let mut res = Instant::MAX;
        self.retain(|p| {
            let task = p.header();
            let expires = task.expires_at.get();
            res = min(res, expires);
            expires != Instant::MAX
        });
        res
    }

    pub(crate) unsafe fn dequeue_expired(&self, now: Instant, on_task: impl Fn(TaskRef)) {
        self.retain(|p| {
            let task = p.header();
            if task.expires_at.get() <= now {
                on_task(p);
                false
            } else {
                true
            }
        });
    }

    pub(crate) unsafe fn retain(&self, mut f: impl FnMut(TaskRef) -> bool) {
        let mut prev = &self.head;
        while let Some(p) = prev.get() {
            let task = p.header();
            if f(p) {
                // Skip to next
                prev = &task.timer_queue_item.next;
            } else {
                // Remove it
                prev.set(task.timer_queue_item.next.get());
                task.state.timer_dequeue();
            }
        }
    }
}
