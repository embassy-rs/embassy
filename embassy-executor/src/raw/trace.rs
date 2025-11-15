//! # Tracing
//!
//! The `trace` feature enables a number of callbacks that can be used to track the
//! lifecycle of tasks and/or executors.
//!
//! Callbacks will have one or both of the following IDs passed to them:
//!
//! 1. A `task_id`, a `u32` value unique to a task for the duration of the time it is valid
//! 2. An `executor_id`, a `u32` value unique to an executor for the duration of the time it is
//!    valid
//!
//! Today, both `task_id` and `executor_id` are u32s containing the least significant 32 bits of
//! the address of the task or executor, however this is NOT a stable guarantee, and MAY change
//! at any time.
//!
//! IDs are only guaranteed to be unique for the duration of time the item is valid. If a task
//! ends, and is re-spawned, it MAY or MAY NOT have the same ID. For tasks, this valid time is defined
//! as the time between `_embassy_trace_task_new` and `_embassy_trace_task_end` for a given task.
//! For executors, this time is not defined, but is often "forever" for practical embedded
//! programs.
//!
//! Callbacks can be used by enabling the `trace` feature, and providing implementations of the
//! `extern "Rust"` functions below. All callbacks must be implemented.
//!
//! ## Task Tracing lifecycle
//!
//! ```text
//! ┌ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─
//!        │(1)                                            │
//! │      │
//!   ╔════▼════╗ (2) ┌─────────┐ (3) ┌─────────┐          │
//! │ ║ SPAWNED ║────▶│ WAITING │────▶│ RUNNING │
//!   ╚═════════╝     └─────────┘     └─────────┘          │
//! │                 ▲         ▲     │    │    │
//!                   │           (4)      │    │(6)       │
//! │                 │(7)      └ ─ ─ ┘    │    │
//!                   │                    │    │          │
//! │             ┌──────┐             (5) │    │  ┌─────┐
//!               │ IDLE │◀────────────────┘    └─▶│ END │ │
//! │             └──────┘                         └─────┘
//!   ┌──────────────────────┐                             │
//! └ ┤ Task Trace Lifecycle │─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─
//!   └──────────────────────┘
//! ```
//!
//! 1. A task is spawned, `_embassy_trace_task_new` is called
//! 2. A task is enqueued for the first time, `_embassy_trace_task_ready_begin` is called
//! 3. A task is polled, `_embassy_trace_task_exec_begin` is called
//! 4. WHILE a task is polled, the task is re-awoken, and `_embassy_trace_task_ready_begin` is
//!      called. The task does not IMMEDIATELY move state, until polling is complete and the
//!      RUNNING state is existed. `_embassy_trace_task_exec_end` is called when polling is
//!      complete, marking the transition to WAITING
//! 5. Polling is complete, `_embassy_trace_task_exec_end` is called
//! 6. The task has completed, and `_embassy_trace_task_end` is called
//! 7. A task is awoken, `_embassy_trace_task_ready_begin` is called
//!
//! ## Executor Tracing lifecycle
//!
//! ```text
//! ┌ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─
//!       │(1)                                             │
//! │     │
//!   ╔═══▼══╗   (2)     ┌────────────┐  (3)  ┌─────────┐  │
//! │ ║ IDLE ║──────────▶│ SCHEDULING │──────▶│ POLLING │
//!   ╚══════╝           └────────────┘       └─────────┘  │
//! │     ▲              │            ▲            │
//!       │      (5)     │            │  (4)       │       │
//! │     └──────────────┘            └────────────┘
//!   ┌──────────────────────────┐                         │
//! └ ┤ Executor Trace Lifecycle │─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─
//!   └──────────────────────────┘
//! ```
//!
//! 1. The executor is started (no associated trace)
//! 2. A task on this executor is awoken. `_embassy_trace_task_ready_begin` is called
//!      when this occurs, and `_embassy_trace_poll_start` is called when the executor
//!      actually begins running
//! 3. The executor has decided a task to poll. `_embassy_trace_task_exec_begin` is called
//! 4. The executor finishes polling the task. `_embassy_trace_task_exec_end` is called
//! 5. The executor has finished polling tasks. `_embassy_trace_executor_idle` is called

#![allow(unused)]

use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};

#[cfg(feature = "rtos-trace")]
use rtos_trace::TaskInfo;

use crate::raw::{SyncExecutor, TaskHeader, TaskRef};
use crate::spawner::{SpawnError, SpawnToken, Spawner};

/// Global task tracker instance
///
/// This static provides access to the global task tracker which maintains
/// a list of all tasks in the system. It's automatically updated by the
/// task lifecycle hooks in the trace module.
#[cfg(feature = "rtos-trace")]
pub(crate) static TASK_TRACKER: TaskTracker = TaskTracker::new();

/// A thread-safe tracker for all tasks in the system
///
/// This struct uses an intrusive linked list approach to track all tasks
/// without additional memory allocations. It maintains a global list of
/// tasks that can be traversed to find all currently existing tasks.
#[cfg(feature = "rtos-trace")]
pub(crate) struct TaskTracker {
    head: AtomicPtr<TaskHeader>,
}

#[cfg(feature = "rtos-trace")]
impl TaskTracker {
    /// Creates a new empty task tracker
    ///
    /// Initializes a tracker with no tasks in its list.
    pub const fn new() -> Self {
        Self {
            head: AtomicPtr::new(core::ptr::null_mut()),
        }
    }

    /// Adds a task to the tracker
    ///
    /// This method inserts a task at the head of the intrusive linked list.
    /// The operation is thread-safe and lock-free, using atomic operations
    /// to ensure consistency even when called from different contexts.
    ///
    /// # Arguments
    /// * `task` - The task reference to add to the tracker
    pub fn add(&self, task: TaskRef) {
        let task_ptr = task.as_ptr();

        loop {
            let current_head = self.head.load(Ordering::Acquire);
            unsafe {
                (*task_ptr).all_tasks_next.store(current_head, Ordering::Relaxed);
            }

            if self
                .head
                .compare_exchange(current_head, task_ptr.cast_mut(), Ordering::Release, Ordering::Relaxed)
                .is_ok()
            {
                break;
            }
        }
    }

    /// Performs an operation on each task in the tracker
    ///
    /// This method traverses the entire list of tasks and calls the provided
    /// function for each task. This allows inspecting or processing all tasks
    /// in the system without modifying the tracker's structure.
    ///
    /// # Arguments
    /// * `f` - A function to call for each task in the tracker
    pub fn for_each<F>(&self, mut f: F)
    where
        F: FnMut(TaskRef),
    {
        let mut current = self.head.load(Ordering::Acquire);
        while !current.is_null() {
            let task = unsafe { TaskRef::from_ptr(current) };
            f(task);

            current = unsafe { (*current).all_tasks_next.load(Ordering::Acquire) };
        }
    }
}

#[cfg(feature = "trace")]
unsafe extern "Rust" {
    /// This callback is called when the executor begins polling. This will always
    /// be paired with a later call to `_embassy_trace_executor_idle`.
    ///
    /// This marks the EXECUTOR state transition from IDLE -> SCHEDULING.
    fn _embassy_trace_poll_start(executor_id: u32);

    /// This callback is called AFTER a task is initialized/allocated, and BEFORE
    /// it is enqueued to run for the first time. If the task ends (and does not
    /// loop "forever"), there will be a matching call to `_embassy_trace_task_end`.
    ///
    /// Tasks start life in the SPAWNED state.
    fn _embassy_trace_task_new(executor_id: u32, task_id: u32);

    /// This callback is called AFTER a task is destructed/freed. This will always
    /// have a prior matching call to `_embassy_trace_task_new`.
    fn _embassy_trace_task_end(executor_id: u32, task_id: u32);

    /// This callback is called AFTER a task has been dequeued from the runqueue,
    /// and BEFORE the task is polled. There will always be a matching call to
    /// `_embassy_trace_task_exec_end`.
    ///
    /// This marks the TASK state transition from WAITING -> RUNNING
    /// This marks the EXECUTOR state transition from SCHEDULING -> POLLING
    fn _embassy_trace_task_exec_begin(executor_id: u32, task_id: u32);

    /// This callback is called AFTER a task has completed polling. There will
    /// always be a matching call to `_embassy_trace_task_exec_begin`.
    ///
    /// This marks the TASK state transition from either:
    /// * RUNNING -> IDLE - if there were no `_embassy_trace_task_ready_begin` events
    ///     for this task since the last `_embassy_trace_task_exec_begin` for THIS task
    /// * RUNNING -> WAITING - if there WAS a `_embassy_trace_task_ready_begin` event
    ///     for this task since the last `_embassy_trace_task_exec_begin` for THIS task
    ///
    /// This marks the EXECUTOR state transition from POLLING -> SCHEDULING
    fn _embassy_trace_task_exec_end(excutor_id: u32, task_id: u32);

    /// This callback is called AFTER the waker for a task is awoken, and BEFORE it
    /// is added to the run queue.
    ///
    /// If the given task is currently RUNNING, this marks no state change, BUT the
    /// RUNNING task will then move to the WAITING stage when polling is complete.
    ///
    /// If the given task is currently IDLE, this marks the TASK state transition
    /// from IDLE -> WAITING.
    ///
    /// NOTE: This may be called from an interrupt, outside the context of the current
    /// task or executor.
    fn _embassy_trace_task_ready_begin(executor_id: u32, task_id: u32);

    /// This callback is called AFTER all dequeued tasks in a single call to poll
    /// have been processed. This will always be paired with a call to
    /// `_embassy_trace_executor_idle`.
    ///
    /// This marks the EXECUTOR state transition from SCHEDULING -> IDLE
    fn _embassy_trace_executor_idle(executor_id: u32);
}

#[inline]
pub(crate) fn poll_start(executor: &SyncExecutor) {
    #[cfg(feature = "trace")]
    unsafe {
        _embassy_trace_poll_start(executor as *const _ as u32)
    }
}

#[inline]
pub(crate) fn task_new(executor: &SyncExecutor, task: &TaskRef) {
    #[cfg(feature = "trace")]
    unsafe {
        _embassy_trace_task_new(executor as *const _ as u32, task.as_ptr() as u32)
    }

    #[cfg(feature = "rtos-trace")]
    {
        rtos_trace::trace::task_new(task.as_ptr() as u32);
        let name = task.metadata().name().unwrap_or("unnamed task\0");
        let info = rtos_trace::TaskInfo {
            name,
            priority: 0,
            stack_base: 0,
            stack_size: 0,
        };
        rtos_trace::trace::task_send_info(task.id(), info);
    }

    #[cfg(feature = "rtos-trace")]
    TASK_TRACKER.add(*task);
}

#[inline]
pub(crate) fn task_end(executor: *const SyncExecutor, task: &TaskRef) {
    #[cfg(feature = "trace")]
    unsafe {
        _embassy_trace_task_end(executor as u32, task.as_ptr() as u32)
    }
}

#[inline]
pub(crate) fn task_ready_begin(executor: &SyncExecutor, task: &TaskRef) {
    #[cfg(feature = "trace")]
    unsafe {
        _embassy_trace_task_ready_begin(executor as *const _ as u32, task.as_ptr() as u32)
    }
    #[cfg(feature = "rtos-trace")]
    rtos_trace::trace::task_ready_begin(task.as_ptr() as u32);
}

#[inline]
pub(crate) fn task_exec_begin(executor: &SyncExecutor, task: &TaskRef) {
    #[cfg(feature = "trace")]
    unsafe {
        _embassy_trace_task_exec_begin(executor as *const _ as u32, task.as_ptr() as u32)
    }
    #[cfg(feature = "rtos-trace")]
    rtos_trace::trace::task_exec_begin(task.as_ptr() as u32);
}

#[inline]
pub(crate) fn task_exec_end(executor: &SyncExecutor, task: &TaskRef) {
    #[cfg(feature = "trace")]
    unsafe {
        _embassy_trace_task_exec_end(executor as *const _ as u32, task.as_ptr() as u32)
    }
    #[cfg(feature = "rtos-trace")]
    rtos_trace::trace::task_exec_end();
}

#[inline]
pub(crate) fn executor_idle(executor: &SyncExecutor) {
    #[cfg(feature = "trace")]
    unsafe {
        _embassy_trace_executor_idle(executor as *const _ as u32)
    }
    #[cfg(feature = "rtos-trace")]
    rtos_trace::trace::system_idle();
}

/// Returns an iterator over all active tasks in the system
///
/// This function provides a convenient way to iterate over all tasks
/// that are currently tracked in the system. The returned iterator
/// yields each task in the global task tracker.
///
/// # Returns
/// An iterator that yields `TaskRef` items for each task
#[cfg(feature = "rtos-trace")]
fn get_all_active_tasks() -> impl Iterator<Item = TaskRef> + 'static {
    struct TaskIterator<'a> {
        tracker: &'a TaskTracker,
        current: *mut TaskHeader,
    }

    impl<'a> Iterator for TaskIterator<'a> {
        type Item = TaskRef;

        fn next(&mut self) -> Option<Self::Item> {
            if self.current.is_null() {
                return None;
            }

            let task = unsafe { TaskRef::from_ptr(self.current) };
            self.current = unsafe { (*self.current).all_tasks_next.load(Ordering::Acquire) };

            Some(task)
        }
    }

    TaskIterator {
        tracker: &TASK_TRACKER,
        current: TASK_TRACKER.head.load(Ordering::Acquire),
    }
}

/// Perform an action on each active task
#[cfg(feature = "rtos-trace")]
fn with_all_active_tasks<F>(f: F)
where
    F: FnMut(TaskRef),
{
    TASK_TRACKER.for_each(f);
}

#[cfg(feature = "rtos-trace")]
impl rtos_trace::RtosTraceOSCallbacks for crate::raw::SyncExecutor {
    fn task_list() {
        with_all_active_tasks(|task| {
            let info = rtos_trace::TaskInfo {
                name: task.metadata().name().unwrap_or("unnamed task\0"),
                priority: 0,
                stack_base: 0,
                stack_size: 0,
            };
            rtos_trace::trace::task_send_info(task.id(), info);
        });
    }
    fn time() -> u64 {
        const fn gcd(a: u64, b: u64) -> u64 {
            if b == 0 { a } else { gcd(b, a % b) }
        }

        const GCD_1M: u64 = gcd(embassy_time_driver::TICK_HZ, 1_000_000);
        embassy_time_driver::now() * (1_000_000 / GCD_1M) / (embassy_time_driver::TICK_HZ / GCD_1M)
    }
}

#[cfg(feature = "rtos-trace")]
rtos_trace::global_os_callbacks! {SyncExecutor}
