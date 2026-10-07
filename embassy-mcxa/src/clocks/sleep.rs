//! Checked CMC sleep entry, exit, and clock recovery.
//!
//! Entry follows the NXP CMC configuration, readback, and barrier sequence.
//! Sleep and Deep Sleep retain WFE event wakeups.
//! The caller holds a critical section through clock and idle-mode recovery.
//! SCR sleep control is saved per call and restored before returning, and
//! status is captured before recovery modifies the CMC/SPC registers.

use core::cell::Ref;
use core::ops::Deref;

use cortex_m::peripheral::SCB;
use critical_section::CriticalSection;

use super::CLOCKS;
use super::config::CoreSleep;
use super::types::{Clocks, PoweredClock};
use crate::pac;
use crate::pac::cmc::Ckmode;
#[cfg(feature = "mcxa2xx")]
use crate::pac::scg::Fircvld;
use crate::pac::scg::{Sircvld, SpllLock};
use crate::pac::spc::{PdLpReq, SpcLpReq};

const LOW_POWER_MODE_MASK: u32 = 0x0f;
const ALLOW_DEEP_SLEEP: u32 = 0x01;
const SLEEPDEEP: u32 = 1 << 2;
const SEVONPEND: u32 = 1 << 4;
const SLEEP_CONTROL_MASK: u32 = SLEEPDEEP | SEVONPEND;

/// Power mode entry error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum PowerModeError {
    /// Clock initialization has not completed.
    ClockNotInitialized,
    /// CKCTRL is locked with an incompatible clocking mode.
    ClockControlLocked,
    /// PMPROT is locked without allowing the requested power mode.
    PowerModeProtectionLocked,
    /// The CMC rejected a requested power mode configuration.
    ConfigurationRejected,
    /// The configured idle mode could not be restored.
    RecoveryRejected,
}

/// Power-state registers captured immediately after the sleep instruction returns.
///
/// A requested mode and `CKSTAT.VALID` do not prove the power domain reached
/// that mode. In particular, WFE can consume an event without gating the core.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct SleepStatus {
    /// MAIN domain mode requested before executing WFE.
    pub requested_main_mode: u32,
    /// CMC clock status before recovery clears it.
    pub ckstat: u32,
    /// MAIN domain mode observed immediately after wake.
    pub wake_main_mode: u32,
    /// SPC power-domain status before recovery clears it.
    pub pd_status0: u32,
    /// SPC status before recovery clears it.
    pub spc_sc: u32,
}

impl SleepStatus {
    /// Whether CMC recorded core clock gating during this entry attempt.
    pub fn core_clock_was_gated(&self) -> bool {
        pac::cmc::Ckstat(self.ckstat).valid()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PowerMode {
    Sleep,
    DeepSleep,
}

impl PowerMode {
    fn configuration(self) -> (Ckmode, u32, u8) {
        match self {
            Self::Sleep => (Ckmode::Ckmode0001, 0, 0),
            Self::DeepSleep => (Ckmode::Ckmode1111, ALLOW_DEEP_SLEEP, 1),
        }
    }
}

/// Attempt to go to deep sleep if possible.
///
/// Returns `Ok(None)` when active `WakeGuard`s inhibit entry. `Ok(Some(status))`
/// means the WFE entry sequence ran, including when an event prevented sleep.
/// After any attempted entry, the executor must poll runnable work again.
///
/// ## SAFETY
///
/// Care must be taken that we have ensured that the system is ready to go to deep
/// sleep, otherwise HAL peripherals may misbehave. `crate::clocks::init()` must
/// have been called and returned successfully, with a `CoreSleep` configuration
/// set to DeepSleep (or lower).
/// Hold the critical section through recovery and execute ISB after restoring
/// the caller's interrupt mask.
pub unsafe fn deep_sleep_if_possible(cs: &CriticalSection) -> Result<Option<SleepStatus>, PowerModeError> {
    let inhibit = crate::clocks::active_wake_guards(cs);
    if inhibit {
        return Ok(None);
    }

    // SAFETY: this function has the same clock-initialization and critical-section
    // requirements as go_to_deep_sleep.
    unsafe { go_to_deep_sleep_with_status(cs) }.map(Some)
}

/// Enter Deep Sleep, preserving event wakeups and restoring the configured idle mode.
///
/// # Safety
///
/// Clock initialization must have completed and peripherals must be quiescent.
/// The caller must hold the critical section through entry and recovery, then
/// execute ISB after restoring its interrupt mask.
pub unsafe fn go_to_deep_sleep(cs: &CriticalSection) -> Result<(), PowerModeError> {
    // SAFETY: this wrapper has the same contract as the status-returning routine.
    unsafe { go_to_deep_sleep_with_status(cs) }.map(|_| ())
}

/// Enter Deep Sleep and capture the hardware result before recovery.
///
/// # Safety
///
/// The caller must satisfy the requirements of [`go_to_deep_sleep`].
pub unsafe fn go_to_deep_sleep_with_status(cs: &CriticalSection) -> Result<SleepStatus, PowerModeError> {
    // SAFETY: the caller holds the critical section through clock recovery.
    unsafe { enter_power_mode(cs, PowerMode::DeepSleep) }
}

/// Enter core-clock-gated Sleep and restore the configured idle mode.
///
/// # Safety
///
/// Clock initialization must have completed. The caller must hold the critical
/// section through entry and recovery, then execute ISB after restoring its
/// interrupt mask.
pub unsafe fn go_to_sleep(cs: &CriticalSection) -> Result<(), PowerModeError> {
    // SAFETY: this wrapper has the same contract as the status-returning routine.
    unsafe { go_to_sleep_with_status(cs) }.map(|_| ())
}

/// Enter Sleep and capture the hardware result before recovery.
///
/// # Safety
///
/// The caller must satisfy the requirements of [`go_to_sleep`].
pub unsafe fn go_to_sleep_with_status(cs: &CriticalSection) -> Result<SleepStatus, PowerModeError> {
    // SAFETY: the caller holds the critical section through idle-mode recovery.
    unsafe { enter_power_mode(cs, PowerMode::Sleep) }
}

unsafe fn enter_power_mode(cs: &CriticalSection, mode: PowerMode) -> Result<SleepStatus, PowerModeError> {
    // Synchronize the caller's interrupt masking before configuring low power.
    cortex_m::asm::isb();
    let idle_mode = {
        let clocks = CLOCKS.borrow_ref(*cs);
        let clocks = clocks.as_ref().ok_or(PowerModeError::ClockNotInitialized)?;
        idle_clock_mode(clocks.core_sleep)
    };
    let cmc = pac::CMC;
    let (clock_mode, protection, low_power_mode) = mode.configuration();
    validate_power_mode(
        cmc.ckctrl().read(),
        cmc.pmprot().read(),
        clock_mode,
        idle_mode,
        protection,
    )?;
    if let Err(error) = prepare_power_mode(cmc, clock_mode, protection, low_power_mode) {
        restore_idle_mode(cmc, idle_mode)?;
        return Err(error);
    }

    // SAFETY: SCB is an MMIO-only zero-sized handle. The caller masks interrupts
    // and restores the affected SCR bits before returning.
    let scb: SCB = unsafe { core::mem::transmute(()) };
    let saved_scr = scb.scr.read();
    // SAFETY: only the architectural SLEEPDEEP/SEVONPEND bits are changed.
    unsafe { scb.scr.modify(sleep_control_for_entry) };
    let requested_main_mode = cmc.pmctrlmain().read().0;
    clear_previous_clock_status();
    clear_spc_low_power_status();
    cortex_m::asm::dsb();
    cortex_m::asm::wfe();
    cortex_m::asm::isb();

    let status = SleepStatus {
        requested_main_mode,
        ckstat: cmc.ckstat().read().0,
        wake_main_mode: cmc.pmctrlmain().read().0,
        pd_status0: pac::SPC0.pd_status0().read().0,
        spc_sc: pac::SPC0.sc().read().0,
    };
    if mode != PowerMode::Sleep {
        // SAFETY: recovery is performed in the same critical section as entry.
        unsafe { restart_active_only_clocks(cs) };
    }
    clear_spc_low_power_status();
    let recovery = restore_idle_mode(cmc, idle_mode);
    // SAFETY: restore only our SCR changes, preserving unrelated control bits.
    unsafe { scb.scr.modify(|current| restore_sleep_control(current, saved_scr)) };
    cortex_m::asm::isb();
    recovery?;

    Ok(status)
}

fn idle_clock_mode(core_sleep: CoreSleep) -> Ckmode {
    match core_sleep {
        CoreSleep::WfeUngated => Ckmode::Ckmode0000,
        CoreSleep::WfeGated | CoreSleep::DeepSleep => Ckmode::Ckmode0001,
    }
}

fn sleep_control_for_entry(scr: u32) -> u32 {
    scr | SLEEP_CONTROL_MASK
}

fn restore_sleep_control(current: u32, saved: u32) -> u32 {
    (current & !SLEEP_CONTROL_MASK) | (saved & SLEEP_CONTROL_MASK)
}

fn validate_power_mode(
    ckctrl: pac::cmc::Ckctrl,
    pmprot: pac::cmc::Pmprot,
    clock_mode: Ckmode,
    idle_mode: Ckmode,
    protection: u32,
) -> Result<(), PowerModeError> {
    // A locked deep configuration is also rejected if it prevents idle recovery.
    if ckctrl.lock() && (ckctrl.ckmode() != clock_mode || ckctrl.ckmode() != idle_mode) {
        return Err(PowerModeError::ClockControlLocked);
    }
    if pmprot.lock() && pmprot.0 & protection != protection {
        return Err(PowerModeError::PowerModeProtectionLocked);
    }
    Ok(())
}

fn prepare_power_mode(
    cmc: pac::cmc::Cmc,
    clock_mode: Ckmode,
    protection: u32,
    low_power_mode: u8,
) -> Result<(), PowerModeError> {
    cmc.ckctrl().modify(|w| w.set_ckmode(clock_mode));
    if cmc.ckctrl().read().ckmode() != clock_mode {
        return Err(PowerModeError::ConfigurationRejected);
    }
    if protection != 0 {
        if !cmc.pmprot().read().lock() {
            cmc.pmprot().modify(|w| w.0 = (w.0 & !LOW_POWER_MODE_MASK) | protection);
        }
        if cmc.pmprot().read().0 & protection != protection {
            return Err(PowerModeError::ConfigurationRejected);
        }
    }
    cmc.gpmctrl().write(|w| w.set_lpmode(low_power_mode));
    // Readback completes the broadcast write; PMCTRLMAIN verifies its effect.
    let _ = cmc.gpmctrl().read();
    if cmc.pmctrlmain().read().lpmode().to_bits() != low_power_mode {
        return Err(PowerModeError::ConfigurationRejected);
    }

    Ok(())
}

fn restore_idle_mode(cmc: pac::cmc::Cmc, idle_mode: Ckmode) -> Result<(), PowerModeError> {
    cmc.gpmctrl().write(|w| w.set_lpmode(0));
    let _ = cmc.gpmctrl().read();
    cmc.ckctrl().modify(|w| w.set_ckmode(idle_mode));
    if cmc.pmctrlmain().read().lpmode().to_bits() != 0 || cmc.ckctrl().read().ckmode() != idle_mode {
        return Err(PowerModeError::RecoveryRejected);
    }
    Ok(())
}

/// Clear SPC request flags independently of CMC wake status.
fn clear_spc_low_power_status() {
    let spc = pac::SPC0;
    spc.pd_status0().modify(|w| w.set_pd_lp_req(PdLpReq::ReqYes));
    spc.sc().modify(|w| w.set_spc_lp_req(SpcLpReq::LowPower));
}

/// Clear the previous CMC result immediately before a new entry attempt.
fn clear_previous_clock_status() {
    // CKSTAT.VALID is write-one-to-clear.
    pac::CMC.ckstat().modify(|w| w.set_valid(true));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode_encodings_match_cmc_reference() {
        assert_eq!(PowerMode::Sleep.configuration(), (Ckmode::Ckmode0001, 0, 0));
        assert_eq!(PowerMode::DeepSleep.configuration(), (Ckmode::Ckmode1111, 1, 1));
    }

    #[test]
    fn idle_policy_preserves_ungated_and_gated_modes() {
        assert_eq!(idle_clock_mode(CoreSleep::WfeUngated), Ckmode::Ckmode0000);
        assert_eq!(idle_clock_mode(CoreSleep::WfeGated), Ckmode::Ckmode0001);
        assert_eq!(idle_clock_mode(CoreSleep::DeepSleep), Ckmode::Ckmode0001);
    }

    #[test]
    fn debug_retention_is_opt_in_by_default() {
        assert!(!super::super::config::ClocksConfig::default().vdd_power.debug_in_sleep);
    }

    #[test]
    fn entry_enables_sleep_and_interrupt_event_wakeups() {
        let sleep_on_exit = 1 << 1;
        assert_eq!(
            sleep_control_for_entry(sleep_on_exit),
            sleep_on_exit | SLEEPDEEP | SEVONPEND
        );
    }

    #[test]
    fn recovery_restores_sleep_bits_and_preserves_other_scr_bits() {
        let unrelated = (1 << 1) | (1 << 8);
        for saved in [0, SLEEPDEEP, SEVONPEND, SLEEP_CONTROL_MASK] {
            assert_eq!(
                restore_sleep_control(unrelated | SLEEP_CONTROL_MASK, saved),
                unrelated | saved
            );
        }
    }

    #[test]
    fn locked_clock_mode_must_allow_entry_and_recovery() {
        let mut ckctrl = pac::cmc::Ckctrl::default();
        ckctrl.set_ckmode(Ckmode::Ckmode1111);
        ckctrl.set_lock(true);
        assert_eq!(
            validate_power_mode(
                ckctrl,
                pac::cmc::Pmprot::default(),
                Ckmode::Ckmode1111,
                Ckmode::Ckmode0001,
                ALLOW_DEEP_SLEEP,
            ),
            Err(PowerModeError::ClockControlLocked)
        );
    }

    #[test]
    fn locked_sleep_configuration_can_be_reused() {
        let mut ckctrl = pac::cmc::Ckctrl::default();
        ckctrl.set_ckmode(Ckmode::Ckmode0001);
        ckctrl.set_lock(true);
        let mut pmprot = pac::cmc::Pmprot::default();
        pmprot.set_lock(true);
        assert_eq!(
            validate_power_mode(ckctrl, pmprot, Ckmode::Ckmode0001, Ckmode::Ckmode0001, 0),
            Ok(())
        );
    }

    #[test]
    fn locked_protection_must_allow_requested_mode() {
        let mut pmprot = pac::cmc::Pmprot::default();
        pmprot.set_lock(true);
        assert_eq!(
            validate_power_mode(
                pac::cmc::Ckctrl::default(),
                pmprot,
                Ckmode::Ckmode1111,
                Ckmode::Ckmode0001,
                ALLOW_DEEP_SLEEP,
            ),
            Err(PowerModeError::PowerModeProtectionLocked)
        );
        pmprot.0 |= ALLOW_DEEP_SLEEP;
        assert_eq!(
            validate_power_mode(
                pac::cmc::Ckctrl::default(),
                pmprot,
                Ckmode::Ckmode1111,
                Ckmode::Ckmode0001,
                ALLOW_DEEP_SLEEP,
            ),
            Ok(())
        );
    }

    #[test]
    fn requested_power_mode_does_not_imply_clock_gating() {
        let mut status = SleepStatus {
            requested_main_mode: 1,
            ckstat: 0,
            wake_main_mode: 1,
            pd_status0: 0,
            spc_sc: 0,
        };
        assert!(!status.core_clock_was_gated());
        let mut ckstat = pac::cmc::Ckstat::default();
        ckstat.set_valid(true);
        status.ckstat = ckstat.0;
        assert!(status.core_clock_was_gated());
    }
}
