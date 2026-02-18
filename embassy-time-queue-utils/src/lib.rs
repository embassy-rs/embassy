#![no_std]
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

use core::task::Waker;

pub mod queue_generic;
pub mod queue_integrated;

#[cfg(feature = "_generic-queue")]
type QueueImpl = queue_generic::Queue;
#[cfg(not(feature = "_generic-queue"))]
type QueueImpl = queue_integrated::Queue;

/// The default timer queue, configured by the crate's features.
///
/// If any of the `generic-queue-X` features are enabled, this implements a generic
/// timer queue of capacity X. Otherwise, it implements an integrated timer queue.
#[derive(Debug)]
pub struct Queue {
    queue: QueueImpl,
}

impl Queue {
    /// Creates a new timer queue.
    pub const fn new() -> Self {
        Self {
            queue: QueueImpl::new(),
        }
    }

    /// Schedules a task to run at a specific time, and returns whether any changes were made.
    ///
    /// If this function returns `true`, the caller should find the next expiration time and set
    /// a new alarm for that time.
    pub fn schedule_wake(&mut self, at: u64, waker: &Waker) -> bool {
        self.queue.schedule_wake(at, waker)
    }

    /// Dequeues expired timers and returns the next alarm time.
    pub fn next_expiration(&mut self, now: u64) -> u64 {
        self.queue.next_expiration(now)
    }
}
