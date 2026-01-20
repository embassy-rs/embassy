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
use core::sync::atomic::{AtomicU32, Ordering, compiler_fence};
use core::task::Poll;

use embassy_sync::waitqueue::AtomicWaker;

use super::context;

type TaskFn = unsafe extern "C" fn();

unsafe extern "Rust" {
    fn __pender(context: *mut ());
}

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
    tasks: TaskTable,
    pending_tasks: AtomicU32,
    events: AtomicU32,
    waker: AtomicWaker,
}

static SEQUENCER: Sequencer = Sequencer {
    tasks: TaskTable::new(),
    pending_tasks: AtomicU32::new(0),
    events: AtomicU32::new(0),
    waker: AtomicWaker::new(),
};

fn mask_to_index(mask: u32) -> Option<usize> {
    if mask == 0 {
        return None;
    }
    let idx = mask.trailing_zeros() as usize;
    if idx < MAX_TASKS { Some(idx) } else { None }
}

pub fn run() {
    SEQUENCER.run();
}

pub fn seq_pend() {
    SEQUENCER.seq_pend();
}

pub async fn wait_for_event() {
    SEQUENCER.wait_for_event().await
}

impl Sequencer {
    pub async fn wait_for_event(&self) {
        poll_fn(|cx| {
            self.waker.register(cx.waker());

            compiler_fence(Ordering::Acquire);

            if self.has_work() {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;
    }

    fn seq_pend(&self) {
        self.waker.wake();
        unsafe { __pender(u32::MAX as *mut _) };
    }

    /// Wait for an event
    ///
    /// Instead of blocking with WFE, this yields back to the runner context
    /// so that the embassy executor can run other tasks.
    #[inline(always)]
    fn seq_yield(&self) {
        // If we're in the sequencer context, yield back to the runner
        // If we're not (e.g., during initialization), use actual WFE
        if context::in_sequencer_context() {
            context::sequencer_yield();
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

    fn select_next_task(&self) -> Option<(usize, TaskFn)> {
        let pending = self.pending_tasks.load(Ordering::Acquire);
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
    pub fn run(&self) {
        compiler_fence(Ordering::Acquire);
        loop {
            loop {
                let next = critical_section::with(|_| self.select_next_task());
                match next {
                    Some((idx, task)) => unsafe {
                        task();
                        // Force a fresh read of the pending bitmask after each task completion.
                        let _ = idx;
                    },
                    None => break,
                }
            }

            if self.pending_tasks.load(Ordering::Acquire) == 0 {
                break;
            }
        }

        compiler_fence(Ordering::Release);
    }
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
    #[cfg(feature = "defmt")]
    defmt::trace!("UTIL_SEQ_SetEvt: mask=0x{:08X}", event_mask);

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

#[unsafe(no_mangle)]
pub extern "C" fn UTIL_SEQ_WaitEvt(event_mask: u32) {
    #[cfg(feature = "defmt")]
    defmt::trace!("UTIL_SEQ_WaitEvt: mask=0x{:08X}", event_mask);

    loop {
        SEQUENCER.run();

        let current = SEQUENCER.events.load(Ordering::Acquire);
        if (current & event_mask) == event_mask {
            SEQUENCER.events.fetch_and(!event_mask, Ordering::AcqRel);
            #[cfg(feature = "defmt")]
            defmt::trace!("UTIL_SEQ_WaitEvt: event received");
            break;
        }

        #[cfg(feature = "defmt")]
        defmt::trace!(
            "UTIL_SEQ_WaitEvt: waiting (in_seq_ctx={})",
            context::in_sequencer_context()
        );

        SEQUENCER.seq_yield();
    }
}
