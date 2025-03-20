use super::{TaskHeader, TaskRef};
use cordyceps::{SortedList, TransferStack};
use core::future::{Future, poll_fn};
use core::task::Poll;

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
    stack: TransferStack<TaskHeader>,
}

impl RunQueue {
    pub const fn new() -> Self {
        Self {
            stack: TransferStack::new(),
        }
    }

    /// Enqueues an item. Returns true if the queue was empty.
    ///
    /// # Safety
    ///
    /// `item` must NOT be already enqueued in any queue.
    #[inline(always)]
    pub(crate) unsafe fn enqueue(&self, task: TaskRef, _: super::state::Token) -> bool {
        self.stack.push_was_empty(task)
    }

    /// Empty the queue, then call `on_task` for each task that was in the queue.
    /// NOTE: It is OK for `on_task` to enqueue more tasks. In this case they're left in the queue
    /// and will be processed by the *next* call to `dequeue_all`, *not* the current one.
    pub(crate) fn dequeue_all(&self, on_task: impl Fn(TaskRef)) {
        let mut sorted = SortedList::<TaskHeader>::new(|lhs, rhs| unsafe {
            // TODO: Do we need any kind of access control here? Not if we say that
            // tasks can only set their own priority, which they can't do if we're in
            // the scheduler
            lhs.deadline.get().cmp(&rhs.deadline.get())
        });

        loop {
            // For each loop, grab any newly pended items
            let taken = self.stack.take_all();

            // Sort these into the list - this is potentially expensive! We do an
            // insertion sort of new items, which iterates the linked list.
            //
            // Something on the order of `O(n * m)`, where `n` is the number
            // of new tasks, and `m` is the number of already pending tasks.
            sorted.extend(taken);

            // Pop the task with the SOONEST deadline. If there are no tasks
            // pending, then we are done.
            let Some(taskref) = sorted.pop_front() else {
                return;
            };

            // We got one task, mark it as dequeued, and process the task.
            taskref.header().state.run_dequeue();
            on_task(taskref);
        }
    }
}

/// A type for interacting with the deadline of the current task
pub struct Deadline {
    /// Deadline value in ticks, same time base and ticks as `embassy-time`
    pub instant_ticks: u64,
}

impl Deadline {
    /// Set the current task's deadline at exactly `instant_ticks`
    ///
    /// This method is a future in order to access the currently executing task's
    /// header which contains the deadline.
    ///
    /// Analogous to `Timer::at`.
    ///
    /// TODO: Should we check/panic if the deadline is in the past?
    #[must_use = "Setting deadline must be polled to be effective"]
    pub fn set_current_task_deadline(instant_ticks: u64) -> impl Future<Output = ()> {
        poll_fn(move |cx| {
            let task = super::task_from_waker(cx.waker());
            // SAFETY: A task can only modify its own deadline, while the task is being
            // polled, meaning that there cannot be concurrent access to the deadline.
            unsafe {
                task.header().deadline.set(instant_ticks);
            }
            Poll::Ready(())
        })
    }

    /// Set the current task's deadline `duration_ticks` in the future from when
    /// this future is polled.
    ///
    /// This method is a future in order to access the currently executing task's
    /// header which contains the deadline
    ///
    /// Analogous to `Timer::after`
    ///
    /// TODO: Do we want to return what the deadline is?
    #[must_use = "Setting deadline must be polled to be effective"]
    pub fn set_current_task_deadline_after(duration_ticks: u64) -> impl Future<Output = ()> {
        poll_fn(move |cx| {
            let task = super::task_from_waker(cx.waker());
            let now = embassy_time_driver::now();

            // Since ticks is a u64, saturating add is PROBABLY overly cautious, leave
            // it for now, we can probably make this wrapping_add for performance
            // reasons later.
            let deadline = now.saturating_add(duration_ticks);

            // SAFETY: A task can only modify its own deadline, while the task is being
            // polled, meaning that there cannot be concurrent access to the deadline.
            unsafe {
                task.header().deadline.set(deadline);
            }
            Poll::Ready(())
        })
    }

    /// Set the current task's deadline `increment_ticks` from the previous deadline.
    ///
    /// Note that by default (unless otherwise set), tasks start life with the deadline
    /// u64::MAX, which means this method will have no effect.
    ///
    /// This method is a future in order to access the currently executing task's
    /// header which contains the deadline
    ///
    /// Analogous to one increment of `Ticker::every().next()`.
    ///
    /// TODO: Do we want to return what the deadline is?
    #[must_use = "Setting deadline must be polled to be effective"]
    pub fn increment_current_task_deadline(increment_ticks: u64) -> impl Future<Output = ()> {
        poll_fn(move |cx| {
            let task = super::task_from_waker(cx.waker());

            // SAFETY: A task can only modify its own deadline, while the task is being
            // polled, meaning that there cannot be concurrent access to the deadline.
            unsafe {
                // Get the last value
                let last = task.header().deadline.get();

                // Since ticks is a u64, saturating add is PROBABLY overly cautious, leave
                // it for now, we can probably make this wrapping_add for performance
                // reasons later.
                let deadline = last.saturating_add(increment_ticks);

                // Store the new value
                task.header().deadline.set(deadline);
            }
            Poll::Ready(())
        })
    }

    /// Get the current task's deadline as a tick value.
    ///
    /// This method is a future in order to access the currently executing task's
    /// header which contains the deadline
    pub fn get_current_task_deadline() -> impl Future<Output = Self> {
        poll_fn(move |cx| {
            let task = super::task_from_waker(cx.waker());

            // SAFETY: A task can only modify its own deadline, while the task is being
            // polled, meaning that there cannot be concurrent access to the deadline.
            let deadline = unsafe {
                task.header().deadline.get()
            };
            Poll::Ready(Self { instant_ticks: deadline })
        })
    }
}
