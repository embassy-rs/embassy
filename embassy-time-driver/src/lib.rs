#![no_std]
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

//! ## Implementing a driver
//!
//! - Define a struct `MyDriver`
//! - Implement [`Driver`] for it
//! - Register it as the global driver with [`time_driver_impl`](crate::time_driver_impl).
//!
//! If your driver has a single set tick rate, enable the corresponding [`tick-hz-*`](crate#tick-rate) feature,
//! which will prevent users from needing to configure it themselves (or selecting an incorrect configuration).
//!
//! If your driver supports a small number of set tick rates, expose your own cargo features and have each one
//! enable the corresponding `embassy-time-driver/tick-*`.
//!
//! Otherwise, don’t enable any `tick-hz-*` feature to let the user configure the tick rate themselves by
//! enabling a feature on `embassy-time`.
//!
//! ### Example
//!
//! ```
//! use core::task::Waker;
//!
//! use embassy_time_driver::Driver;
//!
//! struct MyDriver{} // not public!
//!
//! impl Driver for MyDriver {
//!     fn now(&self) -> u64 {
//!         todo!()
//!     }
//!
//!     fn schedule_wake(&self, at: u64, waker: &Waker) {
//!         todo!()
//!     }
//! }
//!
//! embassy_time_driver::time_driver_impl!(static DRIVER: MyDriver = MyDriver{});
//! ```
//!
//! ## Implementing the timer queue
//!
//! The simplest (but suboptimal) way to implement a timer queue is to define a single queue in the
//! time driver. Declare a field protected by an appropriate mutex (e.g. `critical_section::Mutex`).
//!
//! Then, you'll need to adapt the `schedule_wake` method to use this queue.
//!
//! Note that if you are using multiple queues, you will need to ensure that a single timer
//! queue item is only ever enqueued into a single queue at a time.
//!
//! ```
//! use core::cell::RefCell;
//! use core::task::Waker;
//!
//! use critical_section::{CriticalSection, Mutex};
//! use embassy_time_queue_utils::Queue;
//! use embassy_time_driver::Driver;
//!
//! struct MyDriver {
//!     queue: Mutex<RefCell<Queue>>,
//! }
//!
//! impl MyDriver {
//!    fn set_alarm(&self, cs: &CriticalSection, at: u64) -> bool {
//!        todo!()
//!    }
//! }
//!
//! impl Driver for MyDriver {
//!     fn now(&self) -> u64 { todo!() }
//!
//!     fn schedule_wake(&self, at: u64, waker: &Waker) {
//!         critical_section::with(|cs| {
//!             let mut queue = self.queue.borrow(cs).borrow_mut();
//!             if queue.schedule_wake(at, waker) {
//!                 let mut next = queue.next_expiration(self.now());
//!                 while !self.set_alarm(&cs, next) {
//!                     next = queue.next_expiration(self.now());
//!                 }
//!             }
//!         });
//!     }
//! }
//! ```
//!
//! # Linkage details
//!
//! Instead of the usual "trait + generic params" approach, calls from embassy to the driver are done via `extern` functions.
//!
//! `embassy` internally defines the driver function as `extern "Rust" { fn _embassy_time_now() -> u64; }` and calls it.
//! The driver crate defines the function as `#[no_mangle] fn _embassy_time_now() -> u64`. The linker will resolve the
//! calls from the `embassy` crate to call into the driver crate.
//!
//! If there is none or multiple drivers in the crate tree, linking will fail.
//!
//! This method has a few key advantages for something as foundational as timekeeping:
//!
//! - The time driver is available everywhere easily, without having to thread the implementation
//!   through generic parameters. This is especially helpful for libraries.
//! - It means comparing `Instant`s will always make sense: if there were multiple drivers
//!   active, one could compare an `Instant` from driver A to an `Instant` from driver B, which
//!   would yield incorrect results.

//! ## Feature flags
#![doc = document_features::document_features!(feature_label = r#"<span class="stab portability"><code>{feature}</code></span>"#)]

use core::task::Waker;

mod tick;

/// Ticks per second of the global timebase.
///
/// This value is specified by the [`tick-*` Cargo features](crate#tick-rate)
pub const TICK_HZ: u64 = tick::TICK_HZ;

/// Time driver
pub trait Driver: Send + Sync + 'static {
    /// Return the current timestamp in ticks.
    ///
    /// Implementations MUST ensure that:
    /// - This is guaranteed to be monotonic, i.e. a call to now() will always return
    ///   a greater or equal value than earlier calls. Time can't "roll backwards".
    /// - It "never" overflows. It must not overflow in a sufficiently long time frame, say
    ///   in 10_000 years (Human civilization is likely to already have self-destructed
    ///   10_000 years from now.). This means if your hardware only has 16bit/32bit timers
    ///   you MUST extend them to 64-bit, for example by counting overflows in software,
    ///   or chaining multiple timers together.
    fn now(&self) -> u64;

    /// Schedules a waker to be awoken at moment `at`.
    /// If this moment is in the past, the waker might be awoken immediately.
    fn schedule_wake(&self, at: u64, waker: &Waker);
}

extern "Rust" {
    fn _embassy_time_now() -> u64;
    fn _embassy_time_schedule_wake(at: u64, waker: &Waker);
}

/// See [`Driver::now`]
#[inline]
pub fn now() -> u64 {
    unsafe { _embassy_time_now() }
}

/// Schedule the given waker to be woken at `at`.
#[inline]
pub fn schedule_wake(at: u64, waker: &Waker) {
    unsafe { _embassy_time_schedule_wake(at, waker) }
}

/// Set the time Driver implementation.
///
/// See the module documentation for an example.
#[macro_export]
macro_rules! time_driver_impl {
    (static $name:ident: $t: ty = $val:expr) => {
        static $name: $t = $val;

        #[no_mangle]
        #[inline]
        fn _embassy_time_now() -> u64 {
            <$t as $crate::Driver>::now(&$name)
        }

        #[no_mangle]
        #[inline]
        fn _embassy_time_schedule_wake(at: u64, waker: &core::task::Waker) {
            <$t as $crate::Driver>::schedule_wake(&$name, at, waker);
        }
    };
}
