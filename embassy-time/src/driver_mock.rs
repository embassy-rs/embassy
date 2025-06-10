use core::cell::RefCell;
use core::task::Waker;

use critical_section::Mutex as CsMutex;
use embassy_time_driver::Driver;
use embassy_time_queue_utils::Queue;

use crate::{Duration, Instant};

/// A mock driver that can be manually advanced.
/// This is useful for testing code that works with [`Instant`] and [`Duration`].
///
/// This driver can also be used to test runtime functionality, such as
/// timers, delays, etc.
///
/// # Example
///
/// ```ignore
/// fn has_a_second_passed(reference: Instant) -> bool {
///     Instant::now().duration_since(reference) >= Duration::from_secs(1)
/// }
///
/// fn test_second_passed() {
///     let driver = embassy_time::MockDriver::get();
///     let reference = Instant::now();
///     assert_eq!(false, has_a_second_passed(reference));
///     driver.advance(Duration::from_secs(1));
///     assert_eq!(true, has_a_second_passed(reference));
/// }
/// ```
pub struct MockDriver(CsMutex<RefCell<InnerMockDriver>>);

embassy_time_driver::time_driver_impl!(static DRIVER: MockDriver = MockDriver::new());

impl MockDriver {
    /// Creates a new mock driver.
    pub const fn new() -> Self {
        Self(CsMutex::new(RefCell::new(InnerMockDriver::new())))
    }

    /// Gets a reference to the global mock driver.
    pub fn get() -> &'static MockDriver {
        &DRIVER
    }

    /// Resets the internal state of the mock driver
    /// This will clear and deallocate all alarms, and reset the current time to 0.
    pub fn reset(&self) {
        critical_section::with(|cs| {
            self.0.borrow(cs).replace(InnerMockDriver::new());
        });
    }

    /// Advances the time by the specified [`Duration`].
    /// Calling any alarm callbacks that are due.
    pub fn advance(&self, duration: Duration) {
        critical_section::with(|cs| {
            let inner = &mut *self.0.borrow_ref_mut(cs);

            inner.now += duration;
            // wake expired tasks.
            inner.queue.next_expiration(inner.now.as_ticks());
        })
    }
}

impl Driver for MockDriver {
    fn now(&self) -> u64 {
        critical_section::with(|cs| self.0.borrow_ref(cs).now).as_ticks()
    }

    fn schedule_wake(&self, at: u64, waker: &Waker) {
        critical_section::with(|cs| {
            let inner = &mut *self.0.borrow_ref_mut(cs);
            // enqueue it
            inner.queue.schedule_wake(at, waker);
            // wake it if it's in the past.
            inner.queue.next_expiration(inner.now.as_ticks());
        })
    }
}

struct InnerMockDriver {
    now: Instant,
    queue: Queue,
}

impl InnerMockDriver {
    const fn new() -> Self {
        Self {
            now: Instant::from_ticks(0),
            queue: Queue::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use core::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::task::Wake;

    use serial_test::serial;

    use super::*;

    fn setup() {
        DRIVER.reset();
    }

    #[test]
    #[serial]
    fn test_advance() {
        setup();

        let driver = MockDriver::get();
        let reference = driver.now();
        driver.advance(Duration::from_secs(1));
        assert_eq!(Duration::from_secs(1).as_ticks(), driver.now() - reference);
    }

    #[test]
    #[serial]
    fn test_schedule_wake() {
        setup();

        static CALLBACK_CALLED: AtomicBool = AtomicBool::new(false);

        struct MockWaker;

        impl Wake for MockWaker {
            fn wake(self: Arc<Self>) {
                CALLBACK_CALLED.store(true, Ordering::Relaxed);
            }
        }
        let waker = Arc::new(MockWaker).into();

        let driver = MockDriver::get();

        driver.schedule_wake(driver.now() + 1, &waker);
        assert_eq!(false, CALLBACK_CALLED.load(Ordering::Relaxed));
        driver.advance(Duration::from_secs(1));
        assert_eq!(true, CALLBACK_CALLED.load(Ordering::Relaxed));
    }
}
