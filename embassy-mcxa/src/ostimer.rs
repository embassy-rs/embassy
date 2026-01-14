//! Time Driver.
use core::cell::{Cell, RefCell};
use core::sync::atomic::{AtomicBool, Ordering};

use critical_section::CriticalSection;
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_time_driver::Driver;
use embassy_time_queue_utils::Queue;

use crate::clocks::periph_helpers::{OsTimerConfig, OstimerClockSel};
use crate::clocks::{PoweredClock, enable_and_reset};
use crate::interrupt;
use crate::interrupt::InterruptExt;
use crate::pac::OSTIMER0;
use crate::peripherals::OSTIMER0;

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

/// Convert gray to decimal
///
/// Os Event provides a 64-bit timestamp gray-encoded. All we have to
/// do here is read both 32-bit halves of the register and convert
/// from gray to regular binary.
fn gray_to_dec(gray: u64) -> u64 {
    let mut dec = gray;

    dec ^= dec >> 1;
    dec ^= dec >> 2;
    dec ^= dec >> 4;
    dec ^= dec >> 8;
    dec ^= dec >> 16;
    dec ^= dec >> 32;

    dec
}

/// Convert decimal to gray
///
/// Before writing match value to the target register, we must convert
/// it back into gray code.
fn dec_to_gray(dec: u64) -> u64 {
    let gray = dec;
    gray ^ (gray >> 1)
}

embassy_time_driver::time_driver_impl!(static DRIVER: OsTimer = OsTimer {
    alarms:  Mutex::const_new(CriticalSectionRawMutex::new(), AlarmState::new()),
    queue: Mutex::new(RefCell::new(Queue::new())),
});

struct OsTimer {
    /// Timestamp at which to fire alarm. u64::MAX if no alarm is scheduled.
    alarms: Mutex<CriticalSectionRawMutex, AlarmState>,
    queue: Mutex<CriticalSectionRawMutex, RefCell<Queue>>,
}

impl OsTimer {
    fn init(&'static self, irq_prio: crate::interrupt::Priority) {
        // init alarms
        critical_section::with(|cs| {
            let alarm = DRIVER.alarms.borrow(cs);
            alarm.timestamp.set(u64::MAX);
        });

        let parts = unsafe {
            enable_and_reset::<OSTIMER0>(&OsTimerConfig {
                power: PoweredClock::AlwaysEnabled,
                source: OstimerClockSel::Clk1M,
            })
            .expect("Enabling OsTimer clock should not fail")
        };

        // Currently does nothing as Clk1M is always enabled anyway, this is here
        // to make sure that doesn't change in a refactoring.
        core::mem::forget(parts.wake_guard);

        interrupt::OS_EVENT.disable();

        // Make sure interrupt is masked
        OSTIMER0.osevent_ctrl().modify(|w| w.set_ostimer_intena(false));

        // Default to the end of time
        OSTIMER0.match_l().write(|w| w.set_match_value(u32::MAX));
        OSTIMER0.match_h().write(|w| w.set_match_value(u16::MAX));

        interrupt::OS_EVENT.unpend();
        interrupt::OS_EVENT.set_priority(irq_prio);
        unsafe { interrupt::OS_EVENT.enable() };
    }

    fn set_alarm(&self, cs: CriticalSection, timestamp: u64) -> bool {
        let alarm = self.alarms.borrow(cs);
        alarm.timestamp.set(timestamp);

        // Wait until we're allowed to write to MATCH_L/MATCH_H registers
        while OSTIMER0.osevent_ctrl().read().match_wr_rdy() {}

        let t = self.now();
        if timestamp <= t {
            OSTIMER0.osevent_ctrl().modify(|w| w.set_ostimer_intena(false));
            alarm.timestamp.set(u64::MAX);
            return false;
        }

        let gray_timestamp = dec_to_gray(timestamp);

        OSTIMER0
            .match_l()
            .write(|w| w.set_match_value(gray_timestamp as u32 & 0xffff_ffff));
        OSTIMER0
            .match_h()
            .write(|w| w.set_match_value((gray_timestamp >> 32) as u16));
        OSTIMER0.osevent_ctrl().modify(|w| w.set_ostimer_intena(true));

        true
    }

    fn trigger_alarm(&self, cs: CriticalSection) {
        let mut next = self.queue.borrow(cs).borrow_mut().next_expiration(self.now());
        while !self.set_alarm(cs, next) {
            next = self.queue.borrow(cs).borrow_mut().next_expiration(self.now());
        }
    }

    fn on_interrupt(&self) {
        crate::perf_counters::incr_interrupt_ostimer();
        critical_section::with(|cs| {
            if OSTIMER0.osevent_ctrl().read().ostimer_intrflag() {
                OSTIMER0.osevent_ctrl().modify(|w| {
                    w.set_ostimer_intena(false);
                    w.set_ostimer_intrflag(true)
                });
                crate::perf_counters::incr_interrupt_ostimer_alarm();
                self.trigger_alarm(cs);
            }
        });
    }
}

static INIT: AtomicBool = AtomicBool::new(false);

impl Driver for OsTimer {
    fn now(&self) -> u64 {
        // Don't try to read the timer before the OsTimer is actually enabled.
        // This leads to faults on the MCX-A.
        if !INIT.load(Ordering::Relaxed) {
            return 0;
        }

        let mut t = OSTIMER0.evtimerh().read().0 as u64;
        t <<= 32;
        t |= OSTIMER0.evtimerl().read().evtimer_count_value() as u64;
        gray_to_dec(t)
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

#[allow(non_snake_case)]
#[interrupt]
fn OS_EVENT() {
    DRIVER.on_interrupt()
}

pub(crate) fn init(irq_prio: crate::interrupt::Priority) {
    DRIVER.init(irq_prio);
    INIT.store(true, Ordering::Relaxed);
}
