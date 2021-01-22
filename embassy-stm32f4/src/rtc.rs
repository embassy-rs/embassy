use core::cell::Cell;
use core::sync::atomic::Ordering;
use embassy::time::Clock;

use crate::hal::bb;
use crate::hal::rcc::Clocks;
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
        Self {
            rtc,
            irq,
            timestamp: 0,
            clocks: clocks,
            alarms: Mutex::new([AlarmState::new(), AlarmState::new(), AlarmState::new()]),
        }
    }

    #[inline(always)]
    fn prtc(&self) -> &tim6::RegisterBlock {
        unsafe { &(*T::ptr()) }
    }

    pub fn start(&'static mut self) {
        // pause
        self.prtc().cr1.modify(|_, w| w.cen().clear_bit());
        // reset counter
        self.prtc().cnt.reset();

        let frequency = 1; // timeout.into().0;
        let pclk_mul = if T::ppre(self.clocks) == 1 { 1 } else { 2 };
        let ticks = T::pclk(self.clocks) * pclk_mul / frequency;

        let psc = ((ticks - 1) / (1 << 16)) as u16;
        self.prtc().psc.write(|w| w.psc().bits(psc));

        let arr = (ticks / (psc + 1) as u32) as u16;
        self.prtc().arr.write(|w| unsafe { w.bits(arr as u32) });

        // Trigger update event to load the registers
        self.prtc().cr1.modify(|_, w| w.urs().set_bit());
        self.prtc().egr.write(|w| w.ug().set_bit());
        self.prtc().cr1.modify(|_, w| w.urs().clear_bit());

        // enable interrupt
        self.prtc().dier.write(|w| w.uie().set_bit());

        // set alarm value
        self.prtc().arr.write(|w| unsafe { w.bits(0xFFFF) });

        // start counter
        self.prtc().cr1.modify(|_, w| w.cen().set_bit());

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

    unsafe fn on_interrupt(&mut self) {
        // Clear interrupt flag
        self.prtc().sr.write(|w| w.uif().clear_bit());

        self.timestamp = self.now();

        let mut arr = 0xFFFF;

        interrupt::free(|cs| {
            for n in 0..2 {
                let alarm = &self.alarms.borrow(cs)[n];
                let diff = alarm.timestamp.get() - self.timestamp;

                if diff < 5 {
                    self.trigger_alarm(n, cs);
                } else if diff < arr {
                    arr = diff;
                }
            }
        });

        self.prtc().arr.write(|w| unsafe { w.bits(arr as u32) });

        /*
            iterate through the alarms, and if they are close, trigger them
        */
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

            /*
                use 5 ticks for now; later optimize based on testing and docs
            */
            if timestamp <= t + 5 {
                self.trigger_alarm(n, cs);
                return;
            }

            let diff = timestamp - t;
            /*
                if diff is less than arr, modify arr to end sooner
            */
            let arr = self.prtc().arr.read().bits();
            if diff < arr as u64 {
                self.prtc().arr.write(|w| unsafe { w.bits(arr as u32) });
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
        self.timestamp + unsafe { (*T::ptr()).cnt.read().bits() } as u64
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
    fn ppre(clocks: Clocks) -> u8;
    fn pclk(clocks: Clocks) -> u32;
}

impl Instance for crate::pac::TIM7 {
    type Interrupt = interrupt::TIM7Interrupt;

    fn ptr() -> *const tim6::RegisterBlock {
        crate::pac::TIM7::ptr() as *const _
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
