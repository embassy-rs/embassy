#![cfg(feature = "wba")]

use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use critical_section::with as critical;

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

#[inline(always)]
fn wake_event() {
    #[cfg(target_arch = "arm")]
    {
        cortex_m::asm::sev();
    }

    #[cfg(not(target_arch = "arm"))]
    {
        // No-op on architectures without SEV support.
    }
}

#[inline(always)]
fn wait_event() {
    #[cfg(target_arch = "arm")]
    {
        cortex_m::asm::wfe();
    }

    #[cfg(not(target_arch = "arm"))]
    {
        core::hint::spin_loop();
    }
}

static TASKS: TaskTable = TaskTable::new();
static PENDING_TASKS: AtomicU32 = AtomicU32::new(0);
static EVENTS: AtomicU32 = AtomicU32::new(0);
static SCHEDULING: AtomicBool = AtomicBool::new(false);

fn mask_to_index(mask: u32) -> Option<usize> {
    if mask == 0 {
        return None;
    }
    let idx = mask.trailing_zeros() as usize;
    if idx < MAX_TASKS { Some(idx) } else { None }
}

fn drain_pending_tasks() {
    loop {
        if SCHEDULING
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return;
        }

        loop {
            let next = critical(|_| select_next_task());
            match next {
                Some((idx, task)) => unsafe {
                    task();
                    // Force a fresh read of the pending bitmask after each task completion.
                    let _ = idx;
                },
                None => break,
            }
        }

        SCHEDULING.store(false, Ordering::Release);

        if PENDING_TASKS.load(Ordering::Acquire) == 0 {
            break;
        }
    }
}

/// Poll and execute any tasks that have been scheduled via the UTIL sequencer API.
pub fn poll_pending_tasks() {
    drain_pending_tasks();
}

fn select_next_task() -> Option<(usize, TaskFn)> {
    let pending = PENDING_TASKS.load(Ordering::Acquire);
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
            if let Some(func) = TASKS.task(idx) {
                let prio = TASKS.priority(idx);
                if prio <= best_priority {
                    if prio < best_priority || best_idx.map_or(true, |current| idx < current) {
                        best_priority = prio;
                        best_idx = Some(idx);
                        best_fn = Some(func);
                    }
                }
            } else {
                PENDING_TASKS.fetch_and(!(1u32 << idx), Ordering::AcqRel);
            }
        }
    }

    if let (Some(idx), Some(func)) = (best_idx, best_fn) {
        PENDING_TASKS.fetch_and(!(1u32 << idx), Ordering::AcqRel);
        Some((idx, func))
    } else {
        None
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn UTIL_SEQ_RegTask(task_mask: u32, _flags: u32, task: Option<TaskFn>) {
    if let Some(idx) = mask_to_index(task_mask) {
        critical(|_| unsafe {
            TASKS.set_task(idx, task, DEFAULT_PRIORITY);
        });
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn UTIL_SEQ_UnregTask(task_mask: u32) {
    if let Some(idx) = mask_to_index(task_mask) {
        critical(|_| unsafe {
            TASKS.set_task(idx, None, DEFAULT_PRIORITY);
        });
        PENDING_TASKS.fetch_and(!(task_mask), Ordering::AcqRel);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn UTIL_SEQ_SetTask(task_mask: u32, priority: u32) {
    let prio = (priority & 0xFF) as u8;

    if let Some(idx) = mask_to_index(task_mask) {
        let registered = critical(|_| unsafe {
            if TASKS.task(idx).is_some() {
                TASKS.update_priority(idx, prio);
                true
            } else {
                false
            }
        });

        if registered {
            PENDING_TASKS.fetch_or(task_mask, Ordering::Release);
            wake_event();
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn UTIL_SEQ_ResumeTask(task_mask: u32) {
    PENDING_TASKS.fetch_or(task_mask, Ordering::Release);
    wake_event();
}

#[unsafe(no_mangle)]
pub extern "C" fn UTIL_SEQ_PauseTask(task_mask: u32) {
    PENDING_TASKS.fetch_and(!task_mask, Ordering::AcqRel);
}

#[unsafe(no_mangle)]
pub extern "C" fn UTIL_SEQ_SetEvt(event_mask: u32) {
    EVENTS.fetch_or(event_mask, Ordering::Release);
    wake_event();
}

#[unsafe(no_mangle)]
pub extern "C" fn UTIL_SEQ_ClrEvt(event_mask: u32) {
    EVENTS.fetch_and(!event_mask, Ordering::AcqRel);
}

#[unsafe(no_mangle)]
pub extern "C" fn UTIL_SEQ_IsEvtSet(event_mask: u32) -> u32 {
    let state = EVENTS.load(Ordering::Acquire);
    if (state & event_mask) == event_mask { 1 } else { 0 }
}

#[unsafe(no_mangle)]
pub extern "C" fn UTIL_SEQ_WaitEvt(event_mask: u32) {
    loop {
        poll_pending_tasks();

        let current = EVENTS.load(Ordering::Acquire);
        if (current & event_mask) == event_mask {
            EVENTS.fetch_and(!event_mask, Ordering::AcqRel);
            break;
        }

        wait_event();
    }
}
