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
//! `sleep()` determines which peripherals are active by their RCC state; consequently,
//! low-power states can only be entered if peripherals which block stop have been `drop`'d and if
//! peripherals that do not block stop are busy. Peripherals which never block stop include:
//!
//!  * `GPIO`
//!  * `RTC`
//!
//! Other peripherals which block stop when busy include (this list may be stale):
//!
//!  * `I2C`
//!  * `USART`
//!
//! Since entering and leaving low-power modes typically incurs a significant latency, `sleep()`
//! will only attempt to enter when the next timer event is at least [`config.min_stop_pause`] in the future.
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
    stm32f3, stm32f4, stm32f7, stm32g0, stm32g4, stm32l0, stm32l1, stm32l4, stm32l5, stm32u0, stm32u3, stm32u5,
    stm32h5, stm32wb, stm32wba, stm32wl
)))]
compile_error!(
    "the `low-power` feature is not implemented for this STM32 family. \
     Implemented families: f3, f4, f7, g0, g4, l0, l1, l4, l5, u0, u3, u5, h5, wb, wba, wl. \
     See embassy-stm32/src/low_power.rs and docs/low-power/00-hardware-survey.md."
);

// The time spent in Stop is measured with the RTC subsecond counter, which Cat.1 devices lack (RM0038 20.6.11).
#[cfg(rtc_v2_l1_cat1)]
compile_error!(
    "the `low-power` feature is not supported on STM32L1 Cat.1 devices: their RTC has no subsecond counter."
);

use core::mem;
use core::sync::atomic::{Ordering, compiler_fence};

use cortex_m::peripheral::SCB;
use critical_section::CriticalSection;

#[cfg(all(feature = "rt", not(feature = "_lp-time-driver")))]
use crate::interrupt;
pub use crate::rcc::StopMode;
use crate::rcc::get_stop_mode;
use crate::time_driver::{LPTimeDriver, get_driver};

#[cfg(all(feature = "rt", not(feature = "_lp-time-driver")))]
foreach_interrupt! {
    (RTC, rtc, $block:ident, WKUP, $irq:ident) => {
        #[interrupt]
        #[allow(non_snake_case)]
        unsafe fn $irq() {
        }
    };
}

#[cfg(any(stm32l4, stm32l5, stm32u5, stm32u3, stm32wba, stm32wb, stm32wl, stm32u0))]
use crate::pac::pwr::vals::Lpms;

#[cfg(any(stm32l4, stm32l5, stm32u5, stm32u3, stm32wba, stm32wb, stm32wl, stm32u0))]
impl Into<Lpms> for StopMode {
    fn into(self) -> Lpms {
        match self {
            #[cfg(power_stop1)]
            StopMode::Stop1 => Lpms::Stop1,
            #[cfg(not(power_stop1))]
            StopMode::Stop1 => Lpms::Stop0,
            #[cfg(power_stop2)]
            StopMode::Standby | StopMode::Stop2 => Lpms::Stop2,
            #[cfg(all(not(power_stop2), power_stop1))]
            StopMode::Standby | StopMode::Stop2 => Lpms::Stop1,
            #[cfg(not(any(power_stop2, power_stop1)))]
            StopMode::Standby | StopMode::Stop2 => Lpms::Stop0,
        }
    }
}

/// Enter stop mode
fn enter_stop(stop_mode: StopMode) -> Result<(), ()> {
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

    #[cfg(any(stm32l4, stm32l5, stm32u5, stm32u3, stm32u0, stm32wb, stm32wba, stm32wl))]
    {
        #[cfg(not(feature = "_core-cm0p"))]
        crate::pac::PWR.cr1().modify(|m| m.set_lpms(stop_mode.into()));
        #[cfg(feature = "_core-cm0p")]
        crate::pac::PWR.c2cr1().modify(|m| m.set_lpms(stop_mode.into()));
    }
    #[cfg(stm32h5)]
    crate::pac::PWR.pmcr().modify(|v| {
        use crate::pac::pwr::vals;
        v.set_lpms(vals::Lpms::Stop);
        v.set_svos(vals::Svos::Scale3);
    });

    #[cfg(stm32l0)]
    {
        use crate::pac::pwr::vals;
        crate::pac::PWR.cr().modify(|w| {
            w.set_pdds(vals::Pdds::StopMode);
            w.set_lpsdsr(vals::Mode::LowPowerMode);
        });
    }

    #[cfg(stm32l1)]
    crate::pac::PWR.cr().modify(|w| {
        w.set_pdds(crate::pac::pwr::vals::Pdds::StopMode);
        w.set_lpsdsr(true);
    });

    #[cfg(any(stm32f4, stm32f7))]
    crate::pac::PWR.cr1().modify(|w| {
        w.set_pdds(crate::pac::pwr::vals::Pdds::StopMode);
        w.set_lpds(true);
    });

    #[cfg(stm32f3)]
    crate::pac::PWR.cr().modify(|w| {
        w.set_pdds(crate::pac::pwr::vals::Pdds::StopMode);
        w.set_lpds(true);
    });

    // Stop 0 and Stop 1 have the same peripherals and wake-up sources (RM0444 4.3), Stop 1
    // only turns the main regulator off.
    #[cfg(stm32g0)]
    crate::pac::PWR.cr1().modify(|w| w.set_lpms(0b001));

    #[cfg(stm32g4)]
    {
        // G4's stop modes are numbered the opposite way round from the L4/U5 families, and
        // there is no Stop 2 at all: RM0440 §6.3.6/§6.3.7 define "Stop 0" as the main
        // regulator staying on (fastest wake, higher current) and "Stop 1" as the main
        // regulator being off with only the low-power regulator running (lowest current,
        // slower wake). Table 41 in the RM uses a single merged "Stop 0/1" column, i.e. the
        // two modes have *identical* peripheral availability and wake-up sources - the only
        // difference is regulator current versus latency.
        //
        // Therefore every stop request maps to LPMS = 0b001 (G4 "Stop 1"), the lowest-current
        // stop available: there is nothing to gain from the shallower G4 "Stop 0" here.
        // LPMS = 0b011 is Standby and 0b1xx is Shutdown; neither is ever entered from here
        // because both lose program state (see `rcc::get_stop_mode`).
        crate::pac::PWR.cr1().modify(|w| w.set_lpms(0b001));
        let _ = stop_mode;
    }

    #[cfg(stm32wb)]
    drop(mutex);

    let _ = stop_mode;

    Ok(())
}

/// Clear any previous stop flags
fn clear_flags() {
    #[cfg(stm32wl)]
    crate::pac::PWR.extscr().modify(|w| {
        #[cfg(not(feature = "_core-cm0p"))]
        w.set_c1cssf(true);
        #[cfg(feature = "_core-cm0p")]
        w.set_c2cssf(true);
    });
    #[cfg(stm32wb)]
    // CPU1 clears its own Stop/Standby flags (each core owns its own bits, as on
    // stm32wl). C1STOPF is only ever set when the system actually entered Stop mode,
    // so clearing it before each entry is what lets `exit_stop` distinguish a real
    // system Stop (clocks gone) from a mere CPU1 deepsleep in which the clocks were
    // left untouched. CPU2's flags are left alone - they belong to CPU2.
    crate::pac::PWR.extscr().modify(|w| w.set_c1cssf(true));
    #[cfg(stm32wba)]
    crate::pac::PWR.sr().modify(|w| w.set_cssf(true));
    #[cfg(stm32wba)]
    // Clear WKUP1..WKUP8 pending flags (CWUFx) before stop entry.
    crate::pac::PWR.wuscr().write(|w| w.0 = 0xff);

    // LPSDSR also puts the regulator in low-power mode in Sleep, which needs the system clock
    // at or below MSI range 1 (RM0377 6.3.8), so it's only set for Stop.
    #[cfg(stm32l0)]
    crate::pac::PWR.cr().modify(|w| {
        w.set_cwuf(true);
        w.set_lpsdsr(crate::pac::pwr::vals::Mode::MainMode);
    });

    // L1 doesn't enter Stop with WUF set (RM0038 5.3.7), and LPSDSR makes Sleep Low-power
    // sleep, which needs the system clock at or below MSI range 1 (RM0038 5.3.6).
    #[cfg(stm32l1)]
    crate::pac::PWR.cr().modify(|w| {
        w.set_cwuf(true);
        w.set_lpsdsr(false);
    });

    #[cfg(stm32f3)]
    crate::pac::PWR.cr().modify(|w| w.set_cwuf(true));

    #[cfg(stm32g4)]
    // G4 keeps its wake-up flags (WUF1..WUF5) in PWR_SR1 and the standby flag (SBF) there
    // too; both are cleared by writing ones to PWR_SCR. A stale flag left over from a
    // previous wake must not survive into the next entry. Writes are used rather than
    // `modify` so we don't depend on the read-back value of a write-one-to-clear register.
    crate::pac::PWR.scr().write(|w| {
        w.set_cwuf1(true);
        w.set_cwuf2(true);
        w.set_cwuf3(true);
        w.set_cwuf4(true);
        w.set_cwuf5(true);
        w.set_csbf(true);
    });
}

/// Exit stop mode: restore the clocks, clear the flags, reinitialize the timer if required and resume time.
fn exit_stop(cs: CriticalSection) {
    // Both cores share the clocks, so they restore them one at a time.
    #[cfg(any(stm32wb, stm32wl5x))]
    let lock = crate::hsem::get_hsem(3).blocking_lock(0);

    crate::rcc::stop_exit(cs);

    #[cfg(any(stm32wb, stm32wl5x))]
    drop(lock);

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

        // Clear STOPF by writing CSSF
        crate::pac::PWR.sr().modify(|w| w.set_cssf(true));
    }

    #[cfg(stm32h5)]
    crate::pac::PWR.pmcr().modify(|w| w.set_cssf(true));

    #[cfg(any(stm32wl, stm32wb, stm32wba))]
    {
        #[cfg(any(stm32wl, stm32wb))]
        let es = crate::pac::PWR.extscr().read();

        #[cfg(stm32wba)]
        let es = crate::pac::PWR.sr().read();

        #[cfg(any(stm32wl, stm32wb, stm32wba))]
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
                // C1STOPF is only set when the system as a whole entered Stop (RM0473),
                // i.e. exactly when HSE/PLL are gone and the clocks were restored above:
                // the GPTIM is then parked and its prescaler must be recomputed for the
                // restored timer clock. LPTIM keeps running from LSI/LSE through Stop and
                // is skipped by the cfg inside the block below.
                es.c1stopf()
            }
            #[cfg(stm32wba)]
            {
                es.stopf()
            }
        };

        #[cfg(all(stm32wba, power_stop2))]
        match (es.stopf(), es.stop2f()) {
            (true, true) => debug!("low power: WBA woke from STOP2"),
            (true, false) => debug!("low power: WBA woke from STOP0/1"),
            _ => {}
        };

        #[cfg(all(stm32wba, not(power_stop2)))]
        if es.stopf() {
            debug!("low power: WBA woke from STOP0/1");
        }

        #[cfg(stm32wl)]
        match (es.c1stopf(), es.c1stop2f()) {
            (true, false) => debug!("low power: cpu1 has been in STOP1"),
            (false, true) => debug!("low power: cpu1 has been in STOP2"),
            (true, true) => debug!("low power: cpu1 has been in STOP1 and STOP2 ???"),
            (false, false) => trace!("low power: cpu1 stop mode not entered"),
        };

        #[cfg(stm32wl5x)]
        // TODO: only for the current cpu
        match (es.c2stopf(), es.c2stop2f()) {
            (true, false) => debug!("low power: cpu2 has been in STOP1"),
            (false, true) => debug!("low power: cpu2 has been in STOP2"),
            (true, true) => debug!("low power: cpu2 has been in STOP1 and STOP2 ???"),
            (false, false) => trace!("low power: cpu2 stop mode not entered"),
        };

        #[cfg(stm32wb)]
        match (es.c1stopf(), es.c2stopf()) {
            (true, false) => debug!("low power: cpu1 has been in STOP"),
            (false, true) => debug!("low power: cpu2 has been in STOP"),
            (true, true) => debug!("low power: cpu1 and cpu2 have been in STOP"),
            (false, false) => trace!("low power: stop mode not entered"),
        };

        clear_flags();

        #[cfg(any(stm32wl, stm32wb, stm32wba))]
        if re_initialize_timer {
            trace!("low power: re-initializing timer");
            // when we wake from STOP2, we need to re-initialize the time driver
            #[cfg(not(feature = "_lp-time-driver"))]
            get_driver().init_timer(cs);
        }
    }

    #[cfg(any(stm32l0, stm32l1, stm32l4, stm32l5, stm32u0, stm32f3, stm32f4, stm32g4))]
    clear_flags();

    get_driver().resume_time(cs);
    trace!("low power: resumed");
}

fn configure_pwr(cs: CriticalSection) -> bool {
    let mut scb: SCB = unsafe { mem::transmute(()) };

    scb.clear_sleepdeep();
    clear_flags();

    compiler_fence(Ordering::Acquire);

    let Some(stop_mode) = get_stop_mode(cs) else {
        return false;
    };

    #[cfg(not(power_stop2))]
    let stop_mode = match stop_mode {
        StopMode::Standby | StopMode::Stop2 => StopMode::Stop1,
        other => other,
    };

    #[cfg(stm32wba)]
    let stop_mode = {
        let Some(radio_max) = crate::rcc::radio_max_stop_mode(cs) else {
            trace!("low_power: radio busy, using WFI");
            return false;
        };
        let capped = if stop_mode.at_least(radio_max) {
            radio_max
        } else {
            stop_mode
        };
        // `Into<Lpms>` enters Stop 1 (`power_stop1`) or Stop 2 (`power_stop2`),
        // both of which put the 2.4 GHz radio into deep sleep.
        let deep_stop = cfg!(any(power_stop1, power_stop2));
        if crate::rcc::radio_before_stop(cs, deep_stop).is_err() {
            trace!("low_power: radio vetoed stop, using WFI");
            return false;
        }
        capped
    };

    if get_driver().pause_time(cs).is_err() {
        trace!("low_power: failed to pause time, not entering stop");

        false
    } else if enter_stop(stop_mode).is_err() {
        trace!("low_power: failed to enter stop");

        false
    } else {
        crate::rcc::stop_enter(cs);

        #[cfg(any(stm32l0, stm32f4))]
        trace!("low power: enter stop");
        #[cfg(not(any(stm32l0, stm32f4)))]
        trace!("low power: enter stop: {:?}", stop_mode);

        scb.set_sleepdeep();

        true
    }
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

    #[cfg(stm32wba)]
    crate::rcc::radio_before_wfi(cs);

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

    #[cfg(stm32wba)]
    crate::rcc::radio_after_wfi(cs, stop);
}
