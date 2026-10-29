//! Continuous 1 MHz LPTMR timekeeping with one-shot WAKETIMER alarms.
//!
//! LPTMR is never stopped while scheduling. Its compare cannot be freely
//! rewritten while running (MCXA2 35.3.6, MCXA5 40.3.6), so WAKETIMER handles
//! both queue deadlines and half-wrap maintenance with no periodic polling.
//! Alarms have approximately millisecond resolution; timestamps remain in us.

use core::cell::RefCell;
use core::task::Waker;

use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_time_driver::Driver;
use embassy_time_queue_utils::Queue;

use super::{Config, configure, read_counter};
use crate::clocks::{PoweredClock, with_clocks};
use crate::interrupt::InterruptExt;
use crate::pac::lptmr::{Ten, Tfc};
use crate::{interrupt, pac};

const HALF_WRAP: u64 = 1 << 31;
// The chip clocking tables specify 16.384 kHz, divided by the wake timer's
// 4-bit ripple counter (MCXA2 21.3.2/30.5.1.2, MCXA5 28.3.2/37.5.1.2).
const ALARM_HZ: u64 = 16_384 / 16;

fn alarm_ticks(delta: u64) -> u32 {
    // delta is bounded by HALF_WRAP before conversion, so this fits in u32.
    // Add a tick for the unknown phase of the continuously enabled divider.
    // Only the LPTMR timestamp decides when a queued waker is actually due.
    (delta * ALARM_HZ).div_ceil(1_000_000) as u32 + 1
}

fn stop_alarm() {
    // Mask, clear and halt before replacing the countdown. WAKE_FLAG is W1C;
    // use full writes rather than accidentally acknowledging it through RMW.
    // MCXA2 30.5.1.2-3; MCXA5 37.5.1.2-3.
    pac::WAKETIMER0.wake_timer_ctrl().write(|w| {
        w.set_osc_div_ena(true);
        w.set_clr_wake_timer(true);
        w.set_wake_flag(true);
    });
    // Count reads cross a clock domain. Observe the stopped counter with two
    // consecutive matching reads before writing a new start count.
    loop {
        let first = pac::WAKETIMER0.wake_timer_cnt().read().wake_cnt();
        let second = pac::WAKETIMER0.wake_timer_cnt().read().wake_cnt();
        if first == 0 && second == 0 {
            break;
        }
    }
    // An old expiry may have arrived while the clear crossed clock domains.
    pac::WAKETIMER0.wake_timer_ctrl().write(|w| {
        w.set_osc_div_ena(true);
        w.set_wake_flag(true);
    });
    interrupt::WAKETIMER0.unpend();
}

struct State {
    initialized: bool,
    time: u64,
    queue: Queue,
}

fn extend(previous: u64, counter: u32) -> u64 {
    previous + counter.wrapping_sub(previous as u32) as u64
}

impl State {
    fn now(&mut self) -> u64 {
        self.time = extend(self.time, read_counter());
        self.time
    }

    fn update_alarm(&mut self) {
        loop {
            stop_alarm();
            let now = self.now();
            let next = self.queue.next_expiration(now);
            // Queue processing and the stop handshake take time. Re-sample
            // before subtraction to avoid underflow for an expired target.
            let now = self.now();
            if next <= now {
                continue;
            }
            // Even an empty queue needs an observation well before a full
            // LPTMR wrap. Never convert the u64::MAX queue sentinel directly.
            let target = next.min(now.saturating_add(HALF_WRAP));
            let count = alarm_ticks(target - now);
            pac::WAKETIMER0.wake_timer_cnt().write(|w| w.set_wake_cnt(count));
            pac::WAKETIMER0.wake_timer_ctrl().write(|w| {
                w.set_osc_div_ena(true);
                w.set_intr_en(true);
                // Preserve an expiry that arrived after loading the count.
            });
            if target > self.now() {
                return;
            }
            // A deadline passed during programming. Drain it now and replace
            // the alarm instead of waiting for an obsolete countdown.
        }
    }
}

embassy_time_driver::time_driver_impl!(static DRIVER: LptmrTimeDriver = LptmrTimeDriver {
    state: Mutex::new(RefCell::new(State { initialized: false, time: 0, queue: Queue::new() })),
});

struct LptmrTimeDriver {
    state: Mutex<CriticalSectionRawMutex, RefCell<State>>,
}

impl Driver for LptmrTimeDriver {
    fn now(&self) -> u64 {
        critical_section::with(|cs| {
            let mut state = self.state.borrow(cs).borrow_mut();
            if state.initialized { state.now() } else { 0 }
        })
    }

    fn schedule_wake(&self, at: u64, waker: &Waker) {
        critical_section::with(|cs| {
            let mut state = self.state.borrow(cs).borrow_mut();
            if state.queue.schedule_wake(at, waker) {
                if state.initialized {
                    state.update_alarm();
                } else {
                    // Keep the pre-init behavior safe: no peripheral access.
                    // init() will arm any future deadlines already queued.
                    state.queue.next_expiration(0);
                }
            }
        });
    }
}

#[interrupt]
#[allow(non_snake_case)]
fn WAKETIMER0() {
    critical_section::with(|cs| {
        if !pac::WAKETIMER0.wake_timer_ctrl().read().wake_flag() {
            return;
        }
        let mut state = DRIVER.state.borrow(cs).borrow_mut();
        state.update_alarm();
    });
}

pub(crate) fn init(priority: interrupt::Priority) {
    interrupt::LPTMR0.disable();
    interrupt::WAKETIMER0.disable();
    let alarm_source = with_clocks(|clocks| {
        #[cfg(feature = "_mcxa2xx")]
        let frequency = clocks.ensure_clk_16k_vsys_active(&PoweredClock::AlwaysEnabled);
        #[cfg(feature = "_mcxa5xx")]
        let frequency = clocks.ensure_clk_16k_vbat_active(&PoweredClock::AlwaysEnabled);
        frequency
    })
    .expect("WAKETIMER time driver requires initialized clocks")
    .expect("WAKETIMER time driver requires FRO16K in its power domain");
    assert_eq!(alarm_source, 16_384);
    let parts = configure(Config {
        power: PoweredClock::AlwaysEnabled,
        ..Config::default()
    })
    .expect("LPTMR time driver requires an always-enabled 1 MHz clock and bus clock >=2 MHz");
    assert_eq!(parts.freq, 1_000_000);
    core::mem::forget(parts.wake_guard);
    pac::LPTMR0.cmr().write(|w| w.set_compare(u32::MAX));
    critical_section::with(|cs| {
        let mut state = DRIVER.state.borrow(cs).borrow_mut();
        state.time = 0;
        state.initialized = true;
        pac::LPTMR0.csr().write(|w| {
            w.set_tfc(Tfc::Tfc1);
            w.set_ten(Ten::Ten1);
        });
        state.update_alarm();
    });
    interrupt::LPTMR0.unpend();
    interrupt::WAKETIMER0.set_priority(priority);
    unsafe {
        interrupt::WAKETIMER0.enable();
    }
}
