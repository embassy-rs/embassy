use core::cell::Cell;
use core::sync::atomic::{compiler_fence, AtomicU32, AtomicU8, Ordering};
use core::{mem, ptr};
use critical_section::CriticalSection;
use embassy::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy::blocking_mutex::CriticalSectionMutex as Mutex;
use embassy::interrupt::{Interrupt, InterruptExt};
use embassy::time::driver::{AlarmHandle, Driver};

use crate::interrupt;
use crate::pac;

fn rtc() -> &'static pac::rtc0::RegisterBlock {
    unsafe { &*pac::RTC1::ptr() }
}

/// Calculate the timestamp from the period count and the tick count.
///
/// The RTC counter is 24 bit. Ticking at 32768hz, it overflows every ~8 minutes. This is
/// too short. We must make it "never" overflow.
///
/// The obvious way would be to count overflow periods. Every time the counter overflows,
/// increase a `periods` variable. `now()` simply does `periods << 24 + counter`. So, the logic
/// around an overflow would look like this:
///
/// ```not_rust
/// periods = 1, counter = 0xFF_FFFE --> now = 0x1FF_FFFE
/// periods = 1, counter = 0xFF_FFFF --> now = 0x1FF_FFFF
/// **OVERFLOW**
/// periods = 2, counter = 0x00_0000 --> now = 0x200_0000
/// periods = 2, counter = 0x00_0001 --> now = 0x200_0001
/// ```
///
/// The problem is this is vulnerable to race conditions if `now()` runs at the exact time an
/// overflow happens.
///
/// If `now()` reads `periods` first and `counter` later, and overflow happens between the reads,
/// it would return a wrong value:
///
/// ```not_rust
/// periods = 1 (OLD), counter = 0x00_0000 (NEW) --> now = 0x100_0000 -> WRONG
/// ```
///
/// It fails similarly if it reads `counter` first and `periods` second.
///
/// To fix this, we define a "period" to be 2^23 ticks (instead of 2^24). One "overflow cycle" is 2 periods.
///
/// - `period` is incremented on overflow (at counter value 0)
/// - `period` is incremented "midway" between overflows (at counter value 0x80_0000)
///
/// Therefore, when `period` is even, counter is in 0..0x7f_ffff. When odd, counter is in 0x80_0000..0xFF_FFFF
/// This allows for now() to return the correct value even if it races an overflow.
///
/// To get `now()`, `period` is read first, then `counter` is read. If the counter value matches
/// the expected range for the `period` parity, we're done. If it doesn't, this means that
/// a new period start has raced us between reading `period` and `counter`, so we assume the `counter` value
/// corresponds to the next period.
///
/// `period` is a 32bit integer, so It overflows on 2^32 * 2^23 / 32768 seconds of uptime, which is 34865
/// years. For comparison, flash memory like the one containing your firmware is usually rated to retain
/// data for only 10-20 years. 34865 years is long enough!
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

const ALARM_COUNT: usize = 3;

struct RtcDriver {
    /// Number of 2^23 periods elapsed since boot.
    period: AtomicU32,
    alarm_count: AtomicU8,
    /// Timestamp at which to fire alarm. u64::MAX if no alarm is scheduled.
    alarms: Mutex<[AlarmState; ALARM_COUNT]>,
}

const ALARM_STATE_NEW: AlarmState = AlarmState::new();
embassy::time_driver_impl!(static DRIVER: RtcDriver = RtcDriver {
    period: AtomicU32::new(0),
    alarm_count: AtomicU8::new(0),
    alarms: Mutex::const_new(CriticalSectionRawMutex::new(), [ALARM_STATE_NEW; ALARM_COUNT]),
});

impl RtcDriver {
    fn init(&'static self, irq_prio: crate::interrupt::Priority) {
        let r = rtc();
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

        let irq = unsafe { interrupt::RTC1::steal() };
        irq.set_priority(irq_prio);
        irq.enable();
    }

    fn on_interrupt(&self) {
        let r = rtc();
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
                critical_section::with(|cs| {
                    self.trigger_alarm(n, cs);
                })
            }
        }
    }

    fn next_period(&self) {
        critical_section::with(|cs| {
            let r = rtc();
            let period = self.period.fetch_add(1, Ordering::Relaxed) + 1;
            let t = (period as u64) << 23;

            for n in 0..ALARM_COUNT {
                let alarm = &self.alarms.borrow(cs)[n];
                let at = alarm.timestamp.get();

                if at < t + 0xc00000 {
                    // just enable it. `set_alarm` has already set the correct CC val.
                    r.intenset.write(|w| unsafe { w.bits(compare_n(n)) });
                }
            }
        })
    }

    fn get_alarm<'a>(&'a self, cs: CriticalSection<'a>, alarm: AlarmHandle) -> &'a AlarmState {
        // safety: we're allowed to assume the AlarmState is created by us, and
        // we never create one that's out of bounds.
        unsafe { self.alarms.borrow(cs).get_unchecked(alarm.id() as usize) }
    }

    fn trigger_alarm(&self, n: usize, cs: CriticalSection) {
        let r = rtc();
        r.intenclr.write(|w| unsafe { w.bits(compare_n(n)) });

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
        // `period` MUST be read before `counter`, see comment at the top for details.
        let period = self.period.load(Ordering::Relaxed);
        compiler_fence(Ordering::Acquire);
        let counter = rtc().counter.read().bits();
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
            let n = alarm.id() as _;
            let alarm = self.get_alarm(cs, alarm);
            alarm.timestamp.set(timestamp);

            let t = self.now();

            // If alarm timestamp has passed, trigger it instantly.
            if timestamp <= t {
                self.trigger_alarm(n, cs);
                return;
            }

            let r = rtc();

            // If it hasn't triggered yet, setup it in the compare channel.

            // Write the CC value regardless of whether we're going to enable it now or not.
            // This way, when we enable it later, the right value is already set.

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

            let diff = timestamp - t;
            if diff < 0xc00000 {
                r.intenset.write(|w| unsafe { w.bits(compare_n(n)) });
            } else {
                // If it's too far in the future, don't setup the compare channel yet.
                // It will be setup later by `next_period`.
                r.intenclr.write(|w| unsafe { w.bits(compare_n(n)) });
            }
        })
    }
}

#[interrupt]
fn RTC1() {
    DRIVER.on_interrupt()
}

pub(crate) fn init(irq_prio: crate::interrupt::Priority) {
    DRIVER.init(irq_prio)
}
