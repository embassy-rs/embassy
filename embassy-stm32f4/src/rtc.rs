use core::cell::Cell;
use core::mem;
use core::ops::Deref;
use core::sync::atomic::Ordering;
use embassy::time::Clock;

use crate::hal::bb;
use crate::hal::rcc::Clocks;
use crate::interrupt;
use crate::interrupt::{CriticalSection, Mutex, OwnedInterrupt};
use crate::pac::{tim6, RCC};

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

const ALARM_COUNT: usize = 3;

pub struct RTC<T: Instance> {
    rtc: T,
    irq: T::Interrupt,
    // Timestamp marks the elapsed time, excluding the current timer.
    timestamp: u64,
    clocks: Clocks,

    /// Timestamp at which to fire alarm. u64::MAX if no alarm is scheduled.
    alarms: Mutex<[AlarmState; ALARM_COUNT]>,
}

unsafe impl<T: Instance> Send for RTC<T> {}
unsafe impl<T: Instance> Sync for RTC<T> {}

impl<T: Instance> RTC<T> {
    pub fn new(rtc: T, irq: T::Interrupt, clocks: Clocks) -> Self {
        T::enable_clock();

        Self {
            rtc,
            irq,
            timestamp: 0,
            clocks: clocks,
            alarms: Mutex::new([AlarmState::new(), AlarmState::new(), AlarmState::new()]),
        }
    }

    #[inline(always)]
    fn reset_timestamp(&mut self) {
        self.timestamp += self.rtc.arr().read().bits() as u64;
    }

    #[inline(always)]
    fn set_arr(&self, arr: u32) {
        if arr != self.rtc.arr().read().bits() {
            self.rtc.arr().write(|w| unsafe { w.bits(arr) });
            self.update();
        }
    }

    #[inline(always)]
    fn update(&self) {
        self.rtc.cr1().modify(|_, w| w.urs().set_bit());
        self.rtc.egr().write(|w| w.ug().set_bit());
        self.rtc.cr1().modify(|_, w| w.urs().clear_bit());
    }

    pub fn start(&'static mut self) {
        // stop counter
        self.rtc.cr1().modify(|_, w| w.cen().clear_bit());
        // reset counter
        self.rtc.cnt().reset();

        let frequency = 1; // timeout.into().0;
        let pclk_mul = if T::ppre(self.clocks) == 1 { 1 } else { 2 };
        let ticks = T::pclk(self.clocks) * pclk_mul / frequency;

        let psc = ((ticks - 1) / (1 << 16)) as u16;
        self.rtc.psc().write(|w| w.psc().bits(psc));
        self.update();

        self.set_arr(u16::MAX as u32);

        // enable interrupt
        self.rtc.dier().write(|w| w.uie().set_bit());

        self.irq.set_handler(
            |ptr| unsafe {
                let this = &mut *(ptr as *mut Self);
                this.on_interrupt();
            },
            self as *const _ as *mut _,
        );
        self.irq.unpend();
        self.irq.enable();

        // enable "one-pulse" mode
        // self.rtc.deref().cr1.modify(|_, w| w.opm().set_bit());

        self.rtc.cr1().modify(|_, w| w.cen().set_bit());
    }

    fn recompute(&self) {
        interrupt::free(|cs| {
            let now = self.now();
            let mut arr = u16::MAX as u32;

            for n in 0..2 {
                let alarm = &self.alarms.borrow(cs)[n];
                let alarm_timestamp = alarm.timestamp.get();

                let diff: u64;
                if alarm_timestamp > now {
                    diff = alarm_timestamp - now;
                } else {
                    diff = 0;
                }

                if diff < 5 {
                    alarm.timestamp.set(u64::MAX);
                    alarm.callback.get().map(|(f, ctx)| f(ctx));
                } else if diff < arr as u64 {
                    arr = diff as u32;
                }
            }

            self.set_arr(arr);
        });
    }

    unsafe fn on_interrupt(&mut self) {
        // Clear interrupt flag
        self.rtc.sr().write(|w| w.uif().clear_bit());

        self.reset_timestamp();
        self.recompute();
        // self.rtc.deref().cr1.modify(|_, w| w.cen().set_bit());
    }

    fn trigger_alarm(&self, n: usize, cs: &CriticalSection) {
        // self.rtc.intenclr.write(|w| unsafe { w.bits(compare_n(n)) });

        let alarm = &self.alarms.borrow(cs)[n];
        alarm.timestamp.set(u64::MAX);

        // Call after clearing alarm, so the callback can set another alarm.
        alarm.callback.get().map(|(f, ctx)| f(ctx));
    }

    fn set_alarm_callback(&self, n: usize, callback: fn(*mut ()), ctx: *mut ()) {
        interrupt::free(|cs| {
            let alarm = &self.alarms.borrow(cs)[n];
            alarm.callback.set(Some((callback, ctx)));
        })
    }

    fn set_alarm(&self, n: usize, alarm_timestamp: u64) {
        self.rtc.cr1().modify(|_, w| w.cen().clear_bit());

        interrupt::free(|cs| {
            (&self.alarms.borrow(cs)[n]).timestamp.set(alarm_timestamp);
        });

        self.recompute();
        self.rtc.cr1().modify(|_, w| w.cen().set_bit());
    }

    pub fn alarm0(&'static self) -> Alarm<T> {
        Alarm { n: 0, rtc: self }
    }
    pub fn alarm1(&'static self) -> Alarm<T> {
        Alarm { n: 1, rtc: self }
    }
    pub fn alarm2(&'static self) -> Alarm<T> {
        Alarm { n: 2, rtc: self }
    }
}

impl<T: Instance> embassy::time::Clock for RTC<T> {
    fn now(&self) -> u64 {
        self.timestamp + self.rtc.cnt().read().bits() as u64
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
    pub trait Instance {}

    impl Instance for crate::pac::TIM7 {}
    impl Instance for crate::pac::TIM2 {}
}

/// Implemented by all RTC instances.
pub trait Instance: sealed::Instance + Sized + 'static {
    /// The interrupt associated with this RTC instance.
    type Interrupt: OwnedInterrupt;

    fn cr1(&self) -> &tim6::CR1;
    fn egr(&self) -> &tim6::EGR;
    fn psc(&self) -> &tim6::PSC;
    fn cnt(&self) -> &tim6::CNT;
    fn dier(&self) -> &tim6::DIER;
    fn arr(&self) -> &tim6::ARR;
    fn sr(&self) -> &tim6::SR;

    fn enable_clock();
    fn ppre(clocks: Clocks) -> u8;
    fn pclk(clocks: Clocks) -> u32;
}

impl Instance for crate::pac::TIM7 {
    type Interrupt = interrupt::TIM7Interrupt;

    fn arr(&self) -> &tim6::ARR {
        unsafe { &(&(*crate::pac::TIM7::ptr())).arr }
    }

    fn cr1(&self) -> &tim6::CR1 {
        unsafe { &(&(*crate::pac::TIM7::ptr())).cr1 }
    }

    fn egr(&self) -> &tim6::EGR {
        unsafe { &(&(*crate::pac::TIM7::ptr())).egr }
    }

    fn psc(&self) -> &tim6::PSC {
        unsafe { &(&(*crate::pac::TIM7::ptr())).psc }
    }

    fn cnt(&self) -> &tim6::CNT {
        unsafe { &(&(*crate::pac::TIM7::ptr())).cnt }
    }

    fn dier(&self) -> &tim6::DIER {
        unsafe { &(&(*crate::pac::TIM7::ptr())).dier }
    }

    fn sr(&self) -> &tim6::SR {
        unsafe { &(&(*crate::pac::TIM7::ptr())).sr }
    }

    fn ppre(clocks: Clocks) -> u8 {
        clocks.ppre1()
    }

    fn pclk(clocks: Clocks) -> u32 {
        clocks.pclk1().0
    }

    fn enable_clock() {
        unsafe {
            //NOTE(unsafe) this reference will only be used for atomic writes with no side effects
            let rcc = &(*RCC::ptr());
            // Enable and reset the timer peripheral, it's the same bit position for both registers
            bb::set(&rcc.apb1enr, 5);
            bb::set(&rcc.apb1rstr, 5);
            bb::clear(&rcc.apb1rstr, 5);
        }
    }
}
