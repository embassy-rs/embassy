use core::cmp::min;

use super::util::SyncUnsafeCell;
use super::{AlarmHandle, TaskRef};

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
    alarm: AlarmHandle,
}

impl TimerQueue {
    pub const fn new(alarm: AlarmHandle) -> Self {
        Self {
            head: SyncUnsafeCell::new(None),
            alarm,
        }
    }

    pub(crate) unsafe fn notify_task_exited(&self, p: TaskRef) {
        let task = p.header();
        task.expires_at.set(u64::MAX);
        self.dispatch(super::wake_task);
    }

    pub(crate) unsafe fn notify_task_started(&self, p: TaskRef) {
        let task = p.header();
        task.expires_at.set(u64::MAX);
    }

    pub(crate) unsafe fn update(&self, p: TaskRef, at: u64) {
        let task = p.header();
        if at < task.expires_at.get() {
            task.expires_at.set(at);
            if task.state.timer_enqueue() {
                let prev = self.head.replace(Some(p));
                task.timer_queue_item.next.set(prev);
            }
            self.dispatch(super::wake_task);
        }
    }

    unsafe fn dequeue_expired_internal(&self, now: u64, on_task: fn(TaskRef)) -> bool {
        let mut changed = false;
        self.retain(|p| {
            let task = p.header();
            if task.expires_at.get() <= now {
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
        if self.dequeue_expired_internal(now, on_task) {
            self.dispatch(on_task);
        }
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

    unsafe fn next_expiration(&self) -> u64 {
        let mut res = u64::MAX;

        self.retain(|p| {
            let task = p.header();
            let expires = task.expires_at.get();
            res = min(res, expires);
            expires != u64::MAX
        });

        res
    }

    unsafe fn dispatch(&self, cb: fn(TaskRef)) {
        // If this is already in the past, set_alarm might return false.
        let next_expiration = self.next_expiration();
        if !embassy_time_driver::set_alarm(self.alarm, next_expiration) {
            // Time driver did not schedule the alarm,
            // so we need to dequeue expired timers manually.
            self.dequeue_expired_internal(embassy_time_driver::now(), cb);
        }
    }
}
