//! Deep sleep entry, exit, and clock recovery.

use core::cell::Ref;
use core::ops::Deref;

use cortex_m::peripheral::SCB;
use critical_section::CriticalSection;

use super::CLOCKS;
use super::types::{Clocks, PoweredClock};
use crate::pac;
use crate::pac::cmc::Ckmode;
#[cfg(feature = "mcxa2xx")]
use crate::pac::scg::Fircvld;
use crate::pac::scg::{Sircvld, SpllLock};
use crate::pac::spc::{PdLpReq, SpcLpReq};

const ALLOW_ALL_LOW_POWER_MODES: u32 = 0x0f;

/// Attempt to go to deep sleep if possible.
///
/// If we successfully went and returned from deep sleep, this function returns a `true`.
/// If we were unsuccessful due to active `WaitGuard`s, this function returns a `false`.
///
/// ## SAFETY
///
/// Care must be taken that we have ensured that the system is ready to go to deep
/// sleep, otherwise HAL peripherals may misbehave. `crate::clocks::init()` must
/// have been called and returned successfully, with a `CoreSleep` configuration
/// set to DeepSleep (or lower).
pub unsafe fn deep_sleep_if_possible(cs: &CriticalSection) -> bool {
    let inhibit = crate::clocks::active_wake_guards(cs);
    if inhibit {
        return false;
    }

    // SAFETY: this function has the same clock-initialization and critical-section
    // requirements as go_to_deep_sleep.
    unsafe { go_to_deep_sleep(cs) };

    true
}

pub unsafe fn go_to_deep_sleep(cs: &CriticalSection) {
    unsafe {
        setup_deep_sleep();
        enter_low_power_mode();
        recover_deep_sleep(cs);
    }
}

pub unsafe fn go_to_sleep(cs: &CriticalSection) {
    unsafe {
        setup_sleep();
        enter_low_power_mode();
        recover_sleep(cs);
    }
}

pub unsafe fn go_to_power_down(cs: &CriticalSection) {
    unsafe {
        setup_power_down();
        enter_low_power_mode();
        recover_power_down(cs);
    }
}

/// Enter Deep Power Down.
///
/// This function does not return because waking from Deep Power Down resets the
/// core. Wake sources and any required external-domain isolation must be
/// configured before calling this function.
///
/// ## Safety
///
/// The caller must ensure all peripherals are ready for Deep Power Down,
/// `crate::clocks::init()` completed successfully, and the supplied critical
/// section remains held during entry.
pub unsafe fn go_to_deep_power_down(_cs: &CriticalSection) -> ! {
    unsafe {
        setup_deep_power_down();
    }

    // SAFETY: SCB is a zero-sized singleton register handle. The caller holds
    // the critical section required by this entry routine.
    let mut scb: SCB = unsafe { core::mem::transmute(()) };
    scb.set_sleepdeep();

    // A successful wake from Deep Power Down resets the core. If WFE returns
    // without entering the mode, retry rather than continuing with lost state.
    loop {
        cortex_m::asm::dsb();
        cortex_m::asm::wfe();
        cortex_m::asm::isb();
    }
}

/// Enter the CMC mode selected by the setup routine.
///
/// `SEVONPEND` is enabled during clock initialization, so `WFE` wakes when an
/// interrupt becomes pending even while the caller's critical section masks
/// interrupt handling.
unsafe fn enter_low_power_mode() {
    // SAFETY: SCB is a zero-sized singleton register handle. The caller holds
    // the critical section required by the public entry routines.
    let mut scb: SCB = unsafe { core::mem::transmute(()) };

    scb.set_sleepdeep();
    cortex_m::asm::dsb();
    cortex_m::asm::wfe();
    cortex_m::asm::isb();
    scb.clear_sleepdeep();
}

/// Prepare the system for deep sleep
///
/// ## SAFETY
///
/// Care must be taken that we have ensured that the system is ready to go to deep
/// sleep, otherwise HAL peripherals may misbehave. `crate::clocks::init()` must
/// have been called and returned successfully.
unsafe fn setup_deep_sleep() {
    let cmc = nxp_pac::CMC;

    // To configure for Deep Sleep Low-Power mode entry:
    //
    // Write Fh to Clock Control (CKCTRL)
    cmc.ckctrl().modify(|w| w.set_ckmode(Ckmode::Ckmode1111));
    // Allow all low-power modes in Power Mode Protection (PMPROT)
    cmc.pmprot()
        .modify(|w| w.0 = (w.0 & !ALLOW_ALL_LOW_POWER_MODES) | ALLOW_ALL_LOW_POWER_MODES);
    // Write 1h to Global Power Mode Control (GPMCTRL)
    cmc.gpmctrl().modify(|w| w.set_lpmode(0b0001));

    // From the C SDK:
    //
    // Before executing the sleep instruction read back the last register to
    // ensure all registers writes have completed.
    let _ = cmc.gpmctrl().read();
}

/// Prepare the system for sleep
///
/// ## SAFETY
///
/// Care must be taken that we have ensured that the system is ready to go to
/// sleep, otherwise HAL peripherals may misbehave. `crate::clocks::init()` must
/// have been called and returned successfully.
unsafe fn setup_sleep() {
    let cmc = nxp_pac::CMC;

    // To configure for Sleep Low-Power mode entry:
    //
    // Write 1h to Clock Control (CKCTRL)
    cmc.ckctrl().modify(|w| w.set_ckmode(Ckmode::Ckmode0001));
    // Allow all low-power modes in Power Mode Protection (PMPROT)
    cmc.pmprot()
        .modify(|w| w.0 = (w.0 & !ALLOW_ALL_LOW_POWER_MODES) | ALLOW_ALL_LOW_POWER_MODES);
    // Write 0h to Global Power Mode Control (GPMCTRL)
    cmc.gpmctrl().modify(|w| w.set_lpmode(0b0000));

    // From the C SDK:
    //
    // Before executing the sleep instruction read back the last register to
    // ensure all registers writes have completed.
    let _ = cmc.gpmctrl().read();
}

/// Prepare the system for power down
///
/// ## SAFETY
///
/// Care must be taken that we have ensured that the system is ready to go to
/// Power Down, otherwise HAL peripherals may misbehave. `crate::clocks::init()` must
/// have been called and returned successfully.
unsafe fn setup_power_down() {
    let cmc = nxp_pac::CMC;

    // To configure for Power Down Low-Power mode entry:
    //
    // Write Fh to Clock Control (CKCTRL)
    cmc.ckctrl().modify(|w| w.set_ckmode(Ckmode::Ckmode1111));
    // Allow all low-power modes in Power Mode Protection (PMPROT)
    cmc.pmprot()
        .modify(|w| w.0 = (w.0 & !ALLOW_ALL_LOW_POWER_MODES) | ALLOW_ALL_LOW_POWER_MODES);
    // Write 3h to Global Power Mode Control (GPMCTRL)
    cmc.gpmctrl().modify(|w| w.set_lpmode(0b0011));

    // From the C SDK:
    //
    // Before executing the sleep instruction read back the last register to
    // ensure all registers writes have completed.
    let _ = cmc.gpmctrl().read();
}

/// Prepare the system for Deep Power Down.
///
/// ## Safety
///
/// The caller must ensure the system and wake sources are configured for a
/// reset-on-wake Deep Power Down transition.
unsafe fn setup_deep_power_down() {
    let cmc = nxp_pac::CMC;

    // Gate all system clocks and request Deep Power Down for the power domain.
    cmc.ckctrl().modify(|w| w.set_ckmode(Ckmode::Ckmode1111));
    cmc.pmprot()
        .modify(|w| w.0 = (w.0 & !ALLOW_ALL_LOW_POWER_MODES) | ALLOW_ALL_LOW_POWER_MODES);
    cmc.gpmctrl().modify(|w| w.set_lpmode(0b1111));

    // Ensure all register writes complete before executing WFE.
    let _ = cmc.gpmctrl().read();
}

/// Start back up after deep sleep returns
///
/// ## SAFETY
///
/// Care must be taken that we have ensured that the system is ready to go to deep
/// sleep, otherwise HAL peripherals may misbehave. `crate::clocks::init()` must
/// have been called and returned successfully, with a `CoreSleep` configuration
/// set to DeepSleep (or lower).
unsafe fn recover_deep_sleep(cs: &CriticalSection) {
    let cmc = nxp_pac::CMC;

    // Restart any necessary clocks
    unsafe {
        restart_active_only_clocks(cs);
    }
    clear_low_power_status(cs);

    // Re-raise the sleep level to WFE sleep in the off chance that the
    // user decides to call `wfe` on their own accord, and to avoid having
    // to re-set if we chill in WFE sleep mostly
    cmc.ckctrl().modify(|w| w.set_ckmode(Ckmode::Ckmode0001));
}

/// Restore the normal light-sleep configuration after waking from Sleep.
///
/// ## Safety
///
/// This must be called in the same critical section used to enter Sleep.
unsafe fn recover_sleep(cs: &CriticalSection) {
    let cmc = nxp_pac::CMC;
    clear_low_power_status(cs);
    cmc.ckctrl().modify(|w| w.set_ckmode(Ckmode::Ckmode0001));
}

/// Restore active-only clocks after waking from Power Down.
///
/// ## Safety
///
/// This must be called in the same critical section used to enter Power Down.
unsafe fn recover_power_down(cs: &CriticalSection) {
    // SAFETY: Power Down uses the same active-clock gating and recovery
    // requirements as Deep Sleep.
    unsafe { recover_deep_sleep(cs) };
}

/// Clear the low-power request and clock-gated flags latched by the CMC/SPC.
fn clear_low_power_status(_cs: &CriticalSection) {
    let cmc = nxp_pac::CMC;
    if !cmc.ckstat().read().valid() {
        return;
    }

    let spc = nxp_pac::SPC0;
    spc.pd_status0().modify(|w| w.set_pd_lp_req(PdLpReq::ReqYes));
    spc.sc().modify(|w| w.set_spc_lp_req(SpcLpReq::LowPower));
    cmc.ckstat().modify(|w| w.set_valid(true));
}

/// Perform any actions necessary to re-initialize clocks after returning to active
/// mode after a low power (e.g. deep sleep, power-off) state.
///
/// ## Safety
///
/// This should only be called in a critical section, immediately after waking up.
unsafe fn restart_active_only_clocks(_cs: &CriticalSection) {
    let bref: Ref<'_, Option<Clocks>> = CLOCKS.borrow_ref(*_cs);
    let dref: &Option<Clocks> = bref.deref();
    let Some(clocks) = dref else {
        return;
    };
    let scg = pac::SCG0;

    // TODO: Restart clock monitors if necessary? Needs to be re-enabled
    // AFTER FRO12M has been started, and probably after clocks are
    // valid again.
    //
    // TODO: Timeout? Check error fields (at least for SPLL)? Clear
    // or reset any status bits?

    // Ensure FRO12M is up and running
    if let Some(fro12m) = clocks.fro_12m_root.as_ref()
        && !matches!(fro12m.power, PoweredClock::AlwaysEnabled)
    {
        while scg.sirccsr().read().sircvld() != Sircvld::EnabledAndValid {}
    }

    // Ensure FRO45M is up and running
    #[cfg(feature = "mcxa2xx")]
    if let Some(frohf) = clocks.fro_hf_root.as_ref()
        && !matches!(frohf.power, PoweredClock::AlwaysEnabled)
    {
        while scg.firccsr().read().fircvld() != Fircvld::EnabledAndValid {}
    }

    // Ensure SOSC is up and running
    #[cfg(not(feature = "sosc-as-gpio"))]
    if let Some(clk_in) = clocks.clk_in.as_ref()
        && !matches!(clk_in.power, PoweredClock::AlwaysEnabled)
    {
        while !scg.sosccsr().read().soscvld() {}
    }

    // Ensure SPLL is up and running
    if let Some(spll) = clocks.pll1_clk.as_ref()
        && !matches!(spll.power, PoweredClock::AlwaysEnabled)
    {
        while scg.spllcsr().read().spll_lock() != SpllLock::EnabledAndValid {}
    }
}
