//! Timer queue operations.
use core::cell::Cell;
use core::cmp::min;
use core::task::Waker;

extern "Rust" {
    // Waker -> TimerQueueItem, validates that Waker is an embassy Waker.
    fn __embassy_time_queue_item_from_waker(waker: &Waker) -> &'static super::TimerQueueItem;
    // Waker data -> TimerQueueItem, only call after the pointer has been validated.
    fn __embassy_time_queue_item_from_waker_data(data: *const ()) -> &'static super::TimerQueueItem;
    // To avoid storing a Waker & dynamic dispatch via Waker.
    fn __embassy_time_queue_wake_task_from_data(data: *const ());
}

/// A timer queue, with items integrated into tasks.
#[derive(Debug)]
pub struct Queue {
    head: Cell<Option<*const ()>>,
}

unsafe impl Send for Queue {}
unsafe impl Sync for Queue {}

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
        let item = unsafe { __embassy_time_queue_item_from_waker(waker) };
        if item.next.get().is_none() {
            // If not in the queue, add it and update.
            let prev = self.head.replace(Some(waker.data()));
            item.next.set(if prev.is_none() {
                Some(core::ptr::dangling())
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
            let item = unsafe { __embassy_time_queue_item_from_waker_data(p) };
            let expires = item.expires_at.get();

            if expires <= now {
                // Timer expired, process task.
                unsafe { __embassy_time_queue_wake_task_from_data(p) };
                false
            } else {
                // Timer didn't yet expire, or never expires.
                next_expiration = min(next_expiration, expires);
                expires != u64::MAX
            }
        });

        next_expiration
    }

    fn retain(&self, mut f: impl FnMut(*const ()) -> bool) {
        let mut prev = &self.head;
        while let Some(p) = prev.get() {
            if p == core::ptr::dangling() {
                // prev was the last item, stop
                break;
            }
            let item = unsafe { __embassy_time_queue_item_from_waker_data(p) };
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
