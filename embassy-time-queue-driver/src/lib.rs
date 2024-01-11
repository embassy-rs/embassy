#![no_std]
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

//! ## Implementing a timer queue
//!
//! - Define a struct `MyTimerQueue`
//! - Implement [`TimerQueue`] for it
//! - Register it as the global timer queue with [`timer_queue_impl`](crate::timer_queue_impl).
//!
//! ## Example
//!
//! ```
//! use core::task::Waker;
//!
//! use embassy_time::Instant;
//! use embassy_time::queue::{TimerQueue};
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
