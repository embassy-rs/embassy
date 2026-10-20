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

enum AlarmOutcome {
    Armed,
    AlreadyDue,
    Maintenance,
}

/// A 42-bit binary counter value.
#[derive(Copy, Clone)]
struct EvTicks(u64);

impl EvTicks {
    // Select the hardware counter bits of a software timestamp.
    fn from_timestamp(timestamp: u64) -> Self {
        Self(timestamp & COUNTER_MASK)
    }

    fn value(self) -> u64 {
        self.0
    }

    fn to_gray(self) -> GrayTicks {
        GrayTicks(self.0 ^ (self.0 >> 1))
    }
}

/// A 42-bit Gray-coded counter value.
#[derive(Copy, Clone)]
struct GrayTicks(u64);

impl GrayTicks {
    fn from_registers(high: u16, low: u32) -> Self {
        Self(((u64::from(high) << 32) | u64::from(low)) & COUNTER_MASK)
    }

    fn low(self) -> u32 {
        self.0 as u32
    }

    fn high(self) -> u16 {
        (self.0 >> 32) as u16
    }

    fn decode(self) -> EvTicks {
        let mut binary = self.0;
        binary ^= binary >> 1;
        binary ^= binary >> 2;
        binary ^= binary >> 4;
        binary ^= binary >> 8;
        binary ^= binary >> 16;
        binary ^= binary >> 32;
        EvTicks(binary)
    }
}

embassy_time_driver::time_driver_impl!(static DRIVER: OsTimer = OsTimer {
    time: Mutex::new(Cell::new(0)),
    queue: Mutex::new(RefCell::new(Queue::new())),
});

struct OsTimer {
    time: Mutex<CriticalSectionRawMutex, Cell<u64>>,
    queue: Mutex<CriticalSectionRawMutex, RefCell<Queue>>,
}

impl OsTimer {
    fn init(&'static self, irq_prio: crate::interrupt::Priority) {
        critical_section::with(|cs| self.time.borrow(cs).set(0));

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

    fn program_match(&self, _cs: CriticalSection, deadline: u64) -> AlarmOutcome {
        // MCXA2 RM 29.3.2/29.6.1.8; MCXA5 RM 36.3.2/36.6.1.8.
        OSTIMER0.osevent_ctrl().modify(|w| {
            w.set_ostimer_intena(false);
            w.set_ostimer_intrflag(true);
        });
        while OSTIMER0.osevent_ctrl().read().match_wr_rdy() {}

        let gray_timestamp = EvTicks::from_timestamp(deadline).to_gray();
        OSTIMER0.match_l().write(|w| w.set_match_value(gray_timestamp.low()));
        OSTIMER0.match_h().write(|w| w.set_match_value(gray_timestamp.high()));

        // MATCH_H transfers the shadow pair to the active comparator.
        while OSTIMER0.osevent_ctrl().read().match_wr_rdy() {}
        if deadline <= self.now() {
            OSTIMER0.osevent_ctrl().modify(|w| w.set_ostimer_intena(false));
            return AlarmOutcome::AlreadyDue;
        }

        // Preserve a match latched before the enable write: INTRFLAG is W1C.
        OSTIMER0.osevent_ctrl().modify(|w| {
            w.set_ostimer_intrflag(false);
            w.set_ostimer_intena(true);
        });
        AlarmOutcome::Armed
    }

    fn set_alarm(&self, cs: CriticalSection, timestamp: u64) -> AlarmOutcome {
        // Keep the queue sentinel in software; idle queues still need wrap tracking.
        let maintenance = (self.now() | (PERIOD_TICKS - 1)).saturating_add(1);
        let target = timestamp.min(maintenance);
        if matches!(self.program_match(cs, target), AlarmOutcome::AlreadyDue) {
            return AlarmOutcome::AlreadyDue;
        }
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
            let high = OSTIMER0.evtimerh().read().evtimer_count_value();
            let low = OSTIMER0.evtimerl().read().evtimer_count_value();
            let counter = GrayTicks::from_registers(high, low).decode().value();
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
