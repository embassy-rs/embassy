use core::sync::atomic::{AtomicU32, Ordering};

use super::timer_queue::TimerEnqueueOperation;

/// Task is spawned (has a future)
pub(crate) const STATE_SPAWNED: u32 = 1 << 0;
/// Task is in the executor run queue
pub(crate) const STATE_RUN_QUEUED: u32 = 1 << 1;
/// Task is in the executor timer queue
pub(crate) const STATE_TIMER_QUEUED: u32 = 1 << 2;

pub(crate) struct State {
    state: AtomicU32,
}

impl State {
    pub const fn new() -> State {
        Self {
            state: AtomicU32::new(0),
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

    /// Mark the task as run-queued if it's spawned and isn't already run-queued. Return true on success.
    #[inline(always)]
    pub fn run_enqueue(&self) -> bool {
        self.state
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |state| {
                // If already scheduled, or if not started,
                if (state & STATE_RUN_QUEUED != 0) || (state & STATE_SPAWNED == 0) {
                    None
                } else {
                    // Mark it as scheduled
                    Some(state | STATE_RUN_QUEUED)
                }
            })
            .is_ok()
    }

    /// Unmark the task as run-queued. Return whether the task is spawned.
    #[inline(always)]
    pub fn run_dequeue(&self) -> bool {
        let state = self.state.fetch_and(!STATE_RUN_QUEUED, Ordering::AcqRel);
        state & STATE_SPAWNED != 0
    }

    /// Mark the task as timer-queued. Return whether it can be enqueued.
    #[inline(always)]
    pub fn timer_enqueue(&self) -> TimerEnqueueOperation {
        if self
            .state
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
        self.state.fetch_and(!STATE_TIMER_QUEUED, Ordering::Relaxed);
    }
}
