use core::cell::Cell;
use core::sync::atomic::Ordering;
use embassy::time::Clock;

use crate::hal::bb;
use crate::interrupt;
use crate::interrupt::{CriticalSection, Mutex, OwnedInterrupt};
use crate::pac::{tim6, RCC};

fn compare_n(n: usize) -> u32 {
    1 << (n + 16)
}

struct AlarmState {
    timestamp: Cell<u64>,
    callback: Cell<Option<fn()>>,
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
    timestamp: u64,

    /// Timestamp at which to fire alarm. u64::MAX if no alarm is scheduled.
    alarms: Mutex<[AlarmState; ALARM_COUNT]>,
}

unsafe impl<T: Instance> Send for RTC<T> {}
unsafe impl<T: Instance> Sync for RTC<T> {}

impl<T: Instance> RTC<T> {
    pub fn new(rtc: T, irq: T::Interrupt) -> Self {
        Self {
            rtc,
            irq,
            timestamp: 0,
            alarms: Mutex::new([AlarmState::new(), AlarmState::new(), AlarmState::new()]),
        }
    }

    pub fn start(&'static mut self) {
        unsafe {
            // pause
            &(*T::ptr()).cr1.modify(|_, w| w.cen().clear_bit());
            // reset counter
            &(*T::ptr()).cnt.reset();

            let frequency = 1; // timeout.into().0;
            let pclk_mul = 1; // if self.clocks.$ppre() == 1 { 1 } else { 2 };
            let ticks = 1; // self.clocks.$pclk().0 * pclk_mul / frequency;

            let psc = ((ticks - 1) / (1 << 16)) as u16;
            &(*T::ptr()).psc.write(|w| w.psc().bits(psc));

            let arr = (ticks / (psc + 1) as u32) as u16;
            &(*T::ptr()).arr.write(|w| w.bits(arr as u32));

            // Trigger update event to load the registers
            &(*T::ptr()).cr1.modify(|_, w| w.urs().set_bit());
            &(*T::ptr()).egr.write(|w| w.ug().set_bit());
            &(*T::ptr()).cr1.modify(|_, w| w.urs().clear_bit());

            // enable interrupt
            &(*T::ptr()).dier.write(|w| w.uie().set_bit());

            // start counter
            &(*T::ptr()).cr1.modify(|_, w| w.cen().set_bit());
        }

        self.set_timer_alarm(0xFFFF);

        self.irq.set_handler(
            |ptr| unsafe {
                let this = &mut *(ptr as *mut Self);
                this.on_interrupt();
            },
            self as *const _ as *mut _,
        );
        self.irq.unpend();
        self.irq.enable();
    }

    fn set_timer_alarm(&self, ticks: u32) {
        /*
            auto reload value -- ticks to wait until interrupt
        */
        unsafe { &(*T::ptr()).arr.write(|w| w.bits(ticks)) };
    }

    fn read_timer(&self) -> u32 {
        unsafe { (*T::ptr()).cnt.read().bits() }
    }

    unsafe fn on_interrupt(&mut self) {
        // Clear interrupt flag
        &(*T::ptr()).sr.write(|w| w.uif().clear_bit());

        self.timestamp += self.read_timer() as u64;
    }

    fn trigger_alarm(&self, n: usize, cs: &CriticalSection) {
        // self.rtc.intenclr.write(|w| unsafe { w.bits(compare_n(n)) });

        let alarm = &self.alarms.borrow(cs)[n];
        alarm.timestamp.set(u64::MAX);

        // Call after clearing alarm, so the callback can set another alarm.
        alarm.callback.get().map(|f| f());
    }

    fn set_alarm_callback(&self, n: usize, callback: fn()) {
        interrupt::free(|cs| {
            let alarm = &self.alarms.borrow(cs)[n];
            alarm.callback.set(Some(callback));
        })
    }

    fn set_alarm(&self, n: usize, timestamp: u64) {
        interrupt::free(|cs| {
            let alarm = &self.alarms.borrow(cs)[n];
            alarm.timestamp.set(timestamp);

            let t = self.now();
            if timestamp <= t {
                self.trigger_alarm(n, cs);
                return;
            }

            let diff = timestamp - t;
            if diff < 0xc00000 {
                // nrf52 docs say:
                //    If the COUNTER is N, writing N or N+1 to a CC register may not trigger a COMPARE event.
                // To workaround this, we never write a timestamp smaller than N+3.
                // N+2 is not safe because rtc can tick from N to N+1 between calling now() and writing cc.
                let safe_timestamp = timestamp.max(t + 3);
                // self.rtc.cc[n].write(|w| unsafe { w.bits(safe_timestamp as u32 & 0xFFFFFF) });
                // self.rtc.intenset.write(|w| unsafe { w.bits(compare_n(n)) });
            } else {
                // self.rtc.intenclr.write(|w| unsafe { w.bits(compare_n(n)) });
            }
        })
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
        // let counter = self.rtc.counter.read().bits();
        // let period = self.period.load(Ordering::Relaxed);
        // calc_now(period, counter)
        0
    }
}

pub struct Alarm<T: Instance> {
    n: usize,
    rtc: &'static RTC<T>,
}

impl<T: Instance> embassy::time::Alarm for Alarm<T> {
    fn set_callback(&self, callback: fn()) {
        self.rtc.set_alarm_callback(self.n, callback);
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
}

/// Implemented by all RTC instances.
pub trait Instance: sealed::Instance + Sized + 'static {
    /// The interrupt associated with this RTC instance.
    type Interrupt: OwnedInterrupt;

    fn ptr() -> *const tim6::RegisterBlock;
    fn enable_clock();
}

impl Instance for crate::pac::TIM7 {
    type Interrupt = interrupt::TIM7Interrupt;

    fn ptr() -> *const tim6::RegisterBlock {
        crate::pac::TIM7::ptr() as *const _
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
