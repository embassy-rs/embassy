use core::cell::Cell;
use core::sync::atomic::{AtomicPtr, Ordering};
use core::{cmp::min, ptr};

use crate::time::Instant;

use super::{TaskHeader, STATE_TIMER_QUEUED};

pub(crate) struct TimerQueueItem {
    next: Cell<*mut TaskHeader>,
}

impl TimerQueueItem {
    pub const fn new() -> Self {
        Self {
            next: Cell::new(ptr::null_mut()),
        }
    }
}

pub(crate) struct TimerQueue {
    head: Cell<*mut TaskHeader>,
}

impl TimerQueue {
    pub const fn new() -> Self {
        Self {
            head: Cell::new(ptr::null_mut()),
        }
    }

    pub(crate) unsafe fn update(&self, p: *mut TaskHeader) {
        let header = &*p;
        if header.expires_at.get() != Instant::MAX {
            let old_state = header.state.fetch_or(STATE_TIMER_QUEUED, Ordering::AcqRel);
            let is_new = old_state & STATE_TIMER_QUEUED == 0;

            if is_new {
                header.timer_queue_item.next.set(self.head.get());
                self.head.set(p);
            }
        }
    }

    pub(crate) unsafe fn next_expiration(&self) -> Instant {
        let mut res = Instant::MAX;
        self.retain(|p| {
            let header = &*p;
            let expires = header.expires_at.get();
            res = min(res, expires);
            expires != Instant::MAX
        });
        res
    }

    pub(crate) unsafe fn dequeue_expired(&self, now: Instant, on_task: impl Fn(*mut TaskHeader)) {
        self.retain(|p| {
            let header = &*p;
            if header.expires_at.get() <= now {
                on_task(p);
                false
            } else {
                true
            }
        });
    }

    pub(crate) unsafe fn retain(&self, mut f: impl FnMut(*mut TaskHeader) -> bool) {
        let mut prev = &self.head;
        while !prev.get().is_null() {
            let p = prev.get();
            let header = &*p;
            if f(p) {
                // Skip to next
                prev = &header.timer_queue_item.next;
            } else {
                // Remove it
                prev.set(header.timer_queue_item.next.get());
                header
                    .state
                    .fetch_and(!STATE_TIMER_QUEUED, Ordering::AcqRel);
            }
        }
    }
}
