//! A generic timer queue.

use core::cmp::{min, Ordering};
use core::task::Waker;

use heapless::Vec;

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

/// A timer queue with a pre-determined capacity.
pub struct Queue<const QUEUE_SIZE: usize> {
    queue: Vec<Timer, QUEUE_SIZE>,
}

impl<const QUEUE_SIZE: usize> Queue<QUEUE_SIZE> {
    /// Creates a new timer queue.
    pub const fn new() -> Self {
        Self { queue: Vec::new() }
    }

    /// Schedules a task to run at a specific time, and returns whether any changes were made.
    pub fn schedule_wake(&mut self, at: Instant, waker: &Waker) -> bool {
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
        true
    }

    /// Dequeues expired timers and returns the next alarm time.
    pub fn next_expiration(&mut self, now: Instant) -> Instant {
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

        next_alarm
    }
}

#[cfg(test)]
#[cfg(feature = "mock-driver")]
mod tests {
    use core::sync::atomic::{AtomicBool, Ordering};
    use core::task::Waker;
    use std::sync::Arc;
    use std::task::Wake;

    use serial_test::serial;

    use crate::driver_mock::MockDriver;
    use crate::queue_generic::QUEUE;
    use crate::{Duration, Instant};

    struct TestWaker {
        pub awoken: AtomicBool,
    }

    impl Wake for TestWaker {
        fn wake(self: Arc<Self>) {
            self.awoken.store(true, Ordering::Relaxed);
        }

        fn wake_by_ref(self: &Arc<Self>) {
            self.awoken.store(true, Ordering::Relaxed);
        }
    }

    fn test_waker() -> (Arc<TestWaker>, Waker) {
        let arc = Arc::new(TestWaker {
            awoken: AtomicBool::new(false),
        });
        let waker = Waker::from(arc.clone());

        (arc, waker)
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

        let (flag, waker) = test_waker();

        QUEUE.schedule_wake(Instant::from_secs(1), &waker);

        assert!(!flag.awoken.load(Ordering::Relaxed));
        assert_eq!(queue_len(), 1);
    }

    #[test]
    #[serial]
    fn test_schedule_same() {
        setup();

        let (_flag, waker) = test_waker();

        QUEUE.schedule_wake(Instant::from_secs(1), &waker);

        assert_eq!(queue_len(), 1);

        QUEUE.schedule_wake(Instant::from_secs(1), &waker);

        assert_eq!(queue_len(), 1);

        QUEUE.schedule_wake(Instant::from_secs(100), &waker);

        assert_eq!(queue_len(), 1);

        let (_flag2, waker2) = test_waker();

        QUEUE.schedule_wake(Instant::from_secs(100), &waker2);

        assert_eq!(queue_len(), 2);
    }

    #[test]
    #[serial]
    fn test_trigger() {
        setup();

        let (flag, waker) = test_waker();

        QUEUE.schedule_wake(Instant::from_secs(100), &waker);

        assert!(!flag.awoken.load(Ordering::Relaxed));

        MockDriver::get().advance(Duration::from_secs(99));

        assert!(!flag.awoken.load(Ordering::Relaxed));

        assert_eq!(queue_len(), 1);

        MockDriver::get().advance(Duration::from_secs(1));

        assert!(flag.awoken.load(Ordering::Relaxed));

        assert_eq!(queue_len(), 0);
    }

    #[test]
    #[serial]
    fn test_immediate_trigger() {
        setup();

        let (flag, waker) = test_waker();

        QUEUE.schedule_wake(Instant::from_secs(100), &waker);

        MockDriver::get().advance(Duration::from_secs(50));

        let (flag2, waker2) = test_waker();

        QUEUE.schedule_wake(Instant::from_secs(40), &waker2);

        assert!(!flag.awoken.load(Ordering::Relaxed));
        assert!(flag2.awoken.load(Ordering::Relaxed));
        assert_eq!(queue_len(), 1);
    }

    #[test]
    #[serial]
    fn test_queue_overflow() {
        setup();

        for i in 1..super::QUEUE_SIZE {
            let (flag, waker) = test_waker();

            QUEUE.schedule_wake(Instant::from_secs(310), &waker);

            assert_eq!(queue_len(), i);
            assert!(!flag.awoken.load(Ordering::Relaxed));
        }

        let (flag, waker) = test_waker();

        QUEUE.schedule_wake(Instant::from_secs(300), &waker);

        assert_eq!(queue_len(), super::QUEUE_SIZE);
        assert!(!flag.awoken.load(Ordering::Relaxed));

        let (flag2, waker2) = test_waker();

        QUEUE.schedule_wake(Instant::from_secs(305), &waker2);

        assert_eq!(queue_len(), super::QUEUE_SIZE);
        assert!(flag.awoken.load(Ordering::Relaxed));

        let (_flag3, waker3) = test_waker();
        QUEUE.schedule_wake(Instant::from_secs(320), &waker3);
        assert_eq!(queue_len(), super::QUEUE_SIZE);
        assert!(flag2.awoken.load(Ordering::Relaxed));
    }
}
