//! `embassy-time` driver backed by the RTC (Real Time Counter) peripheral.
//!
//! The EFM32 RTC is a free-running 24-bit up-counter clocked by the `LFA` branch (the LFRCO or
//! LFXO, see [`crate::cmu::Config::lfa_source`]), with two compare channels (`COMP0`, `COMP1`) and an overflow interrupt. This is
//! extended to a full 64-bit, non-overflowing tick count using the same "half period" technique
//! `embassy-nrf`'s RTC-based driver uses for its (also 24-bit) `RTC1` peripheral: see
//! [`calc_now`] for the details. `COMP1` is dedicated to that period bookkeeping, so only
//! `COMP0` is available to applications as the single hardware alarm.

use core::cell::RefCell;
use core::sync::atomic::{AtomicU32, Ordering, compiler_fence};

use critical_section::CriticalSection;
use embassy_sync::blocking_mutex::CriticalSectionMutex as Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_time_driver::Driver;
use embassy_time_queue_utils::Queue;

use crate::interrupt::InterruptExt;
use crate::{interrupt, pac};

#[inline]
fn regs() -> &'static pac::rtc::RegisterBlock {
    unsafe { &*pac::RTC::ptr() }
}

/// Calculate the timestamp from the period count and the tick count.
///
/// See the equivalent function in `embassy-nrf`'s `time_driver.rs` for a full explanation of the
/// race-free "half period" technique this implements; the RTC here is likewise a 24-bit counter.
fn calc_now(period: u32, counter: u32) -> u64 {
    ((period as u64) << 23) + ((counter ^ ((period & 1) << 23)) as u64)
}

struct AlarmState {
    timestamp: core::cell::Cell<u64>,
}

unsafe impl Send for AlarmState {}

impl AlarmState {
    const fn new() -> Self {
        Self {
            timestamp: core::cell::Cell::new(u64::MAX),
        }
    }
}

struct RtcDriver {
    /// Number of 2^23 periods elapsed since boot.
    period: AtomicU32,
    /// Timestamp at which to fire the alarm. `u64::MAX` if no alarm is scheduled.
    alarm: Mutex<AlarmState>,
    queue: Mutex<RefCell<Queue>>,
}

embassy_time_driver::time_driver_impl!(static DRIVER: RtcDriver = RtcDriver {
    period: AtomicU32::new(0),
    alarm: Mutex::const_new(CriticalSectionRawMutex::new(), AlarmState::new()),
    queue: Mutex::new(RefCell::new(Queue::new())),
});

impl RtcDriver {
    fn init(&'static self, irq_prio: crate::interrupt::Priority) {
        let r = regs();

        // Disabling clears the counter, so it starts from 0 once enabled again, as `calc_now`
        // needs (reference manual, 21.3.1). With the RTC's immediate synchronization, this takes
        // effect right away. `CNT` is written as well, to not depend on that alone.
        r.ctrl.write(|w| w.en().clear_bit());
        r.cnt.write(|w| unsafe { w.cnt().bits(0) });
        // Halfway compare, used only to force a `next_period()` at the midpoint of the 24-bit
        // range so `now()` stays race-free across the full-range overflow too (see `calc_now`).
        r.comp1.write(|w| unsafe { w.comp1().bits(0x0080_0000) });

        r.ifc.write(|w| w.of().set_bit().comp0().set_bit().comp1().set_bit());
        r.ien.write(|w| w.of().set_bit().comp1().set_bit());

        r.ctrl.modify(|_, w| w.en().set_bit());

        interrupt::RTC.set_priority(irq_prio);
        unsafe { interrupt::RTC.enable() };
    }

    #[cfg(feature = "rt")]
    fn on_interrupt(&self) {
        let r = regs();
        let flags = r.if_.read();

        if flags.of().bit_is_set() {
            r.ifc.write(|w| w.of().set_bit());
            self.next_period();
        }
        if flags.comp1().bit_is_set() {
            r.ifc.write(|w| w.comp1().set_bit());
            self.next_period();
        }
        // `IF.COMP0` is also set by matches while the alarm is disarmed (e.g. still too far in the
        // future), so only act on it while armed.
        if flags.comp0().bit_is_set() && r.ien.read().comp0().bit_is_set() {
            r.ifc.write(|w| w.comp0().set_bit());
            critical_section::with(|cs| self.trigger_alarm(cs));
        }
    }

    #[cfg(feature = "rt")]
    fn next_period(&self) {
        critical_section::with(|cs| {
            let r = regs();
            let period = self.period.load(Ordering::Relaxed) + 1;
            self.period.store(period, Ordering::Relaxed);
            let t = (period as u64) << 23;

            let alarm = self.alarm.borrow(cs);
            let at = alarm.timestamp.get();

            if at < t + 0xc0_0000 {
                r.ien.modify(|_, w| w.comp0().set_bit());
            }
        })
    }

    #[cfg(feature = "rt")]
    fn trigger_alarm(&self, cs: CriticalSection) {
        regs().ien.modify(|_, w| w.comp0().clear_bit());

        let alarm = self.alarm.borrow(cs);
        alarm.timestamp.set(u64::MAX);

        // Call after clearing the alarm, so the callback can set another one.
        let mut next = self.queue.borrow(cs).borrow_mut().next_expiration(self.now());
        while !self.set_alarm(cs, next) {
            next = self.queue.borrow(cs).borrow_mut().next_expiration(self.now());
        }
    }

    fn set_alarm(&self, cs: CriticalSection, timestamp: u64) -> bool {
        let alarm = self.alarm.borrow(cs);
        alarm.timestamp.set(timestamp);

        let r = regs();

        loop {
            let t = self.now();
            if timestamp <= t {
                r.ien.modify(|_, w| w.comp0().clear_bit());
                alarm.timestamp.set(u64::MAX);
                return false;
            }

            // Never program a compare value within 3 ticks of "now": the RTC may not reliably
            // raise COMPARE if COMP0 is set to the current (or next) counter value. This mirrors
            // the same workaround `embassy-nrf`'s RTC1 driver uses for the same class of counter.
            // There's no need to wait for `SYNCBUSY.COMP0` before writing it again: the Giant Gecko
            // RTC has "immediate synchronization", so writes take effect right away and `SYNCBUSY`
            // stays 0 (reference manual, 5.3.1.1.2). Only the original Gecko family needs the wait
            // (see emlib's `RTC_CompareSet`).
            let safe_timestamp = timestamp.max(t + 3);
            r.comp0
                .write(|w| unsafe { w.comp0().bits((safe_timestamp as u32) & 0x00ff_ffff) });

            let diff = timestamp - t;
            if diff < 0xc0_0000 {
                r.ien.modify(|_, w| w.comp0().set_bit());
                if self.now() + 2 <= timestamp {
                    return true;
                }
                // Raced the timestamp while arming; retry.
            } else {
                // Too far in the future: `next_period()` will arm it once we're close enough.
                r.ien.modify(|_, w| w.comp0().clear_bit());
                return true;
            }
        }
    }
}

impl Driver for RtcDriver {
    fn now(&self) -> u64 {
        // `period` MUST be read before `counter`, see `calc_now` for details.
        let period = self.period.load(Ordering::Relaxed);
        compiler_fence(Ordering::Acquire);
        let counter = regs().cnt.read().cnt().bits();
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
fn RTC() {
    DRIVER.on_interrupt()
}

pub(crate) fn init(irq_prio: crate::interrupt::Priority) {
    DRIVER.init(irq_prio)
}

#[cfg(test)]
mod test {
    use super::calc_now;

    #[test]
    fn test_calc_now() {
        // Even period: the counter is in its lower half.
        assert_eq!(calc_now(0, 0x000000), 0x0_000000);
        assert_eq!(calc_now(0, 0x7FFFFF), 0x0_7FFFFF);
        assert_eq!(calc_now(2, 0x000000), 0x1_000000);
        assert_eq!(calc_now(2, 0x7FFFFF), 0x1_7FFFFF);
        // Odd period: the counter is in its upper half.
        assert_eq!(calc_now(1, 0x800000), 0x0_800000);
        assert_eq!(calc_now(1, 0xFFFFFF), 0x0_FFFFFF);
        // `now()` reads the period first, so the counter may already be past the halfway compare
        // or the overflow whose interrupt hasn't bumped the period yet.
        assert_eq!(calc_now(0, 0x800000), 0x0_800000);
        assert_eq!(calc_now(0, 0xFFFFFF), 0x0_FFFFFF);
        assert_eq!(calc_now(1, 0x000000), 0x1_000000);
        assert_eq!(calc_now(1, 0x7FFFFF), 0x1_7FFFFF);
        assert_eq!(calc_now(2, 0x800000), 0x1_800000);
    }
}
