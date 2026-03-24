//! Context switching primitives for the BLE sequencer
//!
//! This module provides low-level context switching between the embassy executor
//! and the BLE sequencer. The sequencer runs on a separate stack and can yield
//! back to the executor when it would otherwise block (WFE).
//!
//! # Architecture
//!
//! ```text
//! ┌────────────────────────────────────────────────────────────┐
//! │                    Embassy Executor                        │
//! │  ┌────────────────────────────────────────────────────┐    │
//! │  │ Runner Task (uses executor's stack)                │    │
//! │  │                                                    │    │
//! │  │  loop {                                            │    │
//! │  │    util_seq::seq_resume();  // switch to sequencer │    │
//! │  │    // returns when sequencer yields                │    │
//! │  │    yield_now().await;   // let other tasks run     │    │
//! │  │  }                                                 │    │
//! │  └────────────────────────────────────────────────────┘    │
//! └────────────────────────────────────────────────────────────┘
//!
//! ┌────────────────────────────────────────────────────────────┐
//! │  Sequencer Context (separate stack)                        │
//! │                                                            │
//! │  - Runs BLE stack C code via poll_pending_tasks()          │
//! │  - When idle, calls sequencer_yield() to return            │
//! └────────────────────────────────────────────────────────────┘
//! ```

use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicU8, Ordering};

use aligned::{A8, Aligned};

// Pure assembly context switch: saves R4-R11+LR, swaps SP, restores R4-R11+LR, returns.
// Defined via global_asm! so the compiler generates NO prologue/epilogue — the caller
// sees a normal C function call, which is exactly what this is (just on a different stack).
#[cfg(target_arch = "arm")]
core::arch::global_asm!(
    ".thumb_func",
    ".global context_switch_asm",
    ".type context_switch_asm, %function",
    "context_switch_asm:",
    "push {{r4-r11, lr}}",
    "str sp, [r0]",
    "ldr sp, [r1]",
    "pop {{r4-r11, lr}}",
    "bx lr",
    ".size context_switch_asm, . - context_switch_asm",
);

#[cfg(target_arch = "arm")]
unsafe extern "C" {
    /// Raw context switch. Arguments:
    /// - r0: pointer to u32 where current SP will be saved
    /// - r1: pointer to u32 from which new SP will be loaded
    fn context_switch_asm(save_sp: *mut u32, restore_sp: *mut u32);
}

/// Size of the sequencer stack in bytes (32KB)
/// This needs to be large enough for the C BLE stack's call depth,
/// including connection event processing and HCI event parsing
/// (the Event enum is ~300+ bytes due to heapless::Vec variants).
const SEQUENCER_CTX_STACK_SIZE: usize = 32 * 1024;

/// Global sequencer state
pub(crate) struct ContextManager {
    /// The sequencer's saved SP
    task_sp: UnsafeCell<u32>,
    /// The runner's saved SP
    runner_sp: UnsafeCell<u32>,
    /// Current state
    state: AtomicU8,
    /// The sequencer's stack (must be 8-byte aligned)
    task_stack: Aligned<A8, UnsafeCell<[u8; SEQUENCER_CTX_STACK_SIZE]>>,
    task_entry: extern "C" fn() -> !,
}

unsafe impl Sync for ContextManager {}

/// Sequencer state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ContextManagerState {
    /// Not yet started
    Uninitialized = 0,
    /// Running (in sequencer context)
    Running = 1,
    /// Yielded back to runner
    Yielded = 2,
    /// Stopped/finished
    Stopped = 3,
}

impl From<u8> for ContextManagerState {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Uninitialized,
            1 => Self::Running,
            2 => Self::Yielded,
            3 => Self::Stopped,
            _ => Self::Uninitialized,
        }
    }
}

impl ContextManager {
    pub(crate) const fn new(task_entry: extern "C" fn() -> !) -> Self {
        Self {
            task_sp: UnsafeCell::new(0),
            runner_sp: UnsafeCell::new(0),
            state: AtomicU8::new(ContextManagerState::Uninitialized as u8),
            task_stack: Aligned(UnsafeCell::new([0u8; SEQUENCER_CTX_STACK_SIZE])),
            task_entry,
        }
    }

    /// Initialize the sequencer stack
    fn init_sequencer_stack(&'static self) {
        // Stack grows downward, so we start at the top
        // Ensure 8-byte alignment (ARM requirement for function calls)
        let stack_top = &raw const self.task_stack as usize + SEQUENCER_CTX_STACK_SIZE;
        let stack_top = (stack_top & !0x7) as u32; // 8-byte align

        // Set up fake saved context on sequencer stack.
        // `pop {r4-r11, lr}` pops 9 words (36 bytes) starting at SP:
        //   [SP+0]=R4, [SP+4]=R5, ..., [SP+28]=R11, [SP+32]=LR
        //
        // Stack layout (growing down):
        // [stack_top - 4]:  padding (8-byte alignment; entry fn is noreturn)
        // [stack_top - 8]:  LR = task_entry (popped into LR, then bx lr)
        // [stack_top - 12]: R11 = 0
        // [stack_top - 16]: R10 = 0
        // [stack_top - 20]: R9 = 0
        // [stack_top - 24]: R8 = 0
        // [stack_top - 28]: R7 = 0
        // [stack_top - 32]: R6 = 0
        // [stack_top - 36]: R5 = 0
        // [stack_top - 40]: R4 = 0    <-- SP points here
        unsafe {
            let mut sp = stack_top;

            // Padding for 8-byte alignment
            sp -= 4;
            core::ptr::write_volatile(sp as *mut u32, 0);

            // LR = entry point with Thumb bit set (required for bx)
            sp -= 4;
            core::ptr::write_volatile(sp as *mut u32, self.task_entry as u32 | 1);

            // Fake saved registers R11-R4 (all zeros)
            for _ in 0..8 {
                sp -= 4;
                core::ptr::write_volatile(sp as *mut u32, 0);
            }

            // SP is now stack_top - 40, which is 8-byte aligned
            core::ptr::write_volatile(self.task_sp.get(), sp);
        }

        self.set_state(ContextManagerState::Yielded);
    }

    /// Resume the sequencer from the runner context.
    /// Returns when the sequencer yields.
    pub(crate) fn task_resume(&'static self) {
        match self.get_state() {
            ContextManagerState::Uninitialized => {
                self.init_sequencer_stack();
                self.set_state(ContextManagerState::Running);
                self.do_switch(self.runner_sp.get(), self.task_sp.get());
                // Returns here when task yields
            }
            ContextManagerState::Yielded => {
                self.set_state(ContextManagerState::Running);
                self.do_switch(self.runner_sp.get(), self.task_sp.get());
                // Returns here when task yields
            }
            ContextManagerState::Running => {
                #[cfg(feature = "defmt")]
                defmt::warn!("sequencer_resume called while already running");
            }
            ContextManagerState::Stopped => {
                #[cfg(feature = "defmt")]
                defmt::warn!("sequencer_resume called after stop");
            }
        }
    }

    /// Yield from the sequencer back to the runner.
    pub(crate) fn task_yield(&'static self) {
        if self.get_state() == ContextManagerState::Running {
            self.set_state(ContextManagerState::Yielded);
            self.do_switch(self.task_sp.get(), self.runner_sp.get());
            // Returns here when runner resumes us
        }
    }

    #[inline(always)]
    fn do_switch(&'static self, save_sp: *mut u32, restore_sp: *mut u32) {
        #[cfg(target_arch = "arm")]
        unsafe {
            context_switch_asm(save_sp, restore_sp);
        }

        #[cfg(not(target_arch = "arm"))]
        {
            let _ = (save_sp, restore_sp);
            panic!("Context switching only supported on ARM");
        }
    }

    pub fn in_task_context(&self) -> bool {
        self.get_state() == ContextManagerState::Running
    }

    fn get_state(&self) -> ContextManagerState {
        self.state.load(Ordering::Acquire).into()
    }

    fn set_state(&self, state: ContextManagerState) {
        self.state.store(state as u8, Ordering::Release);
    }
}
