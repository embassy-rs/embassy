//! Generic timer queue implementation
//!
//! This module provides a timer queue that works with any executor.
//!
//! In terms of performance, this queue will likely be less efficient in comparison to executor-native queues,
//! like the one provided with e.g. the `embassy-executor` crate.
//!
//! # Enabling the queue
//! - Enable the Cargo feature `generic-queue`. This will automatically instantiate the queue.
//!
//! # Initializing the queue
//! - Call ```unsafe { embassy_time::queue::initialize(); }``` early on in your program, before any of your futures that utilize `embassy-time` are polled.
//!
//! # Customizing the queue
//! - It is possible to customize two aspects of the queue:
//!   - Queue size:
//!     By default, the queue can hold up to 128 timer schedules and their corresponding wakers. While it will not crash if more timer schedules are added,
//!     the performance will degrade, as one of the already added wakers will be awoken, thus making room for the new timer schedule and its waker.
//!   - The mutex (i.e. the [`RawMutex`](embassy_sync::blocking_mutex::raw::RawMutex) implementation) utilized by the queue:
//!     By default, the utilized [`RawMutex`](embassy_sync::blocking_mutex::raw::RawMutex) implementation is [`CriticalSectionRawMutex`](embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex)
//!     which is provided by the `critical-section` crate. This should work just fine, except in a few niche cases like running on
//!     top of an RTOS which provides a [`Driver`](crate::driver::Driver) implementation that will call-back directly from an ISR. As the
//!     `critical-section` implementation for RTOS-es will likely provide an RTOS mutex which cannot be locked from an ISR, user needs to instead
//!     configure the queue with a "disable-all-interrupts" style of mutex.
//! - To customize any of these queue aspects, don't enable the `generic-queue` Cargo feature and instead instantiate the queue with the [`generic_queue`](crate::generic_queue)
//!   macro, as per the example below.
//!
//!
//! # Example
//!
//! ```ignore
//! use embassy_time::queue::Queue;
//!
//! // You only need to invoke this macro in case you need to customize the queue.
//! //
//! // Otherwise, just depend on the `embassy-time` crate with feature `generic-queue` enabled,
//! // and the queue instantiation will be done for you behind the scenes.
//! embassy_time::generic_queue!(static QUEUE: Queue<200, MyCustomRawMutex> = Queue::new());
//!
//! fn main() {
//!     unsafe {
//!         embassy_time::queue::initialize();
//!     }
//! }
//! ```
use core::cell::{Cell, RefCell};
use core::cmp::Ordering;
use core::task::Waker;

use atomic_polyfill::{AtomicU64, Ordering as AtomicOrdering};
use embassy_sync::blocking_mutex::raw::{CriticalSectionRawMutex, RawMutex};
use embassy_sync::blocking_mutex::Mutex;
use heapless::sorted_linked_list::{LinkedIndexU8, Min, SortedLinkedList};

use crate::driver::{allocate_alarm, set_alarm, set_alarm_callback, AlarmHandle};
use crate::Instant;

#[derive(Debug)]
struct Timer {
    at: Instant,
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

struct InnerQueue<const N: usize> {
    queue: SortedLinkedList<Timer, LinkedIndexU8, Min, N>,
    alarm_at: Instant,
}

impl<const N: usize> InnerQueue<N> {
    const fn new() -> Self {
        Self {
            queue: SortedLinkedList::new_u8(),
            alarm_at: Instant::MAX,
        }
    }

    fn schedule(&mut self, at: Instant, waker: &Waker, alarm_schedule: &AtomicU64) {
        self.queue
            .find_mut(|timer| timer.waker.will_wake(waker))
            .map(|mut timer| {
                timer.waker = waker.clone();
                timer.at = at;

                timer.finish();
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
            });

        // Don't wait for the alarm callback to trigger and directly
        // dispatch all timers that are already due
        //
        // Then update the alarm if necessary
        self.dispatch(alarm_schedule);
    }

    fn dispatch(&mut self, alarm_schedule: &AtomicU64) {
        let now = Instant::now();

        while self.queue.peek().filter(|timer| timer.at <= now).is_some() {
            self.queue.pop().unwrap().waker.wake();
        }

        self.update_alarm(alarm_schedule);
    }

    fn update_alarm(&mut self, alarm_schedule: &AtomicU64) {
        if let Some(timer) = self.queue.peek() {
            let new_at = timer.at;

            if self.alarm_at != new_at {
                self.alarm_at = new_at;
                alarm_schedule.store(new_at.as_ticks(), AtomicOrdering::SeqCst);
            }
        } else {
            self.alarm_at = Instant::MAX;
            alarm_schedule.store(Instant::MAX.as_ticks(), AtomicOrdering::SeqCst);
        }
    }

    fn handle_alarm(&mut self, alarm_schedule: &AtomicU64) {
        self.alarm_at = Instant::MAX;

        self.dispatch(alarm_schedule);
    }
}

/// The generic queue implementation
pub struct Queue<const N: usize = 128, R: RawMutex = CriticalSectionRawMutex> {
    inner: Mutex<R, RefCell<InnerQueue<N>>>,
    alarm: Cell<Option<AlarmHandle>>,
    alarm_schedule: AtomicU64,
}

impl<const N: usize, R: RawMutex + 'static> Queue<N, R> {
    /// Create a Queue
    pub const fn new() -> Self {
        Self {
            inner: Mutex::new(RefCell::new(InnerQueue::<N>::new())),
            alarm: Cell::new(None),
            alarm_schedule: AtomicU64::new(u64::MAX),
        }
    }

    /// Initialize the queue
    ///
    /// This method is called from [`initialize`](crate::queue::initialize), so you are not expected to call it directly.
    /// Call [`initialize`](crate::queue::initialize) instead.
    ///
    /// # Safety
    /// It is UB call this function more than once, or to call it after any of your
    /// futures that use `embassy-time` are polled already.
    pub unsafe fn initialize(&'static self) {
        if self.alarm.get().is_some() {
            panic!("Queue is already initialized");
        }

        let handle = allocate_alarm().unwrap();
        self.alarm.set(Some(handle));

        set_alarm_callback(handle, Self::handle_alarm_callback, self as *const _ as _);
    }

    /// Schedule a new waker to be awoken at moment `at`
    ///
    /// This method is called internally by [`embassy-time`](crate), so you are not expected to call it directly.
    pub fn schedule(&'static self, at: Instant, waker: &Waker) {
        self.check_initialized();

        self.inner
            .lock(|inner| inner.borrow_mut().schedule(at, waker, &self.alarm_schedule));

        self.update_alarm();
    }

    fn check_initialized(&self) {
        if self.alarm.get().is_none() {
            panic!("Queue is not initialized yet");
        }
    }

    fn update_alarm(&self) {
        // Need to set the alarm when we are *not* holding the mutex on the inner queue
        // because mutexes are not re-entrant, which is a problem because `set_alarm` might immediately
        // call us back if the timestamp is in the past.
        let alarm_at = self.alarm_schedule.swap(u64::MAX, AtomicOrdering::SeqCst);

        if alarm_at < u64::MAX {
            set_alarm(self.alarm.get().unwrap(), alarm_at);
        }
    }

    fn handle_alarm(&self) {
        self.check_initialized();
        self.inner
            .lock(|inner| inner.borrow_mut().handle_alarm(&self.alarm_schedule));

        self.update_alarm();
    }

    fn handle_alarm_callback(ctx: *mut ()) {
        unsafe { (ctx as *const Self).as_ref().unwrap() }.handle_alarm();
    }
}

unsafe impl<const N: usize, R: RawMutex + 'static> Send for Queue<N, R> {}
unsafe impl<const N: usize, R: RawMutex + 'static> Sync for Queue<N, R> {}

/// Initialize the queue
///
/// Call this function early on in your program, before any of your futures that utilize `embassy-time` are polled.
///
/// # Safety
/// It is UB call this function more than once, or to call it after any of your
/// futures that use `embassy-time` are polled already.
pub unsafe fn initialize() {
    extern "Rust" {
        fn _embassy_time_generic_queue_initialize();
    }

    _embassy_time_generic_queue_initialize();
}

/// Instantiates a global, generic (as in executor-agnostic) timer queue.
///
/// Unless you plan to customize the queue (size or mutex), prefer
/// instantiating the queue via the `generic-queue` feature.
///
/// See the module documentation for an example.
#[macro_export]
macro_rules! generic_queue {
    (static $name:ident: $t: ty = $val:expr) => {
        static $name: $t = $val;

        #[no_mangle]
        fn _embassy_time_generic_queue_initialize() {
            unsafe {
                $crate::queue::Queue::initialize(&$name);
            }
        }

        #[no_mangle]
        fn _embassy_time_schedule_wake(at: $crate::Instant, waker: &core::task::Waker) {
            $crate::queue::Queue::schedule(&$name, at, waker);
        }
    };
}

#[cfg(feature = "generic-queue")]
generic_queue!(static QUEUE: Queue = Queue::new());

#[cfg(test)]
mod tests {
    use core::cell::Cell;
    use core::sync::atomic::Ordering;
    use core::task::{RawWaker, RawWakerVTable, Waker};
    use std::rc::Rc;
    use std::sync::Mutex;

    use embassy_sync::blocking_mutex::raw::RawMutex;
    use serial_test::serial;

    use super::InnerQueue;
    use crate::driver::{AlarmHandle, Driver};
    use crate::Instant;

    struct InnerTestDriver {
        now: u64,
        alarm: u64,
        callback: fn(*mut ()),
        ctx: *mut (),
    }

    impl InnerTestDriver {
        const fn new() -> Self {
            Self {
                now: 0,
                alarm: u64::MAX,
                callback: Self::noop,
                ctx: core::ptr::null_mut(),
            }
        }

        fn noop(_ctx: *mut ()) {}
    }

    unsafe impl Send for InnerTestDriver {}

    struct TestDriver(Mutex<InnerTestDriver>);

    impl TestDriver {
        const fn new() -> Self {
            Self(Mutex::new(InnerTestDriver::new()))
        }

        fn reset(&self) {
            *self.0.lock().unwrap() = InnerTestDriver::new();
        }

        fn set_now(&self, now: u64) {
            let notify = {
                let mut inner = self.0.lock().unwrap();

                if inner.now < now {
                    inner.now = now;

                    if inner.alarm <= now {
                        inner.alarm = u64::MAX;

                        Some((inner.callback, inner.ctx))
                    } else {
                        None
                    }
                } else {
                    panic!("Going back in time?");
                }
            };

            if let Some((callback, ctx)) = notify {
                (callback)(ctx);
            }
        }
    }

    impl Driver for TestDriver {
        fn now(&self) -> u64 {
            self.0.lock().unwrap().now
        }

        unsafe fn allocate_alarm(&self) -> Option<AlarmHandle> {
            Some(AlarmHandle::new(0))
        }

        fn set_alarm_callback(&self, _alarm: AlarmHandle, callback: fn(*mut ()), ctx: *mut ()) {
            let mut inner = self.0.lock().unwrap();

            inner.callback = callback;
            inner.ctx = ctx;
        }

        fn set_alarm(&self, _alarm: AlarmHandle, timestamp: u64) {
            let notify = {
                let mut inner = self.0.lock().unwrap();

                if timestamp <= inner.now {
                    Some((inner.callback, inner.ctx))
                } else {
                    inner.alarm = timestamp;
                    None
                }
            };

            if let Some((callback, ctx)) = notify {
                (callback)(ctx);
            }
        }
    }

    struct TestWaker {
        pub awoken: Rc<Cell<bool>>,
        pub waker: Waker,
    }

    impl TestWaker {
        fn new() -> Self {
            let flag = Rc::new(Cell::new(false));

            const VTABLE: RawWakerVTable = RawWakerVTable::new(
                |data: *const ()| {
                    unsafe {
                        Rc::increment_strong_count(data as *const Cell<bool>);
                    }

                    RawWaker::new(data as _, &VTABLE)
                },
                |data: *const ()| unsafe {
                    let data = data as *const Cell<bool>;
                    data.as_ref().unwrap().set(true);
                    Rc::decrement_strong_count(data);
                },
                |data: *const ()| unsafe {
                    (data as *const Cell<bool>).as_ref().unwrap().set(true);
                },
                |data: *const ()| unsafe {
                    Rc::decrement_strong_count(data);
                },
            );

            let raw = RawWaker::new(Rc::into_raw(flag.clone()) as _, &VTABLE);

            Self {
                awoken: flag.clone(),
                waker: unsafe { Waker::from_raw(raw) },
            }
        }
    }

    // TODO: This impl should be part of `embassy-sync`, hidden behind the "std" feature gate
    pub struct StdRawMutex(std::sync::Mutex<()>);

    unsafe impl RawMutex for StdRawMutex {
        const INIT: Self = StdRawMutex(std::sync::Mutex::new(()));

        fn lock<R>(&self, f: impl FnOnce() -> R) -> R {
            let _guard = self.0.lock().unwrap();

            f()
        }
    }

    const QUEUE_MAX_LEN: usize = 8;

    crate::time_driver_impl!(static DRIVER: TestDriver = TestDriver::new());
    crate::generic_queue!(static QUEUE: super::Queue<{ QUEUE_MAX_LEN }, StdRawMutex> = super::Queue::new());

    fn setup() {
        DRIVER.reset();

        QUEUE.alarm.set(None);
        QUEUE.alarm_schedule.store(u64::MAX, Ordering::SeqCst);
        QUEUE.inner.lock(|inner| {
            *inner.borrow_mut() = InnerQueue::new();
        });

        unsafe { super::initialize() };
    }

    fn queue_len() -> usize {
        QUEUE.inner.lock(|inner| inner.borrow().queue.iter().count())
    }

    #[test]
    #[serial]
    #[should_panic(expected = "Queue is not initialized yet")]
    fn test_not_initialized() {
        static QUEUE: super::Queue<{ QUEUE_MAX_LEN }, StdRawMutex> = super::Queue::new();

        let waker = TestWaker::new();

        QUEUE.schedule(Instant::from_secs(1), &waker.waker);
    }

    #[test]
    #[serial]
    fn test_initialized() {
        static QUEUE: super::Queue<{ QUEUE_MAX_LEN }, StdRawMutex> = super::Queue::new();

        assert!(QUEUE.alarm.get().is_none());

        unsafe { QUEUE.initialize() };

        assert!(QUEUE.alarm.get().is_some());
    }

    #[test]
    #[serial]
    #[should_panic(expected = "Queue is already initialized")]
    fn test_already_initialized() {
        static QUEUE: super::Queue<{ QUEUE_MAX_LEN }, StdRawMutex> = super::Queue::new();

        unsafe { QUEUE.initialize() };

        assert!(QUEUE.alarm.get().is_some());

        unsafe { QUEUE.initialize() };
    }

    #[test]
    #[serial]
    fn test_schedule() {
        setup();

        assert_eq!(queue_len(), 0);

        let waker = TestWaker::new();

        QUEUE.schedule(Instant::from_secs(1), &waker.waker);

        assert!(!waker.awoken.get());
        assert_eq!(queue_len(), 1);
    }

    #[test]
    #[serial]
    fn test_schedule_same() {
        setup();

        let waker = TestWaker::new();

        QUEUE.schedule(Instant::from_secs(1), &waker.waker);

        assert_eq!(queue_len(), 1);

        QUEUE.schedule(Instant::from_secs(1), &waker.waker);

        assert_eq!(queue_len(), 1);

        QUEUE.schedule(Instant::from_secs(100), &waker.waker);

        assert_eq!(queue_len(), 1);

        let waker2 = TestWaker::new();

        QUEUE.schedule(Instant::from_secs(100), &waker2.waker);

        assert_eq!(queue_len(), 2);
    }

    #[test]
    #[serial]
    fn test_trigger() {
        setup();

        let waker = TestWaker::new();

        QUEUE.schedule(Instant::from_secs(100), &waker.waker);

        assert!(!waker.awoken.get());

        DRIVER.set_now(Instant::from_secs(99).as_ticks());

        assert!(!waker.awoken.get());

        assert_eq!(queue_len(), 1);

        DRIVER.set_now(Instant::from_secs(100).as_ticks());

        assert!(waker.awoken.get());

        assert_eq!(queue_len(), 0);
    }

    #[test]
    #[serial]
    fn test_immediate_trigger() {
        setup();

        let waker = TestWaker::new();

        QUEUE.schedule(Instant::from_secs(100), &waker.waker);

        DRIVER.set_now(Instant::from_secs(50).as_ticks());

        let waker2 = TestWaker::new();

        QUEUE.schedule(Instant::from_secs(40), &waker2.waker);

        assert!(!waker.awoken.get());
        assert!(waker2.awoken.get());
        assert_eq!(queue_len(), 1);
    }

    #[test]
    #[serial]
    fn test_queue_overflow() {
        setup();

        for i in 1..QUEUE_MAX_LEN {
            let waker = TestWaker::new();

            QUEUE.schedule(Instant::from_secs(310), &waker.waker);

            assert_eq!(queue_len(), i);
            assert!(!waker.awoken.get());
        }

        let first_waker = TestWaker::new();

        QUEUE.schedule(Instant::from_secs(300), &first_waker.waker);

        assert_eq!(queue_len(), QUEUE_MAX_LEN);
        assert!(!first_waker.awoken.get());

        let second_waker = TestWaker::new();

        QUEUE.schedule(Instant::from_secs(305), &second_waker.waker);

        assert_eq!(queue_len(), QUEUE_MAX_LEN);
        assert!(first_waker.awoken.get());

        QUEUE.schedule(Instant::from_secs(320), &TestWaker::new().waker);
        assert_eq!(queue_len(), QUEUE_MAX_LEN);
        assert!(second_waker.awoken.get());
    }
}
