//! Driver-owned FIRC retention for ordinary MCXA5xx Deep Sleep.
//!
//! This is not a WakeGuard exemption. A validated driver retains the clocks
//! needed by its live hardware instead of claiming that hardware is stopped.
//! CMC restores buses and SRAM for asynchronous DMA requests without waking the
//! core (MCXA5xx RM 26.3.3.2 and 30.3.3). Power Down is not supported.

use core::sync::atomic::{AtomicUsize, Ordering};

use super::Clocks;
use crate::pac;
use crate::pac::scg::{Firccsr, FirccsrLk, Fircerr, Fircsten, Scs};

#[cfg(not(feature = "mcxa5xx"))]
compile_error!("retained-deep-sleep currently requires mcxa5xx");

static LIVE_FIRC_REQUIREMENTS: AtomicUsize = AtomicUsize::new(0);

/// Only validated drivers may create this requirement.
pub(crate) struct RetainedFirc(());

impl RetainedFirc {
    pub(crate) fn new() -> Self {
        LIVE_FIRC_REQUIREMENTS.fetch_add(1, Ordering::AcqRel);
        Self(())
    }

    pub(crate) fn configuration_supported(clocks: &Clocks) -> bool {
        let Some(root) = clocks.fro_hf_root.as_ref() else {
            return false;
        };
        let Some(system) = clocks.cpu_system_clk.as_ref() else {
            return false;
        };
        clocks.active_power == clocks.lp_power
            && clocks.bandgap_active
            && clocks.bandgap_lowpower
            && clocks.fro_hf_div.is_some()
            && root.frequency == system.frequency
            && root.frequency != 0
    }

    pub(crate) fn hardware_supported() -> bool {
        pac::SCG0.csr().read().scs() == Scs::Firc
            && pac::CMC.sramdis0().read().0 == 0
            && pac::CMC.sramret0().read().0 == 0
    }

    pub(crate) fn buffer_supported(address: usize, length: usize) -> bool {
        address >= 0x2000_0000 && length != 0 && address.checked_add(length).is_some_and(|end| end <= 0x200A_0000)
    }
}

impl Drop for RetainedFirc {
    fn drop(&mut self) {
        let previous = LIVE_FIRC_REQUIREMENTS.fetch_sub(1, Ordering::AcqRel);
        assert!(previous != 0, "FIRC requirement underflow");
    }
}

pub(super) fn required() -> bool {
    LIVE_FIRC_REQUIREMENTS.load(Ordering::Acquire) != 0
}

pub(super) fn control(mut value: Firccsr, retain: bool, lock: FirccsrLk) -> Firccsr {
    value.set_fircsten(if retain {
        Fircsten::EnabledInStopModes
    } else {
        Fircsten::DisabledInStopModes
    });
    value.set_lk(lock);
    value.set_fircerr(Fircerr::ErrorNotDetected);
    value
}

pub(super) fn set_retention(scg: pac::scg::Scg, retain: bool, lock: FirccsrLk) -> bool {
    let current = scg.firccsr().read();
    scg.firccsr().write_value(control(
        current,
        current.fircsten() == Fircsten::EnabledInStopModes,
        FirccsrLk::WriteEnabled,
    ));
    scg.firccsr().write_value(control(current, retain, lock));
    let result = scg.firccsr().read();
    (result.fircsten() == Fircsten::EnabledInStopModes) == retain && result.lk() == lock
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;
    use crate::clocks::{Clock, PoweredClock, VddLevel};
    use crate::pac::scg::{FircaccIe, FircerrIe};

    fn clocks() -> Clocks {
        let mut clocks = Clocks::default();
        clocks.active_power = VddLevel::NormalMode;
        clocks.lp_power = VddLevel::NormalMode;
        clocks.bandgap_active = true;
        clocks.bandgap_lowpower = true;
        let clock = Clock {
            frequency: 96_000_000,
            power: PoweredClock::NormalEnabledDeepSleepDisabled,
        };
        clocks.fro_hf_root = Some(clock.clone());
        clocks.fro_hf_div = Some(clock.clone());
        clocks.cpu_system_clk = Some(clock);
        clocks
    }

    #[test]
    fn rejects_unsupported_low_power_voltage_bandgap_and_bus_clock() {
        let mut configuration = clocks();
        assert!(RetainedFirc::configuration_supported(&configuration));
        configuration.lp_power = VddLevel::MidDriveMode;
        assert!(!RetainedFirc::configuration_supported(&configuration));
        configuration.lp_power = VddLevel::NormalMode;
        configuration.bandgap_lowpower = false;
        assert!(!RetainedFirc::configuration_supported(&configuration));
        configuration.bandgap_lowpower = true;
        if let Some(system) = configuration.cpu_system_clk.as_mut() {
            system.frequency = 48_000_000;
        }
        assert!(!RetainedFirc::configuration_supported(&configuration));
    }

    #[test]
    fn only_accepts_nonempty_internal_sram_buffers_without_overflow() {
        assert!(RetainedFirc::buffer_supported(0x2000_0000, 1));
        assert!(RetainedFirc::buffer_supported(0x2009_FFFF, 1));
        assert!(!RetainedFirc::buffer_supported(0x2009_FFFF, 2));
        assert!(!RetainedFirc::buffer_supported(0x2000_0000, 0));
        assert!(!RetainedFirc::buffer_supported(0x1000_0000, 16));
        assert!(!RetainedFirc::buffer_supported(usize::MAX, 16));
    }

    #[test]
    fn retention_never_removes_an_independent_wake_guard() {
        critical_section::with(|cs| {
            let before = LIVE_FIRC_REQUIREMENTS.load(Ordering::Acquire);
            let guard = crate::clocks::WakeGuard::new();
            let retention = RetainedFirc::new();
            assert!(required());
            assert!(crate::clocks::active_wake_guards(&cs));
            drop(retention);
            assert_eq!(LIVE_FIRC_REQUIREMENTS.load(Ordering::Acquire), before);
            assert!(crate::clocks::active_wake_guards(&cs));
            drop(guard);
        });
    }

    #[test]
    fn each_owner_retains_firc_until_the_last_owner_stops() {
        critical_section::with(|_| {
            let before = LIVE_FIRC_REQUIREMENTS.load(Ordering::Acquire);
            let tx = RetainedFirc::new();
            let rx = RetainedFirc::new();
            assert_eq!(LIVE_FIRC_REQUIREMENTS.load(Ordering::Acquire), before + 2);
            drop(tx);
            assert_eq!(LIVE_FIRC_REQUIREMENTS.load(Ordering::Acquire), before + 1);
            drop(rx);
            assert_eq!(LIVE_FIRC_REQUIREMENTS.load(Ordering::Acquire), before);
        });
    }

    #[test]
    fn stop_retention_readback_round_trips_without_changing_clock_enable() {
        let registers = std::boxed::Box::leak(std::boxed::Box::new([0u32; 512]));
        // SAFETY: the aligned, isolated allocation covers the SCG register block.
        let scg = unsafe { pac::scg::Scg::from_ptr(registers.as_mut_ptr().cast()) };
        scg.firccsr().write(|w| {
            w.set_fircen(true);
            w.set_lk(FirccsrLk::WriteDisabled);
        });
        let original = scg.firccsr().read();
        assert!(set_retention(scg, true, original.lk()));
        assert!(scg.firccsr().read().fircen());
        assert!(set_retention(scg, false, original.lk()));
        assert_eq!(scg.firccsr().read().0, original.0);
    }

    #[test]
    fn changing_stop_retention_preserves_enable_and_does_not_acknowledge_errors() {
        let mut initial = Firccsr(0);
        initial.set_fircen(true);
        initial.set_firc_fclk_periph_en(true);
        initial.set_firc_sclk_periph_en(true);
        initial.set_fircerr(Fircerr::ErrorDetected);
        initial.set_fircerr_ie(FircerrIe::ErrorDetected);
        initial.set_fircacc_ie(FircaccIe::Fircaccyes);
        let retained = control(initial, true, FirccsrLk::WriteDisabled);
        assert!(retained.fircen());
        assert!(retained.firc_fclk_periph_en());
        assert!(retained.firc_sclk_periph_en());
        assert_eq!(retained.fircsten(), Fircsten::EnabledInStopModes);
        assert_eq!(retained.lk(), FirccsrLk::WriteDisabled);
        assert_eq!(retained.fircerr(), Fircerr::ErrorNotDetected);
        assert_eq!(retained.fircerr_ie(), initial.fircerr_ie());
        assert_eq!(retained.fircacc_ie(), initial.fircacc_ie());
        let restored = control(retained, false, FirccsrLk::WriteDisabled);
        assert_eq!(restored.fircsten(), initial.fircsten());
        assert!(restored.fircen());
    }
}
