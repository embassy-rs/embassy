use core::arch::asm;
use core::sync::atomic::{compiler_fence, AtomicBool, AtomicU32, Ordering};

use super::timer_queue::TimerEnqueueOperation;

#[derive(Clone, Copy)]
pub(crate) struct Token(());

/// Creates a token and passes it to the closure.
///
/// This is a no-op replacement for `CriticalSection::with` because we don't need any locking.
pub(crate) fn locked<R>(f: impl FnOnce(Token) -> R) -> R {
    f(Token(()))
}

// Must be kept in sync with the layout of `State`!
pub(crate) const STATE_SPAWNED: u32 = 1 << 0;
pub(crate) const STATE_RUN_QUEUED: u32 = 1 << 8;
pub(crate) const STATE_TIMER_QUEUED: u32 = 1 << 16;

#[repr(C, align(4))]
pub(crate) struct State {
    /// Task is spawned (has a future)
    spawned: AtomicBool,
    /// Task is in the executor run queue
    run_queued: AtomicBool,
    /// Task is in the executor timer queue
    timer_queued: AtomicBool,
    pad: AtomicBool,
}

impl State {
    pub const fn new() -> State {
        Self {
            spawned: AtomicBool::new(false),
            run_queued: AtomicBool::new(false),
            timer_queued: AtomicBool::new(false),
            pad: AtomicBool::new(false),
        }
    }

    fn as_u32(&self) -> &AtomicU32 {
        unsafe { &*(self as *const _ as *const AtomicU32) }
    }

    /// If task is idle, mark it as spawned + run_queued and return true.
    #[inline(always)]
    pub fn spawn(&self) -> bool {
        compiler_fence(Ordering::Release);
        let r = self
            .as_u32()
            .compare_exchange(
                0,
                STATE_SPAWNED | STATE_RUN_QUEUED,
                Ordering::Relaxed,
                Ordering::Relaxed,
            )
            .is_ok();
        compiler_fence(Ordering::Acquire);
        r
    }

    /// Unmark the task as spawned.
    #[inline(always)]
    pub fn despawn(&self) {
        compiler_fence(Ordering::Release);
        self.spawned.store(false, Ordering::Relaxed);
    }

    /// Mark the task as run-queued if it's spawned and isn't already run-queued. Run the given
    /// function if the task was successfully marked.
    #[inline(always)]
    pub fn run_enqueue(&self, f: impl FnOnce(Token)) {
        unsafe {
            loop {
                let state: u32;
                asm!("ldrex {}, [{}]", out(reg) state, in(reg) self, options(nostack));

                if (state & STATE_RUN_QUEUED != 0) || (state & STATE_SPAWNED == 0) {
                    asm!("clrex", options(nomem, nostack));
                    return;
                }

                let outcome: usize;
                let new_state = state | STATE_RUN_QUEUED;
                asm!("strex {}, {}, [{}]", out(reg) outcome, in(reg) new_state, in(reg) self, options(nostack));
                if outcome == 0 {
                    locked(f);
                    return;
                }
            }
        }
    }

    /// Unmark the task as run-queued. Return whether the task is spawned.
    #[inline(always)]
    pub fn run_dequeue(&self) -> bool {
        compiler_fence(Ordering::Release);

        let r = self.spawned.load(Ordering::Relaxed);
        self.run_queued.store(false, Ordering::Relaxed);
        r
    }

    /// Mark the task as timer-queued. Return whether it can be enqueued.
    #[inline(always)]
    pub fn timer_enqueue(&self) -> TimerEnqueueOperation {
        if self
            .as_u32()
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |state| {
                // If not started, ignore it
                if state & STATE_SPAWNED == 0 {
                    None
                } else {
                    // Mark it as enqueued
                    Some(state | STATE_TIMER_QUEUED)
                }
            })
            .is_ok()
        {
            TimerEnqueueOperation::Enqueue
        } else {
            TimerEnqueueOperation::Ignore
        }
    }

    /// Unmark the task as timer-queued.
    #[inline(always)]
    pub fn timer_dequeue(&self) {
        self.timer_queued.store(false, Ordering::Relaxed);
    }
}
