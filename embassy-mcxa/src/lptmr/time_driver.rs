//! Free-running LPTMR time backend. A periodic compare is necessary because
//! CMR can only be rewritten with TEN=0 or TCF=1 (MCXA2 35.3.6, MCXA5 40.3.6).
//! Poll the queue every 1 ms; never reset the counter while scheduling alarms.

use core::cell::RefCell;
use core::task::Waker;

use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_time_driver::Driver;
use embassy_time_queue_utils::Queue;

use super::{Config, configure, read_counter};
use crate::clocks::PoweredClock;
use crate::interrupt::InterruptExt;
use crate::pac::lptmr::{Ten, Tfc, Tie};
use crate::{interrupt, pac};

const INTERVAL: u32 = 1_000;

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
            state.queue.schedule_wake(at, waker);
            // Expired/past timestamps are processed immediately. Future
            // timestamps are checked on the next periodic compare interrupt.
            let now = if state.initialized { state.now() } else { 0 };
            state.queue.next_expiration(now);
        });
    }
}

#[interrupt]
#[allow(non_snake_case)]
fn LPTMR0() {
    critical_section::with(|cs| {
        if !pac::LPTMR0.csr().read().tcf() {
            return;
        }
        let mut state = DRIVER.state.borrow(cs).borrow_mut();
        let now = state.now();
        state.queue.next_expiration(now);

        // Keep TCF set until CMR is updated. Sample after queue processing so
        // late IRQs/large queues cannot program a compare already in the past.
        // Allow a full millisecond to finish the few MMIO writes below. The
        // counter stops on debug halt; interrupts are masked by this section.
        let mut programmed = false;
        for _ in 0..4 {
            let counter = read_counter();
            pac::LPTMR0
                .cmr()
                .write(|w| w.set_compare(counter.wrapping_add(INTERVAL - 1)));
            if read_counter().wrapping_sub(counter) < INTERVAL / 2 {
                programmed = true;
                break;
            }
        }
        assert!(programmed, "LPTMR compare programming exceeded its timing margin");
        pac::LPTMR0.csr().modify(|w| w.set_tcf(true));
    });
}

pub(crate) fn init(priority: interrupt::Priority) {
    interrupt::LPTMR0.disable();
    let parts = configure(Config {
        power: PoweredClock::AlwaysEnabled,
        ..Config::default()
    })
    .expect("LPTMR time driver requires an always-enabled 1 MHz clock and bus clock >=2 MHz");
    assert_eq!(parts.freq, 1_000_000);
    core::mem::forget(parts.wake_guard);
    pac::LPTMR0.cmr().write(|w| w.set_compare(INTERVAL - 1));
    critical_section::with(|cs| {
        let mut state = DRIVER.state.borrow(cs).borrow_mut();
        state.time = 0;
        state.initialized = true;
        pac::LPTMR0.csr().write(|w| {
            w.set_tfc(Tfc::Tfc1);
            w.set_tie(Tie::Tie1);
            w.set_ten(Ten::Ten1);
        });
    });
    interrupt::LPTMR0.unpend();
    interrupt::LPTMR0.set_priority(priority);
    unsafe {
        interrupt::LPTMR0.enable();
    }
}
