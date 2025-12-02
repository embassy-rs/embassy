use std::sync::{Condvar, LazyLock, Mutex};
use std::thread;
use std::time::{Duration as StdDuration, Instant as StdInstant};

use embassy_time_driver::Driver;
use embassy_time_queue_utils::Queue;

#[derive(Debug)]
struct TimeDriver {
    signaler: Signaler,
    queue: Mutex<Queue>,
    zero_instant: LazyLock<StdInstant>,
}

embassy_time_driver::time_driver_impl!(static DRIVER: TimeDriver = TimeDriver {
    zero_instant: LazyLock::new(|| init()),
    queue: Mutex::new(Queue::new()),
    signaler: Signaler::new(),
});

fn init() -> StdInstant {
    thread::spawn(alarm_thread);
    StdInstant::now()
}

impl Driver for TimeDriver {
    fn now(&self) -> u64 {
        StdInstant::now().duration_since(*self.zero_instant).as_micros() as u64
    }

    fn schedule_wake(&self, at: u64, waker: &core::task::Waker) {
        let mut queue = self.queue.lock().unwrap();
        if queue.schedule_wake(at, waker) {
            self.signaler.signal();
        }
    }
}

fn alarm_thread() {
    let zero = *DRIVER.zero_instant;
    loop {
        let now = DRIVER.now();

        let next_alarm = DRIVER.queue.lock().unwrap().next_expiration(now);

        // Ensure we don't overflow
        let until = zero
            .checked_add(StdDuration::from_micros(next_alarm))
            .unwrap_or_else(|| StdInstant::now() + StdDuration::from_secs(1));

        DRIVER.signaler.wait_until(until);
    }
}

#[derive(Debug)]
struct Signaler {
    mutex: Mutex<bool>,
    condvar: Condvar,
}

impl Signaler {
    const fn new() -> Self {
        Self {
            mutex: Mutex::new(false),
            condvar: Condvar::new(),
        }
    }

    fn wait_until(&self, until: StdInstant) {
        let mut signaled = self.mutex.lock().unwrap();
        while !*signaled {
            let now = StdInstant::now();

            if now >= until {
                break;
            }

            let dur = until - now;
            let (signaled2, timeout) = self.condvar.wait_timeout(signaled, dur).unwrap();
            signaled = signaled2;
            if timeout.timed_out() {
                break;
            }
        }
        *signaled = false;
    }

    fn signal(&self) {
        let mut signaled = self.mutex.lock().unwrap();
        *signaled = true;
        self.condvar.notify_one();
    }
}
