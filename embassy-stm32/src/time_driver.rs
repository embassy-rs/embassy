#![allow(non_snake_case)]

use core::cell::{Cell, RefCell};
use core::sync::atomic::{AtomicU32, Ordering, compiler_fence};

use critical_section::CriticalSection;
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_time_driver::{Driver, TICK_HZ};
use embassy_time_queue_utils::Queue;
use stm32_metapac::timer::{TimGp16, regs};

use crate::interrupt::typelevel::Interrupt;
use crate::pac::timer::vals;
use crate::rcc::{self, SealedRccPeripheral};
#[cfg(feature = "low-power")]
use crate::rtc::Rtc;
use crate::timer::{CoreInstance, GeneralInstance1Channel};
use crate::{interrupt, peripherals};

// NOTE regarding ALARM_COUNT:
//
// As of 2023-12-04, this driver is implemented using CC1 as the halfway rollover interrupt, and any
// additional CC capabilities to provide timer alarms to embassy-time. embassy-time requires AT LEAST
// one alarm to be allocatable, which means timers that only have CC1, such as TIM16/TIM17, are not
// candidates for use as an embassy-time driver provider. (a.k.a 1CH and 1CH_CMP are not, others are good.)

#[cfg(time_driver_tim1)]
type T = peripherals::TIM1;
#[cfg(time_driver_tim2)]
type T = peripherals::TIM2;
#[cfg(time_driver_tim3)]
type T = peripherals::TIM3;
#[cfg(time_driver_tim4)]
type T = peripherals::TIM4;
#[cfg(time_driver_tim5)]
type T = peripherals::TIM5;
#[cfg(time_driver_tim8)]
type T = peripherals::TIM8;
#[cfg(time_driver_tim9)]
type T = peripherals::TIM9;
#[cfg(time_driver_tim12)]
type T = peripherals::TIM12;
#[cfg(time_driver_tim15)]
type T = peripherals::TIM15;
#[cfg(time_driver_tim20)]
type T = peripherals::TIM20;
#[cfg(time_driver_tim21)]
type T = peripherals::TIM21;
#[cfg(time_driver_tim22)]
type T = peripherals::TIM22;
#[cfg(time_driver_tim23)]
type T = peripherals::TIM23;
#[cfg(time_driver_tim24)]
type T = peripherals::TIM24;

foreach_interrupt! {
    (TIM1, timer, $block:ident, CC, $irq:ident) => {
        #[cfg(time_driver_tim1)]
        #[cfg(feature = "rt")]
        #[interrupt]
        fn $irq() {
            DRIVER.on_interrupt()
        }
    };
    (TIM2, timer, $block:ident, CC, $irq:ident) => {
        #[cfg(time_driver_tim2)]
        #[cfg(feature = "rt")]
        #[interrupt]
        fn $irq() {
            DRIVER.on_interrupt()
        }
    };
    (TIM3, timer, $block:ident, CC, $irq:ident) => {
        #[cfg(time_driver_tim3)]
        #[cfg(feature = "rt")]
        #[interrupt]
        fn $irq() {
            DRIVER.on_interrupt()
        }
    };
    (TIM4, timer, $block:ident, CC, $irq:ident) => {
        #[cfg(time_driver_tim4)]
        #[cfg(feature = "rt")]
        #[interrupt]
        fn $irq() {
            DRIVER.on_interrupt()
        }
    };
    (TIM5, timer, $block:ident, CC, $irq:ident) => {
        #[cfg(time_driver_tim5)]
        #[cfg(feature = "rt")]
        #[interrupt]
        fn $irq() {
            DRIVER.on_interrupt()
        }
    };
    (TIM8, timer, $block:ident, CC, $irq:ident) => {
        #[cfg(time_driver_tim8)]
        #[cfg(feature = "rt")]
        #[interrupt]
        fn $irq() {
            DRIVER.on_interrupt()
        }
    };
    (TIM9, timer, $block:ident, CC, $irq:ident) => {
        #[cfg(time_driver_tim9)]
        #[cfg(feature = "rt")]
        #[interrupt]
        fn $irq() {
            DRIVER.on_interrupt()
        }
    };
    (TIM12, timer, $block:ident, CC, $irq:ident) => {
        #[cfg(time_driver_tim12)]
        #[cfg(feature = "rt")]
        #[interrupt]
        fn $irq() {
            DRIVER.on_interrupt()
        }
    };
    (TIM15, timer, $block:ident, CC, $irq:ident) => {
        #[cfg(time_driver_tim15)]
        #[cfg(feature = "rt")]
        #[interrupt]
        fn $irq() {
            DRIVER.on_interrupt()
        }
    };
    (TIM20, timer, $block:ident, CC, $irq:ident) => {
        #[cfg(time_driver_tim20)]
        #[cfg(feature = "rt")]
        #[interrupt]
        fn $irq() {
            DRIVER.on_interrupt()
        }
    };
    (TIM21, timer, $block:ident, CC, $irq:ident) => {
        #[cfg(time_driver_tim21)]
        #[cfg(feature = "rt")]
        #[interrupt]
        fn $irq() {
            DRIVER.on_interrupt()
        }
    };
    (TIM22, timer, $block:ident, CC, $irq:ident) => {
        #[cfg(time_driver_tim22)]
        #[cfg(feature = "rt")]
        #[interrupt]
        fn $irq() {
            DRIVER.on_interrupt()
        }
    };
    (TIM23, timer, $block:ident, CC, $irq:ident) => {
        #[cfg(time_driver_tim23)]
        #[cfg(feature = "rt")]
        #[interrupt]
        fn $irq() {
            DRIVER.on_interrupt()
        }
    };
    (TIM24, timer, $block:ident, CC, $irq:ident) => {
        #[cfg(time_driver_tim24)]
        #[cfg(feature = "rt")]
        #[interrupt]
        fn $irq() {
            DRIVER.on_interrupt()
        }
    };
}

fn regs_gp16() -> TimGp16 {
    unsafe { TimGp16::from_ptr(T::regs()) }
}

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
}

unsafe impl Send for AlarmState {}

impl AlarmState {
    const fn new() -> Self {
        Self {
            timestamp: Cell::new(u64::MAX),
        }
    }
}

pub(crate) struct RtcDriver {
    /// Number of 2^15 periods elapsed since boot.
    period: AtomicU32,
    alarm: Mutex<CriticalSectionRawMutex, AlarmState>,
    #[cfg(feature = "low-power")]
    rtc: Mutex<CriticalSectionRawMutex, RefCell<Option<Rtc>>>,
    queue: Mutex<CriticalSectionRawMutex, RefCell<Queue>>,
}

embassy_time_driver::time_driver_impl!(static DRIVER: RtcDriver = RtcDriver {
    period: AtomicU32::new(0),
    alarm: Mutex::const_new(CriticalSectionRawMutex::new(), AlarmState::new()),
    #[cfg(feature = "low-power")]
    rtc: Mutex::const_new(CriticalSectionRawMutex::new(), RefCell::new(None)),
    queue: Mutex::new(RefCell::new(Queue::new()))
});

impl RtcDriver {
    fn init(&'static self, cs: critical_section::CriticalSection) {
        let r = regs_gp16();

        rcc::enable_and_reset_with_cs::<T>(cs);

        let timer_freq = T::frequency();

        r.cr1().modify(|w| w.set_cen(false));
        r.cnt().write(|w| w.set_cnt(0));

        let psc = timer_freq.0 / TICK_HZ as u32 - 1;
        let psc: u16 = match psc.try_into() {
            Err(_) => panic!("psc division overflow: {}", psc),
            Ok(n) => n,
        };

        r.psc().write_value(psc);
        r.arr().write(|w| w.set_arr(u16::MAX));

        // Set URS, generate update and clear URS
        r.cr1().modify(|w| w.set_urs(vals::Urs::COUNTER_ONLY));
        r.egr().write(|w| w.set_ug(true));
        r.cr1().modify(|w| w.set_urs(vals::Urs::ANY_EVENT));

        // Mid-way point
        r.ccr(0).write(|w| w.set_ccr(0x8000));

        // Enable overflow and half-overflow interrupts
        r.dier().write(|w| {
            w.set_uie(true);
            w.set_ccie(0, true);
        });

        <T as GeneralInstance1Channel>::CaptureCompareInterrupt::unpend();
        unsafe { <T as GeneralInstance1Channel>::CaptureCompareInterrupt::enable() };

        r.cr1().modify(|w| w.set_cen(true));
    }

    fn on_interrupt(&self) {
        let r = regs_gp16();

        critical_section::with(|cs| {
            let sr = r.sr().read();
            let dier = r.dier().read();

            // Clear all interrupt flags. Bits in SR are "write 0 to clear", so write the bitwise NOT.
            // Other approaches such as writing all zeros, or RMWing won't work, they can
            // miss interrupts.
            r.sr().write_value(regs::SrGp16(!sr.0));

            // Overflow
            if sr.uif() {
                self.next_period();
            }

            // Half overflow
            if sr.ccif(0) {
                self.next_period();
            }

            let n = 0;
            if sr.ccif(n + 1) && dier.ccie(n + 1) {
                self.trigger_alarm(cs);
            }
        })
    }

    fn next_period(&self) {
        let r = regs_gp16();

        // We only modify the period from the timer interrupt, so we know this can't race.
        let period = self.period.load(Ordering::Relaxed) + 1;
        self.period.store(period, Ordering::Relaxed);
        let t = (period as u64) << 15;

        critical_section::with(move |cs| {
            r.dier().modify(move |w| {
                let n = 0;
                let alarm = self.alarm.borrow(cs);
                let at = alarm.timestamp.get();

                if at < t + 0xc000 {
                    // just enable it. `set_alarm` has already set the correct CCR val.
                    w.set_ccie(n + 1, true);
                }
            })
        })
    }

    fn trigger_alarm(&self, cs: CriticalSection) {
        let mut next = self.queue.borrow(cs).borrow_mut().next_expiration(self.now());
        while !self.set_alarm(cs, next) {
            next = self.queue.borrow(cs).borrow_mut().next_expiration(self.now());
        }
    }

    /*
        Low-power private functions: all operate within a critical seciton
    */

    #[cfg(feature = "low-power")]
    /// Compute the approximate amount of time until the next alarm
    fn time_until_next_alarm(&self, cs: CriticalSection) -> embassy_time::Duration {
        let now = self.now() + 32;

        embassy_time::Duration::from_ticks(self.alarm.borrow(cs).timestamp.get().saturating_sub(now))
    }

    #[cfg(feature = "low-power")]
    /// Add the given offset to the current time
    fn add_time(&self, offset: embassy_time::Duration, cs: CriticalSection) {
        let offset = offset.as_ticks();
        let cnt = regs_gp16().cnt().read().cnt() as u32;
        let period = self.period.load(Ordering::SeqCst);

        // Correct the race, if it exists
        let period = if period & 1 == 1 && cnt < u16::MAX as u32 / 2 {
            period + 1
        } else {
            period
        };

        // Normalize to the full overflow
        let period = (period / 2) * 2;

        // Add the offset
        let period = period + 2 * (offset / u16::MAX as u64) as u32;
        let cnt = cnt + (offset % u16::MAX as u64) as u32;

        let (cnt, period) = if cnt > u16::MAX as u32 {
            (cnt - u16::MAX as u32, period + 2)
        } else {
            (cnt, period)
        };

        let period = if cnt > u16::MAX as u32 / 2 { period + 1 } else { period };

        self.period.store(period, Ordering::SeqCst);
        regs_gp16().cnt().write(|w| w.set_cnt(cnt as u16));

        // Now, recompute alarm
        let alarm = self.alarm.borrow(cs);

        if !self.set_alarm(cs, alarm.timestamp.get()) {
            // If the alarm timestamp has passed, we need to trigger it
            self.trigger_alarm(cs);
        }
    }

    #[cfg(feature = "low-power")]
    /// Stop the wakeup alarm, if enabled, and add the appropriate offset
    fn stop_wakeup_alarm(&self, cs: CriticalSection) {
        if let Some(offset) = self.rtc.borrow(cs).borrow_mut().as_mut().unwrap().stop_wakeup_alarm(cs) {
            self.add_time(offset, cs);
        }
    }

    /*
        Low-power public functions: all create a critical section
    */
    #[cfg(feature = "low-power")]
    /// Set the rtc but panic if it's already been set
    pub(crate) fn set_rtc(&self, mut rtc: Rtc) {
        critical_section::with(|cs| {
            rtc.stop_wakeup_alarm(cs);

            assert!(self.rtc.borrow(cs).replace(Some(rtc)).is_none())
        });
    }

    #[cfg(feature = "low-power")]
    /// Set the rtc but panic if it's already been set
    pub(crate) fn reconfigure_rtc(&self, f: impl FnOnce(&mut Rtc)) {
        critical_section::with(|cs| f(self.rtc.borrow(cs).borrow_mut().as_mut().unwrap()));
    }

    #[cfg(feature = "low-power")]
    /// The minimum pause time beyond which the executor will enter a low-power state.
    pub(crate) const MIN_STOP_PAUSE: embassy_time::Duration = embassy_time::Duration::from_millis(250);

    #[cfg(feature = "low-power")]
    /// Pause the timer if ready; return err if not
    pub(crate) fn pause_time(&self) -> Result<(), ()> {
        critical_section::with(|cs| {
            /*
                If the wakeup timer is currently running, then we need to stop it and
                add the elapsed time to the current time, as this will impact the result
                of `time_until_next_alarm`.
            */
            self.stop_wakeup_alarm(cs);

            let time_until_next_alarm = self.time_until_next_alarm(cs);
            if time_until_next_alarm < Self::MIN_STOP_PAUSE {
                Err(())
            } else {
                self.rtc
                    .borrow(cs)
                    .borrow_mut()
                    .as_mut()
                    .unwrap()
                    .start_wakeup_alarm(time_until_next_alarm, cs);

                regs_gp16().cr1().modify(|w| w.set_cen(false));

                Ok(())
            }
        })
    }

    #[cfg(feature = "low-power")]
    /// Resume the timer with the given offset
    pub(crate) fn resume_time(&self) {
        if regs_gp16().cr1().read().cen() {
            // Time isn't currently stopped

            return;
        }

        critical_section::with(|cs| {
            self.stop_wakeup_alarm(cs);

            regs_gp16().cr1().modify(|w| w.set_cen(true));
        })
    }

    fn set_alarm(&self, cs: CriticalSection, timestamp: u64) -> bool {
        let r = regs_gp16();

        let n = 0;
        self.alarm.borrow(cs).timestamp.set(timestamp);

        let t = self.now();
        if timestamp <= t {
            // If alarm timestamp has passed the alarm will not fire.
            // Disarm the alarm and return `false` to indicate that.
            r.dier().modify(|w| w.set_ccie(n + 1, false));

            self.alarm.borrow(cs).timestamp.set(u64::MAX);

            return false;
        }

        // Write the CCR value regardless of whether we're going to enable it now or not.
        // This way, when we enable it later, the right value is already set.
        r.ccr(n + 1).write(|w| w.set_ccr(timestamp as u16));

        // Enable it if it'll happen soon. Otherwise, `next_period` will enable it.
        let diff = timestamp - t;
        r.dier().modify(|w| w.set_ccie(n + 1, diff < 0xc000));

        // Reevaluate if the alarm timestamp is still in the future
        let t = self.now();
        if timestamp <= t {
            // If alarm timestamp has passed since we set it, we have a race condition and
            // the alarm may or may not have fired.
            // Disarm the alarm and return `false` to indicate that.
            // It is the caller's responsibility to handle this ambiguity.
            r.dier().modify(|w| w.set_ccie(n + 1, false));

            self.alarm.borrow(cs).timestamp.set(u64::MAX);

            return false;
        }

        // We're confident the alarm will ring in the future.
        true
    }
}

impl Driver for RtcDriver {
    fn now(&self) -> u64 {
        let r = regs_gp16();

        let period = self.period.load(Ordering::Relaxed);
        compiler_fence(Ordering::Acquire);
        let counter = r.cnt().read().cnt();
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

#[cfg(feature = "low-power")]
pub(crate) fn get_driver() -> &'static RtcDriver {
    &DRIVER
}

pub(crate) fn init(cs: CriticalSection) {
    DRIVER.init(cs)
}
