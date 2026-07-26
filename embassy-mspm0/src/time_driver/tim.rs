use core::cell::{Cell, RefCell};
use core::sync::atomic::{AtomicU32, Ordering, compiler_fence};
use core::task::Waker;

use critical_section::{CriticalSection, Mutex};
use embassy_hal_internal::interrupt::InterruptExt;
use embassy_time_driver::Driver;
use embassy_time_queue_utils::Queue;
use mspm0_metapac::interrupt;
use mspm0_metapac::tim::Tim;
use mspm0_metapac::tim::vals::{Cm, Cvae, CxC, EvtCfg, PwrenKey, Repeat, ResetKey};

use crate::interrupt::typelevel::Interrupt;
use crate::tim::SealedInstance;
use crate::{peripherals, tim};

#[cfg(any(time_driver_timg12, time_driver_timg13))]
compile_error!("TIMG12 and TIMG13 are not supported by the time driver yet");

// Currently TIMG12 and TIMG13 are excluded because those are 32-bit timers.
#[cfg(time_driver_timg0)]
type T = peripherals::TIMG0;
#[cfg(time_driver_timg1)]
type T = peripherals::TIMG1;
#[cfg(time_driver_timg2)]
type T = peripherals::TIMG2;
#[cfg(time_driver_timg3)]
type T = peripherals::TIMG3;
#[cfg(time_driver_timg4)]
type T = peripherals::TIMG4;
#[cfg(time_driver_timg5)]
type T = peripherals::TIMG5;
#[cfg(time_driver_timg6)]
type T = peripherals::TIMG6;
#[cfg(time_driver_timg7)]
type T = peripherals::TIMG7;
#[cfg(time_driver_timg8)]
type T = peripherals::TIMG8;
#[cfg(time_driver_timg9)]
type T = peripherals::TIMG9;
#[cfg(time_driver_timg10)]
type T = peripherals::TIMG10;
#[cfg(time_driver_timg11)]
type T = peripherals::TIMG11;
#[cfg(time_driver_timg14)]
type T = peripherals::TIMG14;
#[cfg(time_driver_tima0)]
type T = peripherals::TIMA0;
#[cfg(time_driver_tima1)]
type T = peripherals::TIMA1;

fn regs() -> Tim {
    T::info().regs
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
// The overflow half must be counted on the *zero* event, not the load event, or that invariant does
// not hold. SLAU847F Figure 28-10 shows that in up counting mode the load event is asserted during
// the `CTR == LOAD` TIMCLK cycle, i.e. one full tick *before* the counter wraps, while the zero
// event is asserted during the `CTR == 0` cycle. Counting on the load event advances `period` while
// `counter` still reads 0xFFFF, which is the one pairing the parity repair below cannot fix: it
// leaves a stale counter against a fresh period for a whole tick on every wrap, worth 2^16 ticks of
// error. The zero event lands exactly on the parity boundary, same as CCU0 lands exactly on
// `CTR == 0x8000`. (LOAD is 2^16 - 1, not 2^16: up counting mode has a period of `LOAD + 1`.)
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

/// TODO: Configurable tick rate
/// TODO: Compensate for per part variance. This can supposedly be done with the FCC system.
/// TODO: Allow using 32-bit timers (TIMG12 and TIMG13).
struct TimxDriver {
    /// Number of 2^15 periods elapsed since boot.
    period: AtomicU32,
    /// Timestamp at which to fire alarm. u64::MAX if no alarm is scheduled.
    alarm: Mutex<Cell<u64>>,
    queue: Mutex<RefCell<Queue>>,
}

impl TimxDriver {
    fn init(&'static self, _cs: CriticalSection) {
        // TODO: Configurable tick rate
        let regs = regs();

        // Reset timer
        regs.gprcm(0).rstctl().write(|w| {
            w.set_resetassert(true);
            w.set_key(ResetKey::KEY);
            w.set_resetstkyclr(true);
        });

        // Power up timer
        regs.gprcm(0).pwren().write(|w| {
            w.set_enable(true);
            w.set_key(PwrenKey::KEY);
        });

        // Following the instructions according to SLAU847D 23.2.1: TIMCLK Configuration

        // 1. Select TIMCLK source
        regs.clksel().modify(|w| {
            w.set_lfclk_sel(true);
        });

        // 2. Divide by TIMCLK, we don't need to divide further for the 32kHz tick rate
        regs.clkdiv().modify(|w| {
            w.set_ratio(0); // + 1
        });

        // Not every timer supports the prescaler so we should zero it.
        regs.commonregs(0).cps().modify(|w| {
            w.set_pcnt(0);
        });

        regs.pdbgctl().modify(|w| {
            w.set_free(true);
        });

        // 4. Enable the TIMCLK.
        regs.commonregs(0).cclkctl().modify(|w| {
            w.set_clken(true);
        });

        regs.counterregs(0).ctrctl().modify(|w| {
            w.set_repeat(Repeat::REPEAT_1);
            w.set_cvae(Cvae::ZEROVAL);
            w.set_cm(Cm::UP);

            // Must explicitly set CZC, CAC and CLC to 0 in order for all the timers to count.
            //
            // The reset value of these registers is 0x07, which is a reserved value.
            //
            // Looking at a bit representation of the reset value, this appears to be an AND
            // of 2-input QEI mode and CCCTL_3 ACOND. Given that TIMG14 and TIMA0 have no QEI
            // and 4 capture and compare channels, this works by accident for those timer units.
            w.set_czc(CxC::CCTL0);
            w.set_cac(CxC::CCTL0);
            w.set_clc(CxC::CCTL0);
        });

        // Middle
        regs.counterregs(0).cc(0).write_value(0x8000 as u32);
        regs.counterregs(0).load().write_value(u16::MAX as u32);

        // Enable the period interrupts
        //
        // This does not appear to ever be set for CPU_INT in the TI SDK and is not technically needed.
        regs.evt_mode().modify(|w| {
            w.set_evt_cfg(0, EvtCfg::SOFTWARE);
        });

        regs.cpu_int(0).imask().modify(|w| {
            w.set_z(true);
            w.set_ccu0(true);
        });

        <T as tim::Instance>::Interrupt::IRQ.unpend();
        unsafe { <T as tim::Instance>::Interrupt::IRQ.enable() };

        // Allow the counter to start counting.
        regs.counterregs(0).ctrctl().modify(|w| {
            w.set_en(true);
        });
    }

    fn next_period(&self, cs: CriticalSection) {
        let r = regs();

        // We only modify the period from the timer interrupt, so we know this can't race.
        let period = self.period.load(Ordering::Relaxed) + 1;
        self.period.store(period, Ordering::Relaxed);
        let t = (period as u64) << 15;

        r.cpu_int(0).imask().modify(move |w| {
            let alarm = self.alarm.borrow(cs);
            let at = alarm.get();

            if at < t + 0xC000 {
                // just enable it. `set_alarm` has already set the correct CC1 val.
                w.set_ccu1(true);
            }
        });
    }

    fn on_interrupt(&self) {
        let r = regs();

        critical_section::with(|cs| {
            let mis = r.cpu_int(0).mis().read();

            // Clear the flags we are about to handle before handling them. MIS and ICLR have the
            // same layout, and writing back only the bits we read leaves any event latched during
            // the handler pending, so it is picked up on re-entry rather than lost.
            r.cpu_int(0).iclr().write_value(mis);

            // Overflow
            if mis.z() {
                self.next_period(cs);
            }

            // Half overflow
            if mis.ccu0() {
                self.next_period(cs);
            }

            if mis.ccu1() {
                self.trigger_alarm(cs);
            }
        });
    }

    fn trigger_alarm(&self, cs: CriticalSection) {
        let mut next = self.queue.borrow(cs).borrow_mut().next_expiration(self.now());

        while !self.set_alarm(cs, next) {
            next = self.queue.borrow(cs).borrow_mut().next_expiration(self.now());
        }
    }

    fn set_alarm(&self, cs: CriticalSection, timestamp: u64) -> bool {
        let r = regs();

        self.alarm.borrow(cs).set(timestamp);

        let t = self.now();

        if timestamp <= t {
            // If alarm timestamp has passed the alarm will not fire.
            // Disarm the alarm and return `false` to indicate that.
            r.cpu_int(0).imask().modify(|w| w.set_ccu1(false));

            self.alarm.borrow(cs).set(u64::MAX);

            return false;
        }

        // Write the CC1 value regardless of whether we're going to enable it now or not.
        // This way, when we enable it later, the right value is already set.
        //
        // Cast to u16 and then u32 to clamp to 16-bit timer limits.
        r.counterregs(0).cc(1).write_value(timestamp as u16 as u32);

        // Enable it if it'll happen soon. Otherwise, `next_period` will enable it.
        let diff = timestamp - t;
        r.cpu_int(0).imask().modify(|w| w.set_ccu1(diff < 0xC000));

        // Reevaluate if the alarm timestamp is still in the future
        let t = self.now();
        if timestamp <= t {
            // If alarm timestamp has passed since we set it, we have a race condition and
            // the alarm may or may not have fired.
            // Disarm the alarm and return `false` to indicate that.
            // It is the caller's responsibility to handle this ambiguity.
            r.cpu_int(0).imask().modify(|w| w.set_ccu1(false));

            self.alarm.borrow(cs).set(u64::MAX);

            return false;
        }

        // We're confident the alarm will ring in the future.
        true
    }
}

impl Driver for TimxDriver {
    fn now(&self) -> u64 {
        let regs = regs();

        // On MSPM0 this sequence reread and comparison must be done or else time may
        // appear to go backwards.
        //
        // The timer counter and the software period counter are not updated as one
        // atomic operation. It is possible for the timer interrupt to increment the
        // period counter while reading the counter.
        //
        // For example, `period` may be read as X while the timer is near the end of
        // that period. The timer then wraps and the interrupt increments `period` to
        // X + 1 before the counter is read. If the counter read returns 0x0000, using
        // the stale period X would produce a timestamp 32768 ticks in the past.
        loop {
            let period = self.period.load(Ordering::Relaxed);
            // Ensure the compiler does not read the counter before the period.
            compiler_fence(Ordering::Acquire);

            let counter = regs.counterregs(0).ctr().read() as u16;

            // Ensure the compiler does not read the period again before the counter.
            compiler_fence(Ordering::Acquire);
            let period2 = self.period.load(Ordering::Relaxed);

            if period != period2 {
                continue;
            }

            return calc_now(period, counter);
        }
    }

    fn schedule_wake(&self, at: u64, waker: &Waker) {
        critical_section::with(|cs| {
            let mut queue = self.queue.borrow(cs).borrow_mut();

            if queue.schedule_wake(at, waker) {
                let mut next = queue.next_expiration(self.now());

                while !self.set_alarm(cs, next) {
                    next = queue.next_expiration(self.now());
                }
            }
        });
    }
}

embassy_time_driver::time_driver_impl!(static DRIVER: TimxDriver = TimxDriver {
    period: AtomicU32::new(0),
    alarm: Mutex::new(Cell::new(u64::MAX)),
    queue: Mutex::new(RefCell::new(Queue::new()))
});

pub(crate) fn init(cs: CriticalSection) {
    DRIVER.init(cs);
}

#[cfg(time_driver_timg0)]
#[interrupt]
fn TIMG0() {
    DRIVER.on_interrupt();
}

#[cfg(time_driver_timg1)]
#[interrupt]
fn TIMG1() {
    DRIVER.on_interrupt();
}

#[cfg(time_driver_timg2)]
#[interrupt]
fn TIMG2() {
    DRIVER.on_interrupt();
}

#[cfg(time_driver_timg3)]
#[interrupt]
fn TIMG3() {
    DRIVER.on_interrupt();
}

#[cfg(time_driver_timg4)]
#[interrupt]
fn TIMG4() {
    DRIVER.on_interrupt();
}

#[cfg(time_driver_timg5)]
#[interrupt]
fn TIMG5() {
    DRIVER.on_interrupt();
}

#[cfg(time_driver_timg6)]
#[interrupt]
fn TIMG6() {
    DRIVER.on_interrupt();
}

#[cfg(time_driver_timg7)]
#[interrupt]
fn TIMG7() {
    DRIVER.on_interrupt();
}

#[cfg(time_driver_timg8)]
#[interrupt]
fn TIMG8() {
    DRIVER.on_interrupt();
}

#[cfg(time_driver_timg9)]
#[interrupt]
fn TIMG9() {
    DRIVER.on_interrupt();
}

#[cfg(time_driver_timg10)]
#[interrupt]
fn TIMG10() {
    DRIVER.on_interrupt();
}

#[cfg(time_driver_timg11)]
#[interrupt]
fn TIMG11() {
    DRIVER.on_interrupt();
}

// TODO: TIMG12 and TIMG13

#[cfg(time_driver_timg14)]
#[interrupt]
fn TIMG14() {
    DRIVER.on_interrupt();
}

#[cfg(time_driver_tima0)]
#[interrupt]
fn TIMA0() {
    DRIVER.on_interrupt();
}

#[cfg(time_driver_tima1)]
#[interrupt]
fn TIMA1() {
    DRIVER.on_interrupt();
}
