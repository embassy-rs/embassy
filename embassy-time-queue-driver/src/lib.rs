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

#[cfg(feature = "_generic-queue")]
pub mod queue_generic;
#[cfg(not(feature = "_generic-queue"))]
pub mod queue_integrated;

#[cfg(not(feature = "_generic-queue"))]
pub use queue_integrated::Queue;

#[cfg(feature = "_generic-queue")]
pub use queue_generic::Queue;
