#![allow(non_snake_case)]

#[cfg(feature = "low-power")]
use core::cell::Cell;
use core::cell::RefCell;
#[cfg(feature = "low-power")]
use core::sync::atomic::AtomicBool;
use core::sync::atomic::{AtomicU32, Ordering, compiler_fence};

use critical_section::CriticalSection;
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_time_driver::{Driver, TICK_HZ};
use embassy_time_queue_utils::Queue;
use stm32_metapac::lptim::{Lptim, regs};

use super::AlarmState;
use crate::interrupt::typelevel::Interrupt;
use crate::lptim::SealedInstance;
use crate::pac::lptim::vals;
use crate::rcc::SealedRccPeripheral;
use crate::{peripherals, rcc};

#[cfg(time_driver_lptim1)]
type T = peripherals::LPTIM1;
#[cfg(time_driver_lptim2)]
type T = peripherals::LPTIM2;
#[cfg(time_driver_lptim3)]
type T = peripherals::LPTIM3;

fn regs_lptim() -> Lptim {
    T::regs()
}

pub(crate) struct RtcDriver {
    /// Number of 2^15 periods elapsed since boot.
    period: AtomicU32,
    alarm: Mutex<CriticalSectionRawMutex, AlarmState>,
    #[cfg(feature = "low-power")]
    is_stopped: AtomicBool,
    #[cfg(feature = "low-power")]
    /// The minimum pause time beyond which the executor will enter a low-power state.
    min_stop_pause: Mutex<CriticalSectionRawMutex, Cell<embassy_time::Duration>>,
    queue: Mutex<CriticalSectionRawMutex, RefCell<Queue>>,
}

embassy_time_driver::time_driver_impl!(static DRIVER: RtcDriver = RtcDriver {
    period: AtomicU32::new(0),
    alarm: Mutex::const_new(CriticalSectionRawMutex::new(), AlarmState::new()),
    #[cfg(feature = "low-power")]
    is_stopped: AtomicBool::new(false),
    #[cfg(feature = "low-power")]
    min_stop_pause: Mutex::const_new(CriticalSectionRawMutex::new(), Cell::new(embassy_time::Duration::from_millis(0))),
    queue: Mutex::new(RefCell::new(Queue::new())),
});

impl RtcDriver {
    /// initialize the timer, but don't start it.  Used for chips like stm32wle5
    /// for low power where the timer config is lost in STOP2.
    pub(crate) fn init_timer(&'static self, _cs: critical_section::CriticalSection) {
        let r = regs_lptim();

        // we want this to increment the stop mode counter (some lp timer can't do STOP2)
        rcc::enable_and_reset_without_stop::<T>();

        let timer_freq = T::frequency();

        r.cnt().write(|w| w.set_cnt(0));

        // let psc = timer_freq.0 / TICK_HZ as u32 - 1;
        let psc = timer_freq.0 / TICK_HZ as u32;
        let psc = match psc {
            128 => vals::Presc::DIV128,
            64 => vals::Presc::DIV64,
            32 => vals::Presc::DIV32,
            16 => vals::Presc::DIV16,
            8 => vals::Presc::DIV8,
            4 => vals::Presc::DIV4,
            2 => vals::Presc::DIV2,
            1 => vals::Presc::DIV1,
            // TODO: we could compute the valid TICK_HZ for the valid prescalers to include in the panic message
            _ => panic!("Invalid prescaler: {} for timer frequency: {}Hz", psc, timer_freq.0),
        };

        trace!(
            "init: setting presc: {} timer_freq: {}Hz TICK_HZ: {}",
            psc, timer_freq, TICK_HZ
        );
        r.cfgr().modify(|w| w.set_presc(psc));

        // RM says timer must be enabled before setting arr or cmp
        r.cr().modify(|w| w.set_enable(true));
        trace!("init: arr: {:?}", r.arr().read());
        // TRM says this is updated immediately if the timer is not started so no need to check for arrok! (stm32wl5 & stm32wle)
        r.arr().write(|w| w.set_arr(u16::MAX));

        // Enable overflow interrupts
        T::regs().ier().modify(|w| w.set_ueie(true));

        <T as crate::lptim::SealedBasicInstance>::GlobalInterrupt::unpend();
        unsafe {
            <T as crate::lptim::SealedBasicInstance>::GlobalInterrupt::enable();
        }
        #[cfg(feature = "low-power")]
        {
            // TODO: use a crate constant for the core number!!!!!!
            #[cfg(feature = "_core-cm4")]
            const CPU: usize = 0;
            #[cfg(feature = "_core-cm0p")]
            const CPU: usize = 1;
            // TODO: define these elsewhere?
            #[cfg(all(any(stm32wlex, stm32wl5x), time_driver_lptim1))]
            const EXTI_WAKEUP_LINE: usize = 29;
            #[cfg(all(any(stm32wlex, stm32wl5x), time_driver_lptim2))]
            const EXTI_WAKEUP_LINE: usize = 30;
            #[cfg(all(any(stm32wlex, stm32wl5x), time_driver_lptim3))]
            const EXTI_WAKEUP_LINE: usize = 31;

            #[cfg(any(stm32wb, stm32wl5x))]
            {
                crate::pac::EXTI
                    .cpu(CPU)
                    .imr(0)
                    .modify(|w| w.set_line(EXTI_WAKEUP_LINE, true));
                // TODO: from the RM: after line 22 all are reserved - try removing this
                crate::pac::EXTI
                    .cpu(CPU)
                    .emr(0)
                    .modify(|w| w.set_line(EXTI_WAKEUP_LINE, true));
            }
            #[cfg(not(any(stm32wb, stm32wl5x)))]
            {
                crate::pac::EXTI.imr(0).modify(|w| w.set_line(EXTI_WAKEUP_LINE, true));
            }
        }
    }

    fn init(&'static self, cs: CriticalSection) {
        self.init_timer(cs);
        regs_lptim().cr().modify(|w| w.set_cntstrt(true));
    }

    pub(crate) fn on_interrupt(&self) {
        let r = regs_lptim();

        critical_section::with(|cs| {
            let sr = r.isr().read();
            let ier = r.ier().read();
            trace!("on_interrupt: sr: {:?}, ier: {:?}", sr, ier);

            // Clear all interrupt flags. Bits in ICR are "write 1 to clear"
            r.icr().write_value(regs::Icr(ier.0));
            r.icr().write_value(regs::Icr(sr.0));

            // Overflow
            if sr.ue() {
                self.next_period();
            }

            if sr.ccif(0) && ier.ccie(0) {
                self.trigger_alarm(cs);
            }
        })
    }

    fn next_period(&self) {
        let r = regs_lptim();

        // We only modify the period from the timer interrupt, so we know this can't race.
        let period = self.period.load(Ordering::Relaxed) + 1;
        self.period.store(period, Ordering::Relaxed);
        let t = (period as u64) << 16;

        critical_section::with(move |cs| {
            r.ier().modify(move |w| {
                let alarm = self.alarm.borrow(cs);
                let at = alarm.timestamp.get();

                if at < t + 0xc000 {
                    // just enable it. `set_alarm` has already set the correct CCR val.
                    w.set_ccie(0, true);
                }
            })
        })
    }

    fn trigger_alarm(&self, cs: CriticalSection) {
        trace!("trigger_alarm");
        let mut next = self.queue.borrow(cs).borrow_mut().next_expiration(self.now());
        while !self.set_alarm(cs, next) {
            next = self.queue.borrow(cs).borrow_mut().next_expiration(self.now());
        }
    }

    fn set_alarm(&self, cs: CriticalSection, timestamp: u64) -> bool {
        trace!("set_alarm: timestamp: {}", timestamp);
        let r = regs_lptim();

        self.alarm.borrow(cs).timestamp.set(timestamp);

        let t = self.now();
        if timestamp <= t {
            trace!("set_alarm: timestamp <= t");
            // If alarm timestamp has passed the alarm will not fire.
            // Disarm the alarm and return `false` to indicate that.
            r.ier().modify(|w| w.set_ccie(0, false));

            self.alarm.borrow(cs).timestamp.set(u64::MAX);

            return false;
        }

        // Write the CCR value regardless of whether we're going to enable it now or not.
        // This way, when we enable it later, the right value is already set.
        r.cmp().write(|w| w.set_cmp(timestamp as u16));
        while !r.isr().read().cmpok(0) {}
        // r.icr().write(|w| w.set_cmpokcf(0, true));

        // Enable it if it'll happen soon. Otherwise, `next_period` will enable it.
        let diff = timestamp - t;
        r.ier().modify(|w| w.set_ccie(0, diff < 0xc000));

        // Reevaluate if the alarm timestamp is still in the future
        let t = self.now();
        if timestamp <= t {
            trace!("set_alarm: timestamp <= t (after set)");
            // If alarm timestamp has passed since we set it, we have a race condition and
            // the alarm may or may not have fired.
            // Disarm the alarm and return `false` to indicate that.
            // It is the caller's responsibility to handle this ambiguity.
            r.ier().modify(|w| w.set_ccie(0, false));

            self.alarm.borrow(cs).timestamp.set(u64::MAX);

            return false;
        }

        trace!("set_alarm: true");
        // We're confident the alarm will ring in the future.
        true
    }
}

#[cfg(feature = "low-power")]
impl super::LPTimeDriver for RtcDriver {
    /// Compute the approximate amount of time until the next alarm
    fn time_until_next_alarm(&self, cs: CriticalSection) -> embassy_time::Duration {
        let now = self.now() + 32;

        embassy_time::Duration::from_ticks(self.alarm.borrow(cs).timestamp.get().saturating_sub(now))
    }

    /// Set the stopped flag or return an error if the time until the next alarm is less than the minimum stop pause
    fn pause_time(&self, cs: CriticalSection) -> Result<(), ()> {
        trace!("pause_time");
        let time_until_next_alarm = self.time_until_next_alarm(cs);
        if time_until_next_alarm < self.min_stop_pause.borrow(cs).get() {
            trace!(
                "time_until_next_alarm < self.min_stop_pause ({})",
                time_until_next_alarm
            );
            Err(())
        } else {
            self.is_stopped.store(true, Ordering::Relaxed);
            Ok(())
        }
    }

    /// Reset the stopped flag
    fn resume_time(&self, _cs: CriticalSection) {
        trace!("resume_time");
        self.is_stopped.store(false, Ordering::Relaxed);
    }

    /// Returns whether time is currently "stopped"
    fn is_stopped(&self) -> bool {
        self.is_stopped.load(Ordering::Relaxed)
    }
}

impl Driver for RtcDriver {
    fn now(&self) -> u64 {
        let r = regs_lptim();
        loop {
            let period = self.period.load(Ordering::Relaxed);
            compiler_fence(Ordering::Acquire);
            let counter = r.cnt().read().cnt();
            let now = ((period as u64) << 16) + (counter as u64);

            if self.period.load(Ordering::Relaxed) == period {
                break now;
            }
        }
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
