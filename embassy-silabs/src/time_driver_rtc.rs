//! `embassy-time` driver on the Series 0 24-bit RTC (`time-driver-rtc`), at 32768 Hz on LFACLK.
//!
//! Series 0 TIMERs are 16-bit, and their power-of-two prescalers cannot make an exact 1 MHz tick
//! from HFRCO at 14 MHz or HFXO at 48 MHz.

use core::cell::{Cell, RefCell};
use core::sync::atomic::{AtomicU32, Ordering, compiler_fence};
use core::task::Waker;

use critical_section::CriticalSection;
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_time_driver::Driver;
use embassy_time_queue_utils::Queue;

use crate::interrupt::typelevel::Interrupt;
use crate::pac::{CMU, RTC};
use crate::peripherals;

/// Half the 24-bit counter range.
///
/// `period` counts half wraps, like the embassy-nrf RTC driver: the overflow (counter 0) and COMP1
/// (counter `HALF`) interrupts each add 1. Its parity gives the counter half, so `now()` stays
/// correct when it races an overflow.
const HALF: u32 = 0x80_0000;
/// 24-bit counter mask.
const MASK: u32 = 0xFF_FFFF;
/// Alarm distance below which the COMP0 interrupt is armed.
const ARM_AHEAD: u64 = 0xC0_0000;
/// Smallest distance between the counter and a COMP0 value.
///
/// A write takes up to 3 LFACLK cycles to synchronize, so a closer compare value can be missed.
/// COMP0, COMP1 and CTRL are in the LF clock domain, and a write while `SYNCBUSY` is set is lost,
/// so each write waits for `SYNCBUSY`.
const MIN_AHEAD: u64 = 3;

#[inline]
fn calc_now(period: u32, counter: u32) -> u64 {
    ((period as u64) << 23) + ((counter ^ ((period & 1) << 23)) as u64)
}

pub(crate) struct RtcDriver {
    period: AtomicU32,
    alarm: Mutex<CriticalSectionRawMutex, Cell<u64>>,
    queue: Mutex<CriticalSectionRawMutex, RefCell<Queue>>,
}

embassy_time_driver::time_driver_impl!(static DRIVER: RtcDriver = RtcDriver {
    period: AtomicU32::new(0),
    alarm: Mutex::new(Cell::new(u64::MAX)),
    queue: Mutex::new(RefCell::new(Queue::new())),
});

// Without `rt` there is no RTC interrupt handler to call these methods.
#[cfg_attr(not(feature = "rt"), allow(dead_code))]
impl RtcDriver {
    fn init(&'static self, _cs: CriticalSection, irq_prio: crate::interrupt::Priority) {
        // `cmu::init_clocks` has turned on the LE interface and LFACLK.
        crate::cmu::enable::<peripherals::RTC>();

        // The RTC runs on LFACLK / LFAPRESC0.RTC. The metapac kernel clock
        // of the RTC is LFACLK and does not include this prescaler, so set
        // it to 1.
        while CMU.syncbusy().read().lfapresc0() {}
        CMU.lfapresc0().modify(|w| w.set_rtc(crate::pac::cmu::vals::Rtc::Div1));
        let lfaclk = crate::cmu::frequency::<peripherals::RTC>().0 as u64;
        assert!(
            lfaclk == embassy_time_driver::TICK_HZ,
            "time-driver-rtc: LFACLK is not TICK_HZ (32768 Hz)"
        );

        // Stop the RTC. A disable and enable restarts the counter at 0.
        while RTC.syncbusy().read().ctrl() {}
        RTC.ctrl().write(|_| {});

        // COMP1 marks the half period, and COMP0 is the alarm.
        while RTC.syncbusy().read().comp1() {}
        RTC.comp1().write(|w| w.set_comp1(HALF));

        RTC.if_clr().write(|w| {
            w.set_of(true);
            w.set_comp0(true);
            w.set_comp1(true);
        });
        RTC.ien().write(|w| {
            w.set_of(true);
            w.set_comp1(true);
        });

        while RTC.syncbusy().read().ctrl() {}
        RTC.ctrl().write(|w| w.set_en(true));

        <crate::interrupt::typelevel::RTC as Interrupt>::unpend();
        <crate::interrupt::typelevel::RTC as Interrupt>::set_priority(irq_prio);
        unsafe { <crate::interrupt::typelevel::RTC as Interrupt>::enable() };
    }

    pub(crate) fn on_interrupt(&self) {
        let flags = RTC.if_().read();
        RTC.if_clr().write_value(flags);

        if flags.of() {
            self.next_period();
        }
        if flags.comp1() {
            self.next_period();
        }
        if flags.comp0() && RTC.ien().read().comp0() {
            critical_section::with(|cs| self.trigger_alarm(cs));
        }
    }

    fn next_period(&self) {
        critical_section::with(|cs| {
            // Only the ISR changes `period`, so a relaxed RMW is safe.
            let period = self.period.load(Ordering::Relaxed) + 1;
            self.period.store(period, Ordering::Relaxed);
            let t = (period as u64) << 23;

            let alarm = self.alarm.borrow(cs).get();
            if alarm < t + ARM_AHEAD {
                RTC.ien().modify(|w| w.set_comp0(true));
            }
        });
    }

    fn trigger_alarm(&self, cs: CriticalSection) {
        RTC.ien().modify(|w| w.set_comp0(false));
        self.alarm.borrow(cs).set(u64::MAX);

        let mut next = self.queue.borrow(cs).borrow_mut().next_expiration(self.now());
        while !self.set_alarm(cs, next) {
            next = self.queue.borrow(cs).borrow_mut().next_expiration(self.now());
        }
    }

    fn set_alarm(&self, cs: CriticalSection, timestamp: u64) -> bool {
        self.alarm.borrow(cs).set(timestamp);

        loop {
            let t = self.now();
            if timestamp <= t {
                RTC.ien().modify(|w| w.set_comp0(false));
                self.alarm.borrow(cs).set(u64::MAX);
                return false;
            }

            // Write COMP0 even for a far alarm, so `next_period` only has to arm the interrupt.
            let safe_timestamp = timestamp.max(t + MIN_AHEAD);
            RTC.if_clr().write(|w| w.set_comp0(true));
            while RTC.syncbusy().read().comp0() {}
            RTC.comp0().write(|w| w.set_comp0(safe_timestamp as u32 & MASK));

            if timestamp - t < ARM_AHEAD {
                RTC.ien().modify(|w| w.set_comp0(true));
                // The counter can reach the compare value while the write synchronizes, so try
                // again. An alarm can be late by a few ticks, but never early.
                if self.now() + MIN_AHEAD <= safe_timestamp {
                    return true;
                }
            } else {
                RTC.ien().modify(|w| w.set_comp0(false));
                return true;
            }
        }
    }
}

impl Driver for RtcDriver {
    fn now(&self) -> u64 {
        let period = self.period.load(Ordering::Relaxed);
        compiler_fence(Ordering::Acquire);
        let counter = RTC.cnt().read().cnt();
        calc_now(period, counter)
    }

    fn schedule_wake(&self, at: u64, waker: &Waker) {
        critical_section::with(|cs| {
            let mut queue = self.queue.borrow(cs).borrow_mut();
            if queue.schedule_wake(at, waker) {
                let mut next = queue.next_expiration(self.now());
                while !self.set_alarm(cs, next) {
                    next = queue.next_expiration(self.now());
                }
            }
        });
    }
}

pub(crate) fn init(cs: CriticalSection, irq_prio: crate::interrupt::Priority) {
    DRIVER.init(cs, irq_prio);
}

#[cfg(feature = "rt")]
pub(crate) const fn get_driver() -> &'static RtcDriver {
    &DRIVER
}
