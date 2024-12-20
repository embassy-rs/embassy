use std::sync::{Condvar, Mutex};
use std::thread;
use std::time::{Duration as StdDuration, Instant as StdInstant};

use embassy_time_driver::Driver;
use embassy_time_queue_utils::Queue;

struct TimeDriver {
    signaler: Signaler,
    inner: Mutex<Inner>,
}

struct Inner {
    zero_instant: Option<StdInstant>,
    queue: Queue,
}

embassy_time_driver::time_driver_impl!(static DRIVER: TimeDriver = TimeDriver {
    inner: Mutex::new(Inner{
        zero_instant: None,
        queue: Queue::new(),
    }),
    signaler: Signaler::new(),
});

impl Inner {
    fn init(&mut self) -> StdInstant {
        *self.zero_instant.get_or_insert_with(|| {
            thread::spawn(alarm_thread);
            StdInstant::now()
        })
    }
}

impl Driver for TimeDriver {
    fn now(&self) -> u64 {
        let mut inner = self.inner.lock().unwrap();
        let zero = inner.init();
        StdInstant::now().duration_since(zero).as_micros() as u64
    }

    fn schedule_wake(&self, at: u64, waker: &core::task::Waker) {
        let mut inner = self.inner.lock().unwrap();
        inner.init();
        if inner.queue.schedule_wake(at, waker) {
            self.signaler.signal();
        }
    }
}

fn alarm_thread() {
    let zero = DRIVER.inner.lock().unwrap().zero_instant.unwrap();
    loop {
        let now = DRIVER.now();

        let next_alarm = DRIVER.inner.lock().unwrap().queue.next_expiration(now);

        // Ensure we don't overflow
        let until = zero
            .checked_add(StdDuration::from_micros(next_alarm))
            .unwrap_or_else(|| StdInstant::now() + StdDuration::from_secs(1));

        DRIVER.signaler.wait_until(until);
    }
}

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
