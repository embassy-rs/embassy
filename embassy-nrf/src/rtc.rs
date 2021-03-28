use core::cell::Cell;
use core::sync::atomic::{compiler_fence, AtomicU32, Ordering};

use embassy::interrupt::InterruptExt;
use embassy::time::Clock;

use crate::interrupt::{CriticalSection, Interrupt, Mutex};
use crate::pac;
use crate::{interrupt, peripherals};

// RTC timekeeping works with something we call "periods", which are time intervals
// of 2^23 ticks. The RTC counter value is 24 bits, so one "overflow cycle" is 2 periods.
//
// A `period` count is maintained in parallel to the RTC hardware `counter`, like this:
// - `period` and `counter` start at 0
// - `period` is incremented on overflow (at counter value 0)
// - `period` is incremented "midway" between overflows (at counter value 0x800000)
//
// Therefore, when `period` is even, counter is in 0..0x7fffff. When odd, counter is in 0x800000..0xFFFFFF
// This allows for now() to return the correct value even if it races an overflow.
//
// To get `now()`, `period` is read first, then `counter` is read. If the counter value matches
// the expected range for the `period` parity, we're done. If it doesn't, this means that
// a new period start has raced us between reading `period` and `counter`, so we assume the `counter` value
// corresponds to the next period.
//
// `period` is a 32bit integer, so It overflows on 2^32 * 2^23 / 32768 seconds of uptime, which is 34865 years.

fn calc_now(period: u32, counter: u32) -> u64 {
    ((period as u64) << 23) + ((counter ^ ((period & 1) << 23)) as u64)
}

fn compare_n(n: usize) -> u32 {
    1 << (n + 16)
}

#[cfg(tests)]
mod test {
    use super::*;

    #[test]
    fn test_calc_now() {
        assert_eq!(calc_now(0, 0x000000), 0x0_000000);
        assert_eq!(calc_now(0, 0x000001), 0x0_000001);
        assert_eq!(calc_now(0, 0x7FFFFF), 0x0_7FFFFF);
        assert_eq!(calc_now(1, 0x7FFFFF), 0x1_7FFFFF);
        assert_eq!(calc_now(0, 0x800000), 0x0_800000);
        assert_eq!(calc_now(1, 0x800000), 0x0_800000);
        assert_eq!(calc_now(1, 0x800001), 0x0_800001);
        assert_eq!(calc_now(1, 0xFFFFFF), 0x0_FFFFFF);
        assert_eq!(calc_now(2, 0xFFFFFF), 0x1_FFFFFF);
        assert_eq!(calc_now(1, 0x000000), 0x1_000000);
        assert_eq!(calc_now(2, 0x000000), 0x1_000000);
    }
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

const ALARM_COUNT: usize = 3;

pub struct RTC<T: Instance> {
    rtc: T,
    irq: T::Interrupt,

    /// Number of 2^23 periods elapsed since boot.
    period: AtomicU32,

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
            period: AtomicU32::new(0),
            alarms: Mutex::new([AlarmState::new(), AlarmState::new(), AlarmState::new()]),
        }
    }

    pub fn start(&'static self) {
        let r = self.rtc.regs();
        r.cc[3].write(|w| unsafe { w.bits(0x800000) });

        r.intenset.write(|w| {
            let w = w.ovrflw().set();
            let w = w.compare3().set();
            w
        });

        r.tasks_clear.write(|w| unsafe { w.bits(1) });
        r.tasks_start.write(|w| unsafe { w.bits(1) });

        // Wait for clear
        while r.counter.read().bits() != 0 {}

        self.irq.set_handler(|ptr| unsafe {
            let this = &*(ptr as *const () as *const Self);
            this.on_interrupt();
        });
        self.irq.set_handler_context(self as *const _ as *mut _);
        self.irq.unpend();
        self.irq.enable();
    }

    fn on_interrupt(&self) {
        let r = self.rtc.regs();
        if r.events_ovrflw.read().bits() == 1 {
            r.events_ovrflw.write(|w| w);
            self.next_period();
        }

        if r.events_compare[3].read().bits() == 1 {
            r.events_compare[3].write(|w| w);
            self.next_period();
        }

        for n in 0..ALARM_COUNT {
            if r.events_compare[n].read().bits() == 1 {
                r.events_compare[n].write(|w| w);
                interrupt::free(|cs| {
                    self.trigger_alarm(n, cs);
                })
            }
        }
    }

    fn next_period(&self) {
        interrupt::free(|cs| {
            let r = self.rtc.regs();
            let period = self.period.fetch_add(1, Ordering::Relaxed) + 1;
            let t = (period as u64) << 23;

            for n in 0..ALARM_COUNT {
                let alarm = &self.alarms.borrow(cs)[n];
                let at = alarm.timestamp.get();

                let diff = at - t;
                if diff < 0xc00000 {
                    r.cc[n].write(|w| unsafe { w.bits(at as u32 & 0xFFFFFF) });
                    r.intenset.write(|w| unsafe { w.bits(compare_n(n)) });
                }
            }
        })
    }

    fn trigger_alarm(&self, n: usize, cs: &CriticalSection) {
        let r = self.rtc.regs();
        r.intenclr.write(|w| unsafe { w.bits(compare_n(n)) });

        let alarm = &self.alarms.borrow(cs)[n];
        alarm.timestamp.set(u64::MAX);

        // Call after clearing alarm, so the callback can set another alarm.
        if let Some((f, ctx)) = alarm.callback.get() {
            f(ctx);
        }
    }

    fn set_alarm_callback(&self, n: usize, callback: fn(*mut ()), ctx: *mut ()) {
        interrupt::free(|cs| {
            let alarm = &self.alarms.borrow(cs)[n];
            alarm.callback.set(Some((callback, ctx)));
        })
    }

    fn set_alarm(&self, n: usize, timestamp: u64) {
        interrupt::free(|cs| {
            let alarm = &self.alarms.borrow(cs)[n];
            alarm.timestamp.set(timestamp);

            let t = self.now();

            // If alarm timestamp has passed, trigger it instantly.
            if timestamp <= t {
                self.trigger_alarm(n, cs);
                return;
            }

            let r = self.rtc.regs();

            // If it hasn't triggered yet, setup it in the compare channel.
            let diff = timestamp - t;
            if diff < 0xc00000 {
                // nrf52 docs say:
                //    If the COUNTER is N, writing N or N+1 to a CC register may not trigger a COMPARE event.
                // To workaround this, we never write a timestamp smaller than N+3.
                // N+2 is not safe because rtc can tick from N to N+1 between calling now() and writing cc.
                //
                // It is impossible for rtc to tick more than once because
                //  - this code takes less time than 1 tick
                //  - it runs with interrupts disabled so nothing else can preempt it.
                //
                // This means that an alarm can be delayed for up to 2 ticks (from t+1 to t+3), but this is allowed
                // by the Alarm trait contract. What's not allowed is triggering alarms *before* their scheduled time,
                // and we don't do that here.
                let safe_timestamp = timestamp.max(t + 3);
                r.cc[n].write(|w| unsafe { w.bits(safe_timestamp as u32 & 0xFFFFFF) });
                r.intenset.write(|w| unsafe { w.bits(compare_n(n)) });
            } else {
                // If it's too far in the future, don't setup the compare channel yet.
                // It will be setup later by `next_period`.
                r.intenclr.write(|w| unsafe { w.bits(compare_n(n)) });
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
        // `period` MUST be read before `counter`, see comment at the top for details.
        let period = self.period.load(Ordering::Relaxed);
        compiler_fence(Ordering::Acquire);
        let counter = self.rtc.regs().counter.read().bits();
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
    use super::*;
    pub trait Instance {
        fn regs(&self) -> &pac::rtc0::RegisterBlock;
    }
}

macro_rules! impl_instance {
    ($type:ident, $irq:ident) => {
        impl sealed::Instance for peripherals::$type {
            fn regs(&self) -> &pac::rtc0::RegisterBlock {
                unsafe { &*pac::$type::ptr() }
            }
        }
        impl Instance for peripherals::$type {
            type Interrupt = interrupt::$irq;
        }
    };
}

/// Implemented by all RTC instances.
pub trait Instance: sealed::Instance + 'static {
    /// The interrupt associated with this RTC instance.
    type Interrupt: Interrupt;
}

impl_instance!(RTC0, RTC0);
impl_instance!(RTC1, RTC1);
#[cfg(any(feature = "52832", feature = "52833", feature = "52840"))]
impl_instance!(RTC2, RTC2);
