//! Generic timer queue implementations.
//!
//! Time queue drivers may use this to simplify their implementation.

use core::cmp::{min, Ordering};
use core::task::Waker;

use heapless::Vec;

#[derive(Debug)]
struct Timer {
    at: u64,
    waker: Waker,
}

impl PartialEq for Timer {
    fn eq(&self, other: &Self) -> bool {
        self.at == other.at
    }
}

impl Eq for Timer {}

impl PartialOrd for Timer {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.at.partial_cmp(&other.at)
    }
}

impl Ord for Timer {
    fn cmp(&self, other: &Self) -> Ordering {
        self.at.cmp(&other.at)
    }
}

/// A timer queue with a pre-determined capacity.
pub struct ConstGenericQueue<const QUEUE_SIZE: usize> {
    queue: Vec<Timer, QUEUE_SIZE>,
}

impl<const QUEUE_SIZE: usize> ConstGenericQueue<QUEUE_SIZE> {
    /// Creates a new timer queue.
    pub const fn new() -> Self {
        Self { queue: Vec::new() }
    }

    /// Schedules a task to run at a specific time, and returns whether any changes were made.
    ///
    /// If this function returns `true`, the called should find the next expiration time and set
    /// a new alarm for that time.
    pub fn schedule_wake(&mut self, at: u64, waker: &Waker) -> bool {
        self.queue
            .iter_mut()
            .find(|timer| timer.waker.will_wake(waker))
            .map(|timer| {
                if timer.at > at {
                    timer.at = at;
                    true
                } else {
                    false
                }
            })
            .unwrap_or_else(|| {
                let mut timer = Timer {
                    waker: waker.clone(),
                    at,
                };

                loop {
                    match self.queue.push(timer) {
                        Ok(()) => break,
                        Err(e) => timer = e,
                    }

                    self.queue.pop().unwrap().waker.wake();
                }

                true
            })
    }

    /// Dequeues expired timers and returns the next alarm time.
    pub fn next_expiration(&mut self, now: u64) -> u64 {
        let mut next_alarm = u64::MAX;

        let mut i = 0;
        while i < self.queue.len() {
            let timer = &self.queue[i];
            if timer.at <= now {
                let timer = self.queue.swap_remove(i);
                timer.waker.wake();
            } else {
                next_alarm = min(next_alarm, timer.at);
                i += 1;
            }
        }

        next_alarm
    }
}

#[cfg(feature = "generic-queue-8")]
const QUEUE_SIZE: usize = 8;
#[cfg(feature = "generic-queue-16")]
const QUEUE_SIZE: usize = 16;
#[cfg(feature = "generic-queue-32")]
const QUEUE_SIZE: usize = 32;
#[cfg(feature = "generic-queue-64")]
const QUEUE_SIZE: usize = 64;
#[cfg(feature = "generic-queue-128")]
const QUEUE_SIZE: usize = 128;
#[cfg(not(any(
    feature = "generic-queue-8",
    feature = "generic-queue-16",
    feature = "generic-queue-32",
    feature = "generic-queue-64",
    feature = "generic-queue-128"
)))]
const QUEUE_SIZE: usize = 64;

/// A timer queue with a pre-determined capacity.
pub struct Queue {
    queue: ConstGenericQueue<QUEUE_SIZE>,
}

impl Queue {
    /// Creates a new timer queue.
    pub const fn new() -> Self {
        Self {
            queue: ConstGenericQueue::new(),
        }
    }

    /// Schedules a task to run at a specific time, and returns whether any changes were made.
    ///
    /// If this function returns `true`, the called should find the next expiration time and set
    /// a new alarm for that time.
    pub fn schedule_wake(&mut self, at: u64, waker: &Waker) -> bool {
        self.queue.schedule_wake(at, waker)
    }

    /// Dequeues expired timers and returns the next alarm time.
    pub fn next_expiration(&mut self, now: u64) -> u64 {
        self.queue.next_expiration(now)
    }
}
