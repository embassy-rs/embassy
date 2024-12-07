use core::cell::Cell;

use critical_section::{CriticalSection, Mutex};

/// Task is claimed (it is being spawned)
pub(crate) const STATE_CLAIMED: u32 = 1 << 0;
/// Task is spawned (has a future)
pub(crate) const STATE_SPAWNED: u32 = 1 << 1;
/// Task is in the executor run queue
pub(crate) const STATE_RUN_QUEUED: u32 = 1 << 2;
/// Task is in the executor timer queue
#[cfg(feature = "integrated-timers")]
pub(crate) const STATE_TIMER_QUEUED: u32 = 1 << 3;

pub(crate) struct State {
    state: Mutex<Cell<u32>>,
}

impl State {
    pub const fn new() -> State {
        Self {
            state: Mutex::new(Cell::new(0)),
        }
    }

    fn update<R>(&self, f: impl FnOnce(&mut u32) -> R) -> R {
        critical_section::with(|cs| self.update_with_cs(cs, f))
    }

    fn update_with_cs<R>(&self, cs: CriticalSection<'_>, f: impl FnOnce(&mut u32) -> R) -> R {
        let s = self.state.borrow(cs);
        let mut val = s.get();
        let r = f(&mut val);
        s.set(val);
        r
    }

    /// If task is idle, mark it as claimed and return true.
    #[inline(always)]
    pub fn claim(&self) -> bool {
        self.update(|s| {
            if *s == 0 {
                *s = STATE_CLAIMED;
                true
            } else {
                false
            }
        })
    }

    /// Mark a claimed task ready to run.
    ///
    /// # Safety
    ///
    /// The task must be claimed, its executor must be configured. This function must
    /// not be called when the task is already spawned.
    #[inline(always)]
    pub unsafe fn mark_spawned(&self) {
        self.update(|s| *s = STATE_SPAWNED | STATE_RUN_QUEUED);
    }

    /// Mark a spawned task `CLAIMED` to prevent enqueueing it again in a run queue.
    #[inline(always)]
    pub fn prepare_despawn(&self) {
        self.update(|s| *s |= STATE_CLAIMED);
    }

    /// Unmark the task as spawned.
    #[inline(always)]
    pub fn despawn(&self) {
        self.update(|s| *s &= !(STATE_SPAWNED | STATE_CLAIMED));
    }

    /// Mark the task as run-queued if it's spawned and isn't already run-queued. Return true on success.
    #[inline(always)]
    pub fn run_enqueue(&self) -> bool {
        self.update(|s| {
            // If CLAIMED is set, the task is being spawned. We don't want to pend, because we
            // may end up adding the task to the wrong run queue.
            let ok = *s & (STATE_RUN_QUEUED | STATE_CLAIMED) == 0;
            *s |= STATE_RUN_QUEUED;
            ok
        })
    }

    /// Unmark the task as run-queued.
    #[inline(always)]
    pub fn run_dequeue(&self, cs: CriticalSection<'_>) {
        self.update_with_cs(cs, |s| *s &= !STATE_RUN_QUEUED);
    }

    /// Mark the task as timer-queued. Return whether it was newly queued (i.e. not queued before)
    #[cfg(feature = "integrated-timers")]
    #[inline(always)]
    pub fn timer_enqueue(&self) -> bool {
        self.update(|s| {
            let ok = *s & STATE_TIMER_QUEUED == 0;
            *s |= STATE_TIMER_QUEUED;
            ok
        })
    }

    /// Unmark the task as timer-queued.
    #[cfg(feature = "integrated-timers")]
    #[inline(always)]
    pub fn timer_dequeue(&self) {
        self.update(|s| *s &= !STATE_TIMER_QUEUED);
    }
}
