use core::cell::RefCell;
use core::cmp::{min, Ordering};
use core::task::Waker;

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::blocking_mutex::Mutex;
use heapless::sorted_linked_list::{LinkedIndexU8, Min, SortedLinkedList};

use crate::driver::{allocate_alarm, set_alarm, set_alarm_callback, AlarmHandle};
use crate::queue::TimerQueue;
use crate::Instant;

#[cfg(feature = "generic-queue-8")]
const QUEUE_SIZE: usize = 8;
#[cfg(feature = "generic-queue-16")]
const QUEUE_SIZE: usize = 16;
#[cfg(feature = "generic-queue-32")]
const QUEUE_SIZE: usize = 32;
#[cfg(feature = "generic-queue-64")]
const QUEUE_SIZE: usize = 32;
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

struct InnerQueue {
    queue: SortedLinkedList<Timer, LinkedIndexU8, Min, { QUEUE_SIZE }>,
    alarm: Option<AlarmHandle>,
    alarm_at: Instant,
}

impl InnerQueue {
    const fn new() -> Self {
        Self {
            queue: SortedLinkedList::new_u8(),
            alarm: None,
            alarm_at: Instant::MAX,
        }
    }

    fn schedule_wake(&mut self, at: Instant, waker: &Waker) {
        self.queue
            .find_mut(|timer| timer.waker.will_wake(waker))
            .map(|mut timer| {
                timer.at = min(timer.at, at);
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
        self.dispatch();
    }

    fn dispatch(&mut self) {
        loop {
            let now = Instant::now();

            while self.queue.peek().filter(|timer| timer.at <= now).is_some() {
                self.queue.pop().unwrap().waker.wake();
            }

            if self.update_alarm() {
                break;
            }
        }
    }

    fn update_alarm(&mut self) -> bool {
        if let Some(timer) = self.queue.peek() {
            let new_at = timer.at;

            if self.alarm_at != new_at {
                self.alarm_at = new_at;

                return set_alarm(self.alarm.unwrap(), self.alarm_at.as_ticks());
            }
        } else {
            self.alarm_at = Instant::MAX;
        }

        true
    }

    fn handle_alarm(&mut self) {
        self.alarm_at = Instant::MAX;

        self.dispatch();
    }
}

struct Queue {
    inner: Mutex<CriticalSectionRawMutex, RefCell<InnerQueue>>,
}

impl Queue {
    const fn new() -> Self {
        Self {
            inner: Mutex::new(RefCell::new(InnerQueue::new())),
        }
    }

    fn schedule_wake(&'static self, at: Instant, waker: &Waker) {
        self.inner.lock(|inner| {
            let mut inner = inner.borrow_mut();

            if inner.alarm.is_none() {
                let handle = unsafe { allocate_alarm() }.unwrap();
                inner.alarm = Some(handle);

                set_alarm_callback(handle, Self::handle_alarm_callback, self as *const _ as _);
            }

            inner.schedule_wake(at, waker)
        });
    }

    fn handle_alarm(&self) {
        self.inner.lock(|inner| inner.borrow_mut().handle_alarm());
    }

    fn handle_alarm_callback(ctx: *mut ()) {
        unsafe { (ctx as *const Self).as_ref().unwrap() }.handle_alarm();
    }
}

impl TimerQueue for Queue {
    fn schedule_wake(&'static self, at: Instant, waker: &Waker) {
        Queue::schedule_wake(self, at, waker);
    }
}

crate::timer_queue_impl!(static QUEUE: Queue = Queue::new());

#[cfg(test)]
mod tests {
    use core::cell::Cell;
    use core::task::{RawWaker, RawWakerVTable, Waker};
    use std::rc::Rc;
    use std::sync::Mutex;

    use serial_test::serial;

    use super::InnerQueue;
    use crate::driver::{AlarmHandle, Driver};
    use crate::queue_generic::QUEUE;
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

        fn set_alarm(&self, _alarm: AlarmHandle, timestamp: u64) -> bool {
            let mut inner = self.0.lock().unwrap();

            if timestamp <= inner.now {
                false
            } else {
                inner.alarm = timestamp;
                true
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

    crate::time_driver_impl!(static DRIVER: TestDriver = TestDriver::new());

    fn setup() {
        DRIVER.reset();

        QUEUE.inner.lock(|inner| {
            *inner.borrow_mut() = InnerQueue::new();
        });
    }

    fn queue_len() -> usize {
        QUEUE.inner.lock(|inner| inner.borrow().queue.iter().count())
    }

    #[test]
    #[serial]
    fn test_schedule() {
        setup();

        assert_eq!(queue_len(), 0);

        let waker = TestWaker::new();

        QUEUE.schedule_wake(Instant::from_secs(1), &waker.waker);

        assert!(!waker.awoken.get());
        assert_eq!(queue_len(), 1);
    }

    #[test]
    #[serial]
    fn test_schedule_same() {
        setup();

        let waker = TestWaker::new();

        QUEUE.schedule_wake(Instant::from_secs(1), &waker.waker);

        assert_eq!(queue_len(), 1);

        QUEUE.schedule_wake(Instant::from_secs(1), &waker.waker);

        assert_eq!(queue_len(), 1);

        QUEUE.schedule_wake(Instant::from_secs(100), &waker.waker);

        assert_eq!(queue_len(), 1);

        let waker2 = TestWaker::new();

        QUEUE.schedule_wake(Instant::from_secs(100), &waker2.waker);

        assert_eq!(queue_len(), 2);
    }

    #[test]
    #[serial]
    fn test_trigger() {
        setup();

        let waker = TestWaker::new();

        QUEUE.schedule_wake(Instant::from_secs(100), &waker.waker);

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

        QUEUE.schedule_wake(Instant::from_secs(100), &waker.waker);

        DRIVER.set_now(Instant::from_secs(50).as_ticks());

        let waker2 = TestWaker::new();

        QUEUE.schedule_wake(Instant::from_secs(40), &waker2.waker);

        assert!(!waker.awoken.get());
        assert!(waker2.awoken.get());
        assert_eq!(queue_len(), 1);
    }

    #[test]
    #[serial]
    fn test_queue_overflow() {
        setup();

        for i in 1..super::QUEUE_SIZE {
            let waker = TestWaker::new();

            QUEUE.schedule_wake(Instant::from_secs(310), &waker.waker);

            assert_eq!(queue_len(), i);
            assert!(!waker.awoken.get());
        }

        let first_waker = TestWaker::new();

        QUEUE.schedule_wake(Instant::from_secs(300), &first_waker.waker);

        assert_eq!(queue_len(), super::QUEUE_SIZE);
        assert!(!first_waker.awoken.get());

        let second_waker = TestWaker::new();

        QUEUE.schedule_wake(Instant::from_secs(305), &second_waker.waker);

        assert_eq!(queue_len(), super::QUEUE_SIZE);
        assert!(first_waker.awoken.get());

        QUEUE.schedule_wake(Instant::from_secs(320), &TestWaker::new().waker);
        assert_eq!(queue_len(), super::QUEUE_SIZE);
        assert!(second_waker.awoken.get());
    }
}
