use atomic_polyfill::{AtomicPtr, Ordering};
use core::ptr;
use core::ptr::NonNull;

use super::TaskHeader;

pub(crate) struct RunQueueItem {
    next: AtomicPtr<TaskHeader>,
}

impl RunQueueItem {
    pub const fn new() -> Self {
        Self {
            next: AtomicPtr::new(ptr::null_mut()),
        }
    }
}

/// Atomic task queue using a very, very simple lock-free linked-list queue:
///
/// To enqueue a task, task.next is set to the old head, and head is atomically set to task.
///
/// Dequeuing is done in batches: the queue is emptied by atomically replacing head with
/// null. Then the batch is iterated following the next pointers until null is reached.
///
/// Note that batches will be iterated in the reverse order as they were enqueued. This is OK
/// for our purposes: it can't crate fairness problems since the next batch won't run until the
/// current batch is completely processed, so even if a task enqueues itself instantly (for example
/// by waking its own waker) can't prevent other tasks from running.
pub(crate) struct RunQueue {
    head: AtomicPtr<TaskHeader>,
}

impl RunQueue {
    pub const fn new() -> Self {
        Self {
            head: AtomicPtr::new(ptr::null_mut()),
        }
    }

    /// Enqueues an item. Returns true if the queue was empty.
    pub(crate) unsafe fn enqueue(&self, item: *mut TaskHeader) -> bool {
        let mut prev = self.head.load(Ordering::Acquire);
        loop {
            (*item).run_queue_item.next.store(prev, Ordering::Relaxed);
            match self
                .head
                .compare_exchange_weak(prev, item, Ordering::AcqRel, Ordering::Acquire)
            {
                Ok(_) => break,
                Err(next_prev) => prev = next_prev,
            }
        }

        prev.is_null()
    }

    pub(crate) unsafe fn dequeue_all(&self, on_task: impl Fn(NonNull<TaskHeader>)) {
        let mut task = self.head.swap(ptr::null_mut(), Ordering::AcqRel);

        while !task.is_null() {
            // If the task re-enqueues itself, the `next` pointer will get overwritten.
            // Therefore, first read the next pointer, and only then process the task.
            let next = (*task).run_queue_item.next.load(Ordering::Relaxed);

            on_task(NonNull::new_unchecked(task));

            task = next
        }
    }
}
