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

#[cfg(not(feature = "integrated-timers"))]
pub mod queue_generic;
#[cfg(feature = "integrated-timers")]
pub mod queue_integrated;

use core::cell::RefCell;
use core::task::Waker;

use critical_section::Mutex;

/// Timer queue
pub trait TimerQueue {
    /// Schedules a waker in the queue to be awoken at moment `at`.
    ///
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

#[cfg(feature = "integrated-timers")]
type InnerQueue = queue_integrated::TimerQueue;

#[cfg(not(feature = "integrated-timers"))]
type InnerQueue = queue_generic::Queue;

/// A timer queue implementation that can be used as a global timer queue.
///
/// This implementation is not thread-safe, and should be protected by a mutex of some sort.
pub struct GenericTimerQueue<F: Fn(u64) -> bool> {
    queue: InnerQueue,
    set_alarm: F,
}

impl<F: Fn(u64) -> bool> GenericTimerQueue<F> {
    /// Creates a new timer queue.
    ///
    /// `set_alarm` is a function that should set the next alarm time. The function should
    /// return `true` if the alarm was set, and `false` if the alarm was in the past.
    pub const fn new(set_alarm: F) -> Self {
        Self {
            queue: InnerQueue::new(),
            set_alarm,
        }
    }

    /// Schedules a task to run at a specific time, and returns whether any changes were made.
    pub fn schedule_wake(&mut self, at: u64, waker: &core::task::Waker) {
        #[cfg(feature = "integrated-timers")]
        let waker = embassy_executor::raw::task_from_waker(waker);

        if self.queue.schedule_wake(at, waker) {
            self.dispatch()
        }
    }

    /// Dequeues expired timers and returns the next alarm time.
    pub fn next_expiration(&mut self, now: u64) -> u64 {
        self.queue.next_expiration(now)
    }

    /// Handle the alarm.
    ///
    /// Call this function when the next alarm is due.
    pub fn dispatch(&mut self) {
        let mut next_expiration = self.next_expiration(embassy_time_driver::now());

        while !(self.set_alarm)(next_expiration) {
            // next_expiration is in the past, dequeue and find a new expiration
            next_expiration = self.next_expiration(next_expiration);
        }
    }
}

/// A [`GenericTimerQueue`] protected by a critical section. Directly useable as a [`TimerQueue`].
pub struct GlobalTimerQueue {
    inner: Mutex<RefCell<GenericTimerQueue<fn(u64) -> bool>>>,
}

impl GlobalTimerQueue {
    /// Creates a new timer queue.
    ///
    /// `set_alarm` is a function that should set the next alarm time. The function should
    /// return `true` if the alarm was set, and `false` if the alarm was in the past.
    pub const fn new(set_alarm: fn(u64) -> bool) -> Self {
        Self {
            inner: Mutex::new(RefCell::new(GenericTimerQueue::new(set_alarm))),
        }
    }

    /// Schedules a task to run at a specific time, and returns whether any changes were made.
    pub fn schedule_wake(&self, at: u64, waker: &core::task::Waker) {
        critical_section::with(|cs| {
            let mut inner = self.inner.borrow_ref_mut(cs);
            inner.schedule_wake(at, waker);
        });
    }

    /// Dequeues expired timers and returns the next alarm time.
    pub fn next_expiration(&self, now: u64) -> u64 {
        critical_section::with(|cs| {
            let mut inner = self.inner.borrow_ref_mut(cs);
            inner.next_expiration(now)
        })
    }

    /// Handle the alarm.
    ///
    /// Call this function when the next alarm is due.
    pub fn dispatch(&self) {
        critical_section::with(|cs| {
            let mut inner = self.inner.borrow_ref_mut(cs);
            inner.dispatch()
        })
    }
}

impl TimerQueue for GlobalTimerQueue {
    fn schedule_wake(&'static self, at: u64, waker: &Waker) {
        GlobalTimerQueue::schedule_wake(self, at, waker)
    }
}
