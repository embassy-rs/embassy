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
            let next_alarm = queue.next_expiration(self.now());
            drop(queue);
            if next_alarm < u64::MAX {
                self.signaler.signal();
            }
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

#[cfg(test)]
mod tests {
    //! Regression test for the deadlock between `schedule_wake()` and
    //! the `alarm_thread()`.
    //!
    //! Root cause: the old code wrapped both `schedule_wake()` and
    //! `alarm_thread()` in `critical_section::with()`, which acquires a
    //! global CS mutex.  Because `schedule_wake()` locked the `queue` mutex
    //! first and then entered the CS, while `alarm_thread()` entered the CS
    //! first and then locked `queue`, concurrent calls could deadlock
    //!
    //! ```text
    //!   schedule_wake():   queue (held) → CS (waiting)
    //!   alarm_thread():    CS   (held) → queue (waiting)  ← deadlock
    //! ```
    //!
    //! The test below would hang indefinitely (and be caught by the watchdog
    //! thread) if the bug were reintroduced.

    use std::sync::Arc;
    use std::task::Wake;
    use std::time::Duration as StdDuration;

    use embassy_time_driver::Driver;

    use super::DRIVER;

    /// A no-op waker used to drive `schedule_wake` calls.
    struct NoopWaker;
    impl Wake for NoopWaker {
        fn wake(self: Arc<Self>) {}
    }

    /// Regression test: `schedule_wake()` must never deadlock with the
    /// background `alarm_thread`.
    ///
    /// Strategy: touch `DRIVER.zero_instant` to ensure the alarm thread is
    /// running, then spam `schedule_wake()` from many concurrent threads.
    /// A watchdog thread panics the process if the whole thing takes longer
    /// than 5 seconds, which would clearly suggest deadlock.
    #[test]
    fn no_deadlock_between_schedule_wake_and_alarm_thread() {
        let _ = DRIVER.now();

        // Watchdog: panic if the test body takes longer than 5 s.
        let watchdog = std::thread::spawn(|| {
            std::thread::sleep(StdDuration::from_secs(5));
            panic!(
                "deadlock detected: schedule_wake() and alarm_thread() \
                 appear to be stuck. Was critical_section::with() \
                 reintroduced around a queue lock?"
            );
        });

        const THREADS: usize = 16;
        const CALLS_PER_THREAD: usize = 1_000;

        let handles: Vec<_> = (0..THREADS)
            .map(|_| {
                std::thread::spawn(|| {
                    let waker: core::task::Waker = Arc::new(NoopWaker).into();
                    for _ in 0..CALLS_PER_THREAD {
                        // Schedule a wake far in the future so the alarm thread
                        // keeps churning and re-locking the queue mutex.
                        let at = DRIVER.now().saturating_add(1_000_000);
                        DRIVER.schedule_wake(at, &waker);
                    }
                })
            })
            .collect();

        for h in handles {
            h.join().expect("worker thread panicked");
        }

        // If we reach this point without the watchdog firing, there was no
        // deadlock.  Detach the watchdog so it doesn't outlive the test run
        // and trigger a spurious panic later.
        drop(watchdog);
    }
}
