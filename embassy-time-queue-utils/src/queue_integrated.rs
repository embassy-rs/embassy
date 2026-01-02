//! Timer queue operations.
use core::cell::Cell;
use core::cmp::min;
use core::ptr::NonNull;
use core::task::Waker;

use embassy_executor_timer_queue::TimerQueueItem;

/// An item in the timer queue.
#[derive(Default)]
struct QueueItem {
    /// The next item in the queue.
    ///
    /// If this field contains `Some`, the item is in the queue. The last item in the queue has a
    /// value of `Some(dangling_pointer)`
    pub next: Cell<Option<NonNull<QueueItem>>>,

    /// The time at which this item expires.
    pub expires_at: u64,

    /// The registered waker. If Some, the item is enqueued in the timer queue.
    pub waker: Option<Waker>,
}

unsafe impl Sync for QueueItem {}

/// A timer queue, with items integrated into tasks.
///
/// # Safety
///
/// **This Queue is only safe when there is a single integrated queue in the system.**
///
/// If there are multiple integrated queues, additional checks are necessary to ensure that a Waker
/// is not attempted to be enqueued in multiple queues.
pub struct Queue {
    head: Cell<Option<NonNull<QueueItem>>>,
}

impl core::fmt::Debug for Queue {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Queue").finish()
    }
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
        let item = unsafe {
            // Safety: the `&mut self`, along with the Safety note of the Queue, are sufficient to
            // ensure that this function creates the only mutable reference to the queue item.
            TimerQueueItem::from_embassy_waker(waker)
        };
        let item = unsafe { item.as_mut::<QueueItem>() };
        match item.waker.as_ref() {
            Some(_) if at <= item.expires_at => {
                // If expiration is sooner than previously set, update.
                item.expires_at = at;
                // The waker is always stored in its own queue item, so we don't need to update it.

                // Trigger a queue update in case this item can be immediately dequeued.
                true
            }
            Some(_) => {
                // Queue item does not need to be updated, the task will be scheduled to be woken
                // before the new expiration.
                false
            }
            None => {
                // If not in the queue, add it and update.
                let mut item_ptr = NonNull::from(item);
                let prev = self.head.replace(Some(item_ptr));

                let item = unsafe { item_ptr.as_mut() };

                item.expires_at = at;
                item.waker = Some(waker.clone());
                item.next.set(prev);
                // The default implementation doesn't care about the
                // opaque payload, leave it unchanged.

                true
            }
        }
    }

    /// Dequeues expired timers and returns the next alarm time.
    ///
    /// The provided callback will be called for each expired task. Tasks that never expire
    /// will be removed, but the callback will not be called.
    pub fn next_expiration(&mut self, now: u64) -> u64 {
        let mut next_expiration = u64::MAX;

        self.retain(|item| {
            if item.expires_at <= now {
                // Timer expired, process task.
                if let Some(waker) = item.waker.take() {
                    waker.wake();
                }
                false
            } else {
                // Timer didn't yet expire, or never expires.
                next_expiration = min(next_expiration, item.expires_at);
                item.expires_at != u64::MAX
            }
        });

        next_expiration
    }

    fn retain(&mut self, mut f: impl FnMut(&mut QueueItem) -> bool) {
        let mut prev = &self.head;
        while let Some(mut p) = prev.get() {
            let mut item = unsafe { p.as_mut() };

            if f(&mut item) {
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
