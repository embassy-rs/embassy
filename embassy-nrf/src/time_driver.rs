use core::cell::{Cell, RefCell};
#[cfg(not(feature = "_grtc"))]
use core::sync::atomic::{AtomicU32, Ordering, compiler_fence};

use critical_section::CriticalSection;
use embassy_sync::blocking_mutex::CriticalSectionMutex as Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_time_driver::Driver;
use embassy_time_queue_utils::Queue;

use crate::interrupt::InterruptExt;
#[cfg(feature = "_grtc")]
use crate::pac::grtc::vals::{Autoen, Busy};
use crate::{interrupt, pac};

#[cfg(feature = "_grtc")]
fn rtc() -> pac::grtc::Grtc {
    pac::GRTC
}

#[cfg(not(feature = "_grtc"))]
fn rtc() -> pac::rtc::Rtc {
    pac::RTC1
}

// On nRF54L/LM GRTC, SYSCOUNTER[n], INTENSETn/INTENCLRn/INTENn, and the
// GRTC_n interrupt all index by "domain":
//   FLPR              = 0
//   App Non-Secure    = 1   (GRTC_1, SYSCOUNTER[1])
//   App Secure        = 2   (GRTC_2, SYSCOUNTER[2])
// See:
//    - nRF54L15: https://github.com/NordicSemiconductor/nrfx/blob/master/bsp/stable/mdk/nrf54l15_interim.h#L64-L75
//    - nRF54LM20A: https://github.com/NordicSemiconductor/nrfx/blob/master/bsp/stable/mdk/nrf54lm20a_interim.h#L64-L75
//    - nRF54LM20B: https://github.com/NordicSemiconductor/nrfx/blob/master/bsp/stable/mdk/nrf54lm20b_interim.h#L64-L75
// And:
//    - https://github.com/NordicSemiconductor/nrfx/blob/master/hal/nrf_grtc.h#L158-L162
#[cfg(feature = "_grtc")]
#[cfg(feature = "_ns")]
const DOMAIN_IDX: usize = 1;

#[cfg(feature = "_grtc")]
#[cfg(not(feature = "_ns"))]
const DOMAIN_IDX: usize = 2;

/// Calculate the timestamp from the period count and the tick count.
///
/// For nRF54 devices and newer, the GRTC counter is 52 bits, so the time driver uses the
/// syscounter and ignores the periods handling, since it overflows every 142 years.
///
/// For most other devices, the RTC counter is 24 bit. Ticking at 32768hz, it overflows every ~8 minutes.
/// This is too short. We must make it "never" overflow.
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
/// `period` is a _32bit integer, so It overflows on 2^32 * 2^23 / 32768 seconds of uptime, which is 34865
/// years. For comparison, flash memory like the one containing your firmware is usually rated to retain
/// data for only 10-20 years. 34865 years is long enough!
#[cfg(not(feature = "_grtc"))]
fn calc_now(period: u32, counter: u32) -> u64 {
    ((period as u64) << 23) + ((counter ^ ((period & 1) << 23)) as u64)
}

/// Read the 52-bit GRTC SYSCOUNTER value.
///
/// Follows the product spec "Recommendation on reading SYSCOUNTER":
///   1. Set SYSCOUNTER[m].ACTIVE to Active
///   2. Wait until SYSCOUNTER[m].SYSCOUNTERH.BUSY is cleared
///   3. Read SYSCOUNTER[m].SYSCOUNTERL/H
///   4. Clear SYSCOUNTER[m].ACTIVE
#[cfg(feature = "_grtc")]
fn syscounter() -> u64 {
    let r = rtc();
    if !r.mode().read().syscounteren() {
        return 0;
    }

    r.syscounter(DOMAIN_IDX).active().write(|w| w.set_active(true));
    loop {
        let countl: u32 = r.syscounter(DOMAIN_IDX).syscounterl().read();
        let counth = r.syscounter(DOMAIN_IDX).syscounterh().read();

        if counth.busy() == Busy::Ready && !counth.overflow() {
            let counth: u32 = counth.value();
            r.syscounter(DOMAIN_IDX).active().write(|w| w.set_active(false));
            return countl as u64 | ((counth as u64) << 32);
        }
        // If overflow or not ready, loop will re-read both registers
    }
}

#[cfg(not(feature = "_grtc"))]
fn compare_n(n: usize) -> u32 {
    1 << (n + 16)
}

#[cfg(feature = "_grtc")]
fn compare_n(n: usize) -> u32 {
    1 << n // GRTC uses bits 0-11 for COMPARE[0-11]
}

#[cfg(test)]
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
}

unsafe impl Send for AlarmState {}

impl AlarmState {
    const fn new() -> Self {
        Self {
            timestamp: Cell::new(u64::MAX),
        }
    }
}

struct RtcDriver {
    /// Number of 2^23 periods elapsed since boot.
    #[cfg(not(feature = "_grtc"))]
    period: AtomicU32,
    /// Timestamp at which to fire alarm. u64::MAX if no alarm is scheduled.
    alarms: Mutex<AlarmState>,
    queue: Mutex<RefCell<Queue>>,
}

embassy_time_driver::time_driver_impl!(static DRIVER: RtcDriver = RtcDriver {
    #[cfg(not(feature = "_grtc"))]
    period: AtomicU32::new(0),
    alarms: Mutex::const_new(CriticalSectionRawMutex::new(), AlarmState::new()),
    queue: Mutex::new(RefCell::new(Queue::new())),
});

impl RtcDriver {
    fn init(&'static self, irq_prio: crate::interrupt::Priority) {
        let r = rtc();
        // Chips without GRTC needs to deal with overflow
        #[cfg(not(feature = "_grtc"))]
        {
            r.cc(3).write(|w| w.set_compare(0x800000));

            r.intenset().write(|w| {
                w.set_ovrflw(true);
                w.set_compare(3, true);
            });

            r.tasks_clear().write_value(1);
            r.tasks_start().write_value(1);

            // Wait for clear
            while r.counter().read().0 != 0 {}

            interrupt::RTC1.set_priority(irq_prio);
            unsafe { interrupt::RTC1.enable() };
        }

        // GRTC initialization for nRF54L/LM series.
        #[cfg(feature = "_grtc")]
        {
            let n = 0;

            // 1. Disable the SYSCOUNTER before reconfiguring
            r.mode().modify(|w| w.set_syscounteren(false));

            // 2. Clear any stale compare event and mask the IRQ for our channel.
            r.events_compare(n).write_value(0);
            r.intenclr(DOMAIN_IDX).write(|w| w.0 = compare_n(n));

            // 3. Clear and start the counter when lftimer is ready
            while !r.status().lftimer().read().ready() {}
            r.tasks_clear().write_value(1);
            while !r.status().lftimer().read().ready() {}
            r.tasks_start().write_value(1);

            // 4. Configure the sleep/wake mechanism and enable the SYSCOUNTER.
            //
            // TIMEOUT and WAKETIME control the GRTC's low-power behaviour.
            // When SYSCOUNTER goes to sleep, the GRTC configures an internal
            // LFCLK compare match based on the next CC[n] value. WAKETIME
            // LFCLK cycles before that match, the GRTC wakes SYSCOUNTER.
            // TIMEOUT extends the active state after the last wake condition
            // clears.
            //
            // Per the product spec: TIMEOUT > WAKETIME + guard_time
            // (minimum guard_time = 1 LFCLK cycle).
            //
            // AUTOEN = CPU_ACTIVE additionally keeps the SYSCOUNTER
            // running whenever any local CPU is active.
            r.timeout().write(|w| w.set_value(5));
            r.waketime().write(|w| w.set_value(4));
            r.mode().modify(|w| {
                w.set_syscounteren(true);
                w.set_autoen(Autoen::CpuActive);
            });

            // 5. Wait for SYSCOUNTER readiness
            let _ = syscounter();

            // 6. Enable the domain IRQ.
            #[cfg(feature = "_ns")]
            {
                interrupt::GRTC_1.set_priority(irq_prio);
                unsafe { interrupt::GRTC_1.enable() };
            }
            #[cfg(not(feature = "_ns"))]
            {
                interrupt::GRTC_2.set_priority(irq_prio);
                unsafe { interrupt::GRTC_2.enable() };
            }
        }
    }

    fn on_interrupt(&self) {
        let r = rtc();

        #[cfg(not(feature = "_grtc"))]
        if r.events_ovrflw().read() == 1 {
            r.events_ovrflw().write_value(0);
            self.next_period();
        }

        #[cfg(not(feature = "_grtc"))]
        if r.events_compare(3).read() == 1 {
            r.events_compare(3).write_value(0);
            self.next_period();
        }

        let n = 0;
        if r.events_compare(n).read() == 1 {
            r.events_compare(n).write_value(0);
            critical_section::with(|cs| {
                self.trigger_alarm(cs);
            });
        }
    }

    #[cfg(not(feature = "_grtc"))]
    fn next_period(&self) {
        critical_section::with(|cs| {
            let r = rtc();
            let period = self.period.load(Ordering::Relaxed) + 1;
            self.period.store(period, Ordering::Relaxed);
            let t = (period as u64) << 23;

            let n = 0;
            let alarm = &self.alarms.borrow(cs);
            let at = alarm.timestamp.get();

            if at < t + 0xc00000 {
                // just enable it. `set_alarm` has already set the correct CC val.
                r.intenset().write(|w| w.0 = compare_n(n));
            }
        })
    }

    fn trigger_alarm(&self, cs: CriticalSection) {
        let n = 0;
        let r = rtc();
        #[cfg(not(feature = "_grtc"))]
        r.intenclr().write(|w| w.0 = compare_n(n));
        #[cfg(feature = "_grtc")]
        r.intenclr(DOMAIN_IDX).write(|w| w.0 = compare_n(n));

        let alarm = &self.alarms.borrow(cs);
        alarm.timestamp.set(u64::MAX);

        // Call after clearing alarm, so the callback can set another alarm.
        let mut next = self.queue.borrow(cs).borrow_mut().next_expiration(self.now());
        while !self.set_alarm(cs, next) {
            next = self.queue.borrow(cs).borrow_mut().next_expiration(self.now());
        }
    }

    fn set_alarm(&self, cs: CriticalSection, timestamp: u64) -> bool {
        let n = 0;
        let alarm = &self.alarms.borrow(cs);
        alarm.timestamp.set(timestamp);

        let r = rtc();

        loop {
            let t = self.now();
            if timestamp <= t {
                // If alarm timestamp has passed the alarm will not fire.
                // Disarm the alarm and return `false` to indicate that.
                #[cfg(not(feature = "_grtc"))]
                r.intenclr().write(|w| w.0 = compare_n(n));
                #[cfg(feature = "_grtc")]
                r.intenclr(DOMAIN_IDX).write(|w| w.0 = compare_n(n));

                alarm.timestamp.set(u64::MAX);

                return false;
            }

            // If it hasn't triggered yet, setup it in the compare channel.

            // Write the CC value regardless of whether we're going to enable it now or not.
            // This way, when we enable it later, the right value is already set.

            // nrf52 docs say :
            //    If the COUNTER is N, writing N or N+1 to a CC register may not trigger a COMPARE event.
            // To workaround this, we never write a timestamp smaller than N+3.
            // N+2 is not safe because rtc can tick from N to N+1 between calling now() and writing cc.
            //
            // Since the critical section does not guarantee that a higher prio interrupt causes
            // this to be delayed, we need to re-check how much time actually passed after setting the
            // alarm, and retry if we are within the unsafe interval still.
            //
            // This means that an alarm can be delayed for up to 2 ticks (from t+1 to t+3), but this is allowed
            // by the Alarm trait contract. What's not allowed is triggering alarms *before* their scheduled time,
            // and we don't do that here.
            #[cfg(not(feature = "_grtc"))]
            {
                let safe_timestamp = timestamp.max(t + 3);
                r.cc(n).write(|w| w.set_compare(safe_timestamp as u32 & 0xFFFFFF));
                let diff = timestamp - t;
                if diff < 0xc00000 {
                    r.intenset().write(|w| w.0 = compare_n(n));

                    // If we have not passed the timestamp, we can be sure the alarm will be invoked. Otherwise,
                    // we need to retry setting the alarm.
                    if self.now() + 2 <= timestamp {
                        return true;
                    }
                } else {
                    // If it's too far in the future, don't setup the compare channel yet.
                    // It will be setup later by `next_period`.
                    r.intenclr().write(|w| w.0 = compare_n(n));
                    return true;
                }
            }

            // The nRF54 datasheet states that 'The EVENTS_COMPARE[n] event is generated immediately if the
            // configured compare value at CC[n] is less than the current SYSCOUNTER value.'. This means we
            // can write the expected timestamp and be sure the alarm is triggered.
            #[cfg(feature = "_grtc")]
            {
                r.events_compare(n).write_value(0);
                // Writes to CC[n].CCL disable the corresponding compare channel and writes to CC[n].CCH enable it.
                // So CC[n].CCL must be written first.
                r.cc(n).ccl().write_value(timestamp as u32);
                r.cc(n).cch().write(|w| w.set_cch((timestamp >> 32) as u32 & 0xFFFFF));
                r.intenset(DOMAIN_IDX).write(|w| w.0 = compare_n(n));
                return true;
            }
        }
    }
}

impl Driver for RtcDriver {
    #[cfg(not(feature = "_grtc"))]
    fn now(&self) -> u64 {
        // `period` MUST be read before `counter`, see comment at the top for details.
        let period = self.period.load(Ordering::Relaxed);
        compiler_fence(Ordering::Acquire);
        let counter = rtc().counter().read().0;
        calc_now(period, counter)
    }

    #[cfg(feature = "_grtc")]
    fn now(&self) -> u64 {
        syscounter()
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

#[cfg(all(feature = "_grtc", feature = "rt", feature = "_ns"))]
#[interrupt]
fn GRTC_1() {
    DRIVER.on_interrupt()
}

#[cfg(all(feature = "_grtc", feature = "rt", not(feature = "_ns")))]
#[interrupt]
fn GRTC_2() {
    DRIVER.on_interrupt()
}

#[cfg(not(feature = "_grtc"))]
#[cfg(feature = "rt")]
#[interrupt]
fn RTC1() {
    DRIVER.on_interrupt()
}

pub(crate) fn init(irq_prio: crate::interrupt::Priority) {
    DRIVER.init(irq_prio)
}
