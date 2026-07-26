//! Time driver.
use core::cell::{Cell, RefCell};
use core::sync::atomic::{AtomicU32, Ordering, compiler_fence};

use critical_section::CriticalSection;
use embassy_hal_internal::interrupt::InterruptExt;
use embassy_sync::blocking_mutex::CriticalSectionMutex as Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_time_driver::Driver;
use embassy_time_queue_utils::Queue;

use crate::interrupt;
use crate::pac::{CCT, ECIA};

/// Calculate the timestamp from the period count and the tick count.
///
/// The Input Capture and Compare Timer is a 32-bit free running timer
/// running at the main clock frequency.
///
/// We define a period to be 2^31 ticks, such that each overflow is 2
/// periods.
///
/// Toe get `now()`, `period` is read first, then `counter` is
/// read. If the counter value matches the expected range for the
/// `period` parity, we're done. If it doesn't, this mean that a new
/// period start has raced us between reading `period` and `counter`,
/// so we assume the `counter` value corresponds to the next period.
///
/// `period` is a 32-bit integer, it overlows on 2^32 * 2^31 /
/// 48_000_000 seconds of uptime, which is about 6093 years.
fn calc_now(period: u32, counter: u32) -> u64 {
    ((period as u64) << 31) + ((counter ^ ((period & 1) << 31)) as u64)
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

pub(crate) struct CctDriver {
    /// Number of 2^31 periods elapsed since boot.
    period: AtomicU32,

    /// Timestamp at which to fire alarm. `u64::MAX` if no alarm is
    /// scheduled.
    alarm: Mutex<AlarmState>,
    queue: Mutex<RefCell<Queue>>,
}

embassy_time_driver::time_driver_impl!(static DRIVER: CctDriver = CctDriver {
    period: AtomicU32::new(0),
    alarm: Mutex::const_new(CriticalSectionRawMutex::new(), AlarmState::new()),
    queue: Mutex::new(RefCell::new(Queue::new())),
});

impl CctDriver {
    fn init(&'static self) {
        interrupt::CCT.disable();
        interrupt::CCT_CMP0.disable();
        interrupt::CCT_CMP1.disable();

        interrupt::CCT.set_priority(interrupt::Priority::P0);
        interrupt::CCT_CMP0.set_priority(interrupt::Priority::P0);
        interrupt::CCT_CMP1.set_priority(interrupt::Priority::P0);

        ECIA.src18().write_value(1 << 20 | 1 << 27 | 1 << 28);
        ECIA.en_set18().write_value(1 << 20 | 1 << 27 | 1 << 28);

        // Reset timer
        CCT.ctrl().write(|w| w.set_free_rst(true));

        // Wait until reset is completed
        while CCT.ctrl().read().free_rst() {}

        // Mid
        CCT.comp0().write(|w| w.set_comp_0(0x8000_0000));

        CCT.ctrl().write(|w| {
            w.set_act(true);
            w.set_free_en(true);
            w.set_cmp_en0(true);
        });

        interrupt::CCT.unpend();
        interrupt::CCT_CMP0.unpend();
        interrupt::CCT_CMP1.unpend();
        unsafe {
            interrupt::CCT.enable();
            interrupt::CCT_CMP0.enable();
            interrupt::CCT_CMP1.enable();
        }
    }

    fn next_period(&self) {
        let period = self.period.load(Ordering::Relaxed) + 1;
        self.period.store(period, Ordering::Relaxed);
        let t = (period as u64) << 31;

        critical_section::with(move |cs| {
            let alarm = self.alarm.borrow(cs);
            let at = alarm.timestamp.get();

            if at < t + 0xc000_0000 {
                CCT.ctrl().modify(|w| w.set_cmp_en1(true));
            }
        });
    }

    fn set_alarm(&self, cs: CriticalSection, timestamp: u64) -> bool {
        self.alarm.borrow(cs).timestamp.set(timestamp);

        let t = self.now();
        if timestamp <= t {
            // If timestamp has passed, alarm will not fire.
            // Disarm it and return `false`.
            CCT.ctrl().modify(|w| w.set_cmp_en1(false));
            self.alarm.borrow(cs).timestamp.set(u64::MAX);
            return false;
        }

        CCT.comp1().write(|w| w.set_comp_1(timestamp as u32));

        let diff = timestamp - t;
        CCT.ctrl().modify(|w| w.set_cmp_en1(diff < 0xc000_0000));

        let t = self.now();
        if timestamp <= t {
            CCT.ctrl().modify(|w| w.set_cmp_en1(true));
            // If timestamp has passed, alarm will not fire.
            // Disarm it and return `false`.
            CCT.ctrl().modify(|w| w.set_cmp_en1(false));
            self.alarm.borrow(cs).timestamp.set(u64::MAX);
            return false;
        }

        true
    }

    fn trigger_alarm(&self, cs: CriticalSection) {
        let mut next = self.queue.borrow(cs).borrow_mut().next_expiration(self.now());
        while !self.set_alarm(cs, next) {
            next = self.queue.borrow(cs).borrow_mut().next_expiration(self.now());
        }
    }
}

impl Driver for CctDriver {
    fn now(&self) -> u64 {
        let period = self.period.load(Ordering::Relaxed);
        compiler_fence(Ordering::Acquire);
        let counter = CCT.free_run().read().tmr();
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

#[cfg(feature = "rt")]
#[interrupt]
fn CCT() {
    ECIA.src18().write_value(1 << 20);
    DRIVER.next_period();
}

#[cfg(feature = "rt")]
#[interrupt]
fn CCT_CMP0() {
    ECIA.src18().write_value(1 << 27);
    DRIVER.next_period();
}

#[cfg(feature = "rt")]
#[interrupt]
fn CCT_CMP1() {
    ECIA.src18().write_value(1 << 28);
    critical_section::with(|cs| DRIVER.trigger_alarm(cs));
}

pub(crate) fn init() {
    DRIVER.init()
}
