#![allow(non_snake_case)]

#[cfg(feature = "low-power")]
use core::cell::Cell;
use core::cell::RefCell;
#[cfg(feature = "low-power")]
use core::sync::atomic::AtomicBool;
use core::sync::atomic::{AtomicU32, Ordering};

use critical_section::CriticalSection;
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_time_driver::{Driver, TICK_HZ};
use embassy_time_queue_utils::Queue;
use stm32_metapac::lptim::Lptim;
#[cfg(lptim_v2b)]
use stm32_metapac::lptim::regs::Dier;
#[cfg(not(lptim_v2a))]
use stm32_metapac::lptim::regs::Icr;
#[cfg(lptim_v2a)]
use stm32_metapac::lptim::regs::{DierAdv as Dier, IcrAdv as Icr};

use super::AlarmState;
use crate::_generated::time_driver_clock;
use crate::interrupt::typelevel::Interrupt;
use crate::lptim::{SealedInstance, vals};
use crate::rcc;
use crate::rcc::SealedRccPeripheral;

/// The EXTI line of the LPTIM1 event on F4 and F7.
#[cfg(all(feature = "low-power", any(stm32f4, stm32f7)))]
const F4_F7_EXTI_LINE: usize = 23;

type T = crate::_generated::TimeDriverPeripheral;

fn regs_lptim() -> Lptim {
    T::regs()
}

// LPTIM v2a/v2b (STM32WBA, STM32U5, STM32U3, STM32H5, STM32U0, STM32C5) renames the interrupt
// enable register from IER to DIER and uses IcrAdv/DierAdv register types
// instead of Icr/Ier. The compare register also changes from a single `cmp()`
// to an indexed `ccr(n)`.

/// Check if the compare-capture interrupt is enabled for channel `n`.
fn ier_ccie(r: Lptim, n: usize) -> bool {
    #[cfg(not(any(lptim_v2a, lptim_v2b)))]
    {
        r.ier().read().ccie(n)
    }
    #[cfg(any(lptim_v2a, lptim_v2b))]
    {
        r.dier().read().ccie(n)
    }
}

/// Whether the counter overflowed. LPTIM v1, v1a and v1b have no update event; their ARRM
/// (autoreload match) flag is set as the counter wraps from ARR to 0.
macro_rules! overflowed {
    ($isr:expr) => {{
        #[cfg(any(lptim_v1, lptim_v1a, lptim_v1b))]
        let overflowed = $isr.arrm();
        #[cfg(not(any(lptim_v1, lptim_v1a, lptim_v1b)))]
        let overflowed = $isr.ue();
        overflowed
    }};
}

/// Enable or disable the counter overflow interrupt.
fn ier_set_overflow_ie(r: Lptim, val: bool) {
    #[cfg(any(lptim_v1, lptim_v1a, lptim_v1b))]
    r.ier().modify(|w| w.set_arrmie(val));
    #[cfg(lptim_v1c)]
    r.ier().modify(|w| w.set_ueie(val));
    #[cfg(any(lptim_v2a, lptim_v2b))]
    dier_modify(r, |w| w.set_ueie(val));
}

fn ier_set_ccie(r: Lptim, n: usize, val: bool) {
    #[cfg(not(any(lptim_v2a, lptim_v2b)))]
    r.ier().modify(|w| w.set_ccie(n, val));
    #[cfg(any(lptim_v2a, lptim_v2b))]
    dier_modify(r, |w| w.set_ccie(n, val));
}

/// Modify DIER and wait for `DIEROK`. RM0456: "Any successive write before the DIEROK flag is
/// set, leads to unpredictable results." A write still in flight when the chip enters Stop can
/// also lose the compare wakeup. A write that changes nothing is skipped, so it costs no wait.
#[cfg(any(lptim_v2a, lptim_v2b))]
fn dier_modify(r: Lptim, f: impl FnOnce(&mut Dier)) {
    let old = r.dier().read();
    let mut new = old;
    f(&mut new);
    if new == old {
        return;
    }
    r.icr().write(|w| w.set_dierokcf(true));
    r.dier().write_value(new);

    // A lost CCIE enable delays the alarm to the next overflow, where `next_period` enables CCIE
    // again, and a lost disable is an interrupt that finds no alarm due.
    wait_for(|| r.isr().read().dierok());
}

/// Counter clock cycles to wait for `DIEROK` or `CMPOK`, which follow the counter clock, not the
/// kernel clock.
#[cfg(any(lptim_v2a, lptim_v2b))]
const ACK_TIMEOUT_TICKS: u32 = 64;

/// Poll `ok` until it returns true, giving up after `ACK_TIMEOUT_TICKS` so a lost acknowledgement
/// cannot hang the critical section it runs in.
#[cfg(any(lptim_v2a, lptim_v2b))]
fn wait_for(ok: impl Fn() -> bool) {
    let sys = unwrap!(unsafe { crate::rcc::get_freqs() }.sys.to_hertz()).0;
    let polls = (sys / TICK_HZ as u32 + 1) * ACK_TIMEOUT_TICKS;
    for _ in 0..polls {
        if ok() {
            return;
        }
    }
}

/// Write a raw value to the ICR (interrupt clear register).
fn icr_write_raw(r: Lptim, val: u32) {
    r.icr().write_value(Icr(val));
}

/// Write the compare value and wait for the write to be acknowledged.
fn write_compare(r: Lptim, val: u16) {
    #[cfg(not(any(lptim_v2a, lptim_v2b)))]
    {
        // On STM32WL, CMPOK remains set after a compare update. Clear the stale
        // acknowledgement so the loop waits for the current CMP write.
        #[cfg(any(stm32wlex, stm32wl5x))]
        r.icr().write(|w| w.set_cmpokcf(0, true));

        r.cmp().write(|w| w.set_cmp(val));
        while !r.isr().read().cmpok(0) {}
    }
    #[cfg(any(lptim_v2a, lptim_v2b))]
    {
        // As on STM32WL, clear the stale CMPOK.
        r.icr().write(|w| w.set_cmpokcf(0, true));
        r.ccr(0).write(|w| w.set_ccr(val));

        // Whatever CCR holds after a lost write, a match on it runs `trigger_alarm`, which writes
        // CCR again, and an alarm it misses is fired by `next_period` at the first overflow after
        // it is due.
        wait_for(|| r.isr().read().cmpok(0));
    }
}

pub(crate) struct RtcDriver {
    /// Number of 2^15 periods elapsed since boot.
    period: AtomicU32,
    alarm: Mutex<CriticalSectionRawMutex, AlarmState>,
    #[cfg(feature = "low-power")]
    is_stopped: AtomicBool,
    /// Whether the driver prevents Stop mode because its clock does not run in Stop.
    #[cfg(feature = "low-power")]
    stop_blocked: AtomicBool,
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
    stop_blocked: AtomicBool::new(false),
    #[cfg(feature = "low-power")]
    min_stop_pause: Mutex::const_new(CriticalSectionRawMutex::new(), Cell::new(embassy_time::Duration::from_millis(0))),
    queue: Mutex::new(RefCell::new(Queue::new())),
});

impl RtcDriver {
    /// initialize the timer, but don't start it.  Used for chips like stm32wle5
    /// for low power where the timer config is lost in STOP2.
    pub(crate) fn init_timer(&'static self, cs: critical_section::CriticalSection) {
        let r = regs_lptim();

        // we want this to increment the stop mode counter (some lp timer can't do STOP2)
        rcc::enable_and_reset_with_cs::<T>(cs);

        let timer_freq = T::frequency();

        r.cnt().write(|w| w.set_cnt(0));

        let freq = timer_freq.0 as u64;
        let div = if freq % TICK_HZ == 0 { freq / TICK_HZ } else { 0 };
        let psc = match div {
            128 => vals::Presc::Div128,
            64 => vals::Presc::Div64,
            32 => vals::Presc::Div32,
            16 => vals::Presc::Div16,
            8 => vals::Presc::Div8,
            4 => vals::Presc::Div4,
            2 => vals::Presc::Div2,
            1 => vals::Presc::Div1,
            _ => panic!(
                "time driver: the LPTIM clock ({}, {} Hz) can't be divided down to the embassy-time tick rate \
                ({} Hz): the ratio must be 1, 2, 4, 8, 16, 32, 64 or 128. Select another clock in \
                `config.rcc.mux` or another embassy-time `tick-hz-*` feature.",
                time_driver_clock::name(),
                freq,
                TICK_HZ
            ),
        };

        trace!(
            "init: setting presc: {} timer_freq: {}Hz TICK_HZ: {}",
            div, timer_freq, TICK_HZ
        );
        r.cfgr().modify(|w| w.set_presc(psc));

        // RM says timer must be enabled before setting arr or cmp
        r.cr().modify(|w| w.set_enable(true));
        trace!("init: arr: {}", r.arr().read().arr());
        // TRM says this is updated immediately if the timer is not started so no need to check for arrok! (stm32wl5 & stm32wle)
        r.arr().write(|w| w.set_arr(u16::MAX));

        // Enable overflow interrupts
        ier_set_overflow_ie(T::regs(), true);

        <T as crate::lptim::SealedBasicInstance>::GlobalInterrupt::unpend();
        unsafe {
            <T as crate::lptim::SealedBasicInstance>::GlobalInterrupt::enable();
        }
        #[cfg(feature = "low-power")]
        {
            // Configure EXTI wakeup lines for LPTIM on chips that require
            // explicit EXTI routing to wake from STOP mode.
            //
            // STM32WL: LPTIM1/2/3 use EXTI lines 29/30/31 respectively.
            // STM32WBA: LPTIM interrupts wake from STOP directly — no EXTI
            //           routing needed.
            #[cfg(any(stm32wlex, stm32wl5x))]
            {
                #[cfg(time_driver_lptim1)]
                const EXTI_WAKEUP_LINE: usize = 29;
                #[cfg(time_driver_lptim2)]
                const EXTI_WAKEUP_LINE: usize = 30;
                #[cfg(time_driver_lptim3)]
                const EXTI_WAKEUP_LINE: usize = 31;

                #[cfg(stm32wl5x)]
                {
                    #[cfg(feature = "_core-cm4")]
                    const CPU: usize = 0;
                    #[cfg(feature = "_core-cm0p")]
                    const CPU: usize = 1;

                    crate::pac::EXTI
                        .cpu(CPU)
                        .imr(0)
                        .modify(|w| w.set_line(EXTI_WAKEUP_LINE, true));
                    crate::pac::EXTI
                        .cpu(CPU)
                        .emr(0)
                        .modify(|w| w.set_line(EXTI_WAKEUP_LINE, true));
                }
                #[cfg(stm32wlex)]
                {
                    crate::pac::EXTI.imr(0).modify(|w| w.set_line(EXTI_WAKEUP_LINE, true));
                }
            }

            // F4/F7: the LPTIM1 event is the configurable EXTI line 23 (RM0401 9.2.5, RM0410
            // 11.8). Its pending bit is cleared in `on_interrupt`: Stop isn't entered while it's set.
            #[cfg(any(stm32f4, stm32f7))]
            {
                crate::pac::EXTI.rtsr(0).modify(|w| w.set_line(F4_F7_EXTI_LINE, true));
                crate::pac::EXTI.imr(0).modify(|w| w.set_line(F4_F7_EXTI_LINE, true));
            }

            // U5/U3: the LP timer used as time driver must remain functional in
            // STOP mode so it can wake the CPU. Two RCC bits are required:
            //  - SRDAMR.*amen: autonomous mode enable, so the peripheral can
            //    request its kernel clock (LSE/LSI) while in STOP.
            //  - APB3SMENR.*smen: keep the APB bus clock available in STOP so
            //    the NVIC interrupt reaches the CPU.
            // Without these, the LPTIM keeps its registers but its interrupt
            // cannot wake the core from STOP, hanging the executor.
            #[cfg(stm32u5)]
            {
                #[cfg(time_driver_lptim1)]
                {
                    crate::pac::RCC.srdamr().modify(|w| w.set_lptim1amen(true));
                    crate::pac::RCC.apb3smenr().modify(|w| w.set_lptim1smen(true));
                }
                #[cfg(time_driver_lptim3)]
                {
                    crate::pac::RCC.srdamr().modify(|w| w.set_lptim3amen(true));
                    crate::pac::RCC.apb3smenr().modify(|w| w.set_lptim3smen(true));
                }
            }
            // U3 has no autonomous mode register; APB3STPENR keeps the clock in STOP.
            #[cfg(stm32u3)]
            {
                #[cfg(time_driver_lptim1)]
                crate::pac::RCC.apb3stpenr().modify(|w| w.set_lptim1stpen(true));
                #[cfg(time_driver_lptim3)]
                crate::pac::RCC.apb3stpenr().modify(|w| w.set_lptim3stpen(true));
            }
        }
    }

    fn init(&'static self, cs: CriticalSection) {
        self.check_clock(cs);
        self.init_timer(cs);
        regs_lptim().cr().modify(|w| w.set_cntstrt(true));
    }

    /// Check the LPTIM kernel clock selected in `config.rcc.mux`. Panics if the driver can't run
    /// from it. With `low-power`, prevents Stop mode if the clock stops in Stop.
    fn check_clock(&self, _cs: CriticalSection) {
        // A clock that isn't running makes `T::frequency()` panic in `init_timer`.

        #[cfg(any(rcc_l5, rcc_u5, rcc_u3, rcc_wle, rcc_wl5, rcc_wba, rcc_u0))]
        if time_driver_clock::is_lse() && !crate::pac::RCC.bdcr().read().lsesysen() {
            panic!(
                "time driver: the LPTIM is clocked from LSE, but LSE doesn't reach peripherals. \
                Set `peripherals_clocked: true` in `config.rcc.ls.lse`."
            );
        }

        #[cfg(feature = "low-power")]
        {
            let runs_in_stop = time_driver_clock::is_lsi() || time_driver_clock::is_lse();
            let blocked = self.stop_blocked.load(Ordering::Relaxed);
            if !runs_in_stop && !blocked {
                warn!(
                    "time driver: the LPTIM clock ({}) stops in Stop mode, so Stop mode is disabled. \
                    Select LSI or LSE for the LPTIM in `config.rcc.mux` to allow Stop.",
                    time_driver_clock::name()
                );
                rcc::increment_stop_refcount(_cs, rcc::StopMode::Stop1);
                self.stop_blocked.store(true, Ordering::Relaxed);
            } else if runs_in_stop && blocked {
                rcc::decrement_stop_refcount(_cs, rcc::StopMode::Stop1);
                self.stop_blocked.store(false, Ordering::Relaxed);
            }
        }
    }

    pub(crate) fn on_interrupt(&self) {
        let r = regs_lptim();

        #[cfg(all(feature = "low-power", any(stm32f4, stm32f7)))]
        crate::pac::EXTI.pr(0).write(|w| w.set_line(F4_F7_EXTI_LINE, true));

        critical_section::with(|cs| {
            let sr = r.isr().read();

            // Clear the flags read, and only those. Bits in ICR are "write 1 to clear". A flag set
            // since the read stays set and enters this handler again: clearing it here would lose
            // it, and a lost UE loses a counter period.
            icr_write_raw(r, sr.0);

            // Overflow
            if overflowed!(sr) {
                self.next_period();
            }

            if sr.ccif(0) && ier_ccie(r, 0) {
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
            let alarm = self.alarm.borrow(cs);
            let at = alarm.timestamp.get();

            // Arm now if it falls in this counter cycle
            if at < t + 0x1_0000 {
                // just enable it. `set_alarm` has already set the correct CCR val.
                ier_set_ccie(r, 0, true);

                // The overflow interrupt may run after an alarm a few ticks into the new cycle
                // has already passed; the compare match was then missed. Fire it now.
                if at <= self.now() {
                    self.trigger_alarm(cs);
                }
            }
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
            ier_set_ccie(r, 0, false);

            self.alarm.borrow(cs).timestamp.set(u64::MAX);

            return false;
        }

        // Write the CCR value regardless of whether we're going to enable it now or not.
        // This way, when we enable it later, the right value is already set.
        write_compare(r, timestamp as u16);

        // Enable it if it'll happen this counter cycle. Otherwise, `next_period` will enable it.
        let next_overflow = ((t >> 16) + 1) << 16;
        ier_set_ccie(r, 0, timestamp < next_overflow);

        // Reevaluate if the alarm timestamp is still in the future
        let t = self.now();
        if timestamp <= t {
            trace!("set_alarm: timestamp <= t (after set)");
            // If alarm timestamp has passed since we set it, we have a race condition and
            // the alarm may or may not have fired.
            // Disarm the alarm and return `false` to indicate that.
            // It is the caller's responsibility to handle this ambiguity.
            ier_set_ccie(r, 0, false);

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

    fn set_min_stop_pause(&self, cs: CriticalSection, min_stop_pause: embassy_time::Duration) {
        trace!("set_min_stop_pause: {} ticks", min_stop_pause.as_ticks());
        self.min_stop_pause.borrow(cs).replace(min_stop_pause);
    }
}

impl Driver for RtcDriver {
    fn now(&self) -> u64 {
        let r = regs_lptim();

        // This CS is needed to prevent race conditions that cause time to go backwards when now()
        // is called at the exact instant the LPTIM_CNT register overflows. When an overflow
        // happens, `period` is advanced by the update-event interrupt, so period and LPTIM_CNT
        // must be sampled atomically, i.e. with that interrupt masked. Otherwise, the counter can
        // wrap while `period` still has the previous value, and composing them yields a timestamp
        // a whole period in the past, causing time to go backwards, and any
        // `Instant::duration_since` spanning that window panics.
        critical_section::with(|_| {
            loop {
                // With the period-incrementing, a pending update event unambiguously means
                // the counter has wrapped and `next_period` has not yet accounted for it. NB: we DO
                // NOT clear the ue bit, that (and consequently incrementing self.period) is for the
                // ISR to do once we release this CS.
                //
                // The CS masks the interrupt but not the flag: the counter can still wrap between
                // reading LPTIM_CNT and LPTIM_ISR, pairing 0xFFFF with a pending update event, a
                // whole period in the future. Sample the flag on both sides and retry if it changed.
                let pending = overflowed!(r.isr().read());

                // RM0456 says: "When the LPTIM is running, reading the LPTIM_CNT register may return
                // unreliable values. In this case it is necessary to perform consecutive reads until
                // two returned values are identical."
                let mut counter = r.cnt().read().cnt();
                loop {
                    let again = r.cnt().read().cnt();
                    if again == counter {
                        break;
                    }
                    counter = again;
                }

                if overflowed!(r.isr().read()) == pending {
                    let period = self.period.load(Ordering::Relaxed) + pending as u32;
                    break ((period as u64) << 16) + counter as u64;
                }
            }
        })
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
