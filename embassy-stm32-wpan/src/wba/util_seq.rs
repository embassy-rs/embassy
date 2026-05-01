#![cfg(feature = "wba")]
//! UTIL Sequencer implementation for STM32WBA BLE stack
//!
//! This module provides the sequencer functions required by the ST BLE stack.
//! The sequencer manages background tasks and event waiting.
//!
//! # Context Switching Architecture
//!
//! The key insight is that `UTIL_SEQ_WaitEvt` would normally call WFE (wait for event),
//! which blocks the entire CPU. Instead, we use context switching to yield back to
//! the embassy executor, allowing other async tasks to run.
//!
//! When the sequencer has no work to do, instead of WFE, it yields to the runner task.
//! The runner task can then yield to the embassy executor, and resume the sequencer
//! when there's new work (signaled by interrupts).

use core::cell::UnsafeCell;
use core::future::poll_fn;
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering, compiler_fence};
use core::task::Poll;

use embassy_sync::waitqueue::AtomicWaker;

use super::context::ContextManager;

type TaskFn = unsafe extern "C" fn();

const MAX_TASKS: usize = 32;
const DEFAULT_PRIORITY: u8 = u8::MAX;

struct TaskTable {
    funcs: UnsafeCell<[Option<TaskFn>; MAX_TASKS]>,
    priorities: UnsafeCell<[u8; MAX_TASKS]>,
}

impl TaskTable {
    const fn new() -> Self {
        Self {
            funcs: UnsafeCell::new([None; MAX_TASKS]),
            priorities: UnsafeCell::new([DEFAULT_PRIORITY; MAX_TASKS]),
        }
    }

    unsafe fn set_task(&self, idx: usize, func: Option<TaskFn>, priority: u8) {
        (*self.funcs.get())[idx] = func;
        (*self.priorities.get())[idx] = priority;
    }

    unsafe fn update_priority(&self, idx: usize, priority: u8) {
        (*self.priorities.get())[idx] = priority;
    }

    unsafe fn task(&self, idx: usize) -> Option<TaskFn> {
        (*self.funcs.get())[idx]
    }

    unsafe fn priority(&self, idx: usize) -> u8 {
        (*self.priorities.get())[idx]
    }
}

unsafe impl Sync for TaskTable {}

struct Sequencer {
    context: ContextManager,
    tasks: TaskTable,
    pending_tasks: AtomicU32,
    events: AtomicU32,
    /// Set by seq_pend() (ISRs, timers, etc.) to indicate the runner should wake.
    /// Checked and cleared by wait_for_event(). This ensures wakeups from radio ISRs
    /// that don't set sequencer tasks are not lost.
    pended: AtomicBool,
    waker: AtomicWaker,
    super_mask: AtomicU32,
    current_task_idx: AtomicU32,
}

const ALL_TASKS_MASK: u32 = 0xFFFFFFFF;
const NO_TASK_RUNNING: u32 = 0xFFFFFFFF;

static SEQUENCER: Sequencer = Sequencer {
    context: ContextManager::new(task_entry),
    tasks: TaskTable::new(),
    pending_tasks: AtomicU32::new(0),
    events: AtomicU32::new(0),
    pended: AtomicBool::new(false),
    waker: AtomicWaker::new(),
    super_mask: AtomicU32::new(ALL_TASKS_MASK),
    current_task_idx: AtomicU32::new(NO_TASK_RUNNING),
};

fn mask_to_index(mask: u32) -> Option<usize> {
    if mask == 0 {
        return None;
    }
    let idx = mask.trailing_zeros() as usize;
    if idx < MAX_TASKS { Some(idx) } else { None }
}

/// Run the sequencer with the given task mask.
/// Use UTIL_SEQ_DEFAULT (0xFFFFFFFF) to run all tasks.
/// Returns true if at least one task was executed.
pub fn run(mask: u32) -> bool {
    SEQUENCER.run(mask)
}

/// Check if there are any pending tasks or events
pub fn has_pending_work() -> bool {
    SEQUENCER.has_work()
}

/// Default mask value for running all tasks (matches ST's UTIL_SEQ_DEFAULT)
pub const UTIL_SEQ_DEFAULT: u32 = ALL_TASKS_MASK;

pub fn seq_pend() {
    SEQUENCER.seq_pend();
}

pub fn seq_resume() {
    SEQUENCER.seq_resume();
}

pub async fn wait_for_event() {
    SEQUENCER.wait_for_event().await
}

/// Entry point for the sequencer context
///
/// This function runs in the sequencer's stack context and repeatedly
/// polls for pending tasks, yielding when there's nothing to do.
extern "C" fn task_entry() -> ! {
    loop {
        // Poll and execute any pending sequencer tasks
        // Use UTIL_SEQ_DEFAULT to run all tasks
        SEQUENCER.run(UTIL_SEQ_DEFAULT);

        // Yield back to the runner
        // This will return when the runner resumes us
        SEQUENCER.context.task_yield();
    }
}

impl Sequencer {
    pub async fn wait_for_event(&self) {
        poll_fn(|cx| {
            self.waker.register(cx.waker());

            compiler_fence(Ordering::Acquire);

            // Check both explicit sequencer work AND the pended flag (set by ISRs/timers)
            if self.has_work() || self.pended.swap(false, Ordering::AcqRel) {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;
    }

    pub fn seq_resume(&'static self) {
        self.context.task_resume();
    }

    fn seq_pend(&self) {
        self.pended.store(true, Ordering::Release);
        self.waker.wake();
    }

    /// Wait for an event
    ///
    /// Instead of blocking with WFE, this yields back to the runner context
    /// so that the embassy executor can run other tasks.
    #[inline(always)]
    fn seq_yield(&'static self) {
        // If we're in the sequencer context, yield back to the runner
        // If we're not (e.g., during initialization), use actual WFE
        if self.context.in_task_context() {
            self.context.task_yield();
        } else {
            #[cfg(target_arch = "arm")]
            {
                cortex_m::asm::wfe();
            }

            #[cfg(not(target_arch = "arm"))]
            {
                core::hint::spin_loop();
            }
        }
    }

    /// Check if there are any pending tasks or events
    pub fn has_work(&self) -> bool {
        self.pending_tasks.load(Ordering::Acquire) != 0 || self.events.load(Ordering::Acquire) != 0
    }

    fn select_next_task(&self, super_mask: u32) -> Option<(usize, TaskFn)> {
        let pending = self.pending_tasks.load(Ordering::Acquire);
        // Apply super_mask to restrict which tasks can run
        let pending = pending & super_mask;
        if pending == 0 {
            return None;
        }

        let mut remaining = pending;
        let mut best_idx: Option<usize> = None;
        let mut best_priority = DEFAULT_PRIORITY;
        let mut best_fn: Option<TaskFn> = None;

        while remaining != 0 {
            let idx = remaining.trailing_zeros() as usize;
            remaining &= remaining - 1;

            if idx >= MAX_TASKS {
                continue;
            }

            unsafe {
                if let Some(func) = self.tasks.task(idx) {
                    let prio = self.tasks.priority(idx);
                    if prio <= best_priority {
                        if prio < best_priority || best_idx.map_or(true, |current| idx < current) {
                            best_priority = prio;
                            best_idx = Some(idx);
                            best_fn = Some(func);
                        }
                    }
                } else {
                    self.pending_tasks.fetch_and(!(1u32 << idx), Ordering::AcqRel);
                }
            }
        }

        if let (Some(idx), Some(func)) = (best_idx, best_fn) {
            self.pending_tasks.fetch_and(!(1u32 << idx), Ordering::AcqRel);
            Some((idx, func))
        } else {
            None
        }
    }

    /// Poll and execute any tasks that have been scheduled via the UTIL sequencer API.
    ///
    /// This function supports re-entrant calls (UTIL_SEQ_WaitEvt calls this recursively).
    /// The mask parameter restricts which tasks can run in this invocation.
    ///
    /// Returns true if at least one task was executed.
    pub fn run(&self, mask: u32) -> bool {
        compiler_fence(Ordering::Acquire);

        let mut executed_any = false;

        // Save and update SuperMask for nested calls
        // Each nested call makes the mask MORE restrictive (following ST's implementation)
        let super_mask_backup = self.super_mask.fetch_and(mask, Ordering::AcqRel);

        loop {
            loop {
                let current_super_mask = self.super_mask.load(Ordering::Acquire);
                let next = critical_section::with(|_| self.select_next_task(current_super_mask));
                match next {
                    Some((idx, task)) => {
                        // Set current task index before executing
                        self.current_task_idx.store(idx as u32, Ordering::Release);

                        unsafe {
                            task();
                        }

                        executed_any = true;

                        // Force a fresh read of the pending bitmask after each task completion.
                        // TODO: this appears to do nothing (will be optimized away)
                        let _ = idx;
                    }
                    None => break,
                }
            }

            let current_super_mask = self.super_mask.load(Ordering::Acquire);
            let pending = self.pending_tasks.load(Ordering::Acquire);
            if (pending & current_super_mask) == 0 {
                break;
            }
        }

        // Restore SuperMask when returning from nested call
        self.super_mask.store(super_mask_backup, Ordering::Release);

        // Clear current task index when no task is running
        self.current_task_idx.store(NO_TASK_RUNNING, Ordering::Release);

        compiler_fence(Ordering::Release);

        executed_any
    }
}

/// UTIL_SEQ_Run C API
/// Runs the sequencer with the given mask.
/// Per ST API: UTIL_SEQ_Run(UTIL_SEQ_DEFAULT) runs all tasks.
#[unsafe(no_mangle)]
pub extern "C" fn UTIL_SEQ_Run(mask: u32) {
    let _ = run(mask); // Discard return value for C compatibility
}

#[unsafe(no_mangle)]
pub extern "C" fn UTIL_SEQ_RegTask(task_mask: u32, _flags: u32, task: Option<TaskFn>) {
    #[cfg(feature = "defmt")]
    defmt::trace!("UTIL_SEQ_RegTask: mask=0x{:08X}, task={:?}", task_mask, task);

    if let Some(idx) = mask_to_index(task_mask) {
        critical_section::with(|_| unsafe {
            SEQUENCER.tasks.set_task(idx, task, DEFAULT_PRIORITY);
        });
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn UTIL_SEQ_UnregTask(task_mask: u32) {
    if let Some(idx) = mask_to_index(task_mask) {
        critical_section::with(|_| unsafe {
            SEQUENCER.tasks.set_task(idx, None, DEFAULT_PRIORITY);
        });
        SEQUENCER.pending_tasks.fetch_and(!(task_mask), Ordering::AcqRel);
    }
}

/// Check if a task is registered
/// Returns 1 if registered, 0 if not
#[unsafe(no_mangle)]
pub extern "C" fn UTIL_SEQ_IsRegisteredTask(task_mask: u32) -> u32 {
    if let Some(idx) = mask_to_index(task_mask) {
        critical_section::with(|_| unsafe { if SEQUENCER.tasks.task(idx).is_some() { 1 } else { 0 } })
    } else {
        0
    }
}

/// Check if a task is schedulable (pending and not masked)
/// Returns 1 if schedulable, 0 if not
#[unsafe(no_mangle)]
pub extern "C" fn UTIL_SEQ_IsSchedulableTask(task_mask: u32) -> u32 {
    let pending = SEQUENCER.pending_tasks.load(Ordering::Acquire);
    let super_mask = SEQUENCER.super_mask.load(Ordering::Acquire);

    if (pending & super_mask & task_mask) == task_mask {
        1
    } else {
        0
    }
}

/// Check if a task is paused
/// Returns 1 if paused, 0 if not paused
/// Note: In our simplified implementation, we don't have a separate pause mask like ST's TaskMask
/// We just check if the task is NOT pending
#[unsafe(no_mangle)]
pub extern "C" fn UTIL_SEQ_IsPauseTask(task_mask: u32) -> u32 {
    // In ST's implementation, pausing removes from TaskMask but not from TaskSet
    // In our implementation, pausing is equivalent to clearing the pending bit
    let pending = SEQUENCER.pending_tasks.load(Ordering::Acquire);
    if (pending & task_mask) == 0 {
        1 // Not pending = paused
    } else {
        0 // Pending = not paused
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn UTIL_SEQ_SetTask(task_mask: u32, priority: u32) {
    #[cfg(feature = "defmt")]
    defmt::trace!("UTIL_SEQ_SetTask: mask=0x{:08X}, prio={}", task_mask, priority);

    let prio = (priority & 0xFF) as u8;

    if let Some(idx) = mask_to_index(task_mask) {
        let registered = critical_section::with(|_| unsafe {
            if SEQUENCER.tasks.task(idx).is_some() {
                SEQUENCER.tasks.update_priority(idx, prio);
                true
            } else {
                false
            }
        });

        if registered {
            SEQUENCER.pending_tasks.fetch_or(task_mask, Ordering::Release);
            SEQUENCER.seq_pend();
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn UTIL_SEQ_ResumeTask(task_mask: u32) {
    SEQUENCER.pending_tasks.fetch_or(task_mask, Ordering::Release);
    SEQUENCER.seq_pend();
}

#[unsafe(no_mangle)]
pub extern "C" fn UTIL_SEQ_PauseTask(task_mask: u32) {
    SEQUENCER.pending_tasks.fetch_and(!task_mask, Ordering::AcqRel);
}

#[unsafe(no_mangle)]
pub extern "C" fn UTIL_SEQ_SetEvt(event_mask: u32) {
    trace!("UTIL_SEQ_SetEvt: mask=0x{:08X}", event_mask);

    SEQUENCER.events.fetch_or(event_mask, Ordering::Release);
    SEQUENCER.seq_pend();
}

#[unsafe(no_mangle)]
pub extern "C" fn UTIL_SEQ_ClrEvt(event_mask: u32) {
    SEQUENCER.events.fetch_and(!event_mask, Ordering::AcqRel);
}

#[unsafe(no_mangle)]
pub extern "C" fn UTIL_SEQ_IsEvtSet(event_mask: u32) -> u32 {
    let state = SEQUENCER.events.load(Ordering::Acquire);
    if (state & event_mask) == event_mask { 1 } else { 0 }
}

/// Check if any pending event matches the currently waited event
/// Returns the event_id if pending, 0 otherwise
/// This matches ST's UTIL_SEQ_IsEvtPend API
#[unsafe(no_mangle)]
pub extern "C" fn UTIL_SEQ_IsEvtPend() -> u32 {
    // In ST's implementation, this checks EvtSet & EvtWaited
    // Since we don't track a global EvtWaited (we handle it locally in WaitEvt),
    // we just return the current events mask
    SEQUENCER.events.load(Ordering::Acquire)
}

/// Initialize the sequencer (matches ST's API)
/// In our implementation, initialization is done statically, so this is a no-op
#[unsafe(no_mangle)]
pub extern "C" fn UTIL_SEQ_Init() {
    // Sequencer is initialized statically in our implementation
    trace!("UTIL_SEQ_Init called (no-op in Embassy implementation)");
}

/// Deinitialize the sequencer (matches ST's API)
#[unsafe(no_mangle)]
pub extern "C" fn UTIL_SEQ_DeInit() {
    // No-op in our implementation
    trace!("UTIL_SEQ_DeInit called (no-op in Embassy implementation)");
}

/// Idle function called when sequencer has no work
/// This is a weak function in ST's implementation that can be overridden
/// In our implementation, this is handled by seq_yield()
#[unsafe(no_mangle)]
pub extern "C" fn UTIL_SEQ_Idle() {
    // This would be called by ST's sequencer when idle
    // In our implementation, we handle this via seq_yield in the main loop
    SEQUENCER.seq_yield();
}

/// Pre-idle hook (called before entering idle)
#[unsafe(no_mangle)]
pub extern "C" fn UTIL_SEQ_PreIdle() {
    // Hook for power management - can be overridden by application
    // Default implementation does nothing (matches ST's weak function)
}

/// Post-idle hook (called after waking from idle)
#[unsafe(no_mangle)]
pub extern "C" fn UTIL_SEQ_PostIdle() {
    // Hook for power management - can be overridden by application
    // Default implementation does nothing (matches ST's weak function)
}

#[unsafe(no_mangle)]
pub extern "C" fn UTIL_SEQ_WaitEvt(event_mask: u32) {
    trace!("UTIL_SEQ_WaitEvt: mask=0x{:08X}", event_mask);

    // Store the current task index (we may be called from a task or outside any task)
    let current_task_idx = SEQUENCER.current_task_idx.load(Ordering::Acquire);

    loop {
        // Compute the exclusion mask: run all tasks EXCEPT the current one
        // This matches ST's UTIL_SEQ_EvtIdle behavior: UTIL_SEQ_Run(~TaskId_bm)
        let run_mask = if current_task_idx == NO_TASK_RUNNING {
            // Called outside any task (e.g., during init), run all tasks
            ALL_TASKS_MASK
        } else {
            // Called from a task, exclude that task from running
            ALL_TASKS_MASK & !(1u32 << current_task_idx)
        };

        // This is a RE-ENTRANT call - it will run other tasks while we wait
        SEQUENCER.run(run_mask);

        let current = SEQUENCER.events.load(Ordering::Acquire);
        if (current & event_mask) == event_mask {
            SEQUENCER.events.fetch_and(!event_mask, Ordering::AcqRel);
            trace!("UTIL_SEQ_WaitEvt: event received");
            break;
        }
        trace!(
            "UTIL_SEQ_WaitEvt: waiting (in_seq_ctx={}, current_task={})",
            SEQUENCER.context.in_task_context(),
            current_task_idx
        );

        SEQUENCER.seq_yield();
    }

    // Restore current task index after waiting
    // (it may have been changed by nested run() calls)
    SEQUENCER.current_task_idx.store(current_task_idx, Ordering::Release);
}
