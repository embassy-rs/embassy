use core::cell::RefCell;
use core::cmp::{min, Ordering};
use core::task::Waker;

use critical_section::Mutex;
use embassy_time_driver::{allocate_alarm, set_alarm, set_alarm_callback, AlarmHandle};
use embassy_time_queue_driver::TimerQueue;
use heapless::Vec;

use crate::Instant;

#[cfg(feature = "generic-queue-8")]
const QUEUE_SIZE: usize = 8;
#[cfg(feature = "generic-queue-16")]
const QUEUE_SIZE: usize = 16;
#[cfg(feature = "generic-queue-32")]
const QUEUE_SIZE: usize = 32;
#[cfg(feature = "generic-queue-64")]
const QUEUE_SIZE: usize = 64;
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
    queue: Vec<Timer, QUEUE_SIZE>,
    alarm: AlarmHandle,
}

impl InnerQueue {
    fn schedule_wake(&mut self, at: Instant, waker: &Waker) {
        self.queue
            .iter_mut()
            .find(|timer| timer.waker.will_wake(waker))
            .map(|timer| {
                timer.at = min(timer.at, at);
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

            let mut next_alarm = Instant::MAX;

            let mut i = 0;
            while i < self.queue.len() {
                let timer = &self.queue[i];
                if timer.at <= now {
                    let timer = self.queue.swap_remove(i);
                    timer.waker.wake();
                } else {
                    next_alarm = min(next_alarm, timer.at);
                    i += 1;
                }
            }

            if self.update_alarm(next_alarm) {
                break;
            }
        }
    }

    fn update_alarm(&mut self, next_alarm: Instant) -> bool {
        if next_alarm == Instant::MAX {
            true
        } else {
            set_alarm(self.alarm, next_alarm.as_ticks())
        }
    }

    fn handle_alarm(&mut self) {
        self.dispatch();
    }
}

struct Queue {
    inner: Mutex<RefCell<Option<InnerQueue>>>,
}

impl Queue {
    const fn new() -> Self {
        Self {
            inner: Mutex::new(RefCell::new(None)),
        }
    }

    fn schedule_wake(&'static self, at: Instant, waker: &Waker) {
        critical_section::with(|cs| {
            let mut inner = self.inner.borrow_ref_mut(cs);

            if inner.is_none() {}

            inner
                .get_or_insert_with(|| {
                    let handle = unsafe { allocate_alarm() }.unwrap();
                    set_alarm_callback(handle, Self::handle_alarm_callback, self as *const _ as _);
                    InnerQueue {
                        queue: Vec::new(),
                        alarm: handle,
                    }
                })
                .schedule_wake(at, waker)
        });
    }

    fn handle_alarm(&self) {
        critical_section::with(|cs| self.inner.borrow_ref_mut(cs).as_mut().unwrap().handle_alarm())
    }

    fn handle_alarm_callback(ctx: *mut ()) {
        unsafe { (ctx as *const Self).as_ref().unwrap() }.handle_alarm();
    }
}

impl TimerQueue for Queue {
    fn schedule_wake(&'static self, at: u64, waker: &Waker) {
        Queue::schedule_wake(self, Instant::from_ticks(at), waker);
    }
}

embassy_time_queue_driver::timer_queue_impl!(static QUEUE: Queue = Queue::new());

#[cfg(test)]
#[cfg(feature = "mock-driver")]
mod tests {
    use core::cell::Cell;
    use core::task::{RawWaker, RawWakerVTable, Waker};
    use std::rc::Rc;

    use serial_test::serial;

    use crate::driver_mock::MockDriver;
    use crate::queue_generic::QUEUE;
    use crate::{Duration, Instant};

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

    fn setup() {
        MockDriver::get().reset();
        critical_section::with(|cs| *QUEUE.inner.borrow_ref_mut(cs) = None);
    }

    fn queue_len() -> usize {
        critical_section::with(|cs| {
            QUEUE
                .inner
                .borrow_ref(cs)
                .as_ref()
                .map(|inner| inner.queue.iter().count())
                .unwrap_or(0)
        })
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

        MockDriver::get().advance(Duration::from_secs(99));

        assert!(!waker.awoken.get());

        assert_eq!(queue_len(), 1);

        MockDriver::get().advance(Duration::from_secs(1));

        assert!(waker.awoken.get());

        assert_eq!(queue_len(), 0);
    }

    #[test]
    #[serial]
    fn test_immediate_trigger() {
        setup();

        let waker = TestWaker::new();

        QUEUE.schedule_wake(Instant::from_secs(100), &waker.waker);

        MockDriver::get().advance(Duration::from_secs(50));

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
