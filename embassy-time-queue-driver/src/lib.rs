#![no_std]
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

//! This crate is an implementation detail of `embassy-time-driver`.
//!
//! As a HAL user, you should only depend on this crate if your application does not use
//! `embassy-executor` and your HAL does not configure a generic queue by itself.
//!
//! As a HAL implementer, you need to depend on this crate if you want to implement a time driver,
//! but how you should do so is documented in [`embassy_time_driver`].

use core::task::Waker;

#[cfg(feature = "_generic-queue")]
pub mod queue_generic;
#[cfg(not(feature = "_generic-queue"))]
pub mod queue_integrated;

#[cfg(feature = "_generic-queue")]
pub use queue_generic::Queue;
#[cfg(not(feature = "_generic-queue"))]
pub use queue_integrated::Queue;

extern "Rust" {
    fn _embassy_time_schedule_wake(at: u64, waker: &Waker);
}

/// Schedule the given waker to be woken at `at`.
pub fn schedule_wake(at: u64, waker: &Waker) {
    // This function is not implemented in embassy-time-driver because it needs access to executor
    // internals. The function updates task state, then delegates to the implementation provided
    // by the time driver.
    #[cfg(not(feature = "_generic-queue"))]
    {
        use embassy_executor::raw::task_from_waker;
        use embassy_executor::raw::timer_queue::TimerEnqueueOperation;
        // The very first thing we must do, before we even access the timer queue, is to
        // mark the task a TIMER_QUEUED. This ensures that the task that is being scheduled
        // can not be respawn while we are accessing the timer queue.
        let task = task_from_waker(waker);
        if unsafe { task.timer_enqueue() } == TimerEnqueueOperation::Ignore {
            // We are not allowed to enqueue the task in the timer queue. This is because the
            // task is not spawned, and so it makes no sense to schedule it.
            return;
        }
    }
    unsafe { _embassy_time_schedule_wake(at, waker) }
}
