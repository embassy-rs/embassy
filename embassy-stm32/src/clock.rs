#![macro_use]

use core::cell::Cell;
use core::convert::TryInto;
use core::sync::atomic::{compiler_fence, Ordering};

use atomic_polyfill::AtomicU32;
use embassy::interrupt::InterruptExt;
use embassy::time::{Clock as EmbassyClock, TICKS_PER_SECOND};

use crate::interrupt::{CriticalSection, Interrupt, Mutex};
use crate::pac::timer::TimGp16;
use crate::time::Hertz;

// Clock timekeeping works with something we call "periods", which are time intervals
// of 2^15 ticks. The Clock counter value is 16 bits, so one "overflow cycle" is 2 periods.
//
// A `period` count is maintained in parallel to the Timer hardware `counter`, like this:
// - `period` and `counter` start at 0
// - `period` is incremented on overflow (at counter value 0)
// - `period` is incremented "midway" between overflows (at counter value 0x8000)
//
// Therefore, when `period` is even, counter is in 0..0x7FFF. When odd, counter is in 0x8000..0xFFFF
// This allows for now() to return the correct value even if it races an overflow.
//
// To get `now()`, `period` is read first, then `counter` is read. If the counter value matches
// the expected range for the `period` parity, we're done. If it doesn't, this means that
// a new period start has raced us between reading `period` and `counter`, so we assume the `counter` value
// corresponds to the next period.
//
// `period` is a 32bit integer, so It overflows on 2^32 * 2^15 / 32768 seconds of uptime, which is 136 years.
fn calc_now(period: u32, counter: u16) -> u64 {
    ((period as u64) << 15) + ((counter as u32 ^ ((period & 1) << 15)) as u64)
}

struct AlarmState {
    timestamp: Cell<u64>,
    #[allow(clippy::type_complexity)]
    callback: Cell<Option<(fn(*mut ()), *mut ())>>,
}

impl AlarmState {
    fn new() -> Self {
        Self {
            timestamp: Cell::new(u64::MAX),
            callback: Cell::new(None),
        }
    }
}

const ALARM_COUNT: usize = 3;

/// Clock timer that can be used by the executor and to set alarms.
///
/// It can work with Timers 2, 3, 4, 5. This timer works internally with a unit of 2^15 ticks, which
/// means that if a call to [`embassy::time::Clock::now`] is blocked for that amount of ticks the
/// returned value will be wrong (an old value). The current default tick rate is 32768 ticks per
/// second.
pub struct Clock<T: Instance> {
    _inner: T,
    irq: T::Interrupt,
    /// Clock frequency
    frequency: Hertz,
    /// Number of 2^23 periods elapsed since boot.
    period: AtomicU32,
    /// Timestamp at which to fire alarm. u64::MAX if no alarm is scheduled.
    alarms: Mutex<[AlarmState; ALARM_COUNT]>,
}

impl<T: Instance> Clock<T> {
    pub fn new(peripheral: T, irq: T::Interrupt, frequency: Hertz) -> Self {
        Self {
            _inner: peripheral,
            irq,
            frequency,
            period: AtomicU32::new(0),
            alarms: Mutex::new([AlarmState::new(), AlarmState::new(), AlarmState::new()]),
        }
    }

    pub fn start(&'static self) {
        let inner = T::inner();

        // NOTE(unsafe) Critical section to use the unsafe methods
        critical_section::with(|_| {
            unsafe {
                inner.prepare(self.frequency);
            }

            self.irq.set_handler_context(self as *const _ as *mut _);
            self.irq.set_handler(|ptr| unsafe {
                let this = &*(ptr as *const () as *const Self);
                this.on_interrupt();
            });
            self.irq.unpend();
            self.irq.enable();

            unsafe {
                inner.start_counter();
            }
        })
    }

    fn on_interrupt(&self) {
        let inner = T::inner();

        // NOTE(unsafe) Use critical section to access the methods
        // XXX: reduce the size of this critical section ?
        critical_section::with(|cs| unsafe {
            if inner.overflow_interrupt_status() {
                inner.overflow_clear_flag();
                self.next_period();
            }

            // Half overflow
            if inner.compare_interrupt_status(0) {
                inner.compare_clear_flag(0);
                self.next_period();
            }

            for n in 1..=ALARM_COUNT {
                if inner.compare_interrupt_status(n) {
                    inner.compare_clear_flag(n);
                    self.trigger_alarm(n, cs);
                }
            }
        })
    }

    fn next_period(&self) {
        let inner = T::inner();

        let period = self.period.fetch_add(1, Ordering::Relaxed) + 1;
        let t = (period as u64) << 15;

        critical_section::with(move |cs| {
            for n in 1..=ALARM_COUNT {
                let alarm = &self.alarms.borrow(cs)[n - 1];
                let at = alarm.timestamp.get();

                let diff = at - t;
                if diff < 0xc000 {
                    inner.set_compare(n, at as u16);
                    // NOTE(unsafe) We're in a critical section
                    unsafe {
                        inner.set_compare_interrupt(n, true);
                    }
                }
            }
        })
    }

    fn trigger_alarm(&self, n: usize, cs: CriticalSection) {
        let inner = T::inner();
        // NOTE(unsafe) We have a critical section
        unsafe {
            inner.set_compare_interrupt(n, false);
        }

        let alarm = &self.alarms.borrow(cs)[n - 1];
        alarm.timestamp.set(u64::MAX);

        // Call after clearing alarm, so the callback can set another alarm.
        if let Some((f, ctx)) = alarm.callback.get() {
            f(ctx);
        }
    }

    fn set_alarm_callback(&self, n: usize, callback: fn(*mut ()), ctx: *mut ()) {
        critical_section::with(|cs| {
            let alarm = &self.alarms.borrow(cs)[n - 1];
            alarm.callback.set(Some((callback, ctx)));
        })
    }

    fn set_alarm(&self, n: usize, timestamp: u64) {
        critical_section::with(|cs| {
            let inner = T::inner();

            let alarm = &self.alarms.borrow(cs)[n - 1];
            alarm.timestamp.set(timestamp);

            let t = self.now();
            if timestamp <= t {
                self.trigger_alarm(n, cs);
                return;
            }

            let diff = timestamp - t;
            if diff < 0xc000 {
                let safe_timestamp = timestamp.max(t + 3);
                inner.set_compare(n, safe_timestamp as u16);

                // NOTE(unsafe) We're in a critical section
                unsafe {
                    inner.set_compare_interrupt(n, true);
                }
            } else {
                unsafe {
                    inner.set_compare_interrupt(n, false);
                }
            }
        })
    }

    pub fn alarm1(&'static self) -> Alarm<T> {
        Alarm { n: 1, rtc: self }
    }
    pub fn alarm2(&'static self) -> Alarm<T> {
        Alarm { n: 2, rtc: self }
    }
    pub fn alarm3(&'static self) -> Alarm<T> {
        Alarm { n: 3, rtc: self }
    }
}

impl<T: Instance> EmbassyClock for Clock<T> {
    fn now(&self) -> u64 {
        let inner = T::inner();

        let period = self.period.load(Ordering::Relaxed);
        compiler_fence(Ordering::Acquire);
        let counter = inner.counter();
        calc_now(period, counter)
    }
}

pub struct Alarm<T: Instance> {
    n: usize,
    rtc: &'static Clock<T>,
}

impl<T: Instance> embassy::time::Alarm for Alarm<T> {
    fn set_callback(&self, callback: fn(*mut ()), ctx: *mut ()) {
        self.rtc.set_alarm_callback(self.n, callback, ctx);
    }

    fn set(&self, timestamp: u64) {
        self.rtc.set_alarm(self.n, timestamp);
    }

    fn clear(&self) {
        self.rtc.set_alarm(self.n, u64::MAX);
    }
}

pub struct TimerInner(pub(crate) TimGp16);

impl TimerInner {
    unsafe fn prepare(&self, timer_freq: Hertz) {
        self.stop_and_reset();

        let psc = timer_freq.0 / TICKS_PER_SECOND as u32 - 1;
        let psc: u16 = psc.try_into().unwrap();

        self.set_psc_arr(psc, u16::MAX);
        // Mid-way point
        self.set_compare(0, 0x8000);
        self.set_compare_interrupt(0, true);
    }

    unsafe fn start_counter(&self) {
        self.0.cr1().modify(|w| w.set_cen(true));
    }

    unsafe fn stop_and_reset(&self) {
        let regs = self.0;

        regs.cr1().modify(|w| w.set_cen(false));
        regs.cnt().write(|w| w.set_cnt(0));
    }

    fn overflow_interrupt_status(&self) -> bool {
        // NOTE(unsafe) Atomic read with no side-effects
        unsafe { self.0.sr().read().uif() }
    }

    unsafe fn overflow_clear_flag(&self) {
        self.0.sr().modify(|w| w.set_uif(false));
    }

    unsafe fn set_psc_arr(&self, psc: u16, arr: u16) {
        use crate::pac::timer::vals::Urs;

        let regs = self.0;

        regs.psc().write(|w| w.set_psc(psc));
        regs.arr().write(|w| w.set_arr(arr));

        // Set URS, generate update and clear URS
        regs.cr1().modify(|w| w.set_urs(Urs::COUNTERONLY));
        regs.egr().write(|w| w.set_ug(true));
        regs.cr1().modify(|w| w.set_urs(Urs::ANYEVENT));
    }

    fn compare_interrupt_status(&self, n: usize) -> bool {
        if n > 3 {
            false
        } else {
            // NOTE(unsafe) Atomic read with no side-effects
            unsafe { self.0.sr().read().ccif(n) }
        }
    }

    unsafe fn compare_clear_flag(&self, n: usize) {
        if n > 3 {
            return;
        }
        self.0.sr().modify(|w| w.set_ccif(n, false));
    }

    fn set_compare(&self, n: usize, value: u16) {
        if n > 3 {
            return;
        }
        // NOTE(unsafe) Atomic write
        unsafe {
            self.0.ccr(n).write(|w| w.set_ccr(value));
        }
    }

    unsafe fn set_compare_interrupt(&self, n: usize, enable: bool) {
        if n > 3 {
            return;
        }
        self.0.dier().modify(|w| w.set_ccie(n, enable));
    }

    fn counter(&self) -> u16 {
        // NOTE(unsafe) Atomic read with no side-effects
        unsafe { self.0.cnt().read().cnt() }
    }
}

// ------------------------------------------------------

pub(crate) mod sealed {
    use super::*;
    pub trait Instance {
        type Interrupt: Interrupt;

        fn inner() -> TimerInner;
    }
}

pub trait Instance: sealed::Instance + Sized + 'static {}

macro_rules! impl_timer {
    ($inst:ident) => {
        impl crate::clock::sealed::Instance for peripherals::$inst {
            type Interrupt = interrupt::$inst;

            fn inner() -> crate::clock::TimerInner {
                const INNER: crate::clock::TimerInner = crate::clock::TimerInner($inst);
                INNER
            }
        }

        impl crate::clock::Instance for peripherals::$inst {}
    };
}
