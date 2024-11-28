use core::cell::Cell;
use core::cmp::min;

use critical_section::{CriticalSection, Mutex};

use super::{AlarmHandle, TaskRef};

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
    alarm: AlarmHandle,
}

impl TimerQueue {
    pub const fn new(alarm: AlarmHandle) -> Self {
        Self {
            head: Mutex::new(Cell::new(None)),
            alarm,
        }
    }

    pub(crate) unsafe fn notify_task_exited(&self, p: TaskRef) {
        let task = p.header();
        critical_section::with(|cs| {
            task.expires_at.borrow(cs).set(u64::MAX);
            self.dispatch(cs, super::wake_task);
        });
    }

    pub(crate) unsafe fn notify_task_started(&self, p: TaskRef) {
        let task = p.header();
        critical_section::with(|cs| {
            task.expires_at.borrow(cs).set(u64::MAX);
        });
    }

    pub(crate) unsafe fn update(&self, p: TaskRef, at: u64) {
        let task = p.header();
        critical_section::with(|cs| {
            if at < task.expires_at.borrow(cs).get() {
                task.expires_at.borrow(cs).set(at);
                if task.state.timer_enqueue() {
                    let prev = self.head.borrow(cs).replace(Some(p));
                    task.timer_queue_item.next.borrow(cs).set(prev);
                }
                self.dispatch(cs, super::wake_task);
            }
        });
    }

    unsafe fn dequeue_expired_internal(&self, now: u64, cs: CriticalSection<'_>, on_task: fn(TaskRef)) -> bool {
        let mut changed = false;
        self.retain(cs, |p| {
            let task = p.header();
            if task.expires_at.borrow(cs).get() <= now {
                on_task(p);
                changed = true;
                false
            } else {
                true
            }
        });
        changed
    }

    pub(crate) unsafe fn dequeue_expired(&self, now: u64, on_task: fn(TaskRef)) {
        critical_section::with(|cs| {
            if self.dequeue_expired_internal(now, cs, on_task) {
                self.dispatch(cs, on_task);
            }
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

    unsafe fn next_expiration(&self, cs: CriticalSection<'_>) -> u64 {
        let mut res = u64::MAX;

        self.retain(cs, |p| {
            let task = p.header();
            let expires = task.expires_at.borrow(cs).get();
            res = min(res, expires);
            expires != u64::MAX
        });

        res
    }

    unsafe fn dispatch(&self, cs: CriticalSection<'_>, cb: fn(TaskRef)) {
        // If this is already in the past, set_alarm might return false.
        let next_expiration = self.next_expiration(cs);
        if !embassy_time_driver::set_alarm(self.alarm, next_expiration) {
            // Time driver did not schedule the alarm,
            // so we need to dequeue expired timers manually.
            self.dequeue_expired_internal(embassy_time_driver::now(), cs, cb);
        }
    }
}
