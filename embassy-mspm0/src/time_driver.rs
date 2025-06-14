use core::cell::{Cell, RefCell};
use core::sync::atomic::{compiler_fence, AtomicU32, Ordering};
use core::task::Waker;

use critical_section::{CriticalSection, Mutex};
use embassy_time_driver::Driver;
use embassy_time_queue_utils::Queue;
use mspm0_metapac::interrupt;
use mspm0_metapac::tim::vals::{Cm, Cvae, CxC, EvtCfg, PwrenKey, Ratio, Repeat, ResetKey};
use mspm0_metapac::tim::{Counterregs16, Tim};

use crate::peripherals;
use crate::timer::SealedTimer;

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

// TODO: RTC

fn regs() -> Tim {
    unsafe { Tim::from_ptr(T::regs()) }
}

fn regs_counter(tim: Tim) -> Counterregs16 {
    unsafe { Counterregs16::from_ptr(tim.counterregs(0).as_ptr()) }
}

/// Clock timekeeping works with something we call "periods", which are time intervals
/// of 2^15 ticks. The Clock counter value is 16 bits, so one "overflow cycle" is 2 periods.
fn calc_now(period: u32, counter: u16) -> u64 {
    ((period as u64) << 15) + ((counter as u32 ^ ((period & 1) << 15)) as u64)
}

/// The TIMx driver uses one of the `TIMG` or `TIMA` timer instances to implement a timer with a 32.768 kHz
/// tick rate. (TODO: Allow setting the tick rate)
///
/// This driver defines a period to be 2^15 ticks. 16-bit timers of course count to 2^16 ticks.
///
/// To generate a period every 2^15 ticks, the CC0 value is set to 2^15 and the load value set to 2^16.
/// Incrementing the period on a CCU0 and load results in the a period of 2^15 ticks.
///
/// For a specific timestamp, load the lower 16 bits into the CC1 value. When the period where the timestamp
/// should be enabled is reached, then the CCU1 (CC1 up) interrupt runs to actually wake the timer.
///
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
    #[inline(never)]
    fn init(&'static self, _cs: CriticalSection) {
        // Clock config
        // TODO: Configurable tick rate up to 4 MHz (32 kHz for now)
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
            // Use LFCLK for a 32.768kHz tick rate
            w.set_lfclk_sel(true);
            // TODO: Allow MFCLK for configurable tick rate up to 4 MHz
            // w.set_mfclk_sel(ClkSel::ENABLE);
        });

        // 2. Divide by TIMCLK, we don't need to divide further for the 32kHz tick rate
        regs.clkdiv().modify(|w| {
            w.set_ratio(Ratio::DIV_BY_1);
        });

        // 3. To be generic across timer instances, we do not use the prescaler.
        // TODO: mspm0-sdk always sets this, regardless of timer width?
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
            // allow counting during debug
            w.set_repeat(Repeat::REPEAT_3);
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

        // Setup the period
        let ctr = regs_counter(regs);

        // Middle
        ctr.cc(0).modify(|w| {
            w.set_ccval(0x7FFF);
        });

        ctr.load().modify(|w| {
            w.set_ld(u16::MAX);
        });

        // Enable the period interrupts
        //
        // This does not appear to ever be set for CPU_INT in the TI SDK and is not technically needed.
        regs.evt_mode().modify(|w| {
            w.set_evt_cfg(0, EvtCfg::SOFTWARE);
        });

        regs.int_event(0).imask().modify(|w| {
            w.set_l(true);
            w.set_ccu0(true);
        });

        unsafe { T::enable_interrupt() };

        // Allow the counter to start counting.
        regs.counterregs(0).ctrctl().modify(|w| {
            w.set_en(true);
        });
    }

    #[inline(never)]
    fn next_period(&self) {
        let r = regs();

        // We only modify the period from the timer interrupt, so we know this can't race.
        let period = self.period.load(Ordering::Relaxed) + 1;
        self.period.store(period, Ordering::Relaxed);
        let t = (period as u64) << 15;

        critical_section::with(move |cs| {
            r.int_event(0).imask().modify(move |w| {
                let alarm = self.alarm.borrow(cs);
                let at = alarm.get();

                if at < t + 0xC000 {
                    // just enable it. `set_alarm` has already set the correct CC1 val.
                    w.set_ccu1(true);
                }
            })
        });
    }

    #[inline(never)]
    fn on_interrupt(&self) {
        let r = regs();

        critical_section::with(|cs| {
            let mis = r.int_event(0).mis().read();

            // Advance to next period if overflowed
            if mis.l() {
                self.next_period();

                r.int_event(0).iclr().write(|w| {
                    w.set_l(true);
                });
            }

            if mis.ccu0() {
                self.next_period();

                r.int_event(0).iclr().write(|w| {
                    w.set_ccu0(true);
                });
            }

            if mis.ccu1() {
                r.int_event(0).iclr().write(|w| {
                    w.set_ccu1(true);
                });

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
        let ctr = regs_counter(r);

        self.alarm.borrow(cs).set(timestamp);

        let t = self.now();

        if timestamp <= t {
            // If alarm timestamp has passed the alarm will not fire.
            // Disarm the alarm and return `false` to indicate that.
            r.int_event(0).imask().modify(|w| w.set_ccu1(false));

            self.alarm.borrow(cs).set(u64::MAX);

            return false;
        }

        // Write the CC1 value regardless of whether we're going to enable it now or not.
        // This way, when we enable it later, the right value is already set.
        ctr.cc(1).write(|w| {
            w.set_ccval(timestamp as u16);
        });

        // Enable it if it'll happen soon. Otherwise, `next_period` will enable it.
        let diff = timestamp - t;
        r.int_event(0).imask().modify(|w| w.set_ccu1(diff < 0xC000));

        // Reevaluate if the alarm timestamp is still in the future
        let t = self.now();
        if timestamp <= t {
            // If alarm timestamp has passed since we set it, we have a race condition and
            // the alarm may or may not have fired.
            // Disarm the alarm and return `false` to indicate that.
            // It is the caller's responsibility to handle this ambiguity.
            r.int_event(0).imask().modify(|w| w.set_ccu1(false));

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

        let period = self.period.load(Ordering::Relaxed);
        // Ensure the compiler does not read the counter before the period.
        compiler_fence(Ordering::Acquire);

        let counter = regs_counter(regs).ctr().read().cctr() as u16;

        calc_now(period, counter)
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
