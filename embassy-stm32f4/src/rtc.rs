use core::cell::Cell;
use core::convert::TryInto;
use core::sync::atomic::{compiler_fence, AtomicU32, Ordering};

use embassy::time::{Clock, TICKS_PER_SECOND};
use stm32f4xx_hal::bb;
use stm32f4xx_hal::rcc::Clocks;

use crate::interrupt;
use crate::interrupt::{CriticalSection, Interrupt, Mutex};

// RTC timekeeping works with something we call "periods", which are time intervals
// of 2^15 ticks. The RTC counter value is 16 bits, so one "overflow cycle" is 2 periods.
//
// A `period` count is maintained in parallel to the RTC hardware `counter`, like this:
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

// TODO: This is sometimes wasteful, try to find a better way
const ALARM_COUNT: usize = 3;

/// RTC timer that can be used by the executor and to set alarms.
///
/// It can work with Timers 2, 3, 4, 5, 9 and 12. Timers 9 and 12 only have one alarm available,
/// while the others have three each.
/// This timer works internally with a unit of 2^15 ticks, which means that if a call to
/// [`embassy::time::Clock::now`] is blocked for that amount of ticks the returned value will be
/// wrong (an old value). The current default tick rate is 32768 ticks per second.
pub struct RTC<T: Instance> {
    rtc: T,
    irq: T::Interrupt,

    /// Number of 2^23 periods elapsed since boot.
    period: AtomicU32,

    /// Timestamp at which to fire alarm. u64::MAX if no alarm is scheduled.
    alarms: Mutex<[AlarmState; ALARM_COUNT]>,

    clocks: Clocks,
}

impl<T: Instance> RTC<T> {
    pub fn new(rtc: T, irq: T::Interrupt, clocks: Clocks) -> Self {
        Self {
            rtc,
            irq,
            period: AtomicU32::new(0),
            alarms: Mutex::new([AlarmState::new(), AlarmState::new(), AlarmState::new()]),
            clocks,
        }
    }

    pub fn start(&'static self) {
        self.rtc.enable_clock();
        self.rtc.stop_and_reset();

        let multiplier = if T::ppre(&self.clocks) == 1 { 1 } else { 2 };
        let freq = T::pclk(&self.clocks) * multiplier;
        let psc = freq / TICKS_PER_SECOND as u32 - 1;
        let psc: u16 = psc.try_into().unwrap();

        self.rtc.set_psc_arr(psc, u16::MAX);
        // Mid-way point
        self.rtc.set_compare(0, 0x8000);
        self.rtc.set_compare_interrupt(0, true);

        self.irq.set_handler(|ptr| unsafe {
            let this = &*(ptr as *const () as *const Self);
            this.on_interrupt();
        });
        self.irq.set_handler_context(self as *const _ as *mut _);
        self.irq.unpend();
        self.irq.enable();

        self.rtc.start();
    }

    fn on_interrupt(&self) {
        if self.rtc.overflow_interrupt_status() {
            self.rtc.overflow_clear_flag();
            self.next_period();
        }

        // Half overflow
        if self.rtc.compare_interrupt_status(0) {
            self.rtc.compare_clear_flag(0);
            self.next_period();
        }

        for n in 1..=ALARM_COUNT {
            if self.rtc.compare_interrupt_status(n) {
                self.rtc.compare_clear_flag(n);
                interrupt::free(|cs| self.trigger_alarm(n, cs));
            }
        }
    }

    fn next_period(&self) {
        interrupt::free(|cs| {
            let period = self.period.fetch_add(1, Ordering::Relaxed) + 1;
            let t = (period as u64) << 15;

            for n in 1..=ALARM_COUNT {
                let alarm = &self.alarms.borrow(cs)[n - 1];
                let at = alarm.timestamp.get();

                let diff = at - t;
                if diff < 0xc000 {
                    self.rtc.set_compare(n, at as u16);
                    self.rtc.set_compare_interrupt(n, true);
                }
            }
        })
    }

    fn trigger_alarm(&self, n: usize, cs: &CriticalSection) {
        self.rtc.set_compare_interrupt(n, false);

        let alarm = &self.alarms.borrow(cs)[n - 1];
        alarm.timestamp.set(u64::MAX);

        // Call after clearing alarm, so the callback can set another alarm.
        if let Some((f, ctx)) = alarm.callback.get() {
            f(ctx);
        }
    }

    fn set_alarm_callback(&self, n: usize, callback: fn(*mut ()), ctx: *mut ()) {
        interrupt::free(|cs| {
            let alarm = &self.alarms.borrow(cs)[n - 1];
            alarm.callback.set(Some((callback, ctx)));
        })
    }

    fn set_alarm(&self, n: usize, timestamp: u64) {
        interrupt::free(|cs| {
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
                self.rtc.set_compare(n, safe_timestamp as u16);
                self.rtc.set_compare_interrupt(n, true);
            } else {
                self.rtc.set_compare_interrupt(n, false);
            }
        })
    }

    pub fn alarm1(&'static self) -> Alarm<T> {
        Alarm { n: 1, rtc: self }
    }
    pub fn alarm2(&'static self) -> Option<Alarm<T>> {
        if T::REAL_ALARM_COUNT >= 2 {
            Some(Alarm { n: 2, rtc: self })
        } else {
            None
        }
    }
    pub fn alarm3(&'static self) -> Option<Alarm<T>> {
        if T::REAL_ALARM_COUNT >= 3 {
            Some(Alarm { n: 3, rtc: self })
        } else {
            None
        }
    }
}

impl<T: Instance> embassy::time::Clock for RTC<T> {
    fn now(&self) -> u64 {
        let period = self.period.load(Ordering::Relaxed);
        compiler_fence(Ordering::Acquire);
        let counter = self.rtc.counter();
        calc_now(period, counter)
    }
}

pub struct Alarm<T: Instance> {
    n: usize,
    rtc: &'static RTC<T>,
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

mod sealed {
    pub trait Sealed {}
}

pub trait Instance: sealed::Sealed + Sized + 'static {
    type Interrupt: Interrupt;
    const REAL_ALARM_COUNT: usize;

    fn enable_clock(&self);
    fn set_compare(&self, n: usize, value: u16);
    fn set_compare_interrupt(&self, n: usize, enable: bool);
    fn compare_interrupt_status(&self, n: usize) -> bool;
    fn compare_clear_flag(&self, n: usize);
    fn overflow_interrupt_status(&self) -> bool;
    fn overflow_clear_flag(&self);
    // This method should ensure that the values are really updated before returning
    fn set_psc_arr(&self, psc: u16, arr: u16);
    fn stop_and_reset(&self);
    fn start(&self);
    fn counter(&self) -> u16;
    fn ppre(clocks: &Clocks) -> u8;
    fn pclk(clocks: &Clocks) -> u32;
}

#[allow(unused_macros)]
macro_rules! impl_timer {
    ($module:ident: ($TYPE:ident, $INT:ident, $apbenr:ident, $enrbit:expr, $apbrstr:ident, $rstrbit:expr, $ppre:ident, $pclk: ident), 3) => {
        mod $module {
            use super::*;
            use stm32f4xx_hal::pac::{$TYPE, RCC};

            impl sealed::Sealed for $TYPE {}

            impl Instance for $TYPE {
                type Interrupt = interrupt::$INT;
                const REAL_ALARM_COUNT: usize = 3;

                fn enable_clock(&self) {
                    // NOTE(unsafe) It will only be used for atomic operations
                    unsafe {
                        let rcc = &*RCC::ptr();

                        bb::set(&rcc.$apbenr, $enrbit);
                        bb::set(&rcc.$apbrstr, $rstrbit);
                        bb::clear(&rcc.$apbrstr, $rstrbit);
                    }
                }

                fn set_compare(&self, n: usize, value: u16) {
                    // NOTE(unsafe) these registers accept all the range of u16 values
                    match n {
                        0 => self.ccr1.write(|w| unsafe { w.bits(value.into()) }),
                        1 => self.ccr2.write(|w| unsafe { w.bits(value.into()) }),
                        2 => self.ccr3.write(|w| unsafe { w.bits(value.into()) }),
                        3 => self.ccr4.write(|w| unsafe { w.bits(value.into()) }),
                        _ => {}
                    }
                }

                fn set_compare_interrupt(&self, n: usize, enable: bool) {
                    if n > 3 {
                        return;
                    }
                    let bit = n as u8 + 1;
                    unsafe {
                        if enable {
                            bb::set(&self.dier, bit);
                        } else {
                            bb::clear(&self.dier, bit);
                        }
                    }
                }

                fn compare_interrupt_status(&self, n: usize) -> bool {
                    let status = self.sr.read();
                    match n {
                        0 => status.cc1if().bit_is_set(),
                        1 => status.cc2if().bit_is_set(),
                        2 => status.cc3if().bit_is_set(),
                        3 => status.cc4if().bit_is_set(),
                        _ => false,
                    }
                }

                fn compare_clear_flag(&self, n: usize) {
                    if n > 3 {
                        return;
                    }
                    let bit = n as u8 + 1;
                    unsafe {
                        bb::clear(&self.sr, bit);
                    }
                }

                fn overflow_interrupt_status(&self) -> bool {
                    self.sr.read().uif().bit_is_set()
                }

                fn overflow_clear_flag(&self) {
                    unsafe {
                        bb::clear(&self.sr, 0);
                    }
                }

                fn set_psc_arr(&self, psc: u16, arr: u16) {
                    // NOTE(unsafe) All u16 values are valid
                    self.psc.write(|w| unsafe { w.bits(psc.into()) });
                    self.arr.write(|w| unsafe { w.bits(arr.into()) });

                    unsafe {
                        // Set URS, generate update, clear URS
                        bb::set(&self.cr1, 2);
                        self.egr.write(|w| w.ug().set_bit());
                        bb::clear(&self.cr1, 2);
                    }
                }

                fn stop_and_reset(&self) {
                    unsafe {
                        bb::clear(&self.cr1, 0);
                    }
                    self.cnt.reset();
                }

                fn start(&self) {
                    unsafe { bb::set(&self.cr1, 0) }
                }

                fn counter(&self) -> u16 {
                    self.cnt.read().bits() as u16
                }

                fn ppre(clocks: &Clocks) -> u8 {
                    clocks.$ppre()
                }

                fn pclk(clocks: &Clocks) -> u32 {
                    clocks.$pclk().0
                }
            }
        }
    };

    ($module:ident: ($TYPE:ident, $INT:ident, $apbenr:ident, $enrbit:expr, $apbrstr:ident, $rstrbit:expr, $ppre:ident, $pclk: ident), 1) => {
        mod $module {
            use super::*;
            use stm32f4xx_hal::pac::{$TYPE, RCC};

            impl sealed::Sealed for $TYPE {}

            impl Instance for $TYPE {
                type Interrupt = interrupt::$INT;
                const REAL_ALARM_COUNT: usize = 1;

                fn enable_clock(&self) {
                    // NOTE(unsafe) It will only be used for atomic operations
                    unsafe {
                        let rcc = &*RCC::ptr();

                        bb::set(&rcc.$apbenr, $enrbit);
                        bb::set(&rcc.$apbrstr, $rstrbit);
                        bb::clear(&rcc.$apbrstr, $rstrbit);
                    }
                }

                fn set_compare(&self, n: usize, value: u16) {
                    // NOTE(unsafe) these registers accept all the range of u16 values
                    match n {
                        0 => self.ccr1.write(|w| unsafe { w.bits(value.into()) }),
                        1 => self.ccr2.write(|w| unsafe { w.bits(value.into()) }),
                        _ => {}
                    }
                }

                fn set_compare_interrupt(&self, n: usize, enable: bool) {
                    if n > 1 {
                        return;
                    }
                    let bit = n as u8 + 1;
                    unsafe {
                        if enable {
                            bb::set(&self.dier, bit);
                        } else {
                            bb::clear(&self.dier, bit);
                        }
                    }
                }

                fn compare_interrupt_status(&self, n: usize) -> bool {
                    let status = self.sr.read();
                    match n {
                        0 => status.cc1if().bit_is_set(),
                        1 => status.cc2if().bit_is_set(),
                        _ => false,
                    }
                }

                fn compare_clear_flag(&self, n: usize) {
                    if n > 1 {
                        return;
                    }
                    let bit = n as u8 + 1;
                    unsafe {
                        bb::clear(&self.sr, bit);
                    }
                }

                fn overflow_interrupt_status(&self) -> bool {
                    self.sr.read().uif().bit_is_set()
                }

                fn overflow_clear_flag(&self) {
                    unsafe {
                        bb::clear(&self.sr, 0);
                    }
                }

                fn set_psc_arr(&self, psc: u16, arr: u16) {
                    // NOTE(unsafe) All u16 values are valid
                    self.psc.write(|w| unsafe { w.bits(psc.into()) });
                    self.arr.write(|w| unsafe { w.bits(arr.into()) });

                    unsafe {
                        // Set URS, generate update, clear URS
                        bb::set(&self.cr1, 2);
                        self.egr.write(|w| w.ug().set_bit());
                        bb::clear(&self.cr1, 2);
                    }
                }

                fn stop_and_reset(&self) {
                    unsafe {
                        bb::clear(&self.cr1, 0);
                    }
                    self.cnt.reset();
                }

                fn start(&self) {
                    unsafe { bb::set(&self.cr1, 0) }
                }

                fn counter(&self) -> u16 {
                    self.cnt.read().bits() as u16
                }

                fn ppre(clocks: &Clocks) -> u8 {
                    clocks.$ppre()
                }

                fn pclk(clocks: &Clocks) -> u32 {
                    clocks.$pclk().0
                }
            }
        }
    };
}

#[cfg(not(feature = "stm32f410"))]
impl_timer!(tim2: (TIM2, TIM2, apb1enr, 0, apb1rstr, 0, ppre1, pclk1), 3);

#[cfg(not(feature = "stm32f410"))]
impl_timer!(tim3: (TIM3, TIM3, apb1enr, 1, apb1rstr, 1, ppre1, pclk1), 3);

#[cfg(not(feature = "stm32f410"))]
impl_timer!(tim4: (TIM4, TIM4, apb1enr, 2, apb1rstr, 2, ppre1, pclk1), 3);

impl_timer!(tim5: (TIM5, TIM5, apb1enr, 3, apb1rstr, 3, ppre1, pclk1), 3);

impl_timer!(tim9: (TIM9, TIM1_BRK_TIM9, apb2enr, 16, apb2rstr, 16, ppre2, pclk2), 1);

#[cfg(not(any(feature = "stm32f401", feature = "stm32f410", feature = "stm32f411")))]
impl_timer!(tim12: (TIM12, TIM8_BRK_TIM12, apb1enr, 6, apb1rstr, 6, ppre1, pclk1), 1);
