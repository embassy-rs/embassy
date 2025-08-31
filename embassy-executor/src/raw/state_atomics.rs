// Prefer pointer-width atomic operations, as narrower ones may be slower.
#[cfg(all(target_pointer_width = "32", target_has_atomic = "32"))]
type AtomicState = core::sync::atomic::AtomicU32;
#[cfg(not(all(target_pointer_width = "32", target_has_atomic = "32")))]
type AtomicState = core::sync::atomic::AtomicU8;

#[cfg(all(target_pointer_width = "32", target_has_atomic = "32"))]
type StateBits = u32;
#[cfg(not(all(target_pointer_width = "32", target_has_atomic = "32")))]
type StateBits = u8;

use core::sync::atomic::Ordering;

#[derive(Clone, Copy)]
pub(crate) struct Token(());

/// Creates a token and passes it to the closure.
///
/// This is a no-op replacement for `CriticalSection::with` because we don't need any locking.
pub(crate) fn locked<R>(f: impl FnOnce(Token) -> R) -> R {
    f(Token(()))
}

/// Task is spawned (has a future)
pub(crate) const STATE_SPAWNED: StateBits = 1 << 0;
/// Task is in the executor run queue
pub(crate) const STATE_RUN_QUEUED: StateBits = 1 << 1;

pub(crate) struct State {
    state: AtomicState,
}

impl State {
    pub const fn new() -> State {
        Self {
            state: AtomicState::new(0),
        }
    }

    /// If task is idle, mark it as spawned + run_queued and return true.
    #[inline(always)]
    pub fn spawn(&self) -> bool {
        self.state
            .compare_exchange(0, STATE_SPAWNED | STATE_RUN_QUEUED, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    }

    /// Unmark the task as spawned.
    #[inline(always)]
    pub fn despawn(&self) {
        self.state.fetch_and(!STATE_SPAWNED, Ordering::AcqRel);
    }

    /// Mark the task as run-queued if it's spawned and isn't already run-queued. Run the given
    /// function if the task was successfully marked.
    #[inline(always)]
    pub fn run_enqueue(&self, f: impl FnOnce(Token)) {
        let prev = self.state.fetch_or(STATE_RUN_QUEUED, Ordering::AcqRel);
        if prev & STATE_RUN_QUEUED == 0 {
            locked(f);
        }
    }

    /// Unmark the task as run-queued. Return whether the task is spawned.
    #[inline(always)]
    pub fn run_dequeue(&self) {
        self.state.fetch_and(!STATE_RUN_QUEUED, Ordering::AcqRel);
    }
}
