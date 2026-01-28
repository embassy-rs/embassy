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

use core::arch::asm;
use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicU8, Ordering};

use aligned::{A8, Aligned};

/// Size of the sequencer stack in bytes (8KB)
/// This needs to be large enough for the C BLE stack's call depth
const SEQUENCER_CTX_STACK_SIZE: usize = 8 * 1024;

/// Saved CPU context for context switching
#[repr(C)]
struct Context {
    /// Saved stack pointer
    sp: u32,
}

impl Context {
    const fn new() -> Self {
        Self { sp: 0 }
    }
}

/// Sequencer state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ContextManagerState {
    /// Not yet started
    Uninitialized,
    /// Running (in sequencer context)
    Running,
    /// Yielded back to runner
    Yielded,
    /// Stopped/finished
    Stopped,
}

enum ContextSwitchType {
    RunnerToTask,
    TaskToRunner,
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

impl From<ContextManagerState> for u8 {
    fn from(value: ContextManagerState) -> Self {
        match value {
            ContextManagerState::Uninitialized => 0,
            ContextManagerState::Running => 1,
            ContextManagerState::Yielded => 2,
            ContextManagerState::Stopped => 3,
        }
    }
}

/// Global sequencer state
pub(crate) struct ContextManager {
    /// The sequencer's saved context (SP when yielded)
    task_ctx: UnsafeCell<Context>,
    /// The runner's saved context (SP when in sequencer)
    runner_ctx: UnsafeCell<Context>,
    /// Current state
    state: AtomicU8,
    /// The sequencer's stack (must be 8-byte aligned)
    task_stack: Aligned<A8, UnsafeCell<[u8; SEQUENCER_CTX_STACK_SIZE]>>,
    task_entry: extern "C" fn() -> !,
}

unsafe impl Sync for ContextManager {}

impl ContextManager {
    pub(crate) const fn new(task_entry: extern "C" fn() -> !) -> Self {
        Self {
            task_ctx: UnsafeCell::new(Context::new()),
            runner_ctx: UnsafeCell::new(Context::new()),
            state: AtomicU8::new(ContextManagerState::Uninitialized as u8),
            task_stack: Aligned(UnsafeCell::new([0u8; SEQUENCER_CTX_STACK_SIZE])),
            task_entry: task_entry,
        }
    }

    /// Initialize the sequencer stack
    ///
    /// This must be called once before using the sequencer.
    /// Returns the initial stack pointer for the sequencer.
    fn init_sequencer_stack(&'static self) {
        // Stack grows downward, so we start at the top
        // Ensure 8-byte alignment (ARM requirement for function calls)
        let stack_top = &raw const self.task_stack as usize + SEQUENCER_CTX_STACK_SIZE;
        let stack_top = (stack_top & !0x7) as u32; // 8-byte align

        // Set up initial context for sequencer
        // We need to set up the stack so that when we "restore" to it,
        // it will start executing sequencer_entry
        unsafe {
            let task_ctx = self.task_ctx.get();

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
            core::ptr::write_volatile(sp as *mut u32, self.task_entry as u32);

            // Push fake saved registers R11-R4 (all zeros is fine)
            for _ in 0..8 {
                sp -= 4;
                core::ptr::write_volatile(sp as *mut u32, 0);
            }

            (*task_ctx).sp = sp;
        }

        self.set_state(ContextManagerState::Yielded);
    }

    /// Resume the sequencer from the runner context
    ///
    /// This function switches from the runner's context to the sequencer's context.
    /// It returns when the sequencer yields (calls `sequencer_yield`).
    ///
    /// # Safety
    ///
    /// Must be called from a proper task context with a valid stack.
    pub(crate) fn task_resume(&'static self) {
        match self.get_state() {
            ContextManagerState::Uninitialized => {
                self.init_sequencer_stack();
                self.switch_context(ContextSwitchType::RunnerToTask);
            }
            ContextManagerState::Yielded => {
                self.switch_context(ContextSwitchType::RunnerToTask);
            }
            ContextManagerState::Running => {
                // Already running - shouldn't happen
                #[cfg(feature = "defmt")]
                defmt::warn!("sequencer_resume called while already running");
            }
            ContextManagerState::Stopped => {
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
    ///
    /// This must not be called from an interrupt handler.
    pub(crate) fn task_yield(&'static self) {
        if self.get_state() == ContextManagerState::Running {
            self.switch_context(ContextSwitchType::TaskToRunner);
        }
    }

    /// Perform the actual context switch
    ///
    /// This saves the current context and restores the other context.
    /// Works bidirectionally - runner->sequencer and sequencer->runner.
    #[inline(never)]
    fn switch_context(&'static self, switch_type: ContextSwitchType) {
        self.set_state(match switch_type {
            ContextSwitchType::RunnerToTask => ContextManagerState::Running,
            ContextSwitchType::TaskToRunner => ContextManagerState::Yielded,
        });

        let (save_ctx, restore_ctx) = match switch_type {
            ContextSwitchType::RunnerToTask => (self.runner_ctx.get(), self.task_ctx.get()),
            ContextSwitchType::TaskToRunner => (self.task_ctx.get(), self.runner_ctx.get()),
        };

        unsafe {
            self.switch_context_inner(save_ctx, restore_ctx);
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
    unsafe fn switch_context_inner(&'static self, save_ctx: *mut Context, restore_ctx: *mut Context) {
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

    pub fn in_task_context(&self) -> bool {
        self.get_state() == ContextManagerState::Running
    }

    fn get_state(&self) -> ContextManagerState {
        self.state.load(Ordering::Acquire).into()
    }

    fn set_state(&self, state: ContextManagerState) {
        self.state.store(state.into(), Ordering::Release);
    }
}
