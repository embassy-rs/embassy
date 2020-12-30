pub use embassy_macros::task;

use core::future::Future;
use core::marker::PhantomData;
use core::mem;
use core::pin::Pin;
use core::ptr;
use core::ptr::NonNull;
use core::sync::atomic::{AtomicU32, Ordering};
use core::task::{Context, Poll, Waker};
use core::{
    cell::{Cell, UnsafeCell},
    cmp::min,
};

mod run_queue;
pub(crate) mod timer;
mod timer_queue;
mod util;
mod waker;

use self::run_queue::{RunQueue, RunQueueItem};
use self::timer_queue::{TimerQueue, TimerQueueItem};
use self::util::UninitCell;
use crate::{
    fmt::{panic, *},
    time::{Alarm, Instant},
};

/// Task is spawned (has a future)
pub(crate) const STATE_SPAWNED: u32 = 1 << 0;
/// Task is in the executor run queue
pub(crate) const STATE_RUN_QUEUED: u32 = 1 << 1;
/// Task is in the executor timer queue
pub(crate) const STATE_TIMER_QUEUED: u32 = 1 << 2;

pub(crate) struct TaskHeader {
    state: AtomicU32,
    run_queue_item: RunQueueItem,
    expires_at: Cell<Instant>,
    timer_queue_item: TimerQueueItem,
    executor: Cell<*const Executor>, // Valid if state != 0
    poll_fn: UninitCell<unsafe fn(*mut TaskHeader)>, // Valid if STATE_SPAWNED
}

impl TaskHeader {
    const fn new() -> Self {
        Self {
            state: AtomicU32::new(0),
            expires_at: Cell::new(Instant::from_ticks(0)),
            run_queue_item: RunQueueItem::new(),
            timer_queue_item: TimerQueueItem::new(),
            executor: Cell::new(ptr::null()),
            poll_fn: UninitCell::uninit(),
        }
    }

    pub(crate) unsafe fn enqueue(&self) {
        let mut current = self.state.load(Ordering::Acquire);
        loop {
            // If already scheduled, or if not started,
            if (current & STATE_RUN_QUEUED != 0) || (current & STATE_SPAWNED == 0) {
                return;
            }

            // Mark it as scheduled
            let new = current | STATE_RUN_QUEUED;

            match self.state.compare_exchange_weak(
                current,
                new,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => break,
                Err(next_current) => current = next_current,
            }
        }

        // We have just marked the task as scheduled, so enqueue it.
        let executor = &*self.executor.get();
        executor.enqueue(self as *const TaskHeader as *mut TaskHeader);
    }
}

// repr(C) is needed to guarantee that header is located at offset 0
// This makes it safe to cast between Header and Task pointers.
#[repr(C)]
pub struct Task<F: Future + 'static> {
    header: TaskHeader,
    future: UninitCell<F>, // Valid if STATE_SPAWNED
}

impl<F: Future + 'static> Task<F> {
    pub const fn new() -> Self {
        Self {
            header: TaskHeader::new(),
            future: UninitCell::uninit(),
        }
    }

    pub unsafe fn spawn(pool: &'static [Self], future: impl FnOnce() -> F) -> SpawnToken {
        for task in pool {
            let state = STATE_SPAWNED | STATE_RUN_QUEUED;
            if task
                .header
                .state
                .compare_exchange(0, state, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                // Initialize the task
                task.header.poll_fn.write(Self::poll);
                task.future.write(future());

                return SpawnToken {
                    header: Some(NonNull::new_unchecked(
                        &task.header as *const TaskHeader as _,
                    )),
                };
            }
        }

        return SpawnToken { header: None };
    }

    unsafe fn poll(p: *mut TaskHeader) {
        let this = &*(p as *const Task<F>);

        let future = Pin::new_unchecked(this.future.as_mut());
        let waker = waker::from_task(p);
        let mut cx = Context::from_waker(&waker);
        match future.poll(&mut cx) {
            Poll::Ready(_) => {
                this.future.drop_in_place();
                this.header
                    .state
                    .fetch_and(!STATE_SPAWNED, Ordering::AcqRel);
            }
            Poll::Pending => {}
        }
    }
}

unsafe impl<F: Future + 'static> Sync for Task<F> {}

#[must_use = "Calling a task function does nothing on its own. You must pass the returned SpawnToken to Executor::spawn()"]
pub struct SpawnToken {
    header: Option<NonNull<TaskHeader>>,
}

impl Drop for SpawnToken {
    fn drop(&mut self) {
        // TODO deallocate the task instead.
        panic!("SpawnToken instances may not be dropped. You must pass them to Executor::spawn()")
    }
}

#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SpawnError {
    Busy,
}

pub struct Executor {
    alarm: Option<&'static dyn Alarm>,
    run_queue: RunQueue,
    timer_queue: TimerQueue,
    signal_fn: fn(),
    not_send: PhantomData<*mut ()>,
}

impl Executor {
    pub const fn new(signal_fn: fn()) -> Self {
        Self {
            alarm: None,
            run_queue: RunQueue::new(),
            timer_queue: TimerQueue::new(),
            signal_fn: signal_fn,
            not_send: PhantomData,
        }
    }
    pub const fn new_with_alarm(alarm: &'static dyn Alarm, signal_fn: fn()) -> Self {
        Self {
            alarm: Some(alarm),
            run_queue: RunQueue::new(),
            timer_queue: TimerQueue::new(),
            signal_fn: signal_fn,
            not_send: PhantomData,
        }
    }

    unsafe fn enqueue(&self, item: *mut TaskHeader) {
        if self.run_queue.enqueue(item) {
            (self.signal_fn)()
        }
    }

    /// Spawn a future on this executor.
    pub fn spawn(&'static self, token: SpawnToken) -> Result<(), SpawnError> {
        let header = token.header;
        mem::forget(token);

        match header {
            Some(header) => unsafe {
                let header = header.as_ref();
                header.executor.set(self);
                self.enqueue(header as *const _ as _);
                Ok(())
            },
            None => Err(SpawnError::Busy),
        }
    }

    /// Runs the executor until the queue is empty.
    pub fn run(&self) {
        unsafe {
            if self.alarm.is_some() {
                self.timer_queue.dequeue_expired(Instant::now(), |p| {
                    self.enqueue(p);
                });
            }

            self.run_queue.dequeue_all(|p| {
                let header = &*p;
                header.expires_at.set(Instant::MAX);

                let state = header.state.fetch_and(!STATE_RUN_QUEUED, Ordering::AcqRel);
                if state & STATE_SPAWNED == 0 {
                    // If task is not running, ignore it. This can happen in the following scenario:
                    //   - Task gets dequeued, poll starts
                    //   - While task is being polled, it gets woken. It gets placed in the queue.
                    //   - Task poll finishes, returning done=true
                    //   - RUNNING bit is cleared, but the task is already in the queue.
                    return;
                }

                // Run the task
                header.poll_fn.read()(p as _);

                // Enqueue or update into timer_queue
                self.timer_queue.update(p);
            });

            // If this is in the past, set_alarm will immediately trigger the alarm,
            // which will make the wfe immediately return so we do another loop iteration.
            if let Some(alarm) = self.alarm {
                let next_expiration = self.timer_queue.next_expiration();
                alarm.set_callback(self.signal_fn);
                alarm.set(next_expiration.as_ticks());
            }
        }
    }
}

pub(crate) unsafe fn register_timer(at: Instant, waker: &Waker) {
    let p = waker::task_from_waker(waker);
    let header = &*p;
    let expires_at = header.expires_at.get();
    header.expires_at.set(min(expires_at, at));
}
