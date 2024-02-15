//! Raw executor.
//!
//! This module exposes "raw" Executor and Task structs for more low level control.
//!
//! ## WARNING: here be dragons!
//!
//! Using this module requires respecting subtle safety contracts. If you can, prefer using the safe
//! [executor wrappers](crate::Executor) and the [`embassy_executor::task`](embassy_executor_macros::task) macro, which are fully safe.

#[cfg_attr(target_has_atomic = "ptr", path = "run_queue_atomics.rs")]
#[cfg_attr(not(target_has_atomic = "ptr"), path = "run_queue_critical_section.rs")]
mod run_queue;

#[cfg_attr(all(cortex_m, target_has_atomic = "8"), path = "state_atomics_arm.rs")]
#[cfg_attr(all(not(cortex_m), target_has_atomic = "8"), path = "state_atomics.rs")]
#[cfg_attr(not(target_has_atomic = "8"), path = "state_critical_section.rs")]
mod state;

#[cfg(feature = "integrated-timers")]
mod timer_queue;
pub(crate) mod util;
#[cfg_attr(feature = "turbowakers", path = "waker_turbo.rs")]
mod waker;

use core::future::Future;
use core::marker::PhantomData;
use core::mem;
use core::pin::Pin;
use core::ptr::NonNull;
use core::task::{Context, Poll};

#[cfg(feature = "integrated-timers")]
use embassy_time_driver::{self, AlarmHandle};
#[cfg(feature = "rtos-trace")]
use rtos_trace::trace;

use self::run_queue::{RunQueue, RunQueueItem};
use self::state::State;
use self::util::{SyncUnsafeCell, UninitCell};
pub use self::waker::task_from_waker;
use super::SpawnToken;

/// Raw task header for use in task pointers.
pub(crate) struct TaskHeader {
    pub(crate) state: State,
    pub(crate) run_queue_item: RunQueueItem,
    pub(crate) executor: SyncUnsafeCell<Option<&'static SyncExecutor>>,
    poll_fn: SyncUnsafeCell<Option<unsafe fn(TaskRef)>>,

    #[cfg(feature = "integrated-timers")]
    pub(crate) expires_at: SyncUnsafeCell<u64>,
    #[cfg(feature = "integrated-timers")]
    pub(crate) timer_queue_item: timer_queue::TimerQueueItem,
}

/// This is essentially a `&'static TaskStorage<F>` where the type of the future has been erased.
#[derive(Clone, Copy)]
pub struct TaskRef {
    ptr: NonNull<TaskHeader>,
}

unsafe impl Send for TaskRef where &'static TaskHeader: Send {}
unsafe impl Sync for TaskRef where &'static TaskHeader: Sync {}

impl TaskRef {
    fn new<F: Future + 'static>(task: &'static TaskStorage<F>) -> Self {
        Self {
            ptr: NonNull::from(task).cast(),
        }
    }

    /// Safety: The pointer must have been obtained with `Task::as_ptr`
    pub(crate) unsafe fn from_ptr(ptr: *const TaskHeader) -> Self {
        Self {
            ptr: NonNull::new_unchecked(ptr as *mut TaskHeader),
        }
    }

    pub(crate) fn header(self) -> &'static TaskHeader {
        unsafe { self.ptr.as_ref() }
    }

    /// The returned pointer is valid for the entire TaskStorage.
    pub(crate) fn as_ptr(self) -> *const TaskHeader {
        self.ptr.as_ptr()
    }
}

/// Raw storage in which a task can be spawned.
///
/// This struct holds the necessary memory to spawn one task whose future is `F`.
/// At a given time, the `TaskStorage` may be in spawned or not-spawned state. You
/// may spawn it with [`TaskStorage::spawn()`], which will fail if it is already spawned.
///
/// A `TaskStorage` must live forever, it may not be deallocated even after the task has finished
/// running. Hence the relevant methods require `&'static self`. It may be reused, however.
///
/// Internally, the [embassy_executor::task](embassy_executor_macros::task) macro allocates an array of `TaskStorage`s
/// in a `static`. The most common reason to use the raw `Task` is to have control of where
/// the memory for the task is allocated: on the stack, or on the heap with e.g. `Box::leak`, etc.

// repr(C) is needed to guarantee that the Task is located at offset 0
// This makes it safe to cast between TaskHeader and TaskStorage pointers.
#[repr(C)]
pub struct TaskStorage<F: Future + 'static> {
    raw: TaskHeader,
    future: UninitCell<F>, // Valid if STATE_SPAWNED
}

impl<F: Future + 'static> TaskStorage<F> {
    const NEW: Self = Self::new();

    /// Create a new TaskStorage, in not-spawned state.
    pub const fn new() -> Self {
        Self {
            raw: TaskHeader {
                state: State::new(),
                run_queue_item: RunQueueItem::new(),
                executor: SyncUnsafeCell::new(None),
                // Note: this is lazily initialized so that a static `TaskStorage` will go in `.bss`
                poll_fn: SyncUnsafeCell::new(None),

                #[cfg(feature = "integrated-timers")]
                expires_at: SyncUnsafeCell::new(0),
                #[cfg(feature = "integrated-timers")]
                timer_queue_item: timer_queue::TimerQueueItem::new(),
            },
            future: UninitCell::uninit(),
        }
    }

    /// Try to spawn the task.
    ///
    /// The `future` closure constructs the future. It's only called if spawning is
    /// actually possible. It is a closure instead of a simple `future: F` param to ensure
    /// the future is constructed in-place, avoiding a temporary copy in the stack thanks to
    /// NRVO optimizations.
    ///
    /// This function will fail if the task is already spawned and has not finished running.
    /// In this case, the error is delayed: a "poisoned" SpawnToken is returned, which will
    /// cause [`Spawner::spawn()`](super::Spawner::spawn) to return the error.
    ///
    /// Once the task has finished running, you may spawn it again. It is allowed to spawn it
    /// on a different executor.
    pub fn spawn(&'static self, future: impl FnOnce() -> F) -> SpawnToken<impl Sized> {
        let task = AvailableTask::claim(self);
        match task {
            Some(task) => task.initialize(future),
            None => SpawnToken::new_failed(),
        }
    }

    unsafe fn poll(p: TaskRef) {
        let this = &*(p.as_ptr() as *const TaskStorage<F>);

        let future = Pin::new_unchecked(this.future.as_mut());
        let waker = waker::from_task(p);
        let mut cx = Context::from_waker(&waker);
        match future.poll(&mut cx) {
            Poll::Ready(_) => {
                this.future.drop_in_place();
                this.raw.state.despawn();

                #[cfg(feature = "integrated-timers")]
                this.raw.expires_at.set(u64::MAX);
            }
            Poll::Pending => {}
        }

        // the compiler is emitting a virtual call for waker drop, but we know
        // it's a noop for our waker.
        mem::forget(waker);
    }

    #[doc(hidden)]
    #[allow(dead_code)]
    fn _assert_sync(self) {
        fn assert_sync<T: Sync>(_: T) {}

        assert_sync(self)
    }
}

/// An uninitialized [`TaskStorage`].
pub struct AvailableTask<F: Future + 'static> {
    task: &'static TaskStorage<F>,
}

impl<F: Future + 'static> AvailableTask<F> {
    /// Try to claim a [`TaskStorage`].
    ///
    /// This function returns `None` if a task has already been spawned and has not finished running.
    pub fn claim(task: &'static TaskStorage<F>) -> Option<Self> {
        task.raw.state.spawn().then(|| Self { task })
    }

    fn initialize_impl<S>(self, future: impl FnOnce() -> F) -> SpawnToken<S> {
        unsafe {
            self.task.raw.poll_fn.set(Some(TaskStorage::<F>::poll));
            self.task.future.write_in_place(future);

            let task = TaskRef::new(self.task);

            SpawnToken::new(task)
        }
    }

    /// Initialize the [`TaskStorage`] to run the given future.
    pub fn initialize(self, future: impl FnOnce() -> F) -> SpawnToken<F> {
        self.initialize_impl::<F>(future)
    }

    /// Initialize the [`TaskStorage`] to run the given future.
    ///
    /// # Safety
    ///
    /// `future` must be a closure of the form `move || my_async_fn(args)`, where `my_async_fn`
    /// is an `async fn`, NOT a hand-written `Future`.
    #[doc(hidden)]
    pub unsafe fn __initialize_async_fn<FutFn>(self, future: impl FnOnce() -> F) -> SpawnToken<FutFn> {
        // When send-spawning a task, we construct the future in this thread, and effectively
        // "send" it to the executor thread by enqueuing it in its queue. Therefore, in theory,
        // send-spawning should require the future `F` to be `Send`.
        //
        // The problem is this is more restrictive than needed. Once the future is executing,
        // it is never sent to another thread. It is only sent when spawning. It should be
        // enough for the task's arguments to be Send. (and in practice it's super easy to
        // accidentally make your futures !Send, for example by holding an `Rc` or a `&RefCell` across an `.await`.)
        //
        // We can do it by sending the task args and constructing the future in the executor thread
        // on first poll. However, this cannot be done in-place, so it'll waste stack space for a copy
        // of the args.
        //
        // Luckily, an `async fn` future contains just the args when freshly constructed. So, if the
        // args are Send, it's OK to send a !Send future, as long as we do it before first polling it.
        //
        // (Note: this is how the generators are implemented today, it's not officially guaranteed yet,
        // but it's possible it'll be guaranteed in the future. See zulip thread:
        // https://rust-lang.zulipchat.com/#narrow/stream/187312-wg-async/topic/.22only.20before.20poll.22.20Send.20futures )
        //
        // The `FutFn` captures all the args, so if it's Send, the task can be send-spawned.
        // This is why we return `SpawnToken<FutFn>` below.
        //
        // This ONLY holds for `async fn` futures. The other `spawn` methods can be called directly
        // by the user, with arbitrary hand-implemented futures. This is why these return `SpawnToken<F>`.
        self.initialize_impl::<FutFn>(future)
    }
}

/// Raw storage that can hold up to N tasks of the same type.
///
/// This is essentially a `[TaskStorage<F>; N]`.
pub struct TaskPool<F: Future + 'static, const N: usize> {
    pool: [TaskStorage<F>; N],
}

impl<F: Future + 'static, const N: usize> TaskPool<F, N> {
    /// Create a new TaskPool, with all tasks in non-spawned state.
    pub const fn new() -> Self {
        Self {
            pool: [TaskStorage::NEW; N],
        }
    }

    fn spawn_impl<T>(&'static self, future: impl FnOnce() -> F) -> SpawnToken<T> {
        match self.pool.iter().find_map(AvailableTask::claim) {
            Some(task) => task.initialize_impl::<T>(future),
            None => SpawnToken::new_failed(),
        }
    }

    /// Try to spawn a task in the pool.
    ///
    /// See [`TaskStorage::spawn()`] for details.
    ///
    /// This will loop over the pool and spawn the task in the first storage that
    /// is currently free. If none is free, a "poisoned" SpawnToken is returned,
    /// which will cause [`Spawner::spawn()`](super::Spawner::spawn) to return the error.
    pub fn spawn(&'static self, future: impl FnOnce() -> F) -> SpawnToken<impl Sized> {
        self.spawn_impl::<F>(future)
    }

    /// Like spawn(), but allows the task to be send-spawned if the args are Send even if
    /// the future is !Send.
    ///
    /// Not covered by semver guarantees. DO NOT call this directly. Intended to be used
    /// by the Embassy macros ONLY.
    ///
    /// SAFETY: `future` must be a closure of the form `move || my_async_fn(args)`, where `my_async_fn`
    /// is an `async fn`, NOT a hand-written `Future`.
    #[doc(hidden)]
    pub unsafe fn _spawn_async_fn<FutFn>(&'static self, future: FutFn) -> SpawnToken<impl Sized>
    where
        FutFn: FnOnce() -> F,
    {
        // See the comment in AvailableTask::__initialize_async_fn for explanation.
        self.spawn_impl::<FutFn>(future)
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Pender(*mut ());

unsafe impl Send for Pender {}
unsafe impl Sync for Pender {}

impl Pender {
    pub(crate) fn pend(self) {
        extern "Rust" {
            fn __pender(context: *mut ());
        }
        unsafe { __pender(self.0) };
    }
}

pub(crate) struct SyncExecutor {
    run_queue: RunQueue,
    pender: Pender,

    #[cfg(feature = "integrated-timers")]
    pub(crate) timer_queue: timer_queue::TimerQueue,
    #[cfg(feature = "integrated-timers")]
    alarm: AlarmHandle,
}

impl SyncExecutor {
    pub(crate) fn new(pender: Pender) -> Self {
        #[cfg(feature = "integrated-timers")]
        let alarm = unsafe { unwrap!(embassy_time_driver::allocate_alarm()) };

        Self {
            run_queue: RunQueue::new(),
            pender,

            #[cfg(feature = "integrated-timers")]
            timer_queue: timer_queue::TimerQueue::new(),
            #[cfg(feature = "integrated-timers")]
            alarm,
        }
    }

    /// Enqueue a task in the task queue
    ///
    /// # Safety
    /// - `task` must be a valid pointer to a spawned task.
    /// - `task` must be set up to run in this executor.
    /// - `task` must NOT be already enqueued (in this executor or another one).
    #[inline(always)]
    unsafe fn enqueue(&self, task: TaskRef) {
        #[cfg(feature = "rtos-trace")]
        trace::task_ready_begin(task.as_ptr() as u32);

        if self.run_queue.enqueue(task) {
            self.pender.pend();
        }
    }

    #[cfg(feature = "integrated-timers")]
    fn alarm_callback(ctx: *mut ()) {
        let this: &Self = unsafe { &*(ctx as *const Self) };
        this.pender.pend();
    }

    pub(super) unsafe fn spawn(&'static self, task: TaskRef) {
        task.header().executor.set(Some(self));

        #[cfg(feature = "rtos-trace")]
        trace::task_new(task.as_ptr() as u32);

        self.enqueue(task);
    }

    /// # Safety
    ///
    /// Same as [`Executor::poll`], plus you must only call this on the thread this executor was created.
    pub(crate) unsafe fn poll(&'static self) {
        #[cfg(feature = "integrated-timers")]
        embassy_time_driver::set_alarm_callback(self.alarm, Self::alarm_callback, self as *const _ as *mut ());

        #[allow(clippy::never_loop)]
        loop {
            #[cfg(feature = "integrated-timers")]
            self.timer_queue
                .dequeue_expired(embassy_time_driver::now(), wake_task_no_pend);

            self.run_queue.dequeue_all(|p| {
                let task = p.header();

                #[cfg(feature = "integrated-timers")]
                task.expires_at.set(u64::MAX);

                if !task.state.run_dequeue() {
                    // If task is not running, ignore it. This can happen in the following scenario:
                    //   - Task gets dequeued, poll starts
                    //   - While task is being polled, it gets woken. It gets placed in the queue.
                    //   - Task poll finishes, returning done=true
                    //   - RUNNING bit is cleared, but the task is already in the queue.
                    return;
                }

                #[cfg(feature = "rtos-trace")]
                trace::task_exec_begin(p.as_ptr() as u32);

                // Run the task
                task.poll_fn.get().unwrap_unchecked()(p);

                #[cfg(feature = "rtos-trace")]
                trace::task_exec_end();

                // Enqueue or update into timer_queue
                #[cfg(feature = "integrated-timers")]
                self.timer_queue.update(p);
            });

            #[cfg(feature = "integrated-timers")]
            {
                // If this is already in the past, set_alarm might return false
                // In that case do another poll loop iteration.
                let next_expiration = self.timer_queue.next_expiration();
                if embassy_time_driver::set_alarm(self.alarm, next_expiration) {
                    break;
                }
            }

            #[cfg(not(feature = "integrated-timers"))]
            {
                break;
            }
        }

        #[cfg(feature = "rtos-trace")]
        trace::system_idle();
    }
}

/// Raw executor.
///
/// This is the core of the Embassy executor. It is low-level, requiring manual
/// handling of wakeups and task polling. If you can, prefer using one of the
/// [higher level executors](crate::Executor).
///
/// The raw executor leaves it up to you to handle wakeups and scheduling:
///
/// - To get the executor to do work, call `poll()`. This will poll all queued tasks (all tasks
///   that "want to run").
/// - You must supply a pender function, as shown below. The executor will call it to notify you
///   it has work to do. You must arrange for `poll()` to be called as soon as possible.
/// - Enabling `arch-xx` features will define a pender function for you. This means that you
///   are limited to using the executors provided to you by the architecture/platform
///   implementation. If you need a different executor, you must not enable `arch-xx` features.
///
/// The pender can be called from *any* context: any thread, any interrupt priority
/// level, etc. It may be called synchronously from any `Executor` method call as well.
/// You must deal with this correctly.
///
/// In particular, you must NOT call `poll` directly from the pender callback, as this violates
/// the requirement for `poll` to not be called reentrantly.
///
/// The pender function must be exported with the name `__pender` and have the following signature:
///
/// ```rust
/// #[export_name = "__pender"]
/// fn pender(context: *mut ()) {
///    // schedule `poll()` to be called
/// }
/// ```
///
/// The `context` argument is a piece of arbitrary data the executor will pass to the pender.
/// You can set the `context` when calling [`Executor::new()`]. You can use it to, for example,
/// differentiate between executors, or to pass a pointer to a callback that should be called.
#[repr(transparent)]
pub struct Executor {
    pub(crate) inner: SyncExecutor,

    _not_sync: PhantomData<*mut ()>,
}

impl Executor {
    pub(crate) unsafe fn wrap(inner: &SyncExecutor) -> &Self {
        mem::transmute(inner)
    }

    /// Create a new executor.
    ///
    /// When the executor has work to do, it will call the pender function and pass `context` to it.
    ///
    /// See [`Executor`] docs for details on the pender.
    pub fn new(context: *mut ()) -> Self {
        Self {
            inner: SyncExecutor::new(Pender(context)),
            _not_sync: PhantomData,
        }
    }

    /// Spawn a task in this executor.
    ///
    /// # Safety
    ///
    /// `task` must be a valid pointer to an initialized but not-already-spawned task.
    ///
    /// It is OK to use `unsafe` to call this from a thread that's not the executor thread.
    /// In this case, the task's Future must be Send. This is because this is effectively
    /// sending the task to the executor thread.
    pub(super) unsafe fn spawn(&'static self, task: TaskRef) {
        self.inner.spawn(task)
    }

    /// Poll all queued tasks in this executor.
    ///
    /// This loops over all tasks that are queued to be polled (i.e. they're
    /// freshly spawned or they've been woken). Other tasks are not polled.
    ///
    /// You must call `poll` after receiving a call to the pender. It is OK
    /// to call `poll` even when not requested by the pender, but it wastes
    /// energy.
    ///
    /// # Safety
    ///
    /// You must NOT call `poll` reentrantly on the same executor.
    ///
    /// In particular, note that `poll` may call the pender synchronously. Therefore, you
    /// must NOT directly call `poll()` from the pender callback. Instead, the callback has to
    /// somehow schedule for `poll()` to be called later, at a time you know for sure there's
    /// no `poll()` already running.
    pub unsafe fn poll(&'static self) {
        self.inner.poll()
    }

    /// Get a spawner that spawns tasks in this executor.
    ///
    /// It is OK to call this method multiple times to obtain multiple
    /// `Spawner`s. You may also copy `Spawner`s.
    pub fn spawner(&'static self) -> super::Spawner {
        super::Spawner::new(self)
    }
}

/// Wake a task by `TaskRef`.
///
/// You can obtain a `TaskRef` from a `Waker` using [`task_from_waker`].
pub fn wake_task(task: TaskRef) {
    let header = task.header();
    if header.state.run_enqueue() {
        // We have just marked the task as scheduled, so enqueue it.
        unsafe {
            let executor = header.executor.get().unwrap_unchecked();
            executor.enqueue(task);
        }
    }
}

/// Wake a task by `TaskRef` without calling pend.
///
/// You can obtain a `TaskRef` from a `Waker` using [`task_from_waker`].
pub fn wake_task_no_pend(task: TaskRef) {
    let header = task.header();
    if header.state.run_enqueue() {
        // We have just marked the task as scheduled, so enqueue it.
        unsafe {
            let executor = header.executor.get().unwrap_unchecked();
            executor.run_queue.enqueue(task);
        }
    }
}

#[cfg(feature = "integrated-timers")]
struct TimerQueue;

#[cfg(feature = "integrated-timers")]
impl embassy_time_queue_driver::TimerQueue for TimerQueue {
    fn schedule_wake(&'static self, at: u64, waker: &core::task::Waker) {
        let task = waker::task_from_waker(waker);
        let task = task.header();
        unsafe {
            let expires_at = task.expires_at.get();
            task.expires_at.set(expires_at.min(at));
        }
    }
}

#[cfg(feature = "integrated-timers")]
embassy_time_queue_driver::timer_queue_impl!(static TIMER_QUEUE: TimerQueue = TimerQueue);

#[cfg(all(feature = "rtos-trace", feature = "integrated-timers"))]
const fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

#[cfg(feature = "rtos-trace")]
impl rtos_trace::RtosTraceOSCallbacks for Executor {
    fn task_list() {
        // We don't know what tasks exist, so we can't send them.
    }
    #[cfg(feature = "integrated-timers")]
    fn time() -> u64 {
        const GCD_1M: u64 = gcd(embassy_time_driver::TICK_HZ, 1_000_000);
        embassy_time_driver::now() * (1_000_000 / GCD_1M) / (embassy_time_driver::TICK_HZ / GCD_1M)
    }
    #[cfg(not(feature = "integrated-timers"))]
    fn time() -> u64 {
        0
    }
}

#[cfg(feature = "rtos-trace")]
rtos_trace::global_os_callbacks! {Executor}
