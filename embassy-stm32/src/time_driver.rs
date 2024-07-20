#![allow(non_snake_case)]

use core::sync::atomic::{compiler_fence, AtomicU32, AtomicU8, Ordering};
use core::{mem, ptr};

use critical_section::CriticalSection;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::blocking_mutex::Mutex;
use embassy_time_driver::{AlarmHandle, Driver, TICK_HZ};
use stm32_metapac::timer::{regs, TimGp16};

use crate::interrupt::typelevel::Interrupt;
use crate::pac::timer::vals;
use crate::rcc::{self, SealedRccPeripheral};
#[cfg(feature = "low-power")]
use crate::rtc::{low_power::RtcInstant, Rtc};
use crate::timer::{CoreInstance, GeneralInstance1Channel};
use crate::{interrupt, peripherals};

// NOTE regarding ALARM_COUNT:
//
// As of 2023-12-04, this driver is implemented using CC1 as the halfway rollover interrupt, and any
// additional CC capabilities to provide timer alarms to embassy-time. embassy-time requires AT LEAST
// one alarm to be allocatable, which means timers that only have CC1, such as TIM16/TIM17, are not
// candidates for use as an embassy-time driver provider. (a.k.a 1CH and 1CH_CMP are not, others are good.)
//
// The values of ALARM_COUNT below are not the TOTAL CC registers available, but rather the number
// available after reserving CC1 for regular time keeping. For example, TIM2 has four CC registers:
// CC1, CC2, CC3, and CC4, so it can provide ALARM_COUNT = 3.

cfg_if::cfg_if! {
    if #[cfg(any(time_driver_tim9, time_driver_tim12, time_driver_tim15, time_driver_tim21, time_driver_tim22))] {
        const ALARM_COUNT: usize = 1;
    } else {
        const ALARM_COUNT: usize = 3;
    }
}

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
    timestamp: u64,

    // This is really a Option<(fn(*mut ()), *mut ())>
    // but fn pointers aren't allowed in const yet
    callback: *const (),
    ctx: *mut (),
}

unsafe impl Send for AlarmState {}

impl AlarmState {
    const fn new() -> Self {
        Self {
            timestamp: u64::MAX,
            callback: ptr::null(),
            ctx: ptr::null_mut(),
        }
    }

    fn set_alarm(&mut self, n: usize, timestamp: u64) -> bool {
        let r = regs_gp16();
        self.timestamp = timestamp;

        let t = now();
        if timestamp <= t {
            // If alarm timestamp has passed the alarm will not fire.
            // Disarm the alarm and return `false` to indicate that.
            r.dier().modify(|w| w.set_ccie(n + 1, false));

            self.timestamp = u64::MAX;

            return false;
        }

        // Write the CCR value regardless of whether we're going to enable it now or not.
        // This way, when we enable it later, the right value is already set.
        r.ccr(n + 1).write(|w| w.set_ccr(timestamp as u16));

        // Enable it if it'll happen soon. Otherwise, `next_period` will enable it.
        let diff = timestamp - t;
        r.dier().modify(|w| w.set_ccie(n + 1, diff < 0xc000));

        // Reevaluate if the alarm timestamp is still in the future
        let t = now();
        if timestamp <= t {
            // If alarm timestamp has passed since we set it, we have a race condition and
            // the alarm may or may not have fired.
            // Disarm the alarm and return `false` to indicate that.
            // It is the caller's responsibility to handle this ambiguity.
            r.dier().modify(|w| w.set_ccie(n + 1, false));

            self.timestamp = u64::MAX;

            return false;
        }

        // We're confident the alarm will ring in the future.
        true
    }
}

fn now() -> u64 {
    DRIVER.now()
}

pub(crate) struct RtcDriver {
    /// Number of 2^15 periods elapsed since boot.
    period: AtomicU32,
    alarm_count: AtomicU8,
    inner: Mutex<CriticalSectionRawMutex, RtcDriverInner>,
}

struct RtcDriverInner {
    alarms: [AlarmState; ALARM_COUNT],
    #[cfg(feature = "low-power")]
    rtc: Option<&'static Rtc>,
    #[cfg(feature = "low-power")]
    stop_time: Option<RtcInstant>,
}

impl RtcDriverInner {
    const fn new() -> Self {
        Self {
            alarms: [ALARM_STATE_NEW; ALARM_COUNT],
            #[cfg(feature = "low-power")]
            rtc: None,
            #[cfg(feature = "low-power")]
            stop_time: None,
        }
    }

    fn init(&mut self, cs: CriticalSection) {
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
        r.cr1().modify(|w| w.set_urs(vals::Urs::COUNTERONLY));
        r.egr().write(|w| w.set_ug(true));
        r.cr1().modify(|w| w.set_urs(vals::Urs::ANYEVENT));

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

    fn on_interrupt(&mut self, period: &AtomicU32) {
        let r = regs_gp16();

        let sr = r.sr().read();
        let dier = r.dier().read();

        // Clear all interrupt flags. Bits in SR are "write 0 to clear", so write the bitwise NOT.
        // Other approaches such as writing all zeros, or RMWing won't work, they can
        // miss interrupts.
        r.sr().write_value(regs::SrGp16(!sr.0));

        // Overflow
        if sr.uif() {
            self.next_period(period);
        }

        // Half overflow
        if sr.ccif(0) {
            self.next_period(period);
        }

        for (n, alarm) in self.alarms.iter_mut().enumerate() {
            if sr.ccif(n + 1) && dier.ccie(n + 1) {
                // Trigger alarm
                alarm.timestamp = u64::MAX;

                // Call after clearing alarm, so the callback can set another alarm.

                // safety:
                // - we can ignore the possibility of `f` being unset (null) because of the safety contract of `allocate_alarm`.
                // - other than that we only store valid function pointers into alarm.callback
                let f: fn(*mut ()) = unsafe { mem::transmute(alarm.callback) };
                f(alarm.ctx);
            }
        }
    }

    fn next_period(&mut self, d_period: &AtomicU32) {
        // We only modify the period from the timer interrupt, so we know this can't race.
        let period = d_period.load(Ordering::Relaxed) + 1;
        d_period.store(period, Ordering::Relaxed);
        let t = (period as u64) << 15;

        let r = regs_gp16();
        r.dier().modify(move |w| {
            for (n, alarm) in self.alarms.iter_mut().enumerate() {
                let at = alarm.timestamp;

                if at < t + 0xc000 {
                    // just enable it. `set_alarm` has already set the correct CCR val.
                    w.set_ccie(n + 1, true);
                }
            }
        })
    }

    fn allocate_alarm(&mut self, alarm_count: &AtomicU8) -> Option<AlarmHandle> {
        let id = alarm_count.load(Ordering::Relaxed);
        if id < ALARM_COUNT as u8 {
            alarm_count.store(id + 1, Ordering::Relaxed);
            Some(unsafe { AlarmHandle::new(id) })
        } else {
            None
        }
    }

    fn set_alarm_callback(&mut self, alarm: AlarmHandle, callback: fn(*mut ()), ctx: *mut ()) {
        let alarm = self.get_alarm(alarm.id().into());

        alarm.callback = callback as *const ();
        alarm.ctx = ctx;
    }

    fn get_alarm(&mut self, n: usize) -> &mut AlarmState {
        &mut self.alarms[n]
    }

    fn set_alarm(&mut self, alarm: AlarmHandle, timestamp: u64) -> bool {
        let n = alarm.id() as usize;
        let alarm = self.get_alarm(n);
        alarm.set_alarm(n, timestamp)
    }
}

#[cfg(feature = "low-power")]
impl RtcDriverInner {
    /// The minimum pause time beyond which the executor will enter a low-power state.
    pub(crate) const MIN_STOP_PAUSE: embassy_time::Duration = embassy_time::Duration::from_millis(250);

    /// Compute the approximate amount of time until the next alarm
    fn time_until_next_alarm(&mut self) -> embassy_time::Duration {
        let now = now() + 32;
        let min_ticks = self
            .alarms
            .iter()
            .map(|alarm: &AlarmState| alarm.timestamp.saturating_sub(now))
            .min()
            .unwrap_or(u64::MAX);

        embassy_time::Duration::from_ticks(min_ticks)
    }

    /// Pause the timer if ready; return err if not
    fn pause_time(&mut self, period: &AtomicU32) -> Result<(), ()> {
        /*
            If the wakeup timer is currently running, then we need to stop it and
            add the elapsed time to the current time, as this will impact the result
            of `time_until_next_alarm`.
        */
        self.stop_wakeup_alarm(period);

        let time_until_next_alarm = self.time_until_next_alarm();
        if time_until_next_alarm < Self::MIN_STOP_PAUSE {
            Err(())
        } else {
            // SAFETY: We have a critical section while calling `start_wakeup_alarm`
            unsafe {
                self.rtc
                    .as_ref()
                    .unwrap()
                    .start_wakeup_alarm(time_until_next_alarm, &mut self.stop_time);
            }

            regs_gp16().cr1().modify(|w| w.set_cen(false));

            Ok(())
        }
    }

    /// Add the given offset to the current time
    fn add_time(&mut self, offset: embassy_time::Duration, d_period: &AtomicU32) {
        let offset = offset.as_ticks();
        let cnt = regs_gp16().cnt().read().cnt() as u32;
        let period = d_period.load(Ordering::SeqCst);

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

        d_period.store(period, Ordering::SeqCst);
        regs_gp16().cnt().write(|w| w.set_cnt(cnt as u16));

        // Now, recompute all alarms
        for (n, alarm) in self.alarms.iter_mut().enumerate() {
            alarm.set_alarm(n, alarm.timestamp);
        }
    }

    /// Stop the wakeup alarm, if enabled, and add the appropriate offset
    fn stop_wakeup_alarm(&mut self, period: &AtomicU32) {
        // SAFETY: We hold a critical section while calling `stop_wakeup_alarm`
        let res = unsafe { self.rtc.as_ref().unwrap().stop_wakeup_alarm(&mut self.stop_time) };
        if let Some(offset) = res {
            self.add_time(offset, period);
        }
    }
}

#[allow(clippy::declare_interior_mutable_const)]
const ALARM_STATE_NEW: AlarmState = AlarmState::new();

embassy_time_driver::time_driver_impl!(static DRIVER: RtcDriver = RtcDriver {
    period: AtomicU32::new(0),
    alarm_count: AtomicU8::new(0),
    inner: Mutex::const_new(CriticalSectionRawMutex::new(), RtcDriverInner::new()),
});

impl RtcDriver {
    fn init(&'static self, cs: CriticalSection) {
        self.inner.lock(|r| r.init(cs))
    }

    fn on_interrupt(&self) {
        self.inner.lock(|r| r.on_interrupt(&self.period))
    }
}

#[cfg(feature = "low-power")]
impl RtcDriver {
    // /*
    //     Low-power private functions: all operate within a critical seciton
    // */
    /*
        Low-power public functions: all create a critical section
    */
    /// Set the rtc but panic if it's already been set
    pub(crate) fn set_rtc(&self, rtc: &'static Rtc) {
        self.inner.lock(|r| {
            let old = r.rtc.replace(rtc);
            assert!(old.is_none());
        });
    }

    /// Pause the timer if ready; return err if not
    pub(crate) fn pause_time(&self) -> Result<(), ()> {
        self.inner.lock(|r| r.pause_time(&self.period))
    }

    /// Resume the timer with the given offset
    pub(crate) fn resume_time(&self) {
        if regs_gp16().cr1().read().cen() {
            // Time isn't currently stopped

            return;
        }

        self.inner.lock(|r| {
            r.stop_wakeup_alarm(&self.period);
            regs_gp16().cr1().modify(|w| w.set_cen(true));
        })
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

    unsafe fn allocate_alarm(&self) -> Option<AlarmHandle> {
        self.inner.lock(|r| r.allocate_alarm(&self.alarm_count))
    }

    fn set_alarm_callback(&self, alarm: AlarmHandle, callback: fn(*mut ()), ctx: *mut ()) {
        self.inner.lock(|r| r.set_alarm_callback(alarm, callback, ctx))
    }

    fn set_alarm(&self, alarm: AlarmHandle, timestamp: u64) -> bool {
        self.inner.lock(|r| r.set_alarm(alarm, timestamp))
    }
}

#[cfg(feature = "low-power")]
pub(crate) fn get_driver() -> &'static RtcDriver {
    &DRIVER
}

pub(crate) fn init(cs: CriticalSection) {
    DRIVER.init(cs)
}
