use core::cell::Cell;
use core::cmp::min;

use critical_section::{CriticalSection, Mutex};

use super::TaskRef;

pub(crate) struct TimerQueueItem {
    next: Mutex<Cell<Option<TaskRef>>>,
}

impl TimerQueueItem {
    pub const fn new() -> Self {
        Self {
            next: Mutex::new(Cell::new(None)),
        }
    }
}

pub(crate) struct TimerQueue {
    head: Mutex<Cell<Option<TaskRef>>>,
}

impl TimerQueue {
    pub const fn new() -> Self {
        Self {
            head: Mutex::new(Cell::new(None)),
        }
    }

    pub(crate) unsafe fn update(&self, p: TaskRef, at: u64) {
        let task = p.header();
        if at < task.next_expiration.get() {
            task.next_expiration.set(at);
            critical_section::with(|cs| {
                if task.state.timer_enqueue() {
                    let prev = self.head.borrow(cs).replace(Some(p));
                    task.timer_queue_item.next.borrow(cs).set(prev);
                }
            });
        }
    }

    pub(crate) unsafe fn next_expiration(&self) -> u64 {
        let mut res = u64::MAX;
        critical_section::with(|cs| {
            self.retain(cs, |p| {
                let task = p.header();
                let expires = task.next_expiration.get();
                task.expires_at.borrow(cs).set(expires);
                res = min(res, expires);
                expires != u64::MAX
            });
        });
        res
    }

    pub(crate) unsafe fn dequeue_expired(&self, now: u64, on_task: impl Fn(TaskRef)) {
        critical_section::with(|cs| {
            self.retain(cs, |p| {
                let task = p.header();
                if task.expires_at.borrow(cs).get() <= now {
                    on_task(p);
                    false
                } else {
                    true
                }
            });
        });
    }

    unsafe fn retain(&self, cs: CriticalSection<'_>, mut f: impl FnMut(TaskRef) -> bool) {
        let mut prev = &self.head;
        while let Some(p) = prev.borrow(cs).get() {
            let task = p.header();
            if f(p) {
                prev = &task.timer_queue_item.next;
            } else {
                prev.borrow(cs).set(task.timer_queue_item.next.borrow(cs).get());
                task.state.timer_dequeue();
            }
        }
    }
}
