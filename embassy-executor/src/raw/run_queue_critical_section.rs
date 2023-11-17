use core::cell::Cell;

use critical_section::{CriticalSection, Mutex};

use super::TaskRef;

pub(crate) struct RunQueueItem {
    next: Mutex<Cell<Option<TaskRef>>>,
}

impl RunQueueItem {
    pub const fn new() -> Self {
        Self {
            next: Mutex::new(Cell::new(None)),
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
/// for our purposes: it can't create fairness problems since the next batch won't run until the
/// current batch is completely processed, so even if a task enqueues itself instantly (for example
/// by waking its own waker) can't prevent other tasks from running.
pub(crate) struct RunQueue {
    head: Mutex<Cell<Option<TaskRef>>>,
}

impl RunQueue {
    pub const fn new() -> Self {
        Self {
            head: Mutex::new(Cell::new(None)),
        }
    }

    /// Enqueues an item. Returns true if the queue was empty.
    ///
    /// # Safety
    ///
    /// `item` must NOT be already enqueued in any queue.
    #[inline(always)]
    pub(crate) unsafe fn enqueue(&self, task: TaskRef) -> bool {
        critical_section::with(|cs| {
            let prev = self.head.borrow(cs).replace(Some(task));
            task.header().run_queue_item.next.borrow(cs).set(prev);

            prev.is_none()
        })
    }

    /// Empty the queue, then call `on_task` for each task that was in the queue.
    /// NOTE: It is OK for `on_task` to enqueue more tasks. In this case they're left in the queue
    /// and will be processed by the *next* call to `dequeue_all`, *not* the current one.
    pub(crate) fn dequeue_all(&self, on_task: impl Fn(TaskRef)) {
        // Atomically empty the queue.
        let mut next = critical_section::with(|cs| self.head.borrow(cs).take());

        // Iterate the linked list of tasks that were previously in the queue.
        while let Some(task) = next {
            // If the task re-enqueues itself, the `next` pointer will get overwritten.
            // Therefore, first read the next pointer, and only then process the task.

            // safety: we know if the task is enqueued, no one else will touch the `next` pointer.
            let cs = unsafe { CriticalSection::new() };
            next = task.header().run_queue_item.next.borrow(cs).get();

            on_task(task);
        }
    }
}
