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

        // Trigger removal from the timer queue.
        task.expires_at.set(u64::MAX);
        self.dispatch(super::wake_task);
    }

    pub(crate) unsafe fn schedule(&self, p: TaskRef, at: u64) {
        let task = p.header();
        let update = if task.state.timer_enqueue() {
            // Not in the queue, add it and update.
            let prev = self.head.replace(Some(p));
            task.timer_queue_item.next.set(prev);

            true
        } else {
            // Expiration is sooner than previously set, update.
            at < task.expires_at.get()
        };

        if update {
            task.expires_at.set(at);
            self.dispatch(super::wake_task);
        }
    }

    pub(crate) unsafe fn dispatch(&self, on_task: fn(TaskRef)) {
        loop {
            let now = embassy_time_driver::now();

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

            if self.update_alarm(next_expiration) {
                break;
            }
        }
    }

    fn update_alarm(&self, next_alarm: u64) -> bool {
        if next_alarm == u64::MAX {
            true
        } else {
            embassy_time_driver::set_alarm(self.alarm, next_alarm)
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
}
