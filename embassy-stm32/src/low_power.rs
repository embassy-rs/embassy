//! Low-power support.
//!
//! The STM32 line of microcontrollers support various deep-sleep modes which exploit clock-gating
//! to reduce power consumption. The `embassy-stm32` HAL provides a `sleep()` function which
//! can use knowledge of which peripherals are currently blocked upon to transparently and safely
//! enter such low-power modes including `STOP1` and `STOP2` when possible.
//! To archive lower power consumption consider enabling the `gpio-init-analog` feature, in
//! order to start with all GPIOs set to analog input as recommended by STs low-power guidance
//! AN4899 chapter 7.1
//!
//! `sleep()` coordinates Stop-mode entry using three mechanisms:
//!
//!  * **Capabilities & per-transaction [`WakeGuard`](crate::rcc::WakeGuard)s:** constructed-but-idle
//!    peripherals do not block Stop modes. Active operations hold `WakeGuard`s over the Stop modes
//!    they cannot survive (`Stop0..Stop3`), while peripherals running from a Stop-surviving clock
//!    (such as `RTC`, `LPTIM`, or Stop-capable `USART`/`LPUART`) remain eligible for Stop.
//!  * **Armed [`WakeSource`]s:** async waiters (`ExtiInput`, time-driver alarms, and Stop-capable
//!    peripherals) register a `WakeSource`. If neither a `WakeSource` nor a timer alarm is armed,
//!    `sleep()` falls back to plain `WFI`.
//!  * **Per-mode residency & exit latency:** `sleep()` evaluates each mode's `min_residency_us` and
//!    `exit_latency_us` plus the runtime RCC clock-restoration latency (e.g. `HSE` crystal startup
//!    and `PLL` relock) and time-driver overhead, selecting the deepest Stop mode that fits before
//!    the next deadline (with [`Config::min_stop_pause`](crate::Config::min_stop_pause), defaulting
//!    to `0 ms`, as an optional minimum floor) and waking timed alarms early by the exit latency.
//!
//! `embassy-stm32` also provides an `embassy-executor` platform implementation that integrates `sleep()` into the main loop. It is available
//! in the `embassy_stm32::executor` module, and is enabled by the `executor-thread` or `executor-interrupt` features. This stm32-specific
//! executor is the preferred way to lower power consumption if you're using `async`, instead of calling `sleep()` directly.

// Fail loudly on families that have no low-power implementation.
//
// This is a stopgap, not an endorsement of the split: the families listed below are simply the
// ones for which `enter_stop` below actually programs PWR. On any other family,
// enabling `low-power` would still set `SLEEPDEEP` and execute `WFI`, but with PWR left at its
// reset value. The consequences are silent and severe:
//
//   * the mode actually entered is whatever `PWR` defaults to, not the mode `rcc::get_stop_mode()`
//     reports to callers and tests;
//   * the wake-up flags are not cleared before entry, so entry can be silently ignored;
//   * `exit_stop` has no branch for these families, so the system clock is not restored
//     after wake-up.
//
// Negative list `not(any(...))` rather than an explicit deny-list so that a newly added family
// fails here until it is implemented deliberately. See `docs/low-power/00-hardware-survey.md`
// §6.1 for the full analysis.
#[cfg(not(any(
    stm32c5, stm32f0, stm32f3, stm32f4, stm32f7, stm32g0, stm32g4, stm32g5, stm32l0, stm32l1, stm32l4, stm32l5,
    stm32u0, stm32u3, stm32u5, stm32h5, stm32wb, stm32wba, stm32wl
)))]
compile_error!(
    "the `low-power` feature is not implemented for this STM32 family. \
     Implemented families: c5, f0, f3, f4, f7, g0, g4, g5, l0, l1, l4, l5, u0, u3, u5, h5, wb, wba, wl. \
     See embassy-stm32/src/low_power.rs and docs/low-power/00-hardware-survey.md."
);

// The time spent in Stop is measured with the RTC subsecond counter, which Cat.1 devices lack (RM0038 20.6.11).
#[cfg(rtc_v2_l1_cat1)]
compile_error!(
    "the `low-power` feature is not supported on STM32L1 Cat.1 devices: their RTC has no subsecond counter."
);

// Low-pin-count / entry-line STM32F0 devices lack the RTC wake-up timer (`WUTR`) required by `rtc::low_power`.
#[cfg(any(rtc_v2_f0_nowut, rtc_v2_f0_x0_nowut))]
compile_error!("the `low-power` feature is not supported on STM32F0 devices without an RTC wake-up timer.");

use core::mem;
use core::sync::atomic::{Ordering, compiler_fence};

use cortex_m::peripheral::SCB;
use critical_section::CriticalSection;

#[cfg(all(feature = "rt", not(feature = "_lp-time-driver")))]
use crate::interrupt;
use crate::_generated::power::{DIALECT, Dialect, PowerModeEntry};
pub use crate::rcc::StopMode;
use crate::rcc::{StopModes, select_power_mode};
use crate::time_driver::{LPTimeDriver, get_driver};

/// Classification of an armed wake-up source that can bring the core out of Stop mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum WakeClass {
    /// Wake-up alarm from the low-power time driver / RTC.
    Time,
    /// Wake-up edge from EXTI GPIO input wait.
    Exti,
    /// Wake-up event from a Stop-capable peripheral.
    Peripheral,
}

static WAKE_SOURCES: embassy_sync::blocking_mutex::Mutex<
    embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
    [core::cell::Cell<u16>; 3],
> = embassy_sync::blocking_mutex::Mutex::const_new(
    embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex::new(),
    [
        core::cell::Cell::new(0),
        core::cell::Cell::new(0),
        core::cell::Cell::new(0),
    ],
);

/// RAII token registering an armed wake-up source for Stop mode.
///
/// While at least one [`WakeSource`] is active (or the time driver has a pending
/// timer alarm), [`sleep`] may enter Stop mode; otherwise it degrades to plain `WFI`.
pub struct WakeSource {
    class: WakeClass,
}

impl WakeSource {
    /// Register an armed wake-up source of `class` until dropped.
    pub fn new(class: WakeClass) -> Self {
        WAKE_SOURCES.lock(|sources| {
            let cell = &sources[class as usize];
            cell.set(cell.get() + 1);
        });
        Self { class }
    }
}

impl Drop for WakeSource {
    fn drop(&mut self) {
        WAKE_SOURCES.lock(|sources| {
            let cell = &sources[self.class as usize];
            cell.set(cell.get() - 1);
        });
    }
}

fn has_armed_wake_source(cs: CriticalSection) -> bool {
    let registered = WAKE_SOURCES.borrow(cs).iter().any(|c| c.get() > 0);
    registered || get_driver().has_pending_alarm(cs)
}

#[cfg(all(feature = "rt", not(feature = "_lp-time-driver")))]
foreach_interrupt! {
    (RTC, rtc, $block:ident, WKUP, $irq:ident) => {
        #[interrupt]
        #[allow(non_snake_case)]
        unsafe fn $irq() {
        }
    };
}

/// Configure PWR registers to enter the selected Stop `entry`.
fn enter_stop(entry: &PowerModeEntry) -> Result<(), ()> {
    #[cfg(stm32wb)]
    let mutex = {
        use crate::hsem::get_hsem;
        use crate::pac::{PWR, RCC};

        trace!("low power: trying to get sem3");

        let sem3_mutex = get_hsem(3).try_fast_lock_with_interrupt().ok_or(())?;

        trace!("low power: got sem3");

        let mut sem4_mutex = get_hsem(4).try_fast_lock();
        if sem4_mutex.is_some() {
            trace!("low power: got sem4");
        }

        if !(sem4_mutex.is_some() && !PWR.extscr().read().c2ds()) {
            sem4_mutex.take();
            crate::rcc::stop_switch_to_hsi();
        }

        // on PWR
        RCC.apb1enr1().modify(|r| r.0 |= 1 << 28);
        cortex_m::asm::dsb();

        // off SMPS, on Bypass
        PWR.cr5().modify(|r| {
            let mut val = r.0;
            val &= !(1 << 15); // sdeb = 0 (off SMPS)
            val |= 1 << 14; // sdben = 1 (on Bypass)
            r.0 = val
        });

        cortex_m::asm::delay(1000);

        sem3_mutex
    };

    match DIALECT {
        Dialect::Classic => {
            #[cfg(stm32l0)]
            {
                use crate::pac::pwr::vals;
                crate::pac::PWR.cr().modify(|w| {
                    w.set_pdds(vals::Pdds::StopMode);
                    w.set_lpsdsr(if entry.selector != 0 {
                        vals::Mode::LowPowerMode
                    } else {
                        vals::Mode::MainMode
                    });
                });
            }

            #[cfg(stm32l1)]
            crate::pac::PWR.cr().modify(|w| {
                w.set_pdds(crate::pac::pwr::vals::Pdds::StopMode);
                w.set_lpsdsr(entry.selector != 0);
            });

            #[cfg(any(stm32f4, stm32f7))]
            crate::pac::PWR.cr1().modify(|w| {
                w.set_pdds(crate::pac::pwr::vals::Pdds::StopMode);
                w.set_lpds(entry.selector != 0);
            });

            #[cfg(any(stm32f0, stm32f3))]
            crate::pac::PWR.cr().modify(|w| {
                w.set_pdds(crate::pac::pwr::vals::Pdds::StopMode);
                w.set_lpds(entry.selector != 0);
            });
        }
        Dialect::LpmsScr => {
            #[cfg(any(stm32g0, stm32g4))]
            crate::pac::PWR.cr1().modify(|w| w.set_lpms(entry.selector));

            #[cfg(any(stm32l4, stm32l5, stm32u0))]
            crate::pac::PWR
                .cr1()
                .modify(|w| w.set_lpms(crate::pac::pwr::vals::Lpms::from_bits(entry.selector)));
        }
        Dialect::LpmsSrWuscr => {
            #[cfg(any(stm32u3, stm32u5, stm32wba))]
            crate::pac::PWR
                .cr1()
                .modify(|w| w.set_lpms(crate::pac::pwr::vals::Lpms::from_bits(entry.selector)));
        }
        Dialect::Extscr => {
            #[cfg(any(stm32wb, stm32wl))]
            {
                #[cfg(not(feature = "_core-cm0p"))]
                crate::pac::PWR
                    .cr1()
                    .modify(|m| m.set_lpms(crate::pac::pwr::vals::Lpms::from_bits(entry.selector)));
                #[cfg(feature = "_core-cm0p")]
                crate::pac::PWR
                    .c2cr1()
                    .modify(|m| m.set_lpms(crate::pac::pwr::vals::Lpms::from_bits(entry.selector)));
            }
        }
        Dialect::PmcrSvos => {
            #[cfg(stm32h5)]
            crate::pac::PWR.pmcr().modify(|v| {
                use crate::pac::pwr::vals;
                v.set_lpms(vals::Lpms::from_bits(entry.selector));
                if let Some(svos) = entry.min_svos {
                    v.set_svos(vals::Svos::from_bits(svos));
                }
            });

            #[cfg(stm32c5)]
            crate::pac::PWR
                .pmcr()
                .modify(|v| v.set_lpms(crate::pac::pwr::vals::Lpms::from_bits(entry.selector)));

            #[cfg(stm32g5)]
            crate::pac::PWR
                .lpmcr()
                .modify(|v| v.set_lpms(crate::pac::pwr::vals::Lpms::from_bits(entry.selector)));
        }
        Dialect::DomainPdds | Dialect::Wb0Deepstop => {}
    }

    #[cfg(stm32wb)]
    drop(mutex);

    Ok(())
}

/// Read back hardware `STOPF` / `C1STOPF` / `C2STOPF` flags if supported by this chip's
/// PWR dialect. Returns `Some(true)` if Stop was actually entered, `Some(false)` if WFI
/// returned without entering Stop, or `None` if hardware provides no readable Stop flag.
fn did_enter_stop() -> Option<bool> {
    match DIALECT {
        Dialect::LpmsSrWuscr => {
            #[cfg(any(stm32u3, stm32u5, stm32wba))]
            return Some(crate::pac::PWR.sr().read().stopf());
            #[allow(unreachable_code)]
            None
        }
        Dialect::PmcrSvos => {
            #[cfg(any(stm32h5, stm32c5, stm32g5))]
            return Some(crate::pac::PWR.pmsr().read().stopf());
            #[allow(unreachable_code)]
            None
        }
        Dialect::Extscr => {
            #[cfg(stm32wl)]
            {
                let es = crate::pac::PWR.extscr().read();
                #[cfg(not(feature = "_core-cm0p"))]
                return Some(es.c1stopf() || es.c1stop2f());
                #[cfg(feature = "_core-cm0p")]
                return Some(es.c2stopf() || es.c2stop2f());
            }
            #[cfg(stm32wb)]
            return Some(crate::pac::PWR.extscr().read().c1stopf());
            #[allow(unreachable_code)]
            None
        }
        Dialect::LpmsScr => {
            #[cfg(stm32u0)]
            return Some(crate::pac::PWR.sr1().read().stopf().to_bits() != 0);
            #[allow(unreachable_code)]
            None
        }
        Dialect::Classic | Dialect::DomainPdds | Dialect::Wb0Deepstop => None,
    }
}

/// Clear any previous stop/wakeup flags
fn clear_flags() {
    match DIALECT {
        Dialect::Extscr => {
            #[cfg(stm32wl)]
            crate::pac::PWR.extscr().modify(|w| {
                #[cfg(not(feature = "_core-cm0p"))]
                w.set_c1cssf(true);
                #[cfg(feature = "_core-cm0p")]
                w.set_c2cssf(true);
            });
            #[cfg(stm32wb)]
            // CPU1 clears its own Stop/Standby flags; CPU2's flags belong to CPU2.
            crate::pac::PWR.extscr().modify(|w| w.set_c1cssf(true));
        }
        Dialect::LpmsSrWuscr => {
            #[cfg(any(stm32u3, stm32u5, stm32wba))]
            crate::pac::PWR.sr().modify(|w| w.set_cssf(true));
            #[cfg(stm32wba)]
            crate::pac::PWR.wuscr().write(|w| w.0 = 0xff);
        }
        Dialect::PmcrSvos => {
            #[cfg(stm32h5)]
            crate::pac::PWR.pmcr().modify(|w| w.set_cssf(true));
            #[cfg(stm32c5)]
            {
                crate::pac::PWR
                    .pmcr()
                    .modify(|w| w.set_cssf(crate::pac::pwr::vals::Cssf::Clear));
                crate::pac::PWR.wuscr().write(|w| w.0 = 0x7f);
            }
            #[cfg(stm32g5)]
            {
                crate::pac::PWR.lpmcr().modify(|w| w.set_cssf(true));
                crate::pac::PWR.wuscr().write(|w| w.0 = 0xfb);
            }
        }
        Dialect::Classic => {
            #[cfg(stm32l0)]
            crate::pac::PWR.cr().modify(|w| {
                w.set_cwuf(true);
                w.set_lpsdsr(crate::pac::pwr::vals::Mode::MainMode);
            });
            #[cfg(stm32l1)]
            crate::pac::PWR.cr().modify(|w| {
                w.set_cwuf(true);
                w.set_lpsdsr(false);
            });
            #[cfg(any(stm32f0, stm32f3))]
            crate::pac::PWR.cr().modify(|w| w.set_cwuf(true));
        }
        Dialect::LpmsScr => {
            #[cfg(stm32g4)]
            crate::pac::PWR.scr().write(|w| {
                w.set_cwuf1(true);
                w.set_cwuf2(true);
                w.set_cwuf3(true);
                w.set_cwuf4(true);
                w.set_cwuf5(true);
                w.set_csbf(true);
            });
        }
        Dialect::DomainPdds | Dialect::Wb0Deepstop => {}
    }
}

/// Exit stop mode: check hardware entry flag, restore clocks and timers if Stop was entered,
/// clear status flags, and resume time.
fn exit_stop(cs: CriticalSection) {
    let entered = did_enter_stop();

    if entered == Some(false) {
        trace!("low power: WFI returned without entering stop");
    }

    // `stm32wb` switches CPU1 sysclk to HSI in software inside `enter_stop()`, so it always
    // runs `stop_exit(cs)` even if `C1STOPF` is clear. Other chips with hardware `STOPF`
    // only need clock restoration when Stop was actually entered.
    if entered != Some(false) || cfg!(stm32wb) {
        // Both cores share the clocks, so they restore them one at a time.
        #[cfg(any(stm32wb, stm32wl5x))]
        let lock = crate::hsem::get_hsem(3).blocking_lock(0);

        crate::rcc::stop_exit(cs);

        #[cfg(any(stm32wb, stm32wl5x))]
        drop(lock);
    }

    if entered != Some(false) {
        #[cfg(any(stm32u5, stm32u3))]
        {
            let sr = crate::pac::PWR.sr().read();
            if sr.stopf() {
                let lpms = crate::pac::PWR.cr1().read().lpms();
                debug!("low power: U5/U3 woke from STOP (LPMS={})", lpms.to_bits());

                // GPTIM timer state is lost in Stop2/Stop3, needs re-init.
                // LPTIM (_lp-time-driver) survives all STOP modes autonomously.
                #[cfg(not(feature = "_lp-time-driver"))]
                if lpms.to_bits() >= 2 {
                    trace!("low power: re-initializing timer (STOP2+, GPTIM)");
                    get_driver().init_timer(cs);
                }
            }
        }

        #[cfg(any(stm32wl, stm32wb, stm32wba))]
        {
            #[cfg(any(stm32wl, stm32wb))]
            let es = crate::pac::PWR.extscr().read();

            #[cfg(stm32wba)]
            let es = crate::pac::PWR.sr().read();

            let re_initialize_timer = {
                #[cfg(all(stm32wl, not(feature = "_core-cm0p")))]
                {
                    es.c1stop2f()
                }
                #[cfg(all(stm32wl, feature = "_core-cm0p"))]
                {
                    es.c2stop2f()
                }
                #[cfg(stm32wb)]
                {
                    es.c1stopf()
                }
                #[cfg(stm32wba)]
                {
                    es.stop2f()
                }
            };

            #[cfg(stm32wba)]
            match (es.stopf(), es.stop2f()) {
                (true, true) => debug!("low power: WBA woke from STOP2"),
                (true, false) => debug!("low power: WBA woke from STOP0/1"),
                _ => {}
            };

            #[cfg(stm32wl)]
            match (es.c1stopf(), es.c1stop2f()) {
                (true, false) => debug!("low power: cpu1 has been in STOP1"),
                (false, true) => debug!("low power: cpu1 has been in STOP2"),
                (true, true) => debug!("low power: cpu1 has been in STOP1 and STOP2 ???"),
                (false, false) => {}
            };

            #[cfg(stm32wl5x)]
            match (es.c2stopf(), es.c2stop2f()) {
                (true, false) => debug!("low power: cpu2 has been in STOP1"),
                (false, true) => debug!("low power: cpu2 has been in STOP2"),
                (true, true) => debug!("low power: cpu2 has been in STOP1 and STOP2 ???"),
                (false, false) => {}
            };

            #[cfg(stm32wb)]
            match (es.c1stopf(), es.c2stopf()) {
                (true, false) => debug!("low power: cpu1 has been in STOP"),
                (false, true) => debug!("low power: cpu2 has been in STOP"),
                (true, true) => debug!("low power: cpu1 and cpu2 have been in STOP"),
                (false, false) => {}
            };

            if re_initialize_timer {
                trace!("low power: re-initializing timer");
                #[cfg(not(feature = "_lp-time-driver"))]
                get_driver().init_timer(cs);
            }
        }
    }

    clear_flags();
    get_driver().resume_time(cs);
    trace!("low power: resumed");
}

fn configure_pwr(cs: CriticalSection) -> bool {
    let mut scb: SCB = unsafe { mem::transmute(()) };

    scb.clear_sleepdeep();
    clear_flags();

    compiler_fence(Ordering::Acquire);

    if !has_armed_wake_source(cs) {
        trace!("low_power: no armed wake source, using WFI");
        return false;
    }

    let driver = get_driver();
    let has_alarm = driver.has_pending_alarm(cs);
    let time_until_next_alarm = driver.time_until_next_alarm(cs);
    if time_until_next_alarm < driver.min_stop_pause(cs) {
        trace!("low_power: time_until_next_alarm < min_stop_pause, using WFI");
        return false;
    }

    let available_us = has_alarm.then(|| time_until_next_alarm.as_micros());
    let Some((entry, exit_latency_us)) = select_power_mode(cs, available_us, driver.driver_overhead_us()) else {
        return false;
    };

    let exit_latency = embassy_time::Duration::from_micros(exit_latency_us as u64);
    if driver.pause_time(cs, exit_latency).is_err() {
        trace!("low_power: failed to pause time, not entering stop");

        false
    } else if enter_stop(entry).is_err() {
        trace!("low_power: failed to enter stop");
        driver.resume_time(cs);

        false
    } else {
        crate::rcc::stop_enter(cs);

        #[cfg(any(stm32l0, stm32f4))]
        trace!("low power: enter stop");
        #[cfg(not(any(stm32l0, stm32f4)))]
        trace!("low power: enter stop: {:?}", entry.kind);

        scb.set_sleepdeep();

        true
    }
}

/// Mask of Stop modes currently permitted (i.e. unblocked) by active [`WakeGuard`](crate::rcc::WakeGuard)s.
#[allow(dead_code)]
pub(crate) fn allowed_stop_modes(cs: CriticalSection) -> StopModes {
    let mut mask = 0u8;
    for entry in crate::_generated::power::MODES {
        if crate::rcc::get_power_mode(cs).is_some_and(|max| entry.kind.at_least(max.kind) || max.kind.at_least(entry.kind))
        {
            mask |= entry.bit;
        }
    }
    StopModes(mask)
}

/// `WFI` and `ISB`, executed from RAM.
///
/// U0 and G0: with DBG_STOP set (`enable_debug_during_sleep`), the first instruction fetches from
/// flash after a wake-up from Stop with interrupts masked can fault. ST reproduced it on a
/// NUCLEO-U083; there's no erratum yet. It happens on a NUCLEO-G071RB too, also with no debugger
/// attached. Fetching the instructions right after the wake-up from RAM avoids it.
#[cfg(any(stm32u0, stm32g0))]
#[unsafe(link_section = ".data.embassy_stm32_wfi_from_ram")]
#[inline(never)]
fn wfi_from_ram() {
    unsafe { core::arch::asm!("wfi", "isb", options(nostack, preserves_flags)) };
}

/// Sleep with WFI, attempting to enter the deepest STOP mode possible.
///
/// If it's not possible to enter any STOP mode due to running peripherals it will
/// still do a `WFI` sleep. Therefore this function is equivalent to `WFI` except
/// with lower power consumption and higher latency.
///
/// ## SAFETY
///
/// Care must be taken that we have ensured that the system is ready to go to deep
/// sleep, otherwise HAL peripherals may misbehave. HAL drivers automatically prevent
/// sleep as needed, but you might have to do it manually if you're using some peripherals
/// with the PAC directly.
pub unsafe fn sleep(cs: CriticalSection) {
    let stop = configure_pwr(cs);

    // Only flush defmt if we are actually going into Stop. Avoids flush on every tiny WFE
    #[cfg(feature = "low-power-defmt-flush")]
    if stop {
        defmt::flush();
    }

    cortex_m::asm::dsb();
    #[cfg(not(any(stm32u0, stm32g0)))]
    cortex_m::asm::wfi();
    #[cfg(any(stm32u0, stm32g0))]
    wfi_from_ram();

    cortex_m::asm::isb();
    cortex_m::asm::dsb();
    cortex_m::asm::dmb();

    if stop {
        exit_stop(cs);
    }
}
