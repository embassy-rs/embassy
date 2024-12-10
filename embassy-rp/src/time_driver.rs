//! Timer driver.
use core::cell::Cell;

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::blocking_mutex::Mutex;
use embassy_time_driver::Driver;
use embassy_time_queue_driver::GlobalTimerQueue;
#[cfg(feature = "rp2040")]
use pac::TIMER;
#[cfg(feature = "_rp235x")]
use pac::TIMER0 as TIMER;

use crate::interrupt::InterruptExt;
use crate::{interrupt, pac};

struct AlarmState {
    timestamp: Cell<u64>,
}
unsafe impl Send for AlarmState {}

struct TimerDriver {
    alarms: Mutex<CriticalSectionRawMutex, AlarmState>,
}

embassy_time_driver::time_driver_impl!(static DRIVER: TimerDriver = TimerDriver{
    alarms:  Mutex::const_new(CriticalSectionRawMutex::new(), AlarmState {
        timestamp: Cell::new(0),
    }),
});

impl Driver for TimerDriver {
    fn now(&self) -> u64 {
        loop {
            let hi = TIMER.timerawh().read();
            let lo = TIMER.timerawl().read();
            let hi2 = TIMER.timerawh().read();
            if hi == hi2 {
                return (hi as u64) << 32 | (lo as u64);
            }
        }
    }
}

impl TimerDriver {
    fn set_alarm(&self, timestamp: u64) -> bool {
        let n = 0;
        critical_section::with(|cs| {
            let alarm = &self.alarms.borrow(cs);
            alarm.timestamp.set(timestamp);

            // Arm it.
            // Note that we're not checking the high bits at all. This means the irq may fire early
            // if the alarm is more than 72 minutes (2^32 us) in the future. This is OK, since on irq fire
            // it is checked if the alarm time has passed.
            TIMER.alarm(n).write_value(timestamp as u32);

            let now = self.now();
            if timestamp <= now {
                // If alarm timestamp has passed the alarm will not fire.
                // Disarm the alarm and return `false` to indicate that.
                TIMER.armed().write(|w| w.set_armed(1 << n));

                alarm.timestamp.set(u64::MAX);

                false
            } else {
                true
            }
        })
    }

    fn check_alarm(&self) {
        let n = 0;
        critical_section::with(|cs| {
            let alarm = &self.alarms.borrow(cs);
            let timestamp = alarm.timestamp.get();
            if timestamp <= self.now() {
                self.trigger_alarm()
            } else {
                // Not elapsed, arm it again.
                // This can happen if it was set more than 2^32 us in the future.
                TIMER.alarm(n).write_value(timestamp as u32);
            }
        });

        // clear the irq
        TIMER.intr().write(|w| w.set_alarm(n, true));
    }

    fn trigger_alarm(&self) {
        TIMER_QUEUE_DRIVER.dispatch();
    }
}

/// safety: must be called exactly once at bootup
pub unsafe fn init() {
    // init alarms
    critical_section::with(|cs| {
        let alarm = DRIVER.alarms.borrow(cs);
        alarm.timestamp.set(u64::MAX);
    });

    // enable irq
    TIMER.inte().write(|w| {
        w.set_alarm(0, true);
    });
    #[cfg(feature = "rp2040")]
    {
        interrupt::TIMER_IRQ_0.enable();
    }
    #[cfg(feature = "_rp235x")]
    {
        interrupt::TIMER0_IRQ_0.enable();
    }
}

#[cfg(all(feature = "rt", feature = "rp2040"))]
#[interrupt]
fn TIMER_IRQ_0() {
    DRIVER.check_alarm()
}

#[cfg(all(feature = "rt", feature = "_rp235x"))]
#[interrupt]
fn TIMER0_IRQ_0() {
    DRIVER.check_alarm()
}

embassy_time_queue_driver::timer_queue_impl!(
    static TIMER_QUEUE_DRIVER: GlobalTimerQueue
        = GlobalTimerQueue::new(|next_expiration| DRIVER.set_alarm(next_expiration))
);
