use core::cell::{Cell, RefCell};
use core::sync::atomic::{AtomicU32, Ordering, compiler_fence};

use critical_section::CriticalSection;
use embassy_sync::blocking_mutex::CriticalSectionMutex as Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_time_driver::Driver;
use embassy_time_queue_utils::Queue;

use crate::interrupt::InterruptExt;
use crate::{interrupt, pac};

#[cfg(feature = "_nrf54l")]
fn rtc() -> pac::rtc::Rtc {
    pac::RTC30
}
#[cfg(not(feature = "_nrf54l"))]
fn rtc() -> pac::rtc::Rtc {
    pac::RTC1
}

/// Calculate the timestamp from the period count and the tick count.
///
/// The RTC counter is 24 bit. Ticking at 32768hz, it overflows every ~8 minutes. This is
/// too short. We must make it "never" overflow.
///
/// The obvious way would be to count overflow periods. Every time the counter overflows,
/// increase a `periods` variable. `now()` simply does `periods << 24 + counter`. So, the logic
/// around an overflow would look like this:
///
/// ```not_rust
/// periods = 1, counter = 0xFF_FFFE --> now = 0x1FF_FFFE
/// periods = 1, counter = 0xFF_FFFF --> now = 0x1FF_FFFF
/// **OVERFLOW**
/// periods = 2, counter = 0x00_0000 --> now = 0x200_0000
/// periods = 2, counter = 0x00_0001 --> now = 0x200_0001
/// ```
///
/// The problem is this is vulnerable to race conditions if `now()` runs at the exact time an
/// overflow happens.
///
/// If `now()` reads `periods` first and `counter` later, and overflow happens between the reads,
/// it would return a wrong value:
///
/// ```not_rust
/// periods = 1 (OLD), counter = 0x00_0000 (NEW) --> now = 0x100_0000 -> WRONG
/// ```
///
/// It fails similarly if it reads `counter` first and `periods` second.
///
/// To fix this, we define a "period" to be 2^23 ticks (instead of 2^24). One "overflow cycle" is 2 periods.
///
/// - `period` is incremented on overflow (at counter value 0)
/// - `period` is incremented "midway" between overflows (at counter value 0x80_0000)
///
/// Therefore, when `period` is even, counter is in 0..0x7f_ffff. When odd, counter is in 0x80_0000..0xFF_FFFF
/// This allows for now() to return the correct value even if it races an overflow.
///
/// To get `now()`, `period` is read first, then `counter` is read. If the counter value matches
/// the expected range for the `period` parity, we're done. If it doesn't, this means that
/// a new period start has raced us between reading `period` and `counter`, so we assume the `counter` value
/// corresponds to the next period.
///
/// `period` is a 32bit integer, so It overflows on 2^32 * 2^23 / 32768 seconds of uptime, which is 34865
/// years. For comparison, flash memory like the one containing your firmware is usually rated to retain
/// data for only 10-20 years. 34865 years is long enough!
fn calc_now(period: u32, counter: u32) -> u64 {
    ((period as u64) << 23) + ((counter ^ ((period & 1) << 23)) as u64)
}

fn compare_n(n: usize) -> u32 {
    1 << (n + 16)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_calc_now() {
        assert_eq!(calc_now(0, 0x000000), 0x0_000000);
        assert_eq!(calc_now(0, 0x000001), 0x0_000001);
        assert_eq!(calc_now(0, 0x7FFFFF), 0x0_7FFFFF);
        assert_eq!(calc_now(1, 0x7FFFFF), 0x1_7FFFFF);
        assert_eq!(calc_now(0, 0x800000), 0x0_800000);
        assert_eq!(calc_now(1, 0x800000), 0x0_800000);
        assert_eq!(calc_now(1, 0x800001), 0x0_800001);
        assert_eq!(calc_now(1, 0xFFFFFF), 0x0_FFFFFF);
        assert_eq!(calc_now(2, 0xFFFFFF), 0x1_FFFFFF);
        assert_eq!(calc_now(1, 0x000000), 0x1_000000);
        assert_eq!(calc_now(2, 0x000000), 0x1_000000);
    }
}

struct AlarmState {
    timestamp: Cell<u64>,
}

unsafe impl Send for AlarmState {}

impl AlarmState {
    const fn new() -> Self {
        Self {
            timestamp: Cell::new(u64::MAX),
        }
    }
}

struct RtcDriver {
    /// Number of 2^23 periods elapsed since boot.
    period: AtomicU32,
    /// Timestamp at which to fire alarm. u64::MAX if no alarm is scheduled.
    alarms: Mutex<AlarmState>,
    queue: Mutex<RefCell<Queue>>,
}

embassy_time_driver::time_driver_impl!(static DRIVER: RtcDriver = RtcDriver {
    period: AtomicU32::new(0),
    alarms: Mutex::const_new(CriticalSectionRawMutex::new(), AlarmState::new()),
    queue: Mutex::new(RefCell::new(Queue::new())),
});

impl RtcDriver {
    fn init(&'static self, irq_prio: crate::interrupt::Priority) {
        let r = rtc();
        r.cc(3).write(|w| w.set_compare(0x800000));

        r.intenset().write(|w| {
            w.set_ovrflw(true);
            w.set_compare(3, true);
        });

        r.tasks_clear().write_value(1);
        r.tasks_start().write_value(1);

        // Wait for clear
        while r.counter().read().0 != 0 {}

        #[cfg(feature = "_nrf54l")]
        {
            interrupt::RTC30.set_priority(irq_prio);
            unsafe { interrupt::RTC30.enable() };
        }
        #[cfg(not(feature = "_nrf54l"))]
        {
            interrupt::RTC1.set_priority(irq_prio);
            unsafe { interrupt::RTC1.enable() };
        }
    }

    fn on_interrupt(&self) {
        let r = rtc();
        if r.events_ovrflw().read() == 1 {
            r.events_ovrflw().write_value(0);
            self.next_period();
        }

        if r.events_compare(3).read() == 1 {
            r.events_compare(3).write_value(0);
            self.next_period();
        }

        let n = 0;
        if r.events_compare(n).read() == 1 {
            r.events_compare(n).write_value(0);
            critical_section::with(|cs| {
                self.trigger_alarm(cs);
            });
        }
    }

    fn next_period(&self) {
        critical_section::with(|cs| {
            let r = rtc();
            let period = self.period.load(Ordering::Relaxed) + 1;
            self.period.store(period, Ordering::Relaxed);
            let t = (period as u64) << 23;

            let n = 0;
            let alarm = &self.alarms.borrow(cs);
            let at = alarm.timestamp.get();

            if at < t + 0xc00000 {
                // just enable it. `set_alarm` has already set the correct CC val.
                r.intenset().write(|w| w.0 = compare_n(n));
            }
        })
    }

    fn trigger_alarm(&self, cs: CriticalSection) {
        let n = 0;
        let r = rtc();
        r.intenclr().write(|w| w.0 = compare_n(n));

        let alarm = &self.alarms.borrow(cs);
        alarm.timestamp.set(u64::MAX);

        // Call after clearing alarm, so the callback can set another alarm.
        let mut next = self.queue.borrow(cs).borrow_mut().next_expiration(self.now());
        while !self.set_alarm(cs, next) {
            next = self.queue.borrow(cs).borrow_mut().next_expiration(self.now());
        }
    }

    fn set_alarm(&self, cs: CriticalSection, timestamp: u64) -> bool {
        let n = 0;
        let alarm = &self.alarms.borrow(cs);
        alarm.timestamp.set(timestamp);

        let r = rtc();

        loop {
            let t = self.now();
            if timestamp <= t {
                // If alarm timestamp has passed the alarm will not fire.
                // Disarm the alarm and return `false` to indicate that.
                r.intenclr().write(|w| w.0 = compare_n(n));

                alarm.timestamp.set(u64::MAX);

                return false;
            }

            // If it hasn't triggered yet, setup it in the compare channel.

            // Write the CC value regardless of whether we're going to enable it now or not.
            // This way, when we enable it later, the right value is already set.

            // nrf52 docs say:
            //    If the COUNTER is N, writing N or N+1 to a CC register may not trigger a COMPARE event.
            // To workaround this, we never write a timestamp smaller than N+3.
            // N+2 is not safe because rtc can tick from N to N+1 between calling now() and writing cc.
            //
            // Since the critical section does not guarantee that a higher prio interrupt causes
            // this to be delayed, we need to re-check how much time actually passed after setting the
            // alarm, and retry if we are within the unsafe interval still.
            //
            // This means that an alarm can be delayed for up to 2 ticks (from t+1 to t+3), but this is allowed
            // by the Alarm trait contract. What's not allowed is triggering alarms *before* their scheduled time,
            // and we don't do that here.
            let safe_timestamp = timestamp.max(t + 3);
            r.cc(n).write(|w| w.set_compare(safe_timestamp as u32 & 0xFFFFFF));

            let diff = timestamp - t;
            if diff < 0xc00000 {
                r.intenset().write(|w| w.0 = compare_n(n));

                // If we have not passed the timestamp, we can be sure the alarm will be invoked. Otherwise,
                // we need to retry setting the alarm.
                if self.now() + 2 <= timestamp {
                    return true;
                }
            } else {
                // If it's too far in the future, don't setup the compare channel yet.
                // It will be setup later by `next_period`.
                r.intenclr().write(|w| w.0 = compare_n(n));
                return true;
            }
        }
    }
}

impl Driver for RtcDriver {
    fn now(&self) -> u64 {
        // `period` MUST be read before `counter`, see comment at the top for details.
        let period = self.period.load(Ordering::Relaxed);
        compiler_fence(Ordering::Acquire);
        let counter = rtc().counter().read().0;
        calc_now(period, counter)
    }

    fn schedule_wake(&self, at: u64, waker: &core::task::Waker) {
        critical_section::with(|cs| {
            let mut queue = self.queue.borrow(cs).borrow_mut();

            if queue.schedule_wake(at, waker) {
                let mut next = queue.next_expiration(self.now());
                while !self.set_alarm(cs, next) {
                    next = queue.next_expiration(self.now());
                }
            }
        })
    }
}

#[cfg(feature = "_nrf54l")]
#[cfg(feature = "rt")]
#[interrupt]
fn RTC30() {
    DRIVER.on_interrupt()
}

#[cfg(not(feature = "_nrf54l"))]
#[cfg(feature = "rt")]
#[interrupt]
fn RTC1() {
    DRIVER.on_interrupt()
}

pub(crate) fn init(irq_prio: crate::interrupt::Priority) {
    DRIVER.init(irq_prio)
}
