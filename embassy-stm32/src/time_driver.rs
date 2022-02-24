use atomic_polyfill::{AtomicU32, AtomicU8};
use core::cell::Cell;
use core::convert::TryInto;
use core::sync::atomic::{compiler_fence, Ordering};
use core::{mem, ptr};
use embassy::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy::blocking_mutex::Mutex;
use embassy::interrupt::InterruptExt;
use embassy::time::driver::{AlarmHandle, Driver};
use embassy::time::TICKS_PER_SECOND;
use stm32_metapac::timer::regs;

use crate::interrupt;
use crate::interrupt::CriticalSection;
use crate::pac::timer::vals;
use crate::peripherals;
use crate::rcc::sealed::RccPeripheral;
use crate::timer::sealed::Basic16bitInstance as BasicInstance;
use crate::timer::sealed::GeneralPurpose16bitInstance as Instance;

const ALARM_COUNT: usize = 3;

#[cfg(time_driver_tim2)]
type T = peripherals::TIM2;
#[cfg(time_driver_tim3)]
type T = peripherals::TIM3;
#[cfg(time_driver_tim4)]
type T = peripherals::TIM4;
#[cfg(time_driver_tim5)]
type T = peripherals::TIM5;

crate::pac::interrupts! {
    (TIM2, timer, $block:ident, UP, $irq:ident) => {
        #[cfg(time_driver_tim2)]
        #[interrupt]
        fn $irq() {
            DRIVER.on_interrupt()
        }
    };
    (TIM3, timer, $block:ident, UP, $irq:ident) => {
        #[cfg(time_driver_tim3)]
        #[interrupt]
        fn $irq() {
            DRIVER.on_interrupt()
        }
    };
    (TIM4, timer, $block:ident, UP, $irq:ident) => {
        #[cfg(time_driver_tim4)]
        #[interrupt]
        fn $irq() {
            DRIVER.on_interrupt()
        }
    };
    (TIM5, timer, $block:ident, UP, $irq:ident) => {
        #[cfg(time_driver_tim5)]
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

struct RtcDriver {
    timer: T,
    /// Number of 2^15 periods elapsed since boot.
    period: AtomicU32,
    alarm_count: AtomicU8,
    /// Timestamp at which to fire alarm. u64::MAX if no alarm is scheduled.
    alarms: Mutex<CriticalSectionRawMutex, [AlarmState; ALARM_COUNT]>,
}

const ALARM_STATE_NEW: AlarmState = AlarmState::new();

embassy::time_driver_impl!(static DRIVER: RtcDriver = RtcDriver {
    timer: unsafe { core::mem::transmute(()) }, // steal is not const
    period: AtomicU32::new(0),
    alarm_count: AtomicU8::new(0),
    alarms: Mutex::const_new(CriticalSectionRawMutex::new(), [ALARM_STATE_NEW; ALARM_COUNT]),
});

impl RtcDriver {
    fn init(&'static self) {
        let r = self.timer.regs_gp16();

        <T as RccPeripheral>::enable();
        <T as RccPeripheral>::reset();

        let timer_freq = T::frequency();

        // NOTE(unsafe) Critical section to use the unsafe methods
        critical_section::with(|_| unsafe {
            r.cr1().modify(|w| w.set_cen(false));
            r.cnt().write(|w| w.set_cnt(0));

            let psc = timer_freq.0 / TICKS_PER_SECOND as u32 - 1;
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

            // Enable CC0, disable others
            r.dier().write(|w| w.set_ccie(0, true));

            let irq: <T as BasicInstance>::Interrupt = core::mem::transmute(());
            irq.unpend();
            irq.enable();

            r.cr1().modify(|w| w.set_cen(true));
        })
    }

    fn on_interrupt(&self) {
        let r = self.timer.regs_gp16();

        // NOTE(unsafe) Use critical section to access the methods
        // XXX: reduce the size of this critical section ?
        critical_section::with(|cs| unsafe {
            let sr = r.sr().read();
            let dier = r.dier().read();

            // Clear all interrupt flags. Bits in SR are "write 0 to clear", so write the bitwise NOT.
            // Other approaches such as writing all zeros, or RMWing won't work, they can
            // miss interrupts.
            r.sr().write_value(regs::SrGp(!sr.0));

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
        let r = self.timer.regs_gp16();

        let period = self.period.fetch_add(1, Ordering::Relaxed) + 1;
        let t = (period as u64) << 15;

        critical_section::with(move |cs| unsafe {
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
        // - we can ignore the possiblity of `f` being unset (null) because of the safety contract of `allocate_alarm`.
        // - other than that we only store valid function pointers into alarm.callback
        let f: fn(*mut ()) = unsafe { mem::transmute(alarm.callback.get()) };
        f(alarm.ctx.get());
    }
}

impl Driver for RtcDriver {
    fn now(&self) -> u64 {
        let r = self.timer.regs_gp16();

        let period = self.period.load(Ordering::Relaxed);
        compiler_fence(Ordering::Acquire);
        // NOTE(unsafe) Atomic read with no side-effects
        let counter = unsafe { r.cnt().read().cnt() };
        calc_now(period, counter)
    }

    unsafe fn allocate_alarm(&self) -> Option<AlarmHandle> {
        let id = self
            .alarm_count
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |x| {
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

    fn set_alarm(&self, alarm: AlarmHandle, timestamp: u64) {
        critical_section::with(|cs| {
            let r = self.timer.regs_gp16();

            let n = alarm.id() as _;
            let alarm = self.get_alarm(cs, alarm);
            alarm.timestamp.set(timestamp);

            let t = self.now();
            if timestamp <= t {
                unsafe { r.dier().modify(|w| w.set_ccie(n + 1, false)) };
                self.trigger_alarm(n, cs);
                return;
            }

            let safe_timestamp = timestamp.max(t + 3);

            // Write the CCR value regardless of whether we're going to enable it now or not.
            // This way, when we enable it later, the right value is already set.
            unsafe { r.ccr(n + 1).write(|w| w.set_ccr(safe_timestamp as u16)) };

            // Enable it if it'll happen soon. Otherwise, `next_period` will enable it.
            let diff = timestamp - t;
            // NOTE(unsafe) We're in a critical section
            unsafe { r.dier().modify(|w| w.set_ccie(n + 1, diff < 0xc000)) };
        })
    }
}

pub(crate) fn init() {
    DRIVER.init()
}
