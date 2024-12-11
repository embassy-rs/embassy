#![no_std]
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

//! ## Implementing a timer queue
//!
//! - Define a struct `MyTimerQueue`
//! - Implement [`TimerQueue`] for it
//! - Register it as the global timer queue with [`timer_queue_impl`].
//! - Ensure that you process the timer queue when `schedule_wake` is due. This usually involves
//!   waking expired tasks, finding the next expiration time and setting an alarm.
//!
//! If a single global timer queue is sufficient for you, you can use the
//! [`GlobalTimerQueue`] type, which is a wrapper around a global timer queue
//! protected by a critical section.
//!
//! ```
//! use embassy_time_queue_driver::GlobalTimerQueue;
//! embassy_time_queue_driver::timer_queue_impl!(
//!     static TIMER_QUEUE_DRIVER: GlobalTimerQueue
//!         = GlobalTimerQueue::new(|next_expiration| todo!("Set an alarm"))
//! );
//! ```
//!
//! You can also use the `queue_generic` or the `queue_integrated` modules to implement your own
//! timer queue. These modules contain queue implementations which you can wrap and tailor to
//! your needs.
//!
//! If you are providing an embassy-executor implementation besides a timer queue, you can choose to
//! expose the `integrated-timers` feature in your implementation. This feature stores timer items
//! in the tasks themselves, so you don't need a fixed-size queue or dynamic memory allocation.
//!
//! ## Example
//!
//! ```
//! use core::task::Waker;
//!
//! use embassy_time::Instant;
//! use embassy_time::queue::TimerQueue;
//!
//! struct MyTimerQueue{}; // not public!
//!
//! impl TimerQueue for MyTimerQueue {
//!     fn schedule_wake(&'static self, at: u64, waker: &Waker) {
//!         todo!()
//!     }
//! }
//!
//! embassy_time_queue_driver::timer_queue_impl!(static QUEUE: MyTimerQueue = MyTimerQueue{});
//! ```

use core::task::Waker;

#[cfg(feature = "_generic-queue")]
pub mod queue_generic;
#[cfg(not(feature = "_generic-queue"))]
pub mod queue_integrated;

#[cfg(not(feature = "_generic-queue"))]
pub use queue_integrated::Queue;

#[cfg(feature = "_generic-queue")]
pub use queue_generic::Queue;

extern "Rust" {
    fn _embassy_time_schedule_wake(at: u64, waker: &Waker);
}

/// Schedule the given waker to be woken at `at`.
pub fn schedule_wake(at: u64, waker: &Waker) {
    #[cfg(not(feature = "_generic-queue"))]
    {
        // The very first thing we must do, before we even access the timer queue, is to
        // mark the task a TIMER_QUEUED. This ensures that the task that is being scheduled
        // can not be respawn while we are accessing the timer queue.
        let task = embassy_executor::raw::task_from_waker(waker);
        if unsafe { task.timer_enqueue() } == embassy_executor::raw::timer_queue::TimerEnqueueOperation::Ignore {
            // We are not allowed to enqueue the task in the timer queue. This is because the
            // task is not spawned, and so it makes no sense to schedule it.
            return;
        }
    }
    unsafe { _embassy_time_schedule_wake(at, waker) }
}
