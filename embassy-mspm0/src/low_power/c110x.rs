//! C-series deep sleep: STOP0/2 + STANDBY0/1 (no STOP1).
//!
//! Covers mspm0c110x and mspm0c1105/c1106. These families lack the STOP1 (4 MHz SYSOSC) sub-mode,
//! and their STOP0 additionally clears `USELFCLK`.
//!
//! The entry sequence from the TRM is:
//! `PMODECFG.DSLEEP` selects STOP vs STANDBY,
//! `SYSOSCCFG.DISABLESTOP` selects STOP0 vs STOP2 (this family has no 4 MHz STOP1) with STOP0 also clearing `MCLKCFG.USELFCLK`,
//! `MCLKCFG.STOPCLKSTBY` selects STANDBY0 vs STANDBY1.

use critical_section::CriticalSection;
use pac::sysctl::vals::Dsleep;

use crate::pac;

/// Deep-sleep idle modes, ordered by increasing power saving.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SleepMode {
    /// SYSOSC available. Fastest wake, highest STOP current.
    Stop0,
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
/// # Safety
/// The caller is responsible for ensuring deep sleep is safe right now: no transaction that must survive is in
/// flight (PD1 powers down and its peripherals lose state unless retained by the mode), and a wake source is armed.
pub unsafe fn enter_sleep(_cs: CriticalSection, mode: SleepMode) {
    let sysctl = pac::SYSCTL;

    let dsleep = match mode {
        SleepMode::Stop0 | SleepMode::Stop2 => Dsleep::STOP,
        SleepMode::Standby0 | SleepMode::Standby1 => Dsleep::STANDBY,
    };
    sysctl.pmodecfg().modify(|w| w.set_dsleep(dsleep));

    match mode {
        SleepMode::Stop0 => {
            sysctl.sysosccfg().modify(|w| w.set_disablestop(false));
            sysctl.mclkcfg().modify(|w| w.set_uselfclk(false));
        }
        SleepMode::Stop2 => sysctl.sysosccfg().modify(|w| w.set_disablestop(true)),
        SleepMode::Standby0 => sysctl.mclkcfg().modify(|w| w.set_stopclkstby(false)),
        SleepMode::Standby1 => sysctl.mclkcfg().modify(|w| w.set_stopclkstby(true)),
    }

    super::arm_and_wait();
}
