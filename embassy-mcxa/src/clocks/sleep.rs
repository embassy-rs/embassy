//! Deep sleep entry, exit, and clock recovery.

use core::cell::Ref;
use core::ops::Deref;

use critical_section::CriticalSection;

use super::CLOCKS;
use super::types::{Clocks, PoweredClock};
use crate::pac;
use crate::pac::cmc::Ckmode;
#[cfg(feature = "mcxa2xx")]
use crate::pac::scg::Fircvld;
use crate::pac::scg::{Sircvld, SpllLock};

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

    unsafe {
        // Yep, it's time to go to deep sleep. WHILE STILL IN the CS, get ready
        setup_deep_sleep();

        // Here we go!
        //
        // It is okay to WFE with interrupts disabled: we have enabled SEVONPEND
        cortex_m::asm::dsb();
        cortex_m::asm::wfe();

        // Wakey wakey, eggs and bakey
        recover_deep_sleep(cs);
    }

    true
}

/// Prepare the system for deep sleep
///
/// ## SAFETY
///
/// Care must be taken that we have ensured that the system is ready to go to deep
/// sleep, otherwise HAL peripherals may misbehave. `crate::clocks::init()` must
/// have been called and returned successfully, with a `CoreSleep` configuration
/// set to DeepSleep (or lower).
unsafe fn setup_deep_sleep() {
    let cmc = nxp_pac::CMC;
    let spc = nxp_pac::SPC0;

    // Isolate/unpower external voltage domains
    spc.evd_cfg().write(|w| w.0 = 0);

    // To configure for Deep Sleep Low-Power mode entry:
    //
    // Write Fh to Clock Control (CKCTRL)
    cmc.ckctrl().modify(|w| w.set_ckmode(Ckmode::Ckmode1111));
    // Write 1h to Power Mode Protection (PMPROT)
    cmc.pmprot().write(|w| w.0 = 1);
    // Write 1h to Global Power Mode Control (GPMCTRL)
    cmc.gpmctrl().modify(|w| w.set_lpmode(0b0001));
    // Redundant?
    // cmc.pmctrlmain().modify(|w| w.set_lpmode(PmctrlmainLpmode::LPMODE0001));

    // From the C SDK:
    //
    // Before executing WFI instruction read back the last register to
    // ensure all registers writes have completed.
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

    // Re-raise the sleep level to WFE sleep in the off chance that the
    // user decides to call `wfe` on their own accord, and to avoid having
    // to re-set if we chill in WFE sleep mostly
    cmc.ckctrl().modify(|w| w.set_ckmode(Ckmode::Ckmode0001));
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
