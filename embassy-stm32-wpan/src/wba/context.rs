//! Context switching primitives for the BLE sequencer
//!
//! This module provides low-level context switching between the embassy executor
//! and the BLE sequencer. The sequencer runs on a separate stack and can yield
//! back to the executor when it would otherwise block (WFE).
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │                    Embassy Executor                      │
//! │  ┌─────────────────────────────────────────────────┐    │
//! │  │ Runner Task (uses executor's stack)              │    │
//! │  │                                                   │    │
//! │  │  loop {                                          │    │
//! │  │    sequencer_resume();  // switch to sequencer   │    │
//! │  │    // returns when sequencer yields              │    │
//! │  │    yield_now().await;   // let other tasks run   │    │
//! │  │  }                                               │    │
//! │  └─────────────────────────────────────────────────┘    │
//! └─────────────────────────────────────────────────────────┘
//!
//! ┌─────────────────────────────────────────────────────────┐
//! │  Sequencer Context (separate stack)                      │
//! │                                                          │
//! │  - Runs BLE stack C code via poll_pending_tasks()       │
//! │  - When idle, calls sequencer_yield() to return         │
//! └─────────────────────────────────────────────────────────┘
//! ```

use core::arch::asm;
use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicU8, Ordering};

use aligned::{A8, Aligned};

/// Size of the sequencer stack in bytes (8KB)
/// This needs to be large enough for the C BLE stack's call depth
const SEQUENCER_CTX_STACK_SIZE: usize = 8 * 1024;

/// Saved CPU context for context switching
#[repr(C)]
struct SavedContext {
    /// Saved stack pointer
    sp: u32,
    /// Context has been initialized
    initialized: bool,
}

impl SavedContext {
    const fn new() -> Self {
        Self {
            sp: 0,
            initialized: false,
        }
    }
}

/// Sequencer state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum SequencerState {
    /// Not yet started
    Uninitialized = 0,
    /// Running (in sequencer context)
    Running = 1,
    /// Yielded back to runner
    Yielded = 2,
    /// Stopped/finished
    Stopped = 3,
}

impl SequencerState {
    fn from_u8(v: u8) -> Self {
        match v {
            0 => Self::Uninitialized,
            1 => Self::Running,
            2 => Self::Yielded,
            3 => Self::Stopped,
            _ => Self::Uninitialized,
        }
    }
}

/// Global sequencer state
struct SequencerContext {
    /// The sequencer's saved context (SP when yielded)
    seq_ctx: UnsafeCell<SavedContext>,
    /// The runner's saved context (SP when in sequencer)
    runner_ctx: UnsafeCell<SavedContext>,
    /// Current state
    state: AtomicU8,
    /// The sequencer's stack (must be 8-byte aligned)
    seq_stack: Aligned<A8, UnsafeCell<[u8; SEQUENCER_CTX_STACK_SIZE]>>,
}

unsafe impl Sync for SequencerContext {}

impl SequencerContext {
    const fn new() -> Self {
        Self {
            seq_ctx: UnsafeCell::new(SavedContext::new()),
            runner_ctx: UnsafeCell::new(SavedContext::new()),
            state: AtomicU8::new(SequencerState::Uninitialized as u8),
            seq_stack: Aligned(UnsafeCell::new([0u8; SEQUENCER_CTX_STACK_SIZE])),
        }
    }

    fn state(&self) -> SequencerState {
        SequencerState::from_u8(self.state.load(Ordering::Acquire))
    }

    fn set_state(&self, state: SequencerState) {
        self.state.store(state as u8, Ordering::Release);
    }
}

/// Global sequencer context
static SEQUENCER_CTX: SequencerContext = SequencerContext::new();

/// Initialize the sequencer stack
///
/// This must be called once before using the sequencer.
/// Returns the initial stack pointer for the sequencer.
fn init_sequencer_stack() -> u32 {
    let stack = unsafe { &mut *SEQUENCER_CTX.seq_stack.get() };

    // Stack grows downward, so we start at the top
    // Ensure 8-byte alignment (ARM requirement for function calls)
    let stack_top = stack.as_ptr() as usize + SEQUENCER_CTX_STACK_SIZE;
    let aligned_top = stack_top & !0x7; // 8-byte align

    aligned_top as u32
}

/// Entry point for the sequencer context
///
/// This function runs in the sequencer's stack context and repeatedly
/// polls for pending tasks, yielding when there's nothing to do.
extern "C" fn sequencer_entry() -> ! {
    loop {
        // Poll and execute any pending sequencer tasks
        super::util_seq::run();

        // Yield back to the runner
        // This will return when the runner resumes us
        sequencer_yield_inner();
    }
}

/// Resume the sequencer from the runner context
///
/// This function switches from the runner's context to the sequencer's context.
/// It returns when the sequencer yields (calls `sequencer_yield`).
///
/// # Safety
///
/// Must be called from a proper task context with a valid stack.
pub fn sequencer_resume() {
    let state = SEQUENCER_CTX.state();

    match state {
        SequencerState::Uninitialized => {
            // First time - initialize and start the sequencer
            let stack_top = init_sequencer_stack();

            // Set up initial context for sequencer
            // We need to set up the stack so that when we "restore" to it,
            // it will start executing sequencer_entry
            unsafe {
                let seq_ctx = &mut *SEQUENCER_CTX.seq_ctx.get();

                // Set up fake saved context on sequencer stack
                // Stack layout (growing down):
                // [stack_top - 0]:  (8-byte alignment padding if needed)
                // [stack_top - 4]:  Initial LR (not used, sequencer_entry is noreturn)
                // [stack_top - 8]:  R11
                // [stack_top - 12]: R10
                // [stack_top - 16]: R9
                // [stack_top - 20]: R8
                // [stack_top - 24]: R7
                // [stack_top - 28]: R6
                // [stack_top - 32]: R5
                // [stack_top - 36]: R4
                // SP points here after "restore"

                let mut sp = stack_top;

                // Push a fake return address (entry point)
                sp -= 4;
                core::ptr::write_volatile(sp as *mut u32, sequencer_entry as u32);

                // Push fake saved registers R11-R4 (all zeros is fine)
                for _ in 0..8 {
                    sp -= 4;
                    core::ptr::write_volatile(sp as *mut u32, 0);
                }

                seq_ctx.sp = sp;
                seq_ctx.initialized = true;
            }

            SEQUENCER_CTX.set_state(SequencerState::Yielded);

            // Now do the actual switch
            do_context_switch();
        }

        SequencerState::Yielded => {
            // Sequencer has yielded, resume it
            do_context_switch();
        }

        SequencerState::Running => {
            // Already running - shouldn't happen
            #[cfg(feature = "defmt")]
            defmt::warn!("sequencer_resume called while already running");
        }

        SequencerState::Stopped => {
            // Sequencer has stopped - shouldn't happen normally
            #[cfg(feature = "defmt")]
            defmt::warn!("sequencer_resume called after stop");
        }
    }
}

/// Yield from the sequencer back to the runner
///
/// This is called from within the sequencer context when it has no more
/// work to do (would otherwise WFE).
fn sequencer_yield_inner() {
    SEQUENCER_CTX.set_state(SequencerState::Yielded);
    do_context_switch();
    // When we return here, we've been resumed
    SEQUENCER_CTX.set_state(SequencerState::Running);
}

/// Public function to yield from sequencer
///
/// This should be called instead of WFE when the sequencer is idle.
pub fn sequencer_yield() {
    if SEQUENCER_CTX.state() == SequencerState::Running {
        sequencer_yield_inner();
    }
}

/// Perform the actual context switch
///
/// This saves the current context and restores the other context.
/// Works bidirectionally - runner->sequencer and sequencer->runner.
#[inline(never)]
fn do_context_switch() {
    let state = SEQUENCER_CTX.state();
    let switching_to_sequencer = state == SequencerState::Yielded;

    unsafe {
        let (save_ctx, restore_ctx) = if switching_to_sequencer {
            (SEQUENCER_CTX.runner_ctx.get(), SEQUENCER_CTX.seq_ctx.get())
        } else {
            (SEQUENCER_CTX.seq_ctx.get(), SEQUENCER_CTX.runner_ctx.get())
        };

        if switching_to_sequencer {
            SEQUENCER_CTX.set_state(SequencerState::Running);
        }

        context_switch(save_ctx, restore_ctx);
    }
}

/// Low-level context switch using inline assembly
///
/// Saves R4-R11 and SP to `save_ctx`, then restores from `restore_ctx`.
///
/// # Safety
///
/// Both pointers must be valid SavedContext structures.
#[inline(never)]
unsafe fn context_switch(save_ctx: *mut SavedContext, restore_ctx: *mut SavedContext) {
    #[cfg(target_arch = "arm")]
    {
        asm!(
            // Save current context
            // Push callee-saved registers R4-R11 onto current stack
            "push {{r4-r11, lr}}",

            // Save current SP to save_ctx->sp
            "str sp, [{save_ctx}]",

            // Restore new context
            // Load SP from restore_ctx->sp
            "ldr sp, [{restore_ctx}]",

            // Pop callee-saved registers R4-R11 from new stack
            "pop {{r4-r11, lr}}",

            // Return to new context (via LR we just popped)
            "bx lr",

            save_ctx = in(reg) save_ctx,
            restore_ctx = in(reg) restore_ctx,
            options(noreturn)
        );
    }

    #[cfg(not(target_arch = "arm"))]
    {
        // Fallback for non-ARM (e.g., testing on host)
        let _ = (save_ctx, restore_ctx);
        panic!("Context switching only supported on ARM");
    }
}

/// Check if we're currently in the sequencer context
pub fn in_sequencer_context() -> bool {
    SEQUENCER_CTX.state() == SequencerState::Running
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stack_alignment() {
        let stack_ptr = init_sequencer_stack();
        assert_eq!(stack_ptr % 8, 0, "Stack must be 8-byte aligned");
    }
}
