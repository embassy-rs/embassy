use core::cell::Cell;
use core::convert::TryInto;
use core::sync::atomic::{compiler_fence, Ordering};
use core::{mem, ptr};

use atomic_polyfill::{AtomicU32, AtomicU8};
use critical_section::CriticalSection;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::blocking_mutex::Mutex;
use embassy_time::driver::{AlarmHandle, Driver};
use embassy_time::TICK_HZ;
use stm32_metapac::timer::regs;

use crate::interrupt::typelevel::Interrupt;
use crate::pac::timer::vals;
use crate::rcc::sealed::RccPeripheral;
#[cfg(feature = "low-power")]
use crate::rtc::Rtc;
use crate::timer::sealed::{Basic16bitInstance as BasicInstance, GeneralPurpose16bitInstance as Instance};
use crate::{interrupt, peripherals};

#[cfg(not(any(time_driver_tim12, time_driver_tim15)))]
const ALARM_COUNT: usize = 3;

#[cfg(any(time_driver_tim12, time_driver_tim15))]
const ALARM_COUNT: usize = 1;

#[cfg(time_driver_tim2)]
type T = peripherals::TIM2;
#[cfg(time_driver_tim3)]
type T = peripherals::TIM3;
#[cfg(time_driver_tim4)]
type T = peripherals::TIM4;
#[cfg(time_driver_tim5)]
type T = peripherals::TIM5;

#[cfg(time_driver_tim12)]
type T = peripherals::TIM12;
#[cfg(time_driver_tim15)]
type T = peripherals::TIM15;

foreach_interrupt! {
    (TIM2, timer, $block:ident, UP, $irq:ident) => {
        #[cfg(time_driver_tim2)]
        #[cfg(feature = "rt")]
        #[interrupt]
        fn $irq() {
            DRIVER.on_interrupt()
        }
    };
    (TIM3, timer, $block:ident, UP, $irq:ident) => {
        #[cfg(time_driver_tim3)]
        #[cfg(feature = "rt")]
        #[interrupt]
        fn $irq() {
            DRIVER.on_interrupt()
        }
    };
    (TIM4, timer, $block:ident, UP, $irq:ident) => {
        #[cfg(time_driver_tim4)]
        #[cfg(feature = "rt")]
        #[interrupt]
        fn $irq() {
            DRIVER.on_interrupt()
        }
    };
    (TIM5, timer, $block:ident, UP, $irq:ident) => {
        #[cfg(time_driver_tim5)]
        #[cfg(feature = "rt")]
        #[interrupt]
        fn $irq() {
            DRIVER.on_interrupt()
        }
    };
    (TIM12, timer, $block:ident, UP, $irq:ident) => {
        #[cfg(time_driver_tim12)]
        #[cfg(feature = "rt")]
        #[interrupt]
        fn $irq() {
            DRIVER.on_interrupt()
        }
    };
    (TIM15, timer, $block:ident, UP, $irq:ident) => {
        #[cfg(time_driver_tim15)]
        #[cfg(feature = "rt")]
        #[interrupt]
        fn $irq() {
            DRIVER.on_interrupt()
        }
    };
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

    // This is really a Option<(fn(*mut ()), *mut ())>
    // but fn pointers aren't allowed in const yet
    callback: Cell<*const ()>,
    ctx: Cell<*mut ()>,
}

unsafe impl Send for AlarmState {}

impl AlarmState {
    const fn new() -> Self {
        Self {
            timestamp: Cell::new(u64::MAX),
            callback: Cell::new(ptr::null()),
            ctx: Cell::new(ptr::null_mut()),
        }
    }
}

pub(crate) struct RtcDriver {
    /// Number of 2^15 periods elapsed since boot.
    period: AtomicU32,
    alarm_count: AtomicU8,
    /// Timestamp at which to fire alarm. u64::MAX if no alarm is scheduled.
    alarms: Mutex<CriticalSectionRawMutex, [AlarmState; ALARM_COUNT]>,
    #[cfg(feature = "low-power")]
    rtc: Mutex<CriticalSectionRawMutex, Cell<Option<&'static Rtc>>>,
}

const ALARM_STATE_NEW: AlarmState = AlarmState::new();

embassy_time::time_driver_impl!(static DRIVER: RtcDriver = RtcDriver {
    period: AtomicU32::new(0),
    alarm_count: AtomicU8::new(0),
    alarms: Mutex::const_new(CriticalSectionRawMutex::new(), [ALARM_STATE_NEW; ALARM_COUNT]),
    #[cfg(feature = "low-power")]
    rtc: Mutex::const_new(CriticalSectionRawMutex::new(), Cell::new(None)),
});

impl RtcDriver {
    fn init(&'static self) {
        let r = T::regs_gp16();

        <T as RccPeripheral>::enable();
        <T as RccPeripheral>::reset();

        let timer_freq = T::frequency();

        critical_section::with(|_| {
            r.cr1().modify(|w| w.set_cen(false));
            r.cnt().write(|w| w.set_cnt(0));

            let psc = timer_freq.0 / TICK_HZ as u32 - 1;
            let psc: u16 = match psc.try_into() {
                Err(_) => panic!("psc division overflow: {}", psc),
                Ok(n) => n,
            };

            r.psc().write(|w| w.set_psc(psc));
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

            <T as BasicInstance>::Interrupt::unpend();
            unsafe { <T as BasicInstance>::Interrupt::enable() };

            r.cr1().modify(|w| w.set_cen(true));
        })
    }

    fn on_interrupt(&self) {
        let r = T::regs_gp16();

        // XXX: reduce the size of this critical section ?
        critical_section::with(|cs| {
            let sr = r.sr().read();
            let dier = r.dier().read();

            // Clear all interrupt flags. Bits in SR are "write 0 to clear", so write the bitwise NOT.
            // Other approaches such as writing all zeros, or RMWing won't work, they can
            // miss interrupts.
            r.sr().write_value(regs::SrGp(!sr.0));

            // Overflow
            if sr.uif() {
                self.next_period();
            }

            // Half overflow
            if sr.ccif(0) {
                self.next_period();
            }

            for n in 0..ALARM_COUNT {
                if sr.ccif(n + 1) && dier.ccie(n + 1) {
                    self.trigger_alarm(n, cs);
                }
            }
        })
    }

    fn next_period(&self) {
        let r = T::regs_gp16();

        let period = self.period.fetch_add(1, Ordering::Relaxed) + 1;
        let t = (period as u64) << 15;

        critical_section::with(move |cs| {
            r.dier().modify(move |w| {
                for n in 0..ALARM_COUNT {
                    let alarm = &self.alarms.borrow(cs)[n];
                    let at = alarm.timestamp.get();

                    if at < t + 0xc000 {
                        // just enable it. `set_alarm` has already set the correct CCR val.
                        w.set_ccie(n + 1, true);
                    }
                }
            })
        })
    }

    fn get_alarm<'a>(&'a self, cs: CriticalSection<'a>, alarm: AlarmHandle) -> &'a AlarmState {
        // safety: we're allowed to assume the AlarmState is created by us, and
        // we never create one that's out of bounds.
        unsafe { self.alarms.borrow(cs).get_unchecked(alarm.id() as usize) }
    }

    fn trigger_alarm(&self, n: usize, cs: CriticalSection) {
        let alarm = &self.alarms.borrow(cs)[n];
        alarm.timestamp.set(u64::MAX);

        // Call after clearing alarm, so the callback can set another alarm.

        // safety:
        // - we can ignore the possibility of `f` being unset (null) because of the safety contract of `allocate_alarm`.
        // - other than that we only store valid function pointers into alarm.callback
        let f: fn(*mut ()) = unsafe { mem::transmute(alarm.callback.get()) };
        f(alarm.ctx.get());
    }

    #[cfg(feature = "low-power")]
    /// Set the rtc but panic if it's already been set
    pub(crate) fn set_rtc(&self, rtc: &'static Rtc) {
        critical_section::with(|cs| assert!(self.rtc.borrow(cs).replace(Some(rtc)).is_none()));
    }

    #[cfg(feature = "low-power")]
    /// Compute the approximate amount of time until the next alarm
    fn time_until_next_alarm(&self) -> embassy_time::Duration {
        critical_section::with(|cs| {
            let now = self.now() + 32;

            embassy_time::Duration::from_ticks(
                self.alarms
                    .borrow(cs)
                    .iter()
                    .map(|alarm: &AlarmState| alarm.timestamp.get().saturating_sub(now))
                    .min()
                    .unwrap_or(u64::MAX),
            )
        })
    }

    #[cfg(feature = "low-power")]
    /// Add the given offset to the current time
    fn add_time(&self, offset: embassy_time::Duration) {
        let offset = offset.as_ticks();
        let cnt = T::regs_gp16().cnt().read().cnt() as u32;
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
        T::regs_gp16().cnt().write(|w| w.set_cnt(cnt as u16));

        // Now, recompute all alarms
        critical_section::with(|cs| {
            for i in 0..ALARM_COUNT {
                let alarm_handle = unsafe { AlarmHandle::new(i as u8) };
                let alarm = self.get_alarm(cs, alarm_handle);

                self.set_alarm(alarm_handle, alarm.timestamp.get());
            }
        })
    }

    #[cfg(feature = "low-power")]
    /// Stop the wakeup alarm, if enabled, and add the appropriate offset
    fn stop_wakeup_alarm(&self) {
        critical_section::with(|cs| {
            if let Some(offset) = self.rtc.borrow(cs).get().unwrap().stop_wakeup_alarm() {
                self.add_time(offset);
            }
        });
    }

    #[cfg(feature = "low-power")]
    /// Pause the timer if ready; return err if not
    pub(crate) fn pause_time(&self) -> Result<(), ()> {
        /*
            If the wakeup timer is currently running, then we need to stop it and
            add the elapsed time to the current time
        */
        self.stop_wakeup_alarm();

        let time_until_next_alarm = self.time_until_next_alarm();
        if time_until_next_alarm < embassy_time::Duration::from_millis(250) {
            Err(())
        } else {
            critical_section::with(|cs| {
                self.rtc
                    .borrow(cs)
                    .get()
                    .unwrap()
                    .start_wakeup_alarm(time_until_next_alarm);
            });

            T::regs_gp16().cr1().modify(|w| w.set_cen(false));

            Ok(())
        }
    }

    #[cfg(feature = "low-power")]
    /// Resume the timer with the given offset
    pub(crate) fn resume_time(&self) {
        self.stop_wakeup_alarm();

        T::regs_gp16().cr1().modify(|w| w.set_cen(true));
    }
}

impl Driver for RtcDriver {
    fn now(&self) -> u64 {
        let r = T::regs_gp16();

        let period = self.period.load(Ordering::Relaxed);
        compiler_fence(Ordering::Acquire);
        let counter = r.cnt().read().cnt();
        calc_now(period, counter)
    }

    unsafe fn allocate_alarm(&self) -> Option<AlarmHandle> {
        let id = self.alarm_count.fetch_update(Ordering::AcqRel, Ordering::Acquire, |x| {
            if x < ALARM_COUNT as u8 {
                Some(x + 1)
            } else {
                None
            }
        });

        match id {
            Ok(id) => Some(AlarmHandle::new(id)),
            Err(_) => None,
        }
    }

    fn set_alarm_callback(&self, alarm: AlarmHandle, callback: fn(*mut ()), ctx: *mut ()) {
        critical_section::with(|cs| {
            let alarm = self.get_alarm(cs, alarm);

            alarm.callback.set(callback as *const ());
            alarm.ctx.set(ctx);
        })
    }

    fn set_alarm(&self, alarm: AlarmHandle, timestamp: u64) -> bool {
        critical_section::with(|cs| {
            let r = T::regs_gp16();

            let n = alarm.id() as usize;
            let alarm = self.get_alarm(cs, alarm);
            alarm.timestamp.set(timestamp);

            let t = self.now();
            if timestamp <= t {
                // If alarm timestamp has passed the alarm will not fire.
                // Disarm the alarm and return `false` to indicate that.
                r.dier().modify(|w| w.set_ccie(n + 1, false));

                alarm.timestamp.set(u64::MAX);

                return false;
            }

            let safe_timestamp = timestamp.max(t + 3);

            // Write the CCR value regardless of whether we're going to enable it now or not.
            // This way, when we enable it later, the right value is already set.
            r.ccr(n + 1).write(|w| w.set_ccr(safe_timestamp as u16));

            // Enable it if it'll happen soon. Otherwise, `next_period` will enable it.
            let diff = timestamp - t;
            r.dier().modify(|w| w.set_ccie(n + 1, diff < 0xc000));

            true
        })
    }
}

#[cfg(feature = "low-power")]
pub(crate) fn get_driver() -> &'static RtcDriver {
    &DRIVER
}

pub(crate) fn init() {
    DRIVER.init()
}
