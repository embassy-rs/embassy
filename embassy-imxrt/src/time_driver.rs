//! Time Driver.
use core::cell::{Cell, RefCell};
#[cfg(feature = "time-driver-rtc")]
use core::sync::atomic::{AtomicU32, Ordering, compiler_fence};

use critical_section::CriticalSection;
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_time_driver::Driver;
use embassy_time_queue_utils::Queue;

#[cfg(feature = "time-driver-os-timer")]
use crate::clocks::enable;
use crate::interrupt::InterruptExt;
use crate::{interrupt, pac};

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

#[cfg(feature = "time-driver-rtc")]
fn rtc() -> pac::rtc::Rtc {
    pac::RTC
}

/// Calculate the timestamp from the period count and the tick count.
///
/// To get `now()`, `period` is read first, then `counter` is read. If the counter value matches
/// the expected range for the `period` parity, we're done. If it doesn't, this means that
/// a new period start has raced us between reading `period` and `counter`, so we assume the `counter` value
/// corresponds to the next period.
///
/// the 1kHz RTC counter is 16 bits and RTC doesn't have separate compare channels,
/// so using a 32 bit GPREG0-2 as counter, compare, and int_en
/// `period` is a 32bit integer, gpreg 'counter' is 31 bits plus the parity bit for overflow detection
#[cfg(feature = "time-driver-rtc")]
fn calc_now(period: u32, counter: u32) -> u64 {
    ((period as u64) << 31) + ((counter ^ ((period & 1) << 31)) as u64)
}

#[cfg(feature = "time-driver-rtc")]
embassy_time_driver::time_driver_impl!(static DRIVER: Rtc = Rtc {
    period: AtomicU32::new(0),
    alarms:  Mutex::const_new(CriticalSectionRawMutex::new(), AlarmState::new()),
    queue: Mutex::new(RefCell::new(Queue::new())),
});

#[cfg(feature = "time-driver-rtc")]
struct Rtc {
    /// Number of 2^31 periods elapsed since boot.
    period: AtomicU32,
    /// Timestamp at which to fire alarm. u64::MAX if no alarm is scheduled.
    alarms: Mutex<CriticalSectionRawMutex, AlarmState>,
    queue: Mutex<CriticalSectionRawMutex, RefCell<Queue>>,
}

#[cfg(feature = "time-driver-rtc")]
impl Rtc {
    fn init(&'static self, irq_prio: crate::interrupt::Priority) {
        let r = rtc();
        // enable RTC int (1kHz since subsecond doesn't generate an int)
        r.ctrl().modify(|w| w.set_rtc1khz_en(true));
        // TODO: low power support. line above is leaving out write to .set_wakedpd_en(true)
        // which enables wake from deep power down

        // gpreg1 is used as a compare register for triggering an alarm so to avoid
        // unnecessary triggers after initialization, this is set to 0x:FFFF_FFFF
        rtc().gpreg(1).write(|w| w.set_gpdata(u32::MAX));
        // The following loads 10 into the count-down timer.
        r.wake().write_value(pac::rtc::regs::Wake(0xA));
        interrupt::RTC.set_priority(irq_prio);
        unsafe { interrupt::RTC.enable() };
    }

    #[cfg(feature = "rt")]
    fn on_interrupt(&self) {
        use crate::pac::rtc::vals::Wake1khz;

        let r = rtc();
        // This interrupt fires every 10 ticks of the 1kHz RTC high res clk and adds
        // 10 to the 31 bit counter gpreg0. The 32nd bit is used for parity detection
        // This is done to avoid needing to calculate # of ticks spent on interrupt
        // handlers to recalibrate the clock between interrupts
        //
        // TODO: this is admittedly not great for power that we're generating this
        // many interrupts, will probably get updated in future iterations.
        if r.ctrl().read().wake1khz() == Wake1khz::TIME_OUT {
            r.ctrl().modify(|w| w.set_wake1khz(Wake1khz::TIME_OUT));
            // The following reloads 10 into the count-down timer after it triggers an int.
            // The countdown begins anew after the write so time can continue to be measured.
            r.wake().write_value(pac::rtc::regs::Wake(0xA));
            if (rtc().gpreg(0).read().0 + 0xA) > 0x8000_0000 {
                // if we're going to "overflow", increase the period
                self.next_period();
                let rollover_diff = 0x8000_0000 - (rtc().gpreg(0).read().0 + 0xA);
                // In order to not "lose" time when incrementing the period, gpreg0,
                // the extended counter, is restarted at the # of ticks it would overflow by
                rtc().gpreg(0).write_value(pac::rtc::regs::Gpreg(rollover_diff));
            } else {
                rtc().gpreg(0).modify(|w| w.0 = w.0 + 0xA);
            }
        }

        critical_section::with(|cs| {
            // gpreg2 as an "int_en" set by next_period(). This is
            // 1 when the timestamp for the alarm deadline expires
            // before the counter register overflows again.
            if rtc().gpreg(2).read().gpdata() == 1 {
                // gpreg0 is our extended counter register, check if
                // our counter is larger than the compare value
                if rtc().gpreg(0).read().0 > rtc().gpreg(1).read().0 {
                    self.trigger_alarm(cs);
                }
            }
        })
    }

    #[cfg(feature = "rt")]
    fn next_period(&self) {
        critical_section::with(|cs| {
            let period = self
                .period
                .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |p| Some(p + 1))
                .unwrap_or_else(|p| {
                    trace!("Unable to increment period. Time is now inaccurate");
                    // TODO: additional error handling beyond logging

                    p
                });
            let t = (period as u64) << 31;

            let alarm = &self.alarms.borrow(cs);
            let at = alarm.timestamp.get();
            if at < t + 0xc000_0000 {
                // gpreg2 is an alarm enable. If the alarm must trigger within the
                // next period, then just enable it. `set_alarm` has already set the
                // correct CC val.
                rtc().gpreg(2).write(|w| w.set_gpdata(1));
            }
        })
    }

    #[must_use]
    fn set_alarm(&self, cs: CriticalSection, timestamp: u64) -> bool {
        let alarm = self.alarms.borrow(cs);
        alarm.timestamp.set(timestamp);

        let t = self.now();
        if timestamp <= t {
            // gpreg2 is always just used as the alarm enable for the timer driver.
            // If alarm timestamp has passed the alarm will not fire.
            // Disarm the alarm and return `false` to indicate that.
            rtc().gpreg(2).write(|w| w.set_gpdata(0));

            alarm.timestamp.set(u64::MAX);

            return false;
        }

        // If it hasn't triggered yet, setup it by writing to the compare field
        // An alarm can be delayed, but this is allowed by the Alarm trait contract.
        // What's not allowed is triggering alarms *before* their scheduled time,
        let safe_timestamp = timestamp.max(t + 10); //t+3 was done for nrf chip, choosing 10

        // When a new alarm is set, the compare register, gpreg1, is set to the last
        // 31 bits of the timestamp as the 32nd and final bit is used for the parity
        // check in `next_period`. `period` will be used for the upper bits in a
        // timestamp comparison.
        rtc()
            .gpreg(1)
            .write_value(pac::rtc::regs::Gpreg(safe_timestamp as u32 & 0x7FFF_FFFF));

        // The following checks that the difference in timestamp is less than the overflow period
        let diff = timestamp - t;
        if diff < 0xc000_0000 {
            // this is 0b11 << (30). NRF chip used 23 bit periods and checked against 0b11<<22

            // If the alarm must trigger within the next period, set the "int enable"
            rtc().gpreg(2).write(|w| w.set_gpdata(1));
        } else {
            // If alarm must trigger some time after the current period, too far in the
            // future, don't setup the alarm enable, gpreg2, yet. It will be setup
            // later by `next_period`.
            rtc().gpreg(2).write(|w| w.set_gpdata(0));
        }

        true
    }

    #[cfg(feature = "rt")]
    fn trigger_alarm(&self, cs: CriticalSection) {
        let mut next = self.queue.borrow(cs).borrow_mut().next_expiration(self.now());
        while !self.set_alarm(cs, next) {
            next = self.queue.borrow(cs).borrow_mut().next_expiration(self.now());
        }
    }
}

#[cfg(feature = "time-driver-rtc")]
impl Driver for Rtc {
    fn now(&self) -> u64 {
        // `period` MUST be read before `counter`, see comment at the top for details.
        let period = self.period.load(Ordering::Acquire);
        compiler_fence(Ordering::Acquire);
        let counter = rtc().gpreg(0).read().0;
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

#[cfg(all(feature = "rt", feature = "time-driver-rtc"))]
#[allow(non_snake_case)]
#[interrupt]
fn RTC() {
    DRIVER.on_interrupt()
}

#[cfg(feature = "time-driver-os-timer")]
fn os() -> pac::ostimer::Ostimer {
    pac::OSTIMER0
}

/// Convert gray to decimal
///
/// Os Event provides a 64-bit timestamp gray-encoded. All we have to
/// do here is read both 32-bit halves of the register and convert
/// from gray to regular binary.
#[cfg(feature = "time-driver-os-timer")]
fn gray_to_dec(gray: u64) -> u64 {
    let mut dec = gray;

    dec ^= dec >> 1;
    dec ^= dec >> 2;
    dec ^= dec >> 4;
    dec ^= dec >> 8;
    dec ^= dec >> 16;
    dec ^= dec >> 32;

    dec
}

/// Convert decimal to gray
///
/// Before writing match value to the target register, we must convert
/// it back into gray code.
#[cfg(feature = "time-driver-os-timer")]
fn dec_to_gray(dec: u64) -> u64 {
    let gray = dec;
    gray ^ (gray >> 1)
}

#[cfg(feature = "time-driver-os-timer")]
embassy_time_driver::time_driver_impl!(static DRIVER: OsTimer = OsTimer {
    alarms:  Mutex::const_new(CriticalSectionRawMutex::new(), AlarmState::new()),
    queue: Mutex::new(RefCell::new(Queue::new())),
});

#[cfg(feature = "time-driver-os-timer")]
struct OsTimer {
    /// Timestamp at which to fire alarm. u64::MAX if no alarm is scheduled.
    alarms: Mutex<CriticalSectionRawMutex, AlarmState>,
    queue: Mutex<CriticalSectionRawMutex, RefCell<Queue>>,
}

#[cfg(feature = "time-driver-os-timer")]
impl OsTimer {
    fn init(&'static self, irq_prio: crate::interrupt::Priority) {
        // init alarms
        critical_section::with(|cs| {
            let alarm = DRIVER.alarms.borrow(cs);
            alarm.timestamp.set(u64::MAX);
        });

        // Enable clocks. Documentation advises AGAINST resetting this
        // peripheral.
        enable::<crate::peripherals::OS_EVENT>();

        interrupt::OS_EVENT.disable();

        // Make sure interrupt is masked
        os().osevent_ctrl().modify(|w| w.set_ostimer_intena(false));

        // Default to the end of time
        os().match_l().write_value(pac::ostimer::regs::MatchL(0xffff_ffff));
        os().match_h().write_value(pac::ostimer::regs::MatchH(0xffff_ffff));

        interrupt::OS_EVENT.unpend();
        interrupt::OS_EVENT.set_priority(irq_prio);
        unsafe { interrupt::OS_EVENT.enable() };
    }

    fn set_alarm(&self, cs: CriticalSection, timestamp: u64) -> bool {
        let alarm = self.alarms.borrow(cs);
        alarm.timestamp.set(timestamp);

        // Wait until we're allowed to write to MATCH_L/MATCH_H
        // registers
        while os().osevent_ctrl().read().match_wr_rdy() {}

        let t = self.now();
        if timestamp <= t {
            os().osevent_ctrl().modify(|w| w.set_ostimer_intena(false));
            alarm.timestamp.set(u64::MAX);
            return false;
        }

        let gray_timestamp = dec_to_gray(timestamp);

        os().match_l()
            .write_value(pac::ostimer::regs::MatchL(gray_timestamp as u32 & 0xffff_ffff));
        os().match_h()
            .write_value(pac::ostimer::regs::MatchH((gray_timestamp >> 32) as u32));
        os().osevent_ctrl().modify(|w| w.set_ostimer_intena(true));

        true
    }

    #[cfg(feature = "rt")]
    fn trigger_alarm(&self, cs: CriticalSection) {
        let mut next = self.queue.borrow(cs).borrow_mut().next_expiration(self.now());
        while !self.set_alarm(cs, next) {
            next = self.queue.borrow(cs).borrow_mut().next_expiration(self.now());
        }
    }

    #[cfg(feature = "rt")]
    fn on_interrupt(&self) {
        critical_section::with(|cs| {
            if os().osevent_ctrl().read().ostimer_intrflag() {
                os().osevent_ctrl().modify(|w| {
                    w.set_ostimer_intena(false);
                    w.set_ostimer_intrflag(true);
                });
                self.trigger_alarm(cs);
            }
        });
    }
}

#[cfg(feature = "time-driver-os-timer")]
impl Driver for OsTimer {
    fn now(&self) -> u64 {
        let mut t = os().evtimerh().read().0 as u64;
        t <<= 32;
        t |= os().evtimerl().read().0 as u64;
        gray_to_dec(t)
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

#[cfg(all(feature = "rt", feature = "time-driver-os-timer"))]
#[allow(non_snake_case)]
#[interrupt]
fn OS_EVENT() {
    DRIVER.on_interrupt()
}

pub(crate) fn init(irq_prio: crate::interrupt::Priority) {
    DRIVER.init(irq_prio)
}
