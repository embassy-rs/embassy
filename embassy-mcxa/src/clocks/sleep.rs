//! Sleep and deep sleep management for the MCXA2xx and MCXA5xx series
//! This file contains code to manage sleep and deep sleep entry and exit.
//! This file supports the custom executor in embassy-mcxa/src/executor and
//! a user supplied custom executor.
//! The API calls with the name go_to_* are available for the custom executor
//! and do ignore the wake guards.
//! NOTE BENE:
//!   When using a custom executor, it is the users responsibility to manage
//! wake guards correctly.

//! Checked CMC sleep entry, exit, and clock recovery.
//!
//! Entry follows the NXP CMC configuration, readback, and barrier sequence.
//! Sleep and Deep Sleep retain WFE event wakeups.
//! The caller holds a critical section through clock and idle-mode recovery.
//! SCR sleep control is saved per call and restored before returning, and
//! status is captured before recovery modifies the CMC/SPC registers.
//! Entry/recovery follows MCXAP144M240F61RM Rev. 2 sections 18.3.3, 18.6,
//! 22.3.1, and 25.7 for MCXA2xx, and MCXAP172M240F70RM Rev. 1 sections
//! 26.3.3, 26.6, 29.3.1, and 32.7 for MCXA5xx. Both families require an
//! active-only SPLL to be stopped explicitly; when it supplies the CPU, SIRC
//! carries execution until the reference clocks and SPLL are ready again.
//! Active-only FIRC recovery waits for both valid and accurate output.

use cortex_m::peripheral::SCB;
use critical_section::CriticalSection;

use super::CLOCKS;
use super::config::CoreSleep;
use super::types::{Clocks, PoweredClock};
use crate::pac;
use crate::pac::cmc::Ckmode;
use crate::pac::scg::{
    Fircacc, Firccsr, Fircvld, Rccr, Scs, Sirccsr, SirccsrLk, Sircerr, Sircvld, SpllLock, Spllcsr, SpllcsrLk, Spllerr,
};
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
    /// A requested CMC or clock-source configuration was rejected.
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
    let clock_state = CLOCKS.borrow_ref(*cs);
    let clocks = clock_state.as_ref().ok_or(PowerModeError::ClockNotInitialized)?;
    let idle_mode = idle_clock_mode(clocks.core_sleep);
    let cmc = pac::CMC;
    let spc = pac::SPC0;
    let scg = pac::SCG0;
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

    let pll_state = if mode == PowerMode::DeepSleep {
        // SAFETY: peripherals are quiescent and the caller masks interrupts
        // throughout the temporary CPU clock switch and PLL restart.
        match unsafe { suspend_active_only_pll(scg, clocks) } {
            Ok(state) => state,
            Err(error) => {
                restore_idle_mode(cmc, idle_mode)?;
                return Err(error);
            }
        }
    } else {
        None
    };

    // SAFETY: SCB is an MMIO-only zero-sized handle. The caller masks interrupts
    // and restores the affected SCR bits before returning.
    let scb: SCB = unsafe { core::mem::transmute(()) };
    let saved_scr = scb.scr.read();
    // SAFETY: only the architectural SLEEPDEEP/SEVONPEND bits are changed.
    unsafe { scb.scr.modify(sleep_control_for_entry) };
    let requested_main_mode = cmc.pmctrlmain().read().0;
    clear_previous_clock_status(cmc);
    clear_spc_low_power_status(spc);
    cortex_m::asm::dsb();
    cortex_m::asm::wfe();
    cortex_m::asm::isb();

    let status = SleepStatus {
        requested_main_mode,
        ckstat: cmc.ckstat().read().0,
        wake_main_mode: cmc.pmctrlmain().read().0,
        pd_status0: spc.pd_status0().read().0,
        spc_sc: spc.sc().read().0,
    };
    if mode != PowerMode::Sleep {
        // SAFETY: recovery is performed in the same critical section as entry.
        unsafe { restart_active_only_clocks(scg, clocks, pll_state.as_ref()) };
    }
    clear_spc_low_power_status(spc);
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
fn clear_spc_low_power_status(spc: pac::spc::Spc) {
    // Direct W1C writes must not echo unrelated flags such as ISO_CLR or BUSY.
    spc.pd_status0().write(|w| w.set_pd_lp_req(PdLpReq::ReqYes));
    spc.sc().write(|w| w.set_spc_lp_req(SpcLpReq::LowPower));
    let _ = spc.sc().read();
}

/// Clear the previous CMC result immediately before a new entry attempt.
fn clear_previous_clock_status(cmc: pac::cmc::Cmc) {
    // CKSTAT.VALID is write-one-to-clear.
    cmc.ckstat().write(|w| w.set_valid(true));
}

struct PllStopState {
    control: Spllcsr,
    cpu_clock: Option<Rccr>,
    sirc_control: Option<Sirccsr>,
}

fn set_sirc_peripheral_gate(scg: pac::scg::Scg, enabled: bool, lock: SirccsrLk) {
    scg.sirccsr().modify(|w| {
        w.set_lk(SirccsrLk::WriteEnabled);
        w.set_sircerr(Sircerr::ErrorNotDetected);
    });
    scg.sirccsr().modify(|w| {
        w.set_sirc_clk_periph_en(enabled);
        w.set_lk(lock);
        w.set_sircerr(Sircerr::ErrorNotDetected);
    });
}

fn pll_control_for_stop(mut control: Spllcsr) -> Spllcsr {
    control.set_spllpwren(false);
    control.set_spllclken(false);
    control.set_spllcm(false);
    control.set_spllerr(Spllerr::DisabledOrNoError);
    control
}

fn pll_control_for_start(mut control: Spllcsr) -> Spllcsr {
    control.set_lk(SpllcsrLk::WriteEnabled);
    control.set_spllcm(false);
    control.set_spllerr(Spllerr::DisabledOrNoError);
    control
}

/// Both families require SPLLPWREN to be cleared before Deep Sleep when SPLLSTEN is zero.
///
/// # Safety
///
/// The caller must mask interrupts and quiesce clock consumers until recovery.
unsafe fn suspend_active_only_pll(scg: pac::scg::Scg, clocks: &Clocks) -> Result<Option<PllStopState>, PowerModeError> {
    let Some(pll) = clocks.pll1_clk.as_ref() else {
        return Ok(None);
    };
    if pll.power == PoweredClock::AlwaysEnabled {
        return Ok(None);
    }

    let control = scg.spllcsr().read();
    if !control.spllpwren() || !control.spllclken() || control.spllerr() == Spllerr::EnabledAndError {
        return Err(PowerModeError::ConfigurationRejected);
    }
    let rccr = scg.rccr().read();
    let mut sirc_control = None;
    let cpu_clock = if rccr.scs() == Scs::Spll {
        let sirc = scg.sirccsr().read();
        if !sirc.sirc_clk_periph_en() {
            set_sirc_peripheral_gate(scg, true, sirc.lk());
            sirc_control = Some(sirc);
        }
        while scg.sirccsr().read().sircvld() != Sircvld::EnabledAndValid {}
        scg.rccr().modify(|w| w.set_scs(Scs::Sirc));
        while scg.csr().read().scs() != Scs::Sirc {}
        Some(rccr)
    } else {
        None
    };

    scg.spllcsr().modify(|w| {
        w.set_lk(SpllcsrLk::WriteEnabled);
        w.set_spllerr(Spllerr::DisabledOrNoError);
    });
    scg.spllcsr().write_value(pll_control_for_stop(control));
    let state = PllStopState {
        control,
        cpu_clock,
        sirc_control,
    };
    let stopped = scg.spllcsr().read();
    if stopped.spllpwren() || stopped.spllclken() || stopped.spllcm() {
        // SAFETY: no sleep instruction ran; restore the original clocks before
        // returning a rejected configuration to the caller.
        unsafe { resume_active_only_pll(scg, &state) };
        return Err(PowerModeError::ConfigurationRejected);
    }

    Ok(Some(state))
}

/// Restart SPLL only after its analog LDO and reference clocks are ready.
///
/// # Safety
///
/// The caller must still hold the entry critical section.
unsafe fn resume_active_only_pll(scg: pac::scg::Scg, state: &PllStopState) {
    while !scg.ldocsr().read().vout_ok() {}
    scg.spllcsr().modify(|w| {
        w.set_lk(SpllcsrLk::WriteEnabled);
        w.set_spllerr(Spllerr::DisabledOrNoError);
    });
    scg.spllcsr().write_value(pll_control_for_start(state.control));
    while scg.spllcsr().read().spll_lock() != SpllLock::EnabledAndValid {}
    scg.spllcsr().modify(|w| {
        w.set_spllcm(state.control.spllcm());
        w.set_lk(state.control.lk());
        w.set_spllerr(Spllerr::DisabledOrNoError);
    });

    if let Some(rccr) = state.cpu_clock {
        scg.rccr().write_value(rccr);
        while scg.csr().read().scs() != rccr.scs() {}
    }
    if let Some(sirc) = state.sirc_control {
        set_sirc_peripheral_gate(scg, sirc.sirc_clk_periph_en(), sirc.lk());
    }
}

/// Perform any actions necessary to re-initialize clocks after returning to active
/// mode after a low power (e.g. deep sleep, power-off) state.
///
/// ## Safety
///
/// This should only be called in a critical section, immediately after waking up.
unsafe fn restart_active_only_clocks(scg: pac::scg::Scg, clocks: &Clocks, pll_state: Option<&PllStopState>) {
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

    // Ensure FIRC is valid and accurate before restarting its consumers.
    if let Some(frohf) = clocks.fro_hf_root.as_ref()
        && !matches!(frohf.power, PoweredClock::AlwaysEnabled)
    {
        while !firc_ready(scg.firccsr().read()) {}
    }

    // Ensure SOSC is up and running
    #[cfg(not(feature = "sosc-as-gpio"))]
    if let Some(clk_in) = clocks.clk_in.as_ref()
        && !matches!(clk_in.power, PoweredClock::AlwaysEnabled)
    {
        while !scg.sosccsr().read().soscvld() {}
    }

    if let Some(state) = pll_state {
        // SAFETY: references and LDO recover before restoring the CPU's PLL source.
        unsafe { resume_active_only_pll(scg, state) };
    }

    // Ensure SPLL is up and running.
    if let Some(spll) = clocks.pll1_clk.as_ref()
        && !matches!(spll.power, PoweredClock::AlwaysEnabled)
    {
        while scg.spllcsr().read().spll_lock() != SpllLock::EnabledAndValid {}
    }
}

fn firc_ready(control: Firccsr) -> bool {
    control.fircvld() == Fircvld::EnabledAndValid && control.fircacc() == Fircacc::EnabledAndValid
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spc_acknowledges_only_low_power_request_flags() {
        #[repr(C)]
        struct Registers {
            padding: [u32; 4],
            status: pac::spc::Sc,
            padding_to_domain: [u32; 7],
            domain: pac::spc::PdStatus0,
        }
        let mut registers = Registers {
            padding: [0; 4],
            status: pac::spc::Sc(0x0001_0013),
            padding_to_domain: [0; 7],
            domain: pac::spc::PdStatus0(u32::MAX),
        };
        assert_eq!(core::mem::offset_of!(Registers, domain), 0x30);
        // SAFETY: the isolated, aligned RAM block covers every accessed register.
        let spc = unsafe { pac::spc::Spc::from_ptr((&raw mut registers).cast()) };
        clear_spc_low_power_status(spc);
        assert_eq!(registers.status.0, 1 << 1);
        assert_eq!(registers.domain.0, 1 << 4);
    }

    #[test]
    fn cmc_acknowledges_only_clock_status_valid() {
        #[repr(C)]
        struct Registers {
            padding: [u32; 5],
            status: pac::cmc::Ckstat,
        }
        let mut registers = Registers {
            padding: [0; 5],
            status: pac::cmc::Ckstat(u32::MAX),
        };
        assert_eq!(core::mem::offset_of!(Registers, status), 0x14);
        // SAFETY: the isolated, aligned RAM block covers CKSTAT.
        let cmc = unsafe { pac::cmc::Cmc::from_ptr((&raw mut registers).cast()) };
        clear_previous_clock_status(cmc);
        assert_eq!(registers.status.0, 1 << 31);
    }

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

    mod firc_ready {
        use super::super::firc_ready;
        use super::*;

        #[test]
        fn disabled_output_is_not_ready() {
            assert!(!firc_ready(Firccsr::default()));
        }

        #[test]
        fn valid_output_must_also_be_accurate() {
            let mut control = Firccsr::default();
            control.set_fircvld(Fircvld::EnabledAndValid);
            assert!(!firc_ready(control));
        }

        #[test]
        fn accurate_output_must_also_be_valid() {
            let mut control = Firccsr::default();
            control.set_fircacc(Fircacc::EnabledAndValid);
            assert!(!firc_ready(control));
        }

        #[test]
        fn valid_and_accurate_output_is_ready() {
            let mut control = Firccsr::default();
            control.set_fircvld(Fircvld::EnabledAndValid);
            control.set_fircacc(Fircacc::EnabledAndValid);
            assert!(firc_ready(control));
        }
    }

    mod pll {
        use super::*;
        use crate::clocks::Clock;
        use crate::pac::scg::{Csr, Ldocsr, Sirccsr};

        /// Register-write model with preloaded hardware acknowledgements.
        /// Analog startup timing and live CPU clock switching require board tests.
        #[repr(C)]
        struct Registers {
            padding: [u32; 4],
            status: Csr,
            run_clock: Rccr,
            padding_to_sirc: [u32; 122],
            sirc: Sirccsr,
            padding_to_firc: [u32; 63],
            firc: Firccsr,
            padding_to_pll: [u32; 191],
            pll: Spllcsr,
            padding_to_ldo: [u32; 127],
            ldo: Ldocsr,
        }

        impl Registers {
            fn new(source: Scs) -> Self {
                let mut registers = Self {
                    padding: [0; 4],
                    status: Csr::default(),
                    run_clock: Rccr::default(),
                    padding_to_sirc: [0; 122],
                    sirc: Sirccsr::default(),
                    padding_to_firc: [0; 63],
                    firc: Firccsr::default(),
                    padding_to_pll: [0; 191],
                    pll: Spllcsr::default(),
                    padding_to_ldo: [0; 127],
                    ldo: Ldocsr::default(),
                };
                registers.status.set_scs(Scs::Sirc);
                registers.run_clock.set_scs(source);
                registers.sirc.set_sircvld(Sircvld::EnabledAndValid);
                registers.firc.set_fircvld(Fircvld::EnabledAndValid);
                registers.firc.set_fircacc(Fircacc::EnabledAndValid);
                registers.pll.set_spllpwren(true);
                registers.pll.set_spllclken(true);
                registers.pll.set_spllcm(true);
                registers.pll.set_lk(SpllcsrLk::WriteDisabled);
                registers.pll.set_spll_lock(SpllLock::EnabledAndValid);
                registers.ldo.set_vout_ok(true);
                registers
            }

            fn scg(&mut self) -> pac::scg::Scg {
                assert_eq!(core::mem::offset_of!(Self, sirc), 0x200);
                assert_eq!(core::mem::offset_of!(Self, firc), 0x300);
                assert_eq!(core::mem::offset_of!(Self, pll), 0x600);
                assert_eq!(core::mem::offset_of!(Self, ldo), 0x800);
                // SAFETY: the aligned RAM model covers all accessed SCG registers.
                unsafe { pac::scg::Scg::from_ptr((self as *mut Self).cast()) }
            }
        }

        fn active_only_clocks() -> Clocks {
            let mut clocks = Clocks::default();
            clocks.pll1_clk = Some(Clock {
                frequency: 48_000_000,
                power: PoweredClock::NormalEnabledDeepSleepDisabled,
            });
            clocks
        }

        #[test]
        fn stop_and_start_preserve_control_policy_without_acknowledging_errors() {
            let mut control = Spllcsr::default();
            control.set_spllpwren(true);
            control.set_spllclken(true);
            control.set_spllcm(true);
            control.set_lk(SpllcsrLk::WriteDisabled);
            control.set_spllerr(Spllerr::EnabledAndError);

            let stopped = pll_control_for_stop(control);
            assert!(!stopped.spllpwren());
            assert!(!stopped.spllclken());
            assert!(!stopped.spllcm());
            assert_eq!(stopped.lk(), control.lk());
            assert_eq!(stopped.spllerr(), Spllerr::DisabledOrNoError);

            let restarted = pll_control_for_start(control);
            assert!(restarted.spllpwren());
            assert!(restarted.spllclken());
            assert!(!restarted.spllcm());
            assert_eq!(restarted.lk(), SpllcsrLk::WriteEnabled);
            assert_eq!(restarted.spllerr(), Spllerr::DisabledOrNoError);
        }

        #[test]
        fn cpu_pll_uses_sirc_until_recovery() -> Result<(), PowerModeError> {
            let mut registers = Registers::new(Scs::Spll);
            registers.sirc.set_lk(SirccsrLk::WriteDisabled);
            let scg = registers.scg();
            // SAFETY: there are no concurrent users of this isolated RAM model.
            let state = unsafe { suspend_active_only_pll(scg, &active_only_clocks()) }?
                .ok_or(PowerModeError::ConfigurationRejected)?;
            assert_eq!(registers.run_clock.scs(), Scs::Sirc);
            assert!(registers.sirc.sirc_clk_periph_en());
            assert!(!registers.pll.spllpwren());
            assert!(!registers.pll.spllclken());
            assert!(!registers.pll.spllcm());

            registers.status.set_scs(Scs::Spll);
            // SAFETY: ready/lock feedback is preloaded in the isolated RAM model.
            unsafe { resume_active_only_pll(scg, &state) };
            assert_eq!(registers.run_clock.scs(), Scs::Spll);
            assert!(registers.pll.spllpwren());
            assert!(registers.pll.spllclken());
            assert!(registers.pll.spllcm());
            assert_eq!(registers.pll.lk(), SpllcsrLk::WriteDisabled);
            assert!(!registers.sirc.sirc_clk_periph_en());
            assert_eq!(registers.sirc.lk(), SirccsrLk::WriteDisabled);
            Ok(())
        }

        #[test]
        fn clock_recovery_restores_cpu_pll_after_active_only_firc() -> Result<(), PowerModeError> {
            let mut registers = Registers::new(Scs::Spll);
            let scg = registers.scg();
            let mut clocks = active_only_clocks();
            #[cfg(feature = "mcxa2xx")]
            let frequency = 45_000_000;
            #[cfg(feature = "mcxa5xx")]
            let frequency = 48_000_000;
            clocks.fro_hf_root = Some(Clock {
                frequency,
                power: PoweredClock::NormalEnabledDeepSleepDisabled,
            });

            // SAFETY: there are no concurrent users of this isolated RAM model.
            let state =
                unsafe { suspend_active_only_pll(scg, &clocks) }?.ok_or(PowerModeError::ConfigurationRejected)?;
            registers.status.set_scs(Scs::Spll);
            // SAFETY: FIRC validity, accuracy, and PLL/LDO feedback are preloaded
            // in the isolated RAM model before full clock recovery.
            unsafe { restart_active_only_clocks(scg, &clocks, Some(&state)) };

            assert_eq!(registers.run_clock.scs(), Scs::Spll);
            assert!(registers.pll.spllpwren());
            assert!(registers.pll.spllclken());
            assert!(registers.pll.spllcm());
            assert_eq!(registers.pll.lk(), SpllcsrLk::WriteDisabled);
            assert!(!registers.sirc.sirc_clk_periph_en());
            Ok(())
        }

        #[cfg(feature = "mcxa5xx")]
        #[test]
        fn pll_divide_by_two_setting_survives_stop_and_restart() -> Result<(), PowerModeError> {
            // MCXA5xx SPLLCSR bit 4 has no accessor in the pinned PAC.
            const SPLL_DIV2_EN: u32 = 1 << 4;
            let mut registers = Registers::new(Scs::Firc);
            registers.pll.0 |= SPLL_DIV2_EN;
            let scg = registers.scg();
            // SAFETY: there are no concurrent users of this isolated RAM model.
            let state = unsafe { suspend_active_only_pll(scg, &active_only_clocks()) }?
                .ok_or(PowerModeError::ConfigurationRejected)?;
            assert_eq!(registers.pll.0 & SPLL_DIV2_EN, SPLL_DIV2_EN);

            // SAFETY: ready/lock feedback is preloaded in the isolated RAM model.
            unsafe { resume_active_only_pll(scg, &state) };

            assert_eq!(registers.pll.0 & SPLL_DIV2_EN, SPLL_DIV2_EN);
            assert!(registers.pll.spllpwren());
            assert!(registers.pll.spllclken());
            Ok(())
        }

        #[test]
        fn peripheral_pll_does_not_change_cpu_source() -> Result<(), PowerModeError> {
            let mut registers = Registers::new(Scs::Firc);
            let scg = registers.scg();
            // SAFETY: there are no concurrent users of this isolated RAM model.
            let state = unsafe { suspend_active_only_pll(scg, &active_only_clocks()) }?
                .ok_or(PowerModeError::ConfigurationRejected)?;
            assert!(state.cpu_clock.is_none());
            assert_eq!(registers.run_clock.scs(), Scs::Firc);
            assert!(!registers.pll.spllpwren());
            // SAFETY: ready/lock feedback is preloaded in the isolated RAM model.
            unsafe { resume_active_only_pll(scg, &state) };
            assert_eq!(registers.run_clock.scs(), Scs::Firc);
            assert!(registers.pll.spllpwren());
            Ok(())
        }

        #[test]
        fn retained_pll_is_not_stopped() -> Result<(), PowerModeError> {
            let mut registers = Registers::new(Scs::Spll);
            let scg = registers.scg();
            let original = registers.pll;
            let mut clocks = active_only_clocks();
            if let Some(pll) = clocks.pll1_clk.as_mut() {
                pll.power = PoweredClock::AlwaysEnabled;
            }
            // SAFETY: there are no concurrent users of this isolated RAM model.
            assert!(unsafe { suspend_active_only_pll(scg, &clocks) }?.is_none());
            assert_eq!(registers.pll, original);
            assert_eq!(registers.run_clock.scs(), Scs::Spll);
            Ok(())
        }

        #[test]
        fn absent_pll_is_not_touched() -> Result<(), PowerModeError> {
            let mut registers = Registers::new(Scs::Firc);
            let scg = registers.scg();
            let original = registers.pll;
            // SAFETY: there are no concurrent users of this isolated RAM model.
            assert!(unsafe { suspend_active_only_pll(scg, &Clocks::default()) }?.is_none());
            assert_eq!(registers.pll, original);
            Ok(())
        }

        #[test]
        fn disabled_pll_is_rejected_before_changing_cpu_source() {
            let mut registers = Registers::new(Scs::Spll);
            registers.pll.set_spllpwren(false);
            let scg = registers.scg();
            // SAFETY: there are no concurrent users of this isolated RAM model.
            let result = unsafe { suspend_active_only_pll(scg, &active_only_clocks()) };
            assert!(matches!(result, Err(PowerModeError::ConfigurationRejected)));
            assert_eq!(registers.run_clock.scs(), Scs::Spll);
        }
    }
}
