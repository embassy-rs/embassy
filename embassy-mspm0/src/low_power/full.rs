//! Full-capability deep sleep: STOP0/1/2 + STANDBY0/1.
//!
//! Covers every family whose SYSCTL exposes the full STOP policy: all G families and the supported
//! L families.
//!
//! The entry sequence from the TRM is:
//! `PMODECFG.DSLEEP` selects  STOP vs STANDBY,
//! `SYSOSCCFG.{USE4MHZSTOP, DISABLESTOP}` combination selects the STOP sub-mode,
//! `MCLKCFG.STOPCLKSTBY` selects the STANDBY sub-mode.

use critical_section::CriticalSection;
use pac::sysctl::vals::Dsleep;

use crate::pac;

/// Deep-sleep idle modes, ordered by increasing power saving.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SleepMode {
    /// SYSOSC stays at full speed. Fastest wake, highest STOP current.
    Stop0,
    /// SYSOSC limited to 4 MHz.
    Stop1,
    /// SYSOSC disabled; ULPCLK runs from LFCLK. Lowest STOP current.
    Stop2,
    /// low-speed peripherals retained.
    Standby0,
    /// only TIMG0/TIMG1 remain clocked. Lowest wake-capable current.
    Standby1,
}

/// Enter a deep-sleep `mode` and block until an interrupt wakes the core.
///
/// This runs with interrupts masked, but `WFI` still wakes on enabled interrupts with PRIMASK set.
/// They will run once the `CriticalSection` exits.
///
/// A wake source must already be armed before calling: a live `Timer` on TIMG0/TIMG1, or any
/// wake-capable peripheral/IO interrupt. In STANDBY only TIMG0/TIMG1 remain clocked.
///
/// # Safety
/// The caller is responsible for ensuring deep sleep is safe right now: no transaction that must survive is in
/// flight (PD1 powers down and its peripherals lose state unless retained by the mode), and a wake source is armed.
pub unsafe fn enter_sleep(_cs: CriticalSection, mode: SleepMode) {
    let sysctl = pac::SYSCTL;

    let dsleep = match mode {
        SleepMode::Stop0 | SleepMode::Stop1 | SleepMode::Stop2 => Dsleep::STOP,
        SleepMode::Standby0 | SleepMode::Standby1 => Dsleep::STANDBY,
    };
    sysctl.pmodecfg().modify(|w| w.set_dsleep(dsleep));

    match mode {
        SleepMode::Stop0 => sysctl.sysosccfg().modify(|w| {
            w.set_use4mhzstop(false);
            w.set_disablestop(false);
        }),
        SleepMode::Stop1 => sysctl.sysosccfg().modify(|w| {
            w.set_use4mhzstop(true);
            w.set_disablestop(false);
        }),
        SleepMode::Stop2 => sysctl.sysosccfg().modify(|w| {
            w.set_use4mhzstop(false);
            w.set_disablestop(true);
        }),
        SleepMode::Standby0 => sysctl.mclkcfg().modify(|w| w.set_stopclkstby(false)),
        SleepMode::Standby1 => sysctl.mclkcfg().modify(|w| w.set_stopclkstby(true)),
    }

    super::arm_and_wait();
}
