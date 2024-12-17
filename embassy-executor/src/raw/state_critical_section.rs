use core::cell::Cell;

pub(crate) use critical_section::{with as locked, CriticalSection as Token};
use critical_section::{CriticalSection, Mutex};

use super::timer_queue::TimerEnqueueOperation;

/// Task is spawned (has a future)
pub(crate) const STATE_SPAWNED: u32 = 1 << 0;
/// Task is in the executor run queue
pub(crate) const STATE_RUN_QUEUED: u32 = 1 << 1;
/// Task is in the executor timer queue
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
        critical_section::with(|cs| self.update_with_cs(cs, f))
    }

    fn update_with_cs<R>(&self, cs: CriticalSection<'_>, f: impl FnOnce(&mut u32) -> R) -> R {
        let s = self.state.borrow(cs);
        let mut val = s.get();
        let r = f(&mut val);
        s.set(val);
        r
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

    /// Mark the task as run-queued if it's spawned and isn't already run-queued. Run the given
    /// function if the task was successfully marked.
    #[inline(always)]
    pub fn run_enqueue(&self, f: impl FnOnce(Token)) {
        critical_section::with(|cs| {
            if self.update_with_cs(cs, |s| {
                if (*s & STATE_RUN_QUEUED != 0) || (*s & STATE_SPAWNED == 0) {
                    false
                } else {
                    *s |= STATE_RUN_QUEUED;
                    true
                }
            }) {
                f(cs);
            }
        });
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

    /// Mark the task as timer-queued. Return whether it can be enqueued.
    #[inline(always)]
    pub fn timer_enqueue(&self) -> TimerEnqueueOperation {
        self.update(|s| {
            // FIXME: we need to split SPAWNED into two phases, to prevent enqueueing a task that is
            // just being spawned, because its executor pointer may still be changing.
            if *s & STATE_SPAWNED == STATE_SPAWNED {
                *s |= STATE_TIMER_QUEUED;
                TimerEnqueueOperation::Enqueue
            } else {
                TimerEnqueueOperation::Ignore
            }
        })
    }

    /// Unmark the task as timer-queued.
    #[inline(always)]
    pub fn timer_dequeue(&self) {
        self.update(|s| *s &= !STATE_TIMER_QUEUED);
    }
}
