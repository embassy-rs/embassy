#![no_std]
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

//! ## Implementing a timer queue
//!
//! - Define a struct `MyTimerQueue`
//! - Implement [`TimerQueue`] for it
//! - Register it as the global timer queue with [`timer_queue_impl`](crate::timer_queue_impl).
//!
//! ### Design choices
//!
//! If you are providing an embassy-executor implementation besides a timer queue, you can choose to
//! expose the `embassy-executor/integrated-timers` feature in your implementation. This feature
//! stores timer items in the tasks themselves, so you don't need a fixed-size queue or dynamic
//! memory allocation.
//!
//! To implement the `TimerQueue` trait, you can use the `queue_generic` module from the
//! `embassy-time` crate, or the `raw::timer_queue` module from the `embassy-executor` crate.
//! These modules contain queue implementations which you can wrap and tailor to your needs.
//!
//! ## Example
//!
//! ```
//! use core::task::Waker;
//!
//! use embassy_time::Instant;
//! use embassy_time::queue::TimerQueue;
//!
//! struct MyTimerQueue {}; // not public!
//!
//! impl TimerQueue for MyTimerQueue {
//!     fn schedule_wake(&'static self, at: u64, waker: &Waker) {
//!         todo!()
//!     }
//! }
//!
//! embassy_time_queue_driver::timer_queue_impl!(static QUEUE: MyTimerQueue = MyTimerQueue{});
//! ```

pub mod queue_generic;

use core::task::Waker;

/// Timer queue
pub trait TimerQueue {
    /// Schedules a waker in the queue to be awoken at moment `at`.
    /// If this moment is in the past, the waker might be awoken immediately.
    fn schedule_wake(&'static self, at: u64, waker: &Waker);
}

extern "Rust" {
    fn _embassy_time_schedule_wake(at: u64, waker: &Waker);
}

/// Schedule the given waker to be woken at `at`.
pub fn schedule_wake(at: u64, waker: &Waker) {
    unsafe { _embassy_time_schedule_wake(at, waker) }
}

/// Set the TimerQueue implementation.
///
/// See the module documentation for an example.
#[macro_export]
macro_rules! timer_queue_impl {
    (static $name:ident: $t: ty = $val:expr) => {
        static $name: $t = $val;

        #[no_mangle]
        fn _embassy_time_schedule_wake(at: u64, waker: &core::task::Waker) {
            <$t as $crate::TimerQueue>::schedule_wake(&$name, at, waker);
        }
    };
}
