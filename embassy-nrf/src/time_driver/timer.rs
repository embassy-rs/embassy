use crate::timer::Frequency;

use super::*;

fn timer() -> &'static pac::timer0::RegisterBlock {
    unsafe { &*pac::TIMER1::ptr() }
}

/// Provides a Real Time Counter using the HFCLK. Uses
/// the TIMER peripheral in counter mode.
struct TimerDriver {
    /// Number of 2^23 periods elapsed since boot.
    period: AtomicU32,
    alarm_count: AtomicU8,
    /// Timestamp at which to fire alarm. u64::MAX if no alarm is scheduled.
    alarms: Mutex<[AlarmState; ALARM_COUNT]>,
}

const ALARM_STATE_NEW: AlarmState = AlarmState::new();
embassy::time_driver_impl!(static DRIVER: TimerDriver = TimerDriver {
    period: AtomicU32::new(0),
    alarm_count: AtomicU8::new(0),
    alarms: Mutex::const_new(CriticalSectionRawMutex::new(), [ALARM_STATE_NEW; ALARM_COUNT]),
});

impl TimerDriver {
    fn init(&'static self, irq_prio: crate::interrupt::Priority) {
        let r = timer();
        r.mode.write(|w| w.mode().timer());
        r.prescaler
            .write(|w| unsafe { w.prescaler().bits(Frequency::F62500Hz as u8) });

        r.bitmode.write(|w| w.bitmode()._32bit());
        r.cc[3].write(|w| unsafe { w.bits(0x800000) });

        r.intenset.write(|w| w.compare3().set());

        r.shorts
            .modify(|r, w| unsafe { w.bits(r.bits() | (1 << 3)) });

        r.tasks_clear.write(|w| unsafe { w.bits(1) });
        r.tasks_start.write(|w| unsafe { w.bits(1) });

        let irq = unsafe { interrupt::TIMER1::steal() };
        irq.set_priority(irq_prio);
        irq.enable();
    }

    fn on_interrupt(&self) {
        let r = timer();
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
            let r = timer();
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
        let r = timer();
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

impl Driver for TimerDriver {
    fn now(&self) -> u64 {
        // `period` MUST be read before `counter`, see comment at the top for details.
        let period = self.period.load(Ordering::Relaxed);
        compiler_fence(Ordering::Acquire);
        let r = timer();
        r.tasks_capture[3].write(|w| unsafe { w.bits(1) });
        let counter = r.cc[3].read().bits();
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

            let r = timer();

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
fn TIMER1() {
    DRIVER.on_interrupt()
}

pub(crate) fn init(irq_prio: crate::interrupt::Priority) {
    DRIVER.init(irq_prio)
}
