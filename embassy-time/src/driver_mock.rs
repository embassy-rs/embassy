use core::cell::RefCell;

use critical_section::Mutex as CsMutex;
use embassy_time_driver::Driver;

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
        let notify = {
            critical_section::with(|cs| {
                let mut inner = self.0.borrow_ref_mut(cs);

                inner.now += duration;

                let now = inner.now.as_ticks();

                if inner.alarm.timestamp <= now {
                    inner.alarm.timestamp = u64::MAX;

                    Some((inner.alarm.callback, inner.alarm.ctx))
                } else {
                    None
                }
            })
        };

        if let Some((callback, ctx)) = notify {
            (callback)(ctx);
        }
    }

    /// Configures a callback to be called when the alarm fires.
    pub fn set_alarm_callback(&self, callback: fn(*mut ()), ctx: *mut ()) {
        critical_section::with(|cs| {
            let mut inner = self.0.borrow_ref_mut(cs);

            inner.alarm.callback = callback;
            inner.alarm.ctx = ctx;
        });
    }

    /// Sets the alarm to fire at the specified timestamp.
    pub fn set_alarm(&self, timestamp: u64) -> bool {
        critical_section::with(|cs| {
            let mut inner = self.0.borrow_ref_mut(cs);

            if timestamp <= inner.now.as_ticks() {
                false
            } else {
                inner.alarm.timestamp = timestamp;
                true
            }
        })
    }
}

impl Driver for MockDriver {
    fn now(&self) -> u64 {
        critical_section::with(|cs| self.0.borrow_ref(cs).now).as_ticks()
    }
}

struct InnerMockDriver {
    now: Instant,
    alarm: AlarmState,
}

impl InnerMockDriver {
    const fn new() -> Self {
        Self {
            now: Instant::from_ticks(0),
            alarm: AlarmState::new(),
        }
    }
}

struct AlarmState {
    timestamp: u64,
    callback: fn(*mut ()),
    ctx: *mut (),
}

impl AlarmState {
    const fn new() -> Self {
        Self {
            timestamp: u64::MAX,
            callback: Self::noop,
            ctx: core::ptr::null_mut(),
        }
    }

    fn noop(_ctx: *mut ()) {}
}

unsafe impl Send for AlarmState {}

#[cfg(test)]
mod tests {
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
    fn test_set_alarm_not_in_future() {
        setup();

        let driver = MockDriver::get();
        assert_eq!(false, driver.set_alarm(driver.now()));
    }

    #[test]
    #[serial]
    fn test_alarm() {
        setup();

        let driver = MockDriver::get();
        static mut CALLBACK_CALLED: bool = false;
        driver.set_alarm_callback(|_| unsafe { CALLBACK_CALLED = true }, core::ptr::null_mut());
        driver.set_alarm(driver.now() + 1);
        assert_eq!(false, unsafe { CALLBACK_CALLED });
        driver.advance(Duration::from_secs(1));
        assert_eq!(true, unsafe { CALLBACK_CALLED });
    }
}
