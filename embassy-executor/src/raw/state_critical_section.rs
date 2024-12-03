use core::cell::Cell;

use critical_section::Mutex;

/// Task is spawned (has a future)
pub(crate) const STATE_SPAWNED: u32 = 1 << 0;
/// Task is in the executor run queue
pub(crate) const STATE_RUN_QUEUED: u32 = 1 << 1;
/// Task is in the executor timer queue
#[cfg(feature = "integrated-timers")]
pub(crate) const STATE_TIMER_QUEUED: u32 = 1 << 2;

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
        critical_section::with(|cs| {
            let s = self.state.borrow(cs);
            let mut val = s.get();
            let r = f(&mut val);
            s.set(val);
            r
        })
    }

    /// If task is idle, mark it as spawned + run_queued and return true.
    #[inline(always)]
    pub fn spawn(&self) -> bool {
        self.update(|s| {
            if *s == 0 {
                *s = STATE_SPAWNED | STATE_RUN_QUEUED;
                true
            } else {
                false
            }
        })
    }

    /// Unmark the task as spawned.
    #[inline(always)]
    pub fn despawn(&self) {
        self.update(|s| *s &= !STATE_SPAWNED);
    }

    /// Mark the task as run-queued if it's spawned and isn't already run-queued. Return true on success.
    #[inline(always)]
    pub fn run_enqueue(&self) -> bool {
        self.update(|s| {
            if (*s & STATE_RUN_QUEUED != 0) || (*s & STATE_SPAWNED == 0) {
                false
            } else {
                *s |= STATE_RUN_QUEUED;
                true
            }
        })
    }

    /// Unmark the task as run-queued. Return whether the task is spawned.
    #[inline(always)]
    pub fn run_dequeue(&self) -> bool {
        self.update(|s| {
            let ok = *s & STATE_SPAWNED != 0;
            *s &= !STATE_RUN_QUEUED;
            ok
        })
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
