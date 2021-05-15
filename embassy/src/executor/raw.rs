use atomic_polyfill::{AtomicU32, Ordering};
use core::cell::Cell;
use core::cmp::min;
use core::future::Future;
use core::marker::PhantomData;
use core::pin::Pin;
use core::ptr::NonNull;
use core::task::{Context, Poll, Waker};
use core::{mem, ptr};

use super::run_queue::{RunQueue, RunQueueItem};
use super::timer_queue::{TimerQueue, TimerQueueItem};
use super::util::UninitCell;
use super::waker;
use super::SpawnToken;
use crate::time::{Alarm, Instant};

/// Task is spawned (has a future)
pub(crate) const STATE_SPAWNED: u32 = 1 << 0;
/// Task is in the executor run queue
pub(crate) const STATE_RUN_QUEUED: u32 = 1 << 1;
/// Task is in the executor timer queue
pub(crate) const STATE_TIMER_QUEUED: u32 = 1 << 2;

pub struct TaskHeader {
    pub(crate) state: AtomicU32,
    pub(crate) run_queue_item: RunQueueItem,
    pub(crate) expires_at: Cell<Instant>,
    pub(crate) timer_queue_item: TimerQueueItem,
    pub(crate) executor: Cell<*const Executor>, // Valid if state != 0
    pub(crate) poll_fn: UninitCell<unsafe fn(NonNull<TaskHeader>)>, // Valid if STATE_SPAWNED
}

impl TaskHeader {
    pub(crate) const fn new() -> Self {
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

// repr(C) is needed to guarantee that the Task is located at offset 0
// This makes it safe to cast between Task and Task pointers.
#[repr(C)]
pub struct Task<F: Future> {
    raw: TaskHeader,
    future: UninitCell<F>, // Valid if STATE_SPAWNED
}

impl<F: Future> Task<F> {
    pub const fn new() -> Self {
        Self {
            raw: TaskHeader::new(),
            future: UninitCell::uninit(),
        }
    }

    pub fn spawn_pool(pool: &'static [Self], future: impl FnOnce() -> F) -> SpawnToken<F> {
        for task in pool {
            if task.spawn_allocate() {
                return unsafe { task.spawn_initialize(future) };
            }
        }

        SpawnToken {
            raw_task: None,
            phantom: PhantomData,
        }
    }

    pub fn spawn(&'static self, future: impl FnOnce() -> F) -> SpawnToken<F> {
        if self.spawn_allocate() {
            unsafe { self.spawn_initialize(future) }
        } else {
            SpawnToken {
                raw_task: None,
                phantom: PhantomData,
            }
        }
    }

    fn spawn_allocate(&'static self) -> bool {
        let state = STATE_SPAWNED | STATE_RUN_QUEUED;
        self.raw
            .state
            .compare_exchange(0, state, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    }

    unsafe fn spawn_initialize(&'static self, future: impl FnOnce() -> F) -> SpawnToken<F> {
        // Initialize the task
        self.raw.poll_fn.write(Self::poll);
        self.future.write(future());

        return SpawnToken {
            raw_task: Some(NonNull::new_unchecked(&self.raw as *const TaskHeader as _)),
            phantom: PhantomData,
        };
    }

    unsafe fn poll(p: NonNull<TaskHeader>) {
        let this = &*(p.as_ptr() as *const Task<F>);

        let future = Pin::new_unchecked(this.future.as_mut());
        let waker = waker::from_task(p);
        let mut cx = Context::from_waker(&waker);
        match future.poll(&mut cx) {
            Poll::Ready(_) => {
                this.future.drop_in_place();
                this.raw.state.fetch_and(!STATE_SPAWNED, Ordering::AcqRel);
            }
            Poll::Pending => {}
        }

        // the compiler is emitting a virtual call for waker drop, but we know
        // it's a noop for our waker.
        mem::forget(waker);
    }
}

unsafe impl<F: Future> Sync for Task<F> {}

pub struct Executor {
    run_queue: RunQueue,
    timer_queue: TimerQueue,
    signal_fn: fn(*mut ()),
    signal_ctx: *mut (),
    alarm: Option<&'static dyn Alarm>,
}

impl Executor {
    pub const fn new(signal_fn: fn(*mut ()), signal_ctx: *mut ()) -> Self {
        Self {
            run_queue: RunQueue::new(),
            timer_queue: TimerQueue::new(),
            signal_fn,
            signal_ctx,
            alarm: None,
        }
    }

    pub fn set_alarm(&mut self, alarm: &'static dyn Alarm) {
        self.alarm = Some(alarm);
    }

    pub fn set_signal_ctx(&mut self, signal_ctx: *mut ()) {
        self.signal_ctx = signal_ctx;
    }

    unsafe fn enqueue(&self, item: *mut TaskHeader) {
        if self.run_queue.enqueue(item) {
            (self.signal_fn)(self.signal_ctx)
        }
    }

    pub unsafe fn spawn(&'static self, task: NonNull<TaskHeader>) {
        let task = task.as_ref();
        task.executor.set(self);
        self.enqueue(task as *const _ as _);
    }

    pub unsafe fn run_queued(&'static self) {
        if self.alarm.is_some() {
            self.timer_queue.dequeue_expired(Instant::now(), |p| {
                p.as_ref().enqueue();
            });
        }

        self.run_queue.dequeue_all(|p| {
            let task = p.as_ref();
            task.expires_at.set(Instant::MAX);

            let state = task.state.fetch_and(!STATE_RUN_QUEUED, Ordering::AcqRel);
            if state & STATE_SPAWNED == 0 {
                // If task is not running, ignore it. This can happen in the following scenario:
                //   - Task gets dequeued, poll starts
                //   - While task is being polled, it gets woken. It gets placed in the queue.
                //   - Task poll finishes, returning done=true
                //   - RUNNING bit is cleared, but the task is already in the queue.
                return;
            }

            // Run the task
            task.poll_fn.read()(p as _);

            // Enqueue or update into timer_queue
            self.timer_queue.update(p);
        });

        // If this is in the past, set_alarm will immediately trigger the alarm,
        // which will make the wfe immediately return so we do another loop iteration.
        if let Some(alarm) = self.alarm {
            let next_expiration = self.timer_queue.next_expiration();
            alarm.set_callback(self.signal_fn, self.signal_ctx);
            alarm.set(next_expiration.as_ticks());
        }
    }

    pub unsafe fn spawner(&'static self) -> super::Spawner {
        super::Spawner {
            executor: self,
            not_send: PhantomData,
        }
    }
}

pub use super::waker::task_from_waker;

pub unsafe fn wake_task(task: NonNull<TaskHeader>) {
    task.as_ref().enqueue();
}

pub(crate) unsafe fn register_timer(at: Instant, waker: &Waker) {
    let task = waker::task_from_waker(waker);
    let task = task.as_ref();
    let expires_at = task.expires_at.get();
    task.expires_at.set(min(expires_at, at));
}
