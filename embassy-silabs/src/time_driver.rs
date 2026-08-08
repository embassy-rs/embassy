//! `embassy-time` driver backed by a TIMER peripheral.

use core::cell::{Cell, RefCell};
use core::sync::atomic::{AtomicU32, Ordering, compiler_fence};
use core::task::Waker;

use critical_section::CriticalSection;
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_time_driver::Driver;
use embassy_time_queue_utils::Queue;

use crate::interrupt::typelevel::Interrupt;
// TIMER0 routes to a different register-block version per Series 2 config:
// MG22 (config 2) uses timer_v0_w; configs 4/5/6 use timer_v1_w.
#[cfg(silabs_series_2_config = "2")]
use crate::pac::timer_v0_w as timer_mod;
#[cfg(not(silabs_series_2_config = "2"))]
use crate::pac::timer_v1_w as timer_mod;
use crate::peripherals;

mod sealed {
    pub trait Instance {}
}

pub(crate) trait Instance: sealed::Instance + 'static {
    type Interrupt: crate::interrupt::typelevel::Interrupt;
    fn regs() -> *mut ();
}

macro_rules! impl_timer_instance {
    ($name:ident) => {
        impl sealed::Instance for peripherals::$name {}
        impl Instance for peripherals::$name {
            type Interrupt = crate::interrupt::typelevel::$name;
            fn regs() -> *mut () {
                crate::pac::$name.as_ptr()
            }
        }
    };
}

impl_timer_instance!(TIMER0);
impl_timer_instance!(TIMER1);

#[cfg(time_driver_timer0)]
type T = peripherals::TIMER0;
#[cfg(time_driver_timer1)]
type T = peripherals::TIMER1;

fn regs() -> timer_mod::Timer {
    unsafe { timer_mod::Timer::from_ptr(<T as Instance>::regs()) }
}

// Clock timekeeping works in "periods" — time intervals of 2^31 ticks.
// The Clock counter is 32 bits wide, so one overflow cycle spans two periods.
//
// A `period` count is maintained in parallel to the Timer hardware `counter`, like this:
// - `period` and `counter` start at 0
// - `period` is incremented on overflow (at counter value 0)
// - `period` is incremented "midway" between overflows (at counter value 0x8000_0000)
//
// Therefore, when `period` is even, counter is in 0..0x7FFF_FFFF. When odd, counter is in
// 0x8000_0000..0xFFFF_FFFF. This allows now() to return the correct value even if it races
// an overflow.
//
// `period` is a 32-bit integer, so it overflows on 2^32 * 2^31 / TICK_HZ seconds of uptime,
// which at 1 MHz is ~290 000 years.
const HALF: u32 = 0x8000_0000;

/// Window inside which an alarm's CC1 interrupt should already be armed.
const ARM_AHEAD: u64 = 0xC000_0000;

#[inline]
fn calc_now(period: u32, counter: u32) -> u64 {
    ((period as u64) << 31) + ((counter ^ ((period & 1) << 31)) as u64)
}

pub(crate) struct TimerDriver {
    period: AtomicU32,
    alarm: Mutex<CriticalSectionRawMutex, Cell<u64>>,
    queue: Mutex<CriticalSectionRawMutex, RefCell<Queue>>,
}

embassy_time_driver::time_driver_impl!(static DRIVER: TimerDriver = TimerDriver {
    period: AtomicU32::new(0),
    alarm: Mutex::new(Cell::new(u64::MAX)),
    queue: Mutex::new(RefCell::new(Queue::new())),
});

impl TimerDriver {
    fn init(&'static self, _cs: CriticalSection) {
        let t = regs();

        // Series 2 TIMER register-access rules:
        //  - `EN` is always accessible.
        //  - `CFG` and `CCx_CFG` are writable only when `EN.EN = 0`.
        //  - Every other register (CTRL/CNT/TOP/CMD/CCx_OC/IEN/IF) requires
        //    `EN.EN = 1`. Touching them with EN=0 raises a BusFault.
        t.en().write(|w| w.set_en(false));
        // timer_v0_w (MG22) has no DISABLING status bit — disable is immediate.
        #[cfg(not(silabs_series_2_config = "2"))]
        while t.en().read().disabling() {}

        // TIMER0 lives on EM01GRPACLK. The source is configurable
        // (HFRCODPLL at reset, HFXO once the clock tree is reprogrammed,
        // etc.) and the rate is recorded in `rcc::get_freqs().em01grpaclk`.
        // Pick PRESC = (em01grpaclk / tick_hz) - 1 so the timer counts
        // at exactly `embassy_time_driver::TICK_HZ`.
        //
        // PRESC encoding: divisor = value + 1. The field is 10 bits, so
        // divisor must fit in 1..=1024.
        let em01grpaclk = unsafe { crate::rcc::get_freqs().em01grpaclk }.0 as u64;
        let tick_hz = embassy_time_driver::TICK_HZ;
        assert!(
            tick_hz != 0 && em01grpaclk % tick_hz == 0,
            "EM01GRPACLK is not an integer multiple of TICK_HZ"
        );
        let divisor = em01grpaclk / tick_hz;
        assert!((1..=1024).contains(&divisor), "TIMER0 PRESC out of range");
        t.cfg().write(|w| {
            w.set_presc(timer_mod::vals::Presc::from_bits((divisor - 1) as u16));
        });

        // CC0 is the half-overflow marker driving the period extension.
        t.cc0_cfg().write(|w| {
            w.set_mode(timer_mod::vals::Cc0CfgMode::Outputcompare);
        });

        // CC1 is the alarm slot. OC value is written per-alarm by `set_alarm`.
        t.cc1_cfg().write(|w| {
            w.set_mode(timer_mod::vals::Cc1CfgMode::Outputcompare);
        });

        t.en().write(|w| w.set_en(true));

        t.cmd().write(|w| w.set_stop(true));
        t.cnt().write(|w| w.set_cnt(0));
        // TOP = 2*HALF - 1 so that OF and CC0 alternate, period bookkeeping stays valid.
        // For HALF = 0x8000_0000 this is u32::MAX (the full 32-bit range).
        t.top().write(|w| w.set_top(HALF.wrapping_mul(2).wrapping_sub(1)));
        t.cc0_oc().write(|w| w.set_oc(HALF));

        let stale = t.if_().read();
        t.if_clr().write_value(stale);

        t.ien().write(|w| {
            w.set_of(true);
            w.set_cc0(true);
        });

        <T as Instance>::Interrupt::unpend();
        unsafe { <T as Instance>::Interrupt::enable() };

        t.cmd().write(|w| w.set_start(true));
    }

    pub(crate) fn on_interrupt(&self) {
        critical_section::with(|cs| {
            let t = regs();
            let flags = t.if_().read();
            t.if_clr().write_value(flags);

            if flags.of() {
                self.next_period();
            }
            if flags.cc0() {
                self.next_period();
            }
            if flags.cc1() && t.ien().read().cc1() {
                self.trigger_alarm(cs);
            }
        });
    }

    fn next_period(&self) {
        // Only the ISR mutates `period`, so a relaxed RMW is safe.
        let period = self.period.load(Ordering::Relaxed) + 1;
        self.period.store(period, Ordering::Relaxed);
        let lo = (period as u64) << 31;

        critical_section::with(|cs| {
            let alarm = self.alarm.borrow(cs).get();
            if alarm < lo + ARM_AHEAD {
                regs().ien_set().write(|w| w.set_cc1(true));
            } else {
                regs().ien_clr().write(|w| w.set_cc1(true));
            }
        });
    }

    fn trigger_alarm(&self, cs: CriticalSection) {
        let mut next = self.queue.borrow(cs).borrow_mut().next_expiration(self.now_inner());
        while !self.set_alarm(cs, next) {
            next = self.queue.borrow(cs).borrow_mut().next_expiration(self.now_inner());
        }
    }

    fn set_alarm(&self, cs: CriticalSection, timestamp: u64) -> bool {
        let t = regs();
        self.alarm.borrow(cs).set(timestamp);

        let now = self.now_inner();
        if timestamp <= now {
            t.ien_clr().write(|w| w.set_cc1(true));
            self.alarm.borrow(cs).set(u64::MAX);
            return false;
        }

        t.cc1_oc().write(|w| w.set_oc(timestamp as u32));

        let diff = timestamp - now;
        if diff < ARM_AHEAD {
            t.ien_set().write(|w| w.set_cc1(true));
        } else {
            t.ien_clr().write(|w| w.set_cc1(true));
        }

        // Re-check in case the counter raced past while CC1 was being written.
        let now2 = self.now_inner();
        if timestamp <= now2 {
            t.ien_clr().write(|w| w.set_cc1(true));
            self.alarm.borrow(cs).set(u64::MAX);
            return false;
        }

        true
    }

    #[inline]
    fn now_inner(&self) -> u64 {
        let period = self.period.load(Ordering::Relaxed);
        compiler_fence(Ordering::Acquire);
        let counter = regs().cnt().read().cnt();
        calc_now(period, counter)
    }
}

impl Driver for TimerDriver {
    fn now(&self) -> u64 {
        self.now_inner()
    }

    fn schedule_wake(&self, at: u64, waker: &Waker) {
        critical_section::with(|cs| {
            let mut queue = self.queue.borrow(cs).borrow_mut();
            if queue.schedule_wake(at, waker) {
                let mut next = queue.next_expiration(self.now_inner());
                while !self.set_alarm(cs, next) {
                    next = queue.next_expiration(self.now_inner());
                }
            }
        });
    }
}

pub(crate) fn init(cs: CriticalSection) {
    DRIVER.init(cs);
}

#[cfg(feature = "rt")]
pub(crate) const fn get_driver() -> &'static TimerDriver {
    &DRIVER
}
