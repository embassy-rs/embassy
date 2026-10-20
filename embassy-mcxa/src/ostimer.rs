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

const COUNTER_MASK: u64 = (1 << 42) - 1;
// Half-wrap checkpoints leave a half-cycle margin for interrupt latency.
const PERIOD_TICKS: u64 = 1 << 41;

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

enum AlarmOutcome {
    Armed,
    AlreadyDue,
    Maintenance,
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
    time: Mutex::new(Cell::new(0)),
    alarms:  Mutex::const_new(CriticalSectionRawMutex::new(), AlarmState::new()),
    queue: Mutex::new(RefCell::new(Queue::new())),
});

struct OsTimer {
    time: Mutex<CriticalSectionRawMutex, Cell<u64>>,
    /// Timestamp at which to fire alarm. u64::MAX if no alarm is scheduled.
    alarms: Mutex<CriticalSectionRawMutex, AlarmState>,
    queue: Mutex<CriticalSectionRawMutex, RefCell<Queue>>,
}

impl OsTimer {
    fn init(&'static self, irq_prio: crate::interrupt::Priority) {
        // init alarms
        critical_section::with(|cs| {
            self.time.borrow(cs).set(0);
            let alarm = DRIVER.alarms.borrow(cs);
            alarm.timestamp.set(u64::MAX);
        });

        let parts = unsafe {
            enable_and_reset::<OSTIMER0>(&OsTimerConfig {
                power: PoweredClock::AlwaysEnabled,
                // Split counter reads require bus >= 2 MHz, or the same 1 MHz source.
                // MCXA2 RM 29.6.1.2; MCXA5 RM 36.6.1.2.
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

        interrupt::OS_EVENT.unpend();
        interrupt::OS_EVENT.set_priority(irq_prio);
        unsafe { interrupt::OS_EVENT.enable() };
    }

    fn set_alarm(&self, cs: CriticalSection, timestamp: u64) -> AlarmOutcome {
        let alarm = self.alarms.borrow(cs);
        alarm.timestamp.set(timestamp);

        // Disable the previous alarm and clear a potentially latched match before
        // replacing MATCH_L/MATCH_H.
        OSTIMER0.osevent_ctrl().modify(|w| {
            w.set_ostimer_intena(false);
            w.set_ostimer_intrflag(true);
        });

        // Wait until any previous match-register synchronization completes.
        while OSTIMER0.osevent_ctrl().read().match_wr_rdy() {}

        // Keep the queue sentinel in software; idle queues still need wrap tracking.
        let maintenance = (self.now() | (PERIOD_TICKS - 1)).saturating_add(1);
        let target = timestamp.min(maintenance);
        let gray_timestamp = dec_to_gray(target & COUNTER_MASK);

        OSTIMER0.match_l().write(|w| w.set_match_value(gray_timestamp as u32));
        OSTIMER0
            .match_h()
            .write(|w| w.set_match_value((gray_timestamp >> 32) as u16));

        // Writing MATCH_H starts the transfer of the pair from the shadow registers into the
        // active comparator. MATCH_WR_RDY reads 1 until that transfer completes, so wait for
        // it to clear before sampling the timer. See section 36.6.1.8 of the MCXA5 Reference
        // Manual for details
        while OSTIMER0.osevent_ctrl().read().match_wr_rdy() {}

        let t = self.now();
        if target <= t {
            OSTIMER0.osevent_ctrl().modify(|w| w.set_ostimer_intena(false));
            alarm.timestamp.set(u64::MAX);
            return AlarmOutcome::AlreadyDue;
        }

        // INTRFLAG is write-one-to-clear. Explicitly write zero while enabling
        // the alarm so a match that races this register update remains latched.
        OSTIMER0.osevent_ctrl().modify(|w| {
            w.set_ostimer_intrflag(false);
            w.set_ostimer_intena(true);
        });
        if target == timestamp {
            AlarmOutcome::Armed
        } else {
            AlarmOutcome::Maintenance
        }
    }

    fn trigger_alarm(&self, cs: CriticalSection) {
        let mut next = self.queue.borrow(cs).borrow_mut().next_expiration(self.now());
        while matches!(self.set_alarm(cs, next), AlarmOutcome::AlreadyDue) {
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

        // Keep interrupts from delaying the low-half read across multiple ticks.
        critical_section::with(|cs| {
            let high = OSTIMER0.evtimerh().read().evtimer_count_value() as u64;
            let low = OSTIMER0.evtimerl().read().evtimer_count_value() as u64;
            let counter = gray_to_dec((high << 32) | low);
            let time = self.time.borrow(cs);
            let previous = time.get();
            let now = previous + (counter.wrapping_sub(previous) & COUNTER_MASK);
            time.set(now);
            now
        })
    }

    fn schedule_wake(&self, at: u64, waker: &core::task::Waker) {
        critical_section::with(|cs| {
            let mut queue = self.queue.borrow(cs).borrow_mut();

            if queue.schedule_wake(at, waker) {
                let mut next = queue.next_expiration(self.now());
                while matches!(self.set_alarm(cs, next), AlarmOutcome::AlreadyDue) {
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
    critical_section::with(|cs| {
        INIT.store(true, Ordering::Relaxed);
        DRIVER.trigger_alarm(cs);
    });
}
