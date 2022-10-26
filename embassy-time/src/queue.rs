//! Timer queue implementation
//!
//! This module defines the interface a timer queue needs to implement to power the `embassy_time` module.
//!
//! # Implementing a timer queue
//!
//! - Define a struct `MyTimerQueue`
//! - Implement [`TimerQueue`] for it
//! - Register it as the global timer queue with [`timer_queue_impl`](crate::timer_queue_impl).
//!
//! # Linkage details
//!
//! Check the documentation of the [`driver`](crate::driver) module for more information.
//!
//! Similarly to driver, if there is none or multiple timer queues in the crate tree, linking will fail.
//!
//! # Example
//!
//! ```
//! use core::task::Waker;
//!
//! use embassy_time::Instant;
//! use embassy_time::queue::{TimerQueue};
//!
//! struct MyTimerQueue{}; // not public!
//! embassy_time::timer_queue_impl!(static QUEUE: MyTimerQueue = MyTimerQueue{});
//!
//! impl TimerQueue for MyTimerQueue {
//!     fn schedule_wake(&'static self, at: Instant, waker: &Waker) {
//!         todo!()
//!     }
//! }
//! ```
use core::task::Waker;

use crate::Instant;

/// Timer queue
pub trait TimerQueue {
    /// Schedules a waker in the queue to be awoken at moment `at`.
    /// If this moment is in the past, the waker might be awoken immediately.
    fn schedule_wake(&'static self, at: Instant, waker: &Waker);
}

/// Set the TimerQueue implementation.
///
/// See the module documentation for an example.
#[macro_export]
macro_rules! timer_queue_impl {
    (static $name:ident: $t: ty = $val:expr) => {
        static $name: $t = $val;

        #[no_mangle]
        fn _embassy_time_schedule_wake(at: $crate::Instant, waker: &core::task::Waker) {
            <$t as $crate::queue::TimerQueue>::schedule_wake(&$name, at, waker);
        }
    };
}
