use core::cell::Cell;

use critical_section::Mutex as CsMutex;

use crate::driver::{AlarmHandle, Driver};
use crate::{Duration, Instant};

/// A mock driver that can be manually advanced.
/// This is useful for testing code that works with [`Instant`] and [`Duration`].
///
/// This driver cannot currently be used to test runtime functionality, such as
/// timers, delays, etc.
///
/// # Example
///
/// ```ignore
/// fn has_a_second_passed(reference: Instant) -> bool {
///     Instant::now().duration_since(reference) > Duration::from_secs(1)
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
pub struct MockDriver {
    now: CsMutex<Cell<Instant>>,
}

crate::time_driver_impl!(static DRIVER: MockDriver = MockDriver {
    now: CsMutex::new(Cell::new(Instant::from_ticks(0))),
});

impl MockDriver {
    /// Gets a reference to the global mock driver.
    pub fn get() -> &'static MockDriver {
        &DRIVER
    }

    /// Advances the time by the specified [`Duration`].
    pub fn advance(&self, duration: Duration) {
        critical_section::with(|cs| {
            let now = self.now.borrow(cs).get().as_ticks();
            self.now.borrow(cs).set(Instant::from_ticks(now + duration.as_ticks()));
        });
    }
}

impl Driver for MockDriver {
    fn now(&self) -> u64 {
        critical_section::with(|cs| self.now.borrow(cs).get().as_micros() as u64)
    }

    unsafe fn allocate_alarm(&self) -> Option<AlarmHandle> {
        unimplemented!("MockDriver does not support runtime features that require an executor");
    }

    fn set_alarm_callback(&self, _alarm: AlarmHandle, _callback: fn(*mut ()), _ctx: *mut ()) {
        unimplemented!("MockDriver does not support runtime features that require an executor");
    }

    fn set_alarm(&self, _alarm: AlarmHandle, _timestamp: u64) -> bool {
        unimplemented!("MockDriver does not support runtime features that require an executor");
    }
}
