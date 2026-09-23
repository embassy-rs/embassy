#![allow(non_snake_case)]

#[cfg(feature = "low-power")]
use core::cell::Cell;
use core::cell::RefCell;
use core::sync::atomic::{AtomicU32, Ordering, compiler_fence};

use critical_section::CriticalSection;
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_time_driver::{Driver, TICK_HZ};
use embassy_time_queue_utils::Queue;
use stm32_metapac::timer::TimGp16;
#[cfg(feature = "rt")]
use stm32_metapac::timer::regs;

use super::AlarmState;
use crate::interrupt::typelevel::Interrupt;
use crate::pac::timer::vals;
use crate::rcc::{self, SealedRccPeripheral};
#[cfg(feature = "low-power")]
use crate::rtc::Rtc;
use crate::timer::{CoreInstance, GeneralInstance1Channel};

// NOTE regarding ALARM_COUNT:
//
// As of 2023-12-04, this driver is implemented using CC1 as the halfway rollover interrupt, and any
// additional CC capabilities to provide timer alarms to embassy-time. embassy-time requires AT LEAST
// one alarm to be allocatable, which means timers that only have CC1, such as TIM16/TIM17, are not
// candidates for use as an embassy-time driver provider. (a.k.a 1CH and 1CH_CMP are not, others are good.)

type T = crate::_generated::TimeDriverPeripheral;

fn regs_gp16() -> TimGp16 {
    unsafe { TimGp16::from_ptr(T::regs()) }
}

/// Width of the timer counter: `u16` or `u32` depending on the selected timer.
type Counter = cfg_select! {
    time_driver_32bit => u32,
    _ => u16,
};

// The registers whose width depends on the timer: CNT, ARR and CCRx.
// `TimGp32` extends `TimGp16`, so everything else is accessed through the 16-bit view.

#[cfg(time_driver_32bit)]
fn regs_gp32() -> stm32_metapac::timer::TimGp32 {
    unsafe { stm32_metapac::timer::TimGp32::from_ptr(T::regs()) }
}

fn read_cnt() -> Counter {
    cfg_select! {
        time_driver_32bit => regs_gp32().cnt().read(),
        _ => regs_gp16().cnt().read().cnt(),
    }
}

fn write_cnt(val: Counter) {
    cfg_select! {
        time_driver_32bit => regs_gp32().cnt().write_value(val),
        _ => regs_gp16().cnt().write(|w| w.set_cnt(val)),
    }
}

fn write_arr(val: Counter) {
    cfg_select! {
        time_driver_32bit => regs_gp32().arr().write_value(val),
        _ => regs_gp16().arr().write(|w| w.set_arr(val)),
    }
}

fn write_ccr(n: usize, val: Counter) {
    cfg_select! {
        time_driver_32bit => regs_gp32().ccr(n).write_value(val),
        _ => regs_gp16().ccr(n).write(|w| w.set_ccr(val)),
    }
}

/// Width of the timer counter, in bits.
const BITS: u32 = Counter::BITS;
/// Length of a period in ticks: half of a counter overflow cycle.
const PERIOD_TICKS: u64 = 1 << (BITS - 1);
/// Counter value at the midway point of an overflow cycle.
const HALF_COUNTER: Counter = 1 << (BITS - 1);
/// An alarm is armed in hardware only if it's due within this many ticks of the start
/// of the current period. Beyond that the CCR value would be ambiguous, since it
/// matches the counter once per overflow cycle (2 periods). 3/4 of a cycle leaves
/// margin for interrupt latency.
const ARM_AHEAD: u64 = 3 << (BITS - 2);

// Clock timekeeping works with something we call "periods", which are time intervals
// of 2^(BITS-1) ticks. The Clock counter value is BITS bits, so one "overflow cycle" is 2 periods.
//
// A `period` count is maintained in parallel to the Timer hardware `counter`, like this:
// - `period` and `counter` start at 0
// - `period` is incremented on overflow (at counter value 0)
// - `period` is incremented "midway" between overflows (at counter value HALF_COUNTER)
//
// Therefore, when `period` is even, counter is in 0..HALF_COUNTER. When odd, counter is in HALF_COUNTER..=Counter::MAX
// This allows for now() to return the correct value even if it races an overflow.
//
// To get `now()`, `period` is read first, then `counter` is read. If the counter value matches
// the expected range for the `period` parity, we're done. If it doesn't, this means that
// a new period start has raced us between reading `period` and `counter`, so we assume the `counter` value
// corresponds to the next period.
//
// `period` is a 32bit integer. With a 16-bit timer it overflows on 2^32 * 2^15 / 32768 seconds of uptime,
// which is 136 years. With a 32-bit timer it takes 2^16 times longer.
fn calc_now(period: u32, counter: Counter) -> u64 {
    ((period as u64) << (BITS - 1)) + ((counter ^ (((period & 1) as Counter) << (BITS - 1))) as u64)
}

#[cfg(feature = "low-power")]
fn calc_period_counter(ticks: u64) -> (u32, Counter) {
    (
        2 * (ticks >> BITS) as u32 + (ticks as Counter >= HALF_COUNTER) as u32,
        ticks as Counter,
    )
}

pub(crate) struct RtcDriver {
    /// Number of periods (of `PERIOD_TICKS` ticks each) elapsed since boot.
    period: AtomicU32,
    alarm: Mutex<CriticalSectionRawMutex, AlarmState>,
    #[cfg(feature = "low-power")]
    pub(crate) rtc: Mutex<CriticalSectionRawMutex, RefCell<Option<Rtc>>>,
    #[cfg(feature = "low-power")]
    /// The minimum pause time beyond which the executor will enter a low-power state.
    min_stop_pause: Mutex<CriticalSectionRawMutex, Cell<embassy_time::Duration>>,
    queue: Mutex<CriticalSectionRawMutex, RefCell<Queue>>,
}

embassy_time_driver::time_driver_impl!(static DRIVER: RtcDriver = RtcDriver {
    period: AtomicU32::new(0),
    alarm: Mutex::const_new(CriticalSectionRawMutex::new(), AlarmState::new()),
    #[cfg(feature = "low-power")]
    rtc: Mutex::const_new(CriticalSectionRawMutex::new(), RefCell::new(None)),
    #[cfg(feature = "low-power")]
    min_stop_pause: Mutex::const_new(CriticalSectionRawMutex::new(), Cell::new(embassy_time::Duration::from_millis(0))),
    queue: Mutex::new(RefCell::new(Queue::new()))
});

impl RtcDriver {
    /// initialize the timer, but don't start it.  Used for chips like stm32wle5
    /// for low power where the timer config is lost in STOP2.
    pub(crate) fn init_timer(&'static self, cs: critical_section::CriticalSection) {
        let r = regs_gp16();

        rcc::enable_and_reset_with_cs_no_refcount::<T>(cs);

        let timer_freq = T::frequency();

        r.cr1().modify(|w| w.set_cen(false));
        write_cnt(0);

        let psc = timer_freq.0 / TICK_HZ as u32 - 1;
        let psc: u16 = match psc.try_into() {
            Err(_) => panic!("psc division overflow: {}", psc),
            Ok(n) => n,
        };

        r.psc().write_value(psc);
        write_arr(Counter::MAX);

        // Set URS, generate update and clear URS
        r.cr1().modify(|w| w.set_urs(vals::Urs::CounterOnly));
        r.egr().write(|w| w.set_ug(true));
        r.cr1().modify(|w| w.set_urs(vals::Urs::AnyEvent));

        // Mid-way point
        write_ccr(0, HALF_COUNTER);

        // Enable overflow and half-overflow interrupts
        r.dier().write(|w| {
            w.set_uie(true);
            w.set_ccie(0, true);
        });

        <T as GeneralInstance1Channel>::CaptureCompareInterrupt::unpend();
        <T as CoreInstance>::UpdateInterrupt::unpend();
        unsafe {
            <T as GeneralInstance1Channel>::CaptureCompareInterrupt::enable();
            <T as CoreInstance>::UpdateInterrupt::enable();
        }
    }

    fn init(&'static self, cs: CriticalSection) {
        self.init_timer(cs);
        regs_gp16().cr1().modify(|w| w.set_cen(true));
    }

    #[cfg(feature = "rt")]
    pub(crate) fn on_interrupt(&self) {
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

    #[cfg(feature = "rt")]
    fn next_period(&self) {
        let r = regs_gp16();

        // We only modify the period from the timer interrupt, so we know this can't race.
        let period = self.period.load(Ordering::Relaxed) + 1;
        self.period.store(period, Ordering::Relaxed);
        let t = (period as u64) * PERIOD_TICKS;

        critical_section::with(move |cs| {
            r.dier().modify(move |w| {
                let n = 0;
                let alarm = self.alarm.borrow(cs);
                let at = alarm.timestamp.get();

                if at < t + ARM_AHEAD {
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
        write_ccr(n + 1, timestamp as Counter);

        // Enable it if it'll happen soon. Otherwise, `next_period` will enable it.
        let diff = timestamp - t;
        r.dier().modify(|w| w.set_ccie(n + 1, diff < ARM_AHEAD));

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

    #[cfg(feature = "low-power")]
    /// Set the current time and recompute any passed alarms
    fn set_time(&self, instant: u64, cs: CriticalSection) {
        let (period, counter) = calc_period_counter(core::cmp::max(self.now(), instant));

        self.period.store(period, Ordering::SeqCst);
        write_cnt(counter);

        // Now, recompute alarm
        let alarm = self.alarm.borrow(cs);

        if !self.set_alarm(cs, alarm.timestamp.get()) {
            // If the alarm timestamp has passed, we need to trigger it
            self.trigger_alarm(cs);
        }
    }
}

#[cfg(feature = "low-power")]
impl super::LPTimeDriver for RtcDriver {
    fn time_until_next_alarm(&self, cs: CriticalSection) -> embassy_time::Duration {
        let now = self.now() + 32;

        embassy_time::Duration::from_ticks(self.alarm.borrow(cs).timestamp.get().saturating_sub(now))
    }

    fn set_min_stop_pause(&self, cs: CriticalSection, min_stop_pause: embassy_time::Duration) {
        self.min_stop_pause.borrow(cs).replace(min_stop_pause);
    }

    fn set_rtc(&self, cs: CriticalSection, mut rtc: Rtc) {
        rtc.stop_wakeup_alarm();

        assert!(self.rtc.borrow(cs).replace(Some(rtc)).is_none());
    }

    fn pause_time(&self, cs: CriticalSection) -> Result<(), ()> {
        assert!(regs_gp16().cr1().read().cen());

        let time_until_next_alarm = self.time_until_next_alarm(cs);
        if time_until_next_alarm < self.min_stop_pause.borrow(cs).get() {
            Err(())
        } else {
            self.rtc
                .borrow(cs)
                .borrow_mut()
                .as_mut()
                .unwrap()
                .start_wakeup_alarm(time_until_next_alarm);

            regs_gp16().cr1().modify(|w| w.set_cen(false));
            Ok(())
        }
    }

    fn resume_time(&self, cs: CriticalSection) {
        assert!(!regs_gp16().cr1().read().cen());

        self.set_time(
            self.rtc
                .borrow(cs)
                .borrow_mut()
                .as_mut()
                .unwrap()
                .stop_wakeup_alarm()
                .as_ticks(),
            cs,
        );

        regs_gp16().cr1().modify(|w| w.set_cen(true));
    }
}

impl Driver for RtcDriver {
    fn now(&self) -> u64 {
        let period = self.period.load(Ordering::Relaxed);
        compiler_fence(Ordering::Acquire);
        let counter = read_cnt();
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

pub(crate) const fn get_driver() -> &'static RtcDriver {
    &DRIVER
}

pub(crate) fn init(cs: CriticalSection) {
    DRIVER.init(cs)
}
