//! # Tracing
//!
//! The `trace` feature enables a number of callbacks that can be used to track the
//! lifecycle of tasks and/or executors.
//!
//! The passed IDs are only guaranteed to be unique for the duration of time the item is valid. If a task
//! ends, and is re-spawned, it MAY or MAY NOT have the same ID. While a task is active, the id will not change.
//! For executors, the same applies, but the IDs will be stable for practical embedded programs.
//!
//! Callbacks can be used by enabling the `trace` feature, implementing the `Trace`
//! trait, and registering the implementation with the `embassy_executor::trace_impl!` macro.
//! All callbacks must be implemented.
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
//! 1. A task is spawned, `task_new` is called
//! 2. A task is enqueued for the first time, `task_ready_begin` is called
//! 3. A task is polled, `task_exec_begin` is called
//! 4. WHILE a task is polled, the task is re-awoken, and `task_ready_begin` is
//!      called. The task does not IMMEDIATELY move state, until polling is complete and the
//!      RUNNING state is existed. `task_exec_end` is called when polling is
//!      complete, marking the transition to WAITING
//! 5. Polling is complete, `task_exec_end` is called
//! 6. The task has completed, and `task_end` is called
//! 7. A task is awoken, `task_ready_begin` is called
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
//! 2. A task on this executor is awoken. `task_ready_begin` is called
//!      when this occurs, and `poll_start` is called when the executor
//!      actually begins running
//! 3. The executor has decided a task to poll. `task_exec_begin` is called
//! 4. The executor finishes polling the task. `task_exec_end` is called
//! 5. The executor has finished polling tasks. `executor_idle` is called
//!
//! ## Idle
//!
//! `executor_idle` only means that a single executor has run out of work. With
//! multiple executors (e.g. a thread-mode executor plus one or more interrupt
//! executors), an interrupt executor going idle returns to the preempted
//! lower-priority context, which keeps running. The current thread/core is only
//! idle when its thread-mode executor reaches its sleep site, at which point
//! `idle` is called. In multi-core chips, or when using threads under std or an
//! RTOS, this does not mean the entire system is idle.

use crate::ExecutorId;
use crate::raw::TaskRef;

unitrait::unitrait! {
    /// Executor trace hooks.
    ///
    /// Implement this trait and register the implementation with the
    /// `embassy_executor::trace_impl!` macro to receive callbacks on task and executor
    /// lifecycle events. All callbacks must be implemented.
    ///
    /// See the [module documentation](super) for the task and executor tracing lifecycles.
    pub trait Trace {
        /// This callback is called when the executor begins polling. This will always
        /// be paired with a later call to `executor_idle`.
        ///
        /// This marks the EXECUTOR state transition from IDLE -> SCHEDULING.
        #[symbol = "_embassy_trace_v2_poll_start"]
        pub(crate) fn poll_start(executor: ExecutorId);

        /// This callback is called AFTER a task is initialized/allocated, and BEFORE
        /// it is enqueued to run for the first time. If the task ends (and does not
        /// loop "forever"), there will be a matching call to `task_end`.
        ///
        /// Tasks start life in the SPAWNED state.
        #[symbol = "_embassy_trace_v2_task_new"]
        pub(crate) fn task_new(executor: ExecutorId, task: TaskRef);

        /// This callback is called AFTER a task is destructed/freed. This will always
        /// have a prior matching call to `task_new`.
        #[symbol = "_embassy_trace_v2_task_end"]
        pub(crate) fn task_end(executor: ExecutorId, task: TaskRef);

        /// This callback is called AFTER a task has been dequeued from the runqueue,
        /// and BEFORE the task is polled. There will always be a matching call to
        /// `task_exec_end`.
        ///
        /// This marks the TASK state transition from WAITING -> RUNNING
        /// This marks the EXECUTOR state transition from SCHEDULING -> POLLING
        #[symbol = "_embassy_trace_v2_task_exec_begin"]
        pub(crate) fn task_exec_begin(executor: ExecutorId, task: TaskRef);

        /// This callback is called AFTER a task has completed polling. There will
        /// always be a matching call to `task_exec_begin`.
        ///
        /// This marks the TASK state transition from either:
        /// * RUNNING -> IDLE - if there were no `task_ready_begin` events
        ///     for this task since the last `task_exec_begin` for THIS task
        /// * RUNNING -> WAITING - if there WAS a `task_ready_begin` event
        ///     for this task since the last `task_exec_begin` for THIS task
        ///
        /// This marks the EXECUTOR state transition from POLLING -> SCHEDULING
        #[symbol = "_embassy_trace_v2_task_exec_end"]
        pub(crate) fn task_exec_end(executor: ExecutorId, task: TaskRef);

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
        #[symbol = "_embassy_trace_v2_task_ready_begin"]
        pub(crate) fn task_ready_begin(executor: ExecutorId, task: TaskRef);

        /// This callback is called AFTER all dequeued tasks in a single call to poll
        /// have been processed. This will always be paired with a call to
        /// `poll_start`.
        ///
        /// This marks the EXECUTOR state transition from SCHEDULING -> IDLE
        #[symbol = "_embassy_trace_v2_executor_idle"]
        pub(crate) fn executor_idle(executor: ExecutorId);

        /// This callback is called right before the thread-mode executor puts the
        /// current thread/core to sleep (e.g. `wfe`/`wfi`).
        ///
        /// Unlike `executor_idle`, this is never called by interrupt executors,
        /// since they return to a preempted context after polling and the system
        /// keeps running.
        ///
        /// Note in multi-core chips, or when using threads under std or an RTOS, this
        /// doesn't mean the entire system is idle, it only means the current core/thread is.
        #[symbol = "_embassy_trace_v2_idle"]
        pub(crate) fn idle();

        /// This callback is called AFTER the name of a task is set.
        ///
        /// This function can be called when the task is not running and it does not signal a state change.
        #[symbol = "_embassy_trace_v2_task_name_set"]
        pub(crate) fn task_name_set(task: TaskRef, name: &'static str);

        /// This callback is called AFTER the priority of a task is set.
        ///
        /// This function can be called when the task is not running and it does not signal a state change.
        #[symbol = "_embassy_trace_v2_task_priority_set"]
        pub(crate) fn task_priority_set(task: TaskRef, priority: u8);

        /// This callback is called AFTER the deadline of a task is set.
        ///
        /// This function can be called when the task is not running and it does not signal a state change.
        #[symbol = "_embassy_trace_v2_task_deadline_set"]
        pub(crate) fn task_deadline_set(task: TaskRef, deadline: u64);
    }

    /// Register a type as the global executor trace hook implementation.
    ///
    /// See [`raw::trace::Trace`](crate::raw::trace::Trace).
    macro trace_impl(path = $crate::raw::trace);
}
