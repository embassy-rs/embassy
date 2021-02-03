use core::cell::Cell;
use core::cmp::min;
use core::ptr;
use core::ptr::NonNull;
use core::sync::atomic::{AtomicPtr, Ordering};

use super::raw::{Task, STATE_TIMER_QUEUED};
use crate::time::Instant;

pub(crate) struct TimerQueueItem {
    next: Cell<*mut Task>,
}

impl TimerQueueItem {
    pub const fn new() -> Self {
        Self {
            next: Cell::new(ptr::null_mut()),
        }
    }
}

pub(crate) struct TimerQueue {
    head: Cell<*mut Task>,
}

impl TimerQueue {
    pub const fn new() -> Self {
        Self {
            head: Cell::new(ptr::null_mut()),
        }
    }

    pub(crate) unsafe fn update(&self, p: NonNull<Task>) {
        let task = p.as_ref();
        if task.expires_at.get() != Instant::MAX {
            let old_state = task.state.fetch_or(STATE_TIMER_QUEUED, Ordering::AcqRel);
            let is_new = old_state & STATE_TIMER_QUEUED == 0;

            if is_new {
                task.timer_queue_item.next.set(self.head.get());
                self.head.set(p.as_ptr());
            }
        }
    }

    pub(crate) unsafe fn next_expiration(&self) -> Instant {
        let mut res = Instant::MAX;
        self.retain(|p| {
            let task = p.as_ref();
            let expires = task.expires_at.get();
            res = min(res, expires);
            expires != Instant::MAX
        });
        res
    }

    pub(crate) unsafe fn dequeue_expired(&self, now: Instant, on_task: impl Fn(NonNull<Task>)) {
        self.retain(|p| {
            let task = p.as_ref();
            if task.expires_at.get() <= now {
                on_task(p);
                false
            } else {
                true
            }
        });
    }

    pub(crate) unsafe fn retain(&self, mut f: impl FnMut(NonNull<Task>) -> bool) {
        let mut prev = &self.head;
        while !prev.get().is_null() {
            let p = NonNull::new_unchecked(prev.get());
            let task = &*p.as_ptr();
            if f(p) {
                // Skip to next
                prev = &task.timer_queue_item.next;
            } else {
                // Remove it
                prev.set(task.timer_queue_item.next.get());
                task.state.fetch_and(!STATE_TIMER_QUEUED, Ordering::AcqRel);
            }
        }
    }
}
