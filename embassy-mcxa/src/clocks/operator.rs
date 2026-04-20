//! `ClockOperator` — init-time clock configuration sequencing.
//!
//! This module contains the private `ClockOperator` struct and all of its
//! `configure_*` methods. It is only used during [`super::init()`].

use config::{
    ClocksConfig, CoreSleep, FircConfig, FircFreqSel, Fro16KConfig, MainClockSource, SircConfig, VddDriveStrength,
    VddLevel,
};
use cortex_m::peripheral::SCB;

use super::config;
use super::types::{Clock, ClockError, Clocks, PoweredClock};
use crate::chips::{ClockLimits, clock_limits};
use crate::pac;
use crate::pac::cmc::Ckmode;
use crate::pac::scg::{
    Erefs, Fircacc, FircaccIe, FirccsrLk, Fircerr, FircerrIe, Fircsten, Range, Scs, SirccsrLk, Sircerr, Sircvld,
    SosccsrLk, Soscerr, Source, SpllLock, SpllcsrLk, Spllerr, Spllsten, TrimUnlock,
};
use crate::pac::spc::{
    ActiveCfgBgmode, ActiveCfgCoreldoVddDs, ActiveCfgCoreldoVddLvl, LpCfgBgmode, LpCfgCoreldoVddLvl, Vsm,
};
use crate::pac::syscon::{
    AhbclkdivUnstab, FrohfdivHalt, FrohfdivReset, FrohfdivUnstab, FrolfdivHalt, FrolfdivReset, FrolfdivUnstab,
    Pll1clkdivHalt, Pll1clkdivReset, Pll1clkdivUnstab, Unlock,
};

/// The ClockOperator is a private helper type that contains the methods used
/// during system clock initialization.
///
/// # SAFETY
///
/// Concurrent access to clock-relevant peripheral registers, such as `MRCC`, `SCG`,
/// `SYSCON`, and `VBAT` should not be allowed for the duration of the [`init()`](super::init) function.
#[allow(dead_code)]
pub(super) struct ClockOperator<'a> {
    /// A mutable reference to the current state of system clocks
    pub(super) clocks: &'a mut Clocks,
    /// A reference to the requested configuration provided by the caller of [`init()`](super::init)
    pub(super) config: &'a ClocksConfig,

    /// SIRC is forced-on until we set `main_clk`
    pub(super) sirc_forced: bool,

    // We hold on to stolen peripherals
    pub(super) _mrcc0: pac::mrcc::Mrcc,
    pub(super) scg0: pac::scg::Scg,
    pub(super) syscon: pac::syscon::Syscon,
    pub(super) vbat0: pac::vbat::Vbat,
    pub(super) spc0: pac::spc::Spc,
    pub(super) fmu0: pac::fmu::Fmu,
    pub(super) cmc: pac::cmc::Cmc,
}

impl ClockOperator<'_> {
    pub(super) fn unlock_mrcc(&mut self) {
        // On the MCXA5xx, this is default *locked*, preventing any writes to
        // MRCC registers re enable/div settings. For now, just leave it unlocked,
        // we might want to actively unlock/lock in periph helpers in the future.
        self.syscon.clkunlock().modify(|w| w.set_unlock(Unlock::Enable));
    }

    fn active_limits(&self) -> &'static ClockLimits {
        match self.config.vdd_power.active_mode.level {
            VddLevel::MidDriveMode => &ClockLimits::MID_DRIVE,
            #[cfg(feature = "mcxa5xx")]
            VddLevel::NormalMode => &ClockLimits::NORMAL_DRIVE,
            VddLevel::OverDriveMode => &ClockLimits::OVER_DRIVE,
        }
    }

    fn low_power_limits(&self) -> &'static ClockLimits {
        match self.config.vdd_power.low_power_mode.level {
            VddLevel::MidDriveMode => &ClockLimits::MID_DRIVE,
            #[cfg(feature = "mcxa5xx")]
            VddLevel::NormalMode => &ClockLimits::NORMAL_DRIVE,
            VddLevel::OverDriveMode => &ClockLimits::OVER_DRIVE,
        }
    }

    fn lowest_relevant_limits(&self, for_power: &PoweredClock) -> &'static ClockLimits {
        // We always enforce that deep sleep has a drive <= active mode.
        match for_power {
            PoweredClock::NormalEnabledDeepSleepDisabled => self.active_limits(),
            PoweredClock::AlwaysEnabled => self.low_power_limits(),
        }
    }

    /// Configure the FIRC/FRO180M/FRO192M clock family
    pub(super) fn configure_firc_clocks(&mut self) -> Result<(), ClockError> {
        // Three options here:
        //
        // * Firc is disabled -> Switch main clock to SIRC and return
        // * Firc is enabled and !default ->
        //   * Switch main clock to SIRC
        //   * Make FIRC changes
        //   * Switch main clock back to FIRC
        // * Firc is enabled and default -> nop
        #[cfg(feature = "mcxa2xx")]
        let default_freq = FircFreqSel::Mhz45;
        #[cfg(feature = "mcxa5xx")]
        let default_freq = FircFreqSel::Mhz48;
        let is_default = self.config.firc.as_ref().is_some_and(|c| c.frequency == default_freq);

        // If we are not default, then we need to switch to SIRC
        if !is_default {
            // Set SIRC (fro_12m) as the source
            self.scg0.rccr().modify(|w| w.set_scs(Scs::Sirc));

            // Wait for the change to complete
            while self.scg0.csr().read().scs() != Scs::Sirc {}
        }

        // Enable CSR writes
        self.scg0.firccsr().modify(|w| w.set_lk(FirccsrLk::WriteEnabled));

        // Did the user give us a FIRC config?
        let Some(firc) = self.config.firc.as_ref() else {
            // Nope, and we've already switched to fro_12m. Disable FIRC.
            self.scg0.firccsr().modify(|w| {
                w.set_fircsten(Fircsten::DisabledInStopModes);
                w.set_fircerr_ie(FircerrIe::ErrorNotDetected);
                w.set_firc_fclk_periph_en(false);
                w.set_firc_sclk_periph_en(false);
                w.set_fircen(false);
            });

            self.scg0.firccsr().modify(|w| w.set_lk(FirccsrLk::WriteDisabled));
            return Ok(());
        };

        // If we are here, we WANT FIRC. If we are !default, let's disable FIRC before
        // we mess with it. If we are !default, we have already switched to SIRC instead!
        if !is_default {
            // Unlock
            self.scg0.firccsr().modify(|w| w.set_lk(FirccsrLk::WriteEnabled));

            // Disable FIRC
            self.scg0.firccsr().modify(|w| {
                w.set_fircen(false);
                w.set_fircsten(Fircsten::DisabledInStopModes);
                w.set_fircerr_ie(FircerrIe::ErrorNotDetected);
                w.set_fircacc_ie(FircaccIe::Fircaccnot);
                w.set_firc_sclk_periph_en(false);
                w.set_firc_fclk_periph_en(false);
            });
        }

        let limits = self.lowest_relevant_limits(&firc.power);

        // Set frequency (if not the default!), re-enable FIRC, and return the base frequency
        let (base_freq, sel) = firc.frequency.to_freq_and_sel();

        self.scg0.firccfg().modify(|w| w.set_freq_sel(sel));
        self.scg0.firccsr().modify(|w| w.set_fircen(true));

        // Wait for FIRC to be enabled, error-free, and accurate
        let mut firc_ok = false;
        while !firc_ok {
            let csr = self.scg0.firccsr().read();

            firc_ok =
                csr.fircen() && csr.fircacc() == Fircacc::EnabledAndValid && csr.fircerr() == Fircerr::ErrorNotDetected;
        }

        // Note that the fro_hf_root is active
        self.clocks.fro_hf_root = Some(Clock {
            frequency: base_freq,
            power: firc.power,
        });

        // Okay! Now we're past that, let's enable all the downstream clocks.
        let FircConfig {
            frequency: _,
            power,
            fro_hf_enabled,
            clk_hf_fundamental_enabled,
            fro_hf_div,
        } = firc;

        // When is the FRO enabled?
        let (bg_good, pow_set) = match power {
            PoweredClock::NormalEnabledDeepSleepDisabled => {
                // We only need bandgap enabled in active Mode
                (self.clocks.bandgap_active, Fircsten::DisabledInStopModes)
            }
            PoweredClock::AlwaysEnabled => {
                // We need bandgaps enabled in both active and deep sleep mode
                let bg_good = self.clocks.bandgap_active && self.clocks.bandgap_lowpower;
                (bg_good, Fircsten::EnabledInStopModes)
            }
        };
        if !bg_good {
            return Err(ClockError::BadConfig {
                clock: "fro_hf",
                reason: "bandgap required to be enabled when clock enabled",
            });
        }

        // Do we enable the `fro_hf` output?
        let fro_hf_set = if *fro_hf_enabled {
            if base_freq > limits.fro_hf {
                return Err(ClockError::BadConfig {
                    clock: "fro_hf",
                    reason: "exceeds max",
                });
            }

            self.clocks.fro_hf = Some(Clock {
                frequency: base_freq,
                power: *power,
            });
            true
        } else {
            false
        };

        // Do we enable the `clk_45m`/`clk_48m` output?
        let clk_fund_set = if *clk_hf_fundamental_enabled {
            self.clocks.clk_hf_fundamental = Some(Clock {
                frequency: 45_000_000,
                power: *power,
            });
            true
        } else {
            false
        };

        self.scg0.firccsr().modify(|w| {
            w.set_fircsten(pow_set);
            w.set_firc_fclk_periph_en(fro_hf_set);
            w.set_firc_sclk_periph_en(clk_fund_set);
        });

        // Last write to CSR, re-lock
        self.scg0.firccsr().modify(|w| w.set_lk(FirccsrLk::WriteDisabled));

        // Do we enable the `fro_hf_div` output?
        if let Some(d) = fro_hf_div.as_ref() {
            // We need `fro_hf` to be enabled
            if !*fro_hf_enabled {
                return Err(ClockError::BadConfig {
                    clock: "fro_hf_div",
                    reason: "fro_hf not enabled",
                });
            }

            let div_freq = base_freq / d.into_divisor();
            if div_freq > limits.fro_hf_div {
                return Err(ClockError::BadConfig {
                    clock: "fro_hf_root",
                    reason: "exceeds max frequency",
                });
            }

            // Halt and reset the div; then set our desired div.
            self.syscon.frohfdiv().write(|w| {
                w.set_halt(FrohfdivHalt::Halt);
                w.set_reset(FrohfdivReset::Asserted);
                w.set_div(d.into_bits());
            });
            // Then unhalt it, and reset it
            self.syscon.frohfdiv().write(|w| {
                w.set_halt(FrohfdivHalt::Run);
                w.set_reset(FrohfdivReset::Released);
                w.set_div(d.into_bits());
            });

            // Wait for clock to stabilize
            while self.syscon.frohfdiv().read().unstab() == FrohfdivUnstab::Ongoing {}

            // Store off the clock info
            self.clocks.fro_hf_div = Some(Clock {
                frequency: div_freq,
                power: *power,
            });
        }

        Ok(())
    }

    /// Configure the SIRC/FRO12M clock family
    pub(super) fn configure_sirc_clocks_early(&mut self) -> Result<(), ClockError> {
        let SircConfig {
            power,
            fro_12m_enabled,
            fro_lf_div,
        } = &self.config.sirc;
        let base_freq = 12_000_000;

        // Allow writes
        self.scg0.sirccsr().modify(|w| w.set_lk(SirccsrLk::WriteEnabled));
        self.clocks.fro_12m_root = Some(Clock {
            frequency: base_freq,
            power: *power,
        });

        let deep = match power {
            PoweredClock::NormalEnabledDeepSleepDisabled => false,
            PoweredClock::AlwaysEnabled => true,
        };

        // clk_1m is *before* the fro_12m clock gate
        self.clocks.clk_1m = Some(Clock {
            frequency: base_freq / 12,
            power: *power,
        });

        // If the user wants fro_12m to be disabled, FOR now, we ignore their
        // wish to ensure fro_12m is selectable as a main_clk source at least until
        // we select the CPU clock. We still mark it as not enabled though, to prevent
        // other peripherals using it, as we will gate if off at `configure_sirc_clocks_late`.
        if *fro_12m_enabled {
            self.clocks.fro_12m = Some(Clock {
                frequency: base_freq,
                power: *power,
            });
        } else {
            self.sirc_forced = true;
        };

        // Set sleep/peripheral usage
        self.scg0.sirccsr().modify(|w| {
            w.set_sircsten(deep);
            // Always on, for now at least! Will be resolved in `configure_sirc_clocks_late`
            w.set_sirc_clk_periph_en(true);
        });

        while self.scg0.sirccsr().read().sircvld() == Sircvld::DisabledOrNotValid {}
        if self.scg0.sirccsr().read().sircerr() == Sircerr::ErrorDetected {
            return Err(ClockError::BadConfig {
                clock: "sirc",
                reason: "error set",
            });
        }

        // reset lock
        self.scg0.sirccsr().modify(|w| w.set_lk(SirccsrLk::WriteDisabled));

        // Do we enable the `fro_lf_div` output?
        if let Some(d) = fro_lf_div.as_ref() {
            // We need `fro_lf` to be enabled
            if !*fro_12m_enabled {
                return Err(ClockError::BadConfig {
                    clock: "fro_lf_div",
                    reason: "fro_12m not enabled",
                });
            }

            // Halt and reset the div; then set our desired div.
            self.syscon.frolfdiv().write(|w| {
                w.set_halt(FrolfdivHalt::Halt);
                w.set_reset(FrolfdivReset::Asserted);
                w.set_div(d.into_bits());
            });
            // Then unhalt it, and reset it
            self.syscon.frolfdiv().modify(|w| {
                w.set_halt(FrolfdivHalt::Run);
                w.set_reset(FrolfdivReset::Released);
                w.set_div(d.into_bits());
            });

            // Wait for clock to stabilize
            while self.syscon.frolfdiv().read().unstab() == FrolfdivUnstab::Ongoing {}

            // Store off the clock info
            self.clocks.fro_lf_div = Some(Clock {
                frequency: base_freq / d.into_divisor(),
                power: *power,
            });
        }

        Ok(())
    }

    pub(super) fn configure_sirc_clocks_late(&mut self) {
        // If we forced SIRC's fro_12m to be enabled, disable it now.
        if self.sirc_forced {
            // Allow writes
            self.scg0.sirccsr().modify(|w| w.set_lk(SirccsrLk::WriteEnabled));

            // Disable clk_12m
            self.scg0.sirccsr().modify(|w| w.set_sirc_clk_periph_en(false));

            // reset lock
            self.scg0.sirccsr().modify(|w| w.set_lk(SirccsrLk::WriteDisabled));
        }
    }

    /// Configure the ROSC/FRO16K/clk_16k clock family
    pub(super) fn configure_fro16k_clocks(&mut self) -> Result<(), ClockError> {
        // If we have a config: ensure fro16k is enabled. If not: ensure it is disabled.
        let enable = self.config.fro16k.is_some();
        self.vbat0.froctla().modify(|w| w.set_fro_en(enable));

        // Lock the control register
        self.vbat0.frolcka().modify(|w| w.set_lock(true));

        // If we're disabled, we're done!
        let Some(fro16k) = self.config.fro16k.as_ref() else {
            return Ok(());
        };

        // Enabled, now set up.
        let Fro16KConfig {
            vsys_domain_active,
            vdd_core_domain_active,
            #[cfg(feature = "mcxa5xx")]
            vbat_domain_active,
        } = fro16k;

        // Enable clock outputs to both VSYS and VDD_CORE domains
        // Bit 0: clk_16k0 to VSYS domain
        // Bit 1: clk_16k1 to VDD_CORE/CORE_MAIN domain
        // Bit 2: clk_16k2 to VBAT domain (5xx only)
        //
        // TODO: Define sub-fields for this register with a PAC patch?
        let mut bits = 0;
        if *vsys_domain_active {
            bits |= 0b01;
            self.clocks.clk_16k_vsys = Some(Clock {
                frequency: 16_384,
                power: PoweredClock::AlwaysEnabled,
            });
        }
        if *vdd_core_domain_active {
            bits |= 0b10;
            self.clocks.clk_16k_vdd_core = Some(Clock {
                frequency: 16_384,
                power: PoweredClock::AlwaysEnabled,
            });
        }
        #[cfg(feature = "mcxa5xx")]
        if *vbat_domain_active {
            bits |= 0b100;
            self.clocks.clk_16k_vbat = Some(Clock {
                frequency: 16_384,
                power: PoweredClock::AlwaysEnabled,
            });
        }
        self.vbat0.froclke().modify(|w| w.set_clke(bits));

        Ok(())
    }

    /// Configure the ROSC/OSC32K clock family
    #[cfg(all(feature = "mcxa5xx", feature = "unstable-osc32k", not(feature = "rosc-32k-as-gpio")))]
    pub(super) fn configure_osc32k_clocks(&mut self) -> Result<(), ClockError> {
        use config::{Osc32KCapSel, Osc32KCoarseGain, Osc32KMode};
        use nxp_pac::vbat::{
            CoarseAmpGain, ExtalCapSel, InitTrim, ModeEn, StatusaLdoRdy, StatusaOscRdy, SupplyDet, XtalCapSel,
        };

        // Unlock the control first
        self.vbat0.ldolcka().modify(|w| w.set_lock(false));

        let Some(cfg) = self.config.osc32k.as_ref() else {
            // TODO: how to ensure disabled?
            // ???

            // Re-lock after disabling
            self.vbat0.ldolcka().modify(|w| w.set_lock(true));
            return Ok(());
        };

        // To enable and lock the LDO and bandgap:
        //
        // NOTE(AJM): "The FRO16K must be enabled before enabling the SRAM LDO or the bandgap"
        //
        // 1. Enable the FRO16K.
        //   * NOTE(AJM): clk_16k is always enabled if enabled at all.
        //   * TODO(AJM): I'm not sure which domain needs to be active for this requirement.
        //     It seems reasonable that it would be the vbat domain?

        self.clocks.ensure_clk_16k_vbat_active(&PoweredClock::AlwaysEnabled)?;

        // 2. Write 7h to LDO_RAM Control A (LDOCTLA).
        self.vbat0.ldoctla().write(|w| {
            w.set_refresh_en(true);
            w.set_ldo_en(true);
            w.set_bg_en(true);
        });

        // 3. Wait for STATUSA[LDO_RDY] to become 1.
        while self.vbat0.statusa().read().ldo_rdy() != StatusaLdoRdy::SET {}

        // 4. Write 1h to LDOLCKA[LOCK].
        self.vbat0.ldolcka().modify(|w| w.set_lock(true));

        match &cfg.mode {
            Osc32KMode::HighPower {
                coarse_amp_gain,
                xtal_cap_sel,
                extal_cap_sel,
            } => {
                // To configure and lock OSC32kHz for normal mode operation:
                //
                // 1. Configure OSCCTLA[EXTAL_CAP_SEL], OSCCTLA[XTAL_CAP_SEL] and OSCCTLA[COARSE_AMP_GAIN] as
                // required based on the external crystal component ESR and CL values, and by the PCB parasitics on the EXTAL32K and
                // XTAL32K pins. Configure 0h to OSCCTLA[MODE_EN], 1h to OSCCTLA[CAP_SEL_EN], and 1h to OSCCTLA[OSC_EN].
                //   * NOTE(AJM): You must write 1 to this field and OSCCTLA[OSC_EN] simultaneously.
                self.vbat0.oscctla().modify(|w| {
                    w.set_xtal_cap_sel(match xtal_cap_sel {
                        Osc32KCapSel::Cap2PicoF => XtalCapSel::SEL2,
                        Osc32KCapSel::Cap4PicoF => XtalCapSel::SEL4,
                        Osc32KCapSel::Cap6PicoF => XtalCapSel::SEL6,
                        Osc32KCapSel::Cap8PicoF => XtalCapSel::SEL8,
                        Osc32KCapSel::Cap10PicoF => XtalCapSel::SEL10,
                        Osc32KCapSel::Cap12PicoF => XtalCapSel::SEL12,
                        Osc32KCapSel::Cap14PicoF => XtalCapSel::SEL14,
                        Osc32KCapSel::Cap16PicoF => XtalCapSel::SEL16,
                        Osc32KCapSel::Cap18PicoF => XtalCapSel::SEL18,
                        Osc32KCapSel::Cap20PicoF => XtalCapSel::SEL20,
                        Osc32KCapSel::Cap22PicoF => XtalCapSel::SEL22,
                        Osc32KCapSel::Cap24PicoF => XtalCapSel::SEL24,
                        Osc32KCapSel::Cap26PicoF => XtalCapSel::SEL26,
                        Osc32KCapSel::Cap28PicoF => XtalCapSel::SEL28,
                        Osc32KCapSel::Cap30PicoF => XtalCapSel::SEL30,
                    });
                    w.set_extal_cap_sel(match extal_cap_sel {
                        Osc32KCapSel::Cap2PicoF => ExtalCapSel::SEL2,
                        Osc32KCapSel::Cap4PicoF => ExtalCapSel::SEL4,
                        Osc32KCapSel::Cap6PicoF => ExtalCapSel::SEL6,
                        Osc32KCapSel::Cap8PicoF => ExtalCapSel::SEL8,
                        Osc32KCapSel::Cap10PicoF => ExtalCapSel::SEL10,
                        Osc32KCapSel::Cap12PicoF => ExtalCapSel::SEL12,
                        Osc32KCapSel::Cap14PicoF => ExtalCapSel::SEL14,
                        Osc32KCapSel::Cap16PicoF => ExtalCapSel::SEL16,
                        Osc32KCapSel::Cap18PicoF => ExtalCapSel::SEL18,
                        Osc32KCapSel::Cap20PicoF => ExtalCapSel::SEL20,
                        Osc32KCapSel::Cap22PicoF => ExtalCapSel::SEL22,
                        Osc32KCapSel::Cap24PicoF => ExtalCapSel::SEL24,
                        Osc32KCapSel::Cap26PicoF => ExtalCapSel::SEL26,
                        Osc32KCapSel::Cap28PicoF => ExtalCapSel::SEL28,
                        Osc32KCapSel::Cap30PicoF => ExtalCapSel::SEL30,
                    });
                    w.set_coarse_amp_gain(match coarse_amp_gain {
                        Osc32KCoarseGain::EsrRange0 => CoarseAmpGain::GAIN05,
                        Osc32KCoarseGain::EsrRange1 => CoarseAmpGain::GAIN10,
                        Osc32KCoarseGain::EsrRange2 => CoarseAmpGain::GAIN18,
                        Osc32KCoarseGain::EsrRange3 => CoarseAmpGain::GAIN33,
                    });
                    w.set_mode_en(ModeEn::HP);
                    w.set_cap_sel_en(true);
                    w.set_osc_en(true);
                });

                // 2. Wait for STATUSA[OSC_RDY] to become 1.
                while self.vbat0.statusa().read().osc_rdy() != StatusaOscRdy::SET {}

                // 3. Write 1h to OSCLCKA[LOCK].
                self.vbat0.osclcka().modify(|w| w.set_lock(true));

                // 4. Write 0h to OSCCTLA[EXTAL_CAP_SEL] and 0h to OSCCTLA[XTAL_CAP_SEL].
                self.vbat0.oscctla().modify(|w| {
                    w.set_xtal_cap_sel(XtalCapSel::SEL0);
                    w.set_extal_cap_sel(ExtalCapSel::SEL0);
                });

                // 5. Alter OSCCLKE[CLKE] to clock gate different OSC32K outputs to different peripherals to reduce power consumption.
                const ENABLED: Option<Clock> = Some(Clock {
                    frequency: 32_768,
                    power: PoweredClock::NormalEnabledDeepSleepDisabled,
                });
                self.vbat0.oscclke().modify(|w| {
                    let mut val = 0u8;
                    if cfg.vsys_domain_active {
                        val |= 0b001;
                        self.clocks.clk_32k_vsys = ENABLED;
                    }
                    if cfg.vdd_core_domain_active {
                        val |= 0b010;
                        self.clocks.clk_32k_vdd_core = ENABLED;
                    }
                    if cfg.vbat_domain_active {
                        val |= 0b100;
                        self.clocks.clk_32k_vbat = ENABLED;
                    }
                    w.set_clke(val);
                });
            }
            Osc32KMode::LowPower {
                coarse_amp_gain,
                vbat_exceeds_3v0,
            } => {
                // To configure OSC32kHz for low power mode operation:
                //
                // 1. Write 3h to OSCCFGA[INIT_TRIM].
                //   * NOTE(AJM): This is "1 second"?
                self.vbat0.osccfga().modify(|w| w.set_init_trim(InitTrim::SEL3));

                // 2. Configure OSCCTLA[EXTAL_CAP_SEL], OSCCTLA[XTAL_CAP_SEL] and OSCCTLA[COARSE_AMP_GAIN] as
                // required based on the external crystal component ESR and CL values, and by the PCB parasitics on the EXTAL32K and
                // XTAL32K pins. Configure 1h to OSCCTLA[MODE_EN], 1h to OSCCTLA[CAP_SEL_EN], and 1h to OSCCTLA[OSC_EN].
                //   * NOTE(AJM): The configuration EXTAL_CAP_SEL=0000 and CAP_SEL_EN=1 is required in low power
                //     mode and is not supported in other modes
                self.vbat0.oscctla().modify(|w| {
                    // TODO(AJM): Do we need to set these to reasonable values during the "startup" phase, and THEN
                    // restore them to 0? RM is very unclear here.
                    w.set_xtal_cap_sel(XtalCapSel::SEL0);
                    w.set_extal_cap_sel(ExtalCapSel::SEL0);

                    w.set_coarse_amp_gain(match coarse_amp_gain {
                        Osc32KCoarseGain::EsrRange0 => CoarseAmpGain::GAIN05,
                        Osc32KCoarseGain::EsrRange1 => CoarseAmpGain::GAIN10,
                        Osc32KCoarseGain::EsrRange2 => CoarseAmpGain::GAIN18,
                        Osc32KCoarseGain::EsrRange3 => CoarseAmpGain::GAIN33,
                    });

                    // TODO: This naming is bad
                    //
                    // pub enum ModeEn {
                    //     #[doc = "Normal mode"]
                    //     HP = 0x0,
                    //     #[doc = "Startup mode"]
                    //     LP = 0x01,
                    //     _RESERVED_2 = 0x02,
                    //     #[doc = "Low power mode"]
                    //     SW = 0x03,
                    // }

                    w.set_mode_en(ModeEn::LP);
                    w.set_cap_sel_en(true);
                    w.set_osc_en(true);
                });

                // 3. Wait for STATUSA[OSC_RDY] to become 1.
                while self.vbat0.statusa().read().osc_rdy() != StatusaOscRdy::SET {}

                // 4. Write 0h to OSCCFGA[INIT_TRIM].
                self.vbat0.osccfga().modify(|w| w.set_init_trim(InitTrim::SEL0));

                // 5. Configure 3h to OSCCTLA[MODE_EN], 0h to OSCCTLA[EXTAL_CAP_SEL] and 0h to OSCCTLA[XTAL_CAP_SEL].
                // Configure OSCCTLA[SUPPLY_DET] as required by application.
                self.vbat0.oscctla().modify(|w| {
                    w.set_mode_en(ModeEn::SW);
                    w.set_xtal_cap_sel(XtalCapSel::SEL0);
                    w.set_extal_cap_sel(ExtalCapSel::SEL0);
                    w.set_supply_det(if *vbat_exceeds_3v0 {
                        SupplyDet::G3VSUPPLY
                    } else {
                        SupplyDet::L3VSUPPLY
                    });
                });

                // 6. Wait for STATUSA[OSC_RDY] to become 1.
                while self.vbat0.statusa().read().osc_rdy() != StatusaOscRdy::SET {}

                // 7. Alter OSCCLKE[CLKE] to clock gate different OSC32K outputs to different peripherals to reduce power consumption.
                const ENABLED: Option<Clock> = Some(Clock {
                    frequency: 32_768,
                    power: PoweredClock::AlwaysEnabled,
                });
                self.vbat0.oscclke().modify(|w| {
                    let mut val = 0u8;
                    if cfg.vsys_domain_active {
                        val |= 0b001;
                        self.clocks.clk_32k_vsys = ENABLED;
                    }
                    if cfg.vdd_core_domain_active {
                        val |= 0b010;
                        self.clocks.clk_32k_vdd_core = ENABLED;
                    }
                    if cfg.vbat_domain_active {
                        val |= 0b100;
                        self.clocks.clk_32k_vbat = ENABLED;
                    }
                    w.set_clke(val);
                });
            }
        }

        Ok(())
    }

    fn ensure_ldo_active(&mut self, for_clock: &'static str, for_power: &PoweredClock) -> Result<(), ClockError> {
        let bg_good = match for_power {
            PoweredClock::NormalEnabledDeepSleepDisabled => self.clocks.bandgap_active,
            PoweredClock::AlwaysEnabled => self.clocks.bandgap_active && self.clocks.bandgap_lowpower,
        };
        if !bg_good {
            return Err(ClockError::BadConfig {
                clock: for_clock,
                reason: "LDO requires core bandgap enabled",
            });
        }

        // TODO: Config for the LDO? For now, just enable
        // using the default settings:
        // LDOBYPASS: 0/not bypassed
        // VOUT_SEL: 0b100: 1.1v
        // LDOEN: 0/Disabled
        let already_enabled = {
            let ldocsr = self.scg0.ldocsr().read();
            ldocsr.ldoen() && ldocsr.vout_ok()
        };
        if !already_enabled {
            self.scg0.ldocsr().modify(|w| w.set_ldoen(true));
            while !self.scg0.ldocsr().read().vout_ok() {}
        }

        Ok(())
    }

    /// Configure the SOSC/clk_in oscillator
    #[cfg(not(feature = "sosc-as-gpio"))]
    pub(super) fn configure_sosc(&mut self) -> Result<(), ClockError> {
        let Some(parts) = self.config.sosc.as_ref() else {
            return Ok(());
        };

        // Enable (and wait for) LDO to be active
        self.ensure_ldo_active("sosc", &parts.power)?;

        let eref = match parts.mode {
            config::SoscMode::CrystalOscillator => Erefs::Internal,
            config::SoscMode::ActiveClock => Erefs::External,
        };
        let freq = parts.frequency;

        // TODO: Fix PAC names here
        //
        // #[doc = "0: Frequency range select of 8-16 MHz."]
        // Freq16to20mhz = 0,
        // #[doc = "1: Frequency range select of 16-25 MHz."]
        // LowFreq = 1,
        // #[doc = "2: Frequency range select of 25-40 MHz."]
        // MediumFreq = 2,
        // #[doc = "3: Frequency range select of 40-50 MHz."]
        // HighFreq = 3,
        let range = match freq {
            0..8_000_000 => {
                return Err(ClockError::BadConfig {
                    clock: "clk_in",
                    reason: "freq too low",
                });
            }
            8_000_000..16_000_000 => Range::Freq16to20mhz,
            16_000_000..25_000_000 => Range::LowFreq,
            25_000_000..40_000_000 => Range::MediumFreq,
            40_000_000..50_000_001 => Range::HighFreq,
            50_000_001.. => {
                return Err(ClockError::BadConfig {
                    clock: "clk_in",
                    reason: "freq too high",
                });
            }
        };

        // Set source/erefs and range
        self.scg0.sosccfg().modify(|w| {
            w.set_erefs(eref);
            w.set_range(range);
        });

        // Disable lock
        self.scg0.sosccsr().modify(|w| w.set_lk(SosccsrLk::WriteEnabled));

        // TODO: We could enable the SOSC clock monitor. There are some things to
        // figure out first:
        //
        // * This requires SIRC to be enabled, not sure which branch. Maybe fro12m_root?
        // * If SOSC needs to work in deep sleep, AND the monitor is enabled:
        //   * SIRC also need needs to be low power
        // * We need to decide if we need an interrupt or a reset if the monitor trips
        let (bg_good, soscsten) = match parts.power {
            PoweredClock::NormalEnabledDeepSleepDisabled => (self.clocks.bandgap_active, false),
            PoweredClock::AlwaysEnabled => (self.clocks.bandgap_active && self.clocks.bandgap_lowpower, true),
        };

        if !bg_good {
            return Err(ClockError::BadConfig {
                clock: "sosc",
                reason: "bandgap required",
            });
        }

        // Apply remaining config
        self.scg0.sosccsr().modify(|w| {
            // For now, just disable the monitor. See above.
            w.set_sosccm(false);

            // Set deep sleep mode if needed
            w.set_soscsten(soscsten);

            // Enable SOSC
            w.set_soscen(true)
        });

        // Wait for SOSC to be valid, check for errors
        while !self.scg0.sosccsr().read().soscvld() {}
        if self.scg0.sosccsr().read().soscerr() == Soscerr::EnabledAndError {
            return Err(ClockError::BadConfig {
                clock: "clk_in",
                reason: "soscerr is set",
            });
        }

        // Re-lock the sosc
        self.scg0.sosccsr().modify(|w| w.set_lk(SosccsrLk::WriteDisabled));

        self.clocks.clk_in = Some(Clock {
            frequency: freq,
            power: parts.power,
        });

        Ok(())
    }

    pub(super) fn configure_spll(&mut self) -> Result<(), ClockError> {
        // # Vocab
        //
        // | Name   | Meaning                                                     |
        // | :---   | :---                                                        |
        // | Fin    | Frequency of clkin                                          |
        // | clkout | Output clock of the PLL                                     |
        // | Fout   | Frequency of clkout (depends on mode)                       |
        // | clkref | PLL Reference clock, the input clock to the PFD             |
        // | Fref   | Frequency of clkref, Fref = Fin / N                         |
        // | Fcco   | Frequency of the output clock of the CCO, Fcco = M * Fref   |
        // | N      | Predivider value                                            |
        // | M      | Feedback divider value                                      |
        // | P      | Postdivider value                                           |
        // | Tpon   | PLL start-up time                                           |

        // No PLL? Nothing to do!
        let Some(cfg) = self.config.spll.as_ref() else {
            return Ok(());
        };

        // Ensure the LDO is active
        self.ensure_ldo_active("spll", &cfg.power)?;

        // match on the source, ensure it is active already
        let res = match cfg.source {
            #[cfg(not(feature = "sosc-as-gpio"))]
            config::SpllSource::Sosc => self
                .clocks
                .clk_in
                .as_ref()
                .map(|c| (c, Source::Sosc))
                .ok_or("sosc not active"),
            config::SpllSource::Firc => self
                .clocks
                .clk_hf_fundamental
                .as_ref()
                .map(|c| (c, Source::Firc))
                .ok_or("firc not active"),
            config::SpllSource::Sirc => self
                .clocks
                .fro_12m
                .as_ref()
                .map(|c| (c, Source::Sirc))
                .ok_or("sirc not active"),
        };
        // This checks if active
        let (clk, variant) = res.map_err(|s| ClockError::BadConfig {
            clock: "spll",
            reason: s,
        })?;
        // This checks the correct power reqs
        if !clk.power.meets_requirement_of(&cfg.power) {
            return Err(ClockError::BadConfig {
                clock: "spll",
                reason: "needs low power source",
            });
        }

        // Bandwidth calc
        //
        // > In normal applications, you must calculate the bandwidth manually by using the feedback divider M (ranging from 1 to (2^16)-1),
        // > Equation 1, and Equation 2. The PLL is automatically stable in such case. In normal applications, SPLLCTRL[BANDDIRECT] must
        // > be 0; in this case, the bandwidth changes as a function of M.
        if clk.frequency == 0 {
            return Err(ClockError::BadConfig {
                clock: "spll",
                reason: "internal error",
            });
        }

        // These are calculated differently depending on the mode.
        let f_in = clk.frequency;
        let bp_pre: bool;
        let bp_post: bool;
        let bp_post2: bool;
        let m: u16;
        let p: Option<u8>;
        let n: Option<u8>;

        // Calculate both Fout and Fcco so we can ensure they don't overflow
        // and are in range
        let fout: Option<u32>;
        let fcco: Option<u32>;

        let m_check = |m: u16| {
            if !(1..=u16::MAX).contains(&m) {
                Err(ClockError::BadConfig {
                    clock: "spll",
                    reason: "m_mult out of range",
                })
            } else {
                Ok(m)
            }
        };
        let p_check = |p: u8| {
            if !(1..=31).contains(&p) {
                Err(ClockError::BadConfig {
                    clock: "spll",
                    reason: "p_div out of range",
                })
            } else {
                Ok(p)
            }
        };
        let n_check = |n: u8| {
            if !(1..=u8::MAX).contains(&n) {
                Err(ClockError::BadConfig {
                    clock: "spll",
                    reason: "n_div out of range",
                })
            } else {
                Ok(n)
            }
        };

        match cfg.mode {
            // Fout = M x Fin
            config::SpllMode::Mode1a { m_mult } => {
                bp_pre = true;
                bp_post = true;
                bp_post2 = false;
                m = m_check(m_mult)?;
                p = None;
                n = None;
                fcco = f_in.checked_mul(m_mult as u32);
                fout = fcco;
            }
            // if !bypass_p2_div: Fout = (M / (2 x P)) x Fin
            // if  bypass_p2_div: Fout = (M /    P   ) x Fin
            config::SpllMode::Mode1b {
                m_mult,
                p_div,
                bypass_p2_div,
            } => {
                bp_pre = true;
                bp_post = false;
                bp_post2 = bypass_p2_div;
                m = m_check(m_mult)?;
                p = Some(p_check(p_div)?);
                n = None;
                let mut div = p_div as u32;
                if !bypass_p2_div {
                    div *= 2;
                }
                fcco = f_in.checked_mul(m_mult as u32);
                fout = (f_in / div).checked_mul(m_mult as u32);
            }
            // Fout = (M / N) x Fin
            config::SpllMode::Mode1c { m_mult, n_div } => {
                bp_pre = false;
                bp_post = true;
                bp_post2 = false;
                m = m_check(m_mult)?;
                p = None;
                n = Some(n_check(n_div)?);
                fcco = (f_in / (n_div as u32)).checked_mul(m_mult as u32);
                fout = fcco;
            }
            // if !bypass_p2_div: Fout = (M / (N x 2 x P)) x Fin
            // if  bypass_p2_div: Fout = (M / (  N x P  )) x Fin
            config::SpllMode::Mode1d {
                m_mult,
                n_div,
                p_div,
                bypass_p2_div,
            } => {
                bp_pre = false;
                bp_post = false;
                bp_post2 = bypass_p2_div;
                m = m_check(m_mult)?;
                p = Some(p_check(p_div)?);
                n = Some(n_check(n_div)?);
                // This can't overflow: u8 x u8 (x 2) always fits in u32
                let mut div = (p_div as u32) * (n_div as u32);
                if !bypass_p2_div {
                    div *= 2;
                }
                fcco = (f_in / (n_div as u32)).checked_mul(m_mult as u32);
                fout = (f_in / div).checked_mul(m_mult as u32);
            }
        };

        // Dump all the PLL calcs if needed for debugging
        #[cfg(feature = "defmt")]
        {
            defmt::debug!("f_in: {:?}", f_in);
            defmt::debug!("bp_pre: {:?}", bp_pre);
            defmt::debug!("bp_post: {:?}", bp_post);
            defmt::debug!("bp_post2: {:?}", bp_post2);
            defmt::debug!("m: {:?}", m);
            defmt::debug!("p: {:?}", p);
            defmt::debug!("n: {:?}", n);
            defmt::debug!("fout: {:?}", fout);
            defmt::debug!("fcco: {:?}", fcco);
        }

        // Ensure the Fcco and Fout calcs didn't overflow
        let fcco = fcco.ok_or(ClockError::BadConfig {
            clock: "spll",
            reason: "fcco invalid1",
        })?;
        let fout = fout.ok_or(ClockError::BadConfig {
            clock: "spll",
            reason: "fout invalid",
        })?;

        // Fcco: 275MHz to 550MHz
        if !(275_000_000..=550_000_000).contains(&fcco) {
            return Err(ClockError::BadConfig {
                clock: "spll",
                reason: "fcco invalid2",
            });
        }

        let limits = self.lowest_relevant_limits(&cfg.power);

        // Fout: 4.3MHz to 2x Max CPU Frequency
        let fmax = limits.cpu_clk;
        let spll_range_bad1 = !(4_300_000..=(2 * fmax)).contains(&fout);
        let spll_range_bad2 = fout > limits.pll1_clk;

        if spll_range_bad1 || spll_range_bad2 {
            return Err(ClockError::BadConfig {
                clock: "spll",
                reason: "fout invalid",
            });
        }

        // A = floor(m / 4) + 1
        let selp_a = (m / 4) + 1;
        // SELP = A  if A <  31
        //      = 31 if A >= 31
        let selp = selp_a.min(31);

        // A = 1                    if        M >= 8000
        //   = floor(8000 / M)      if 8000 > M >= 122
        //   = 2 x floor(M / 4) / 3 if 122  > M >= 1
        let seli_a = if m >= 8000 {
            1
        } else if m >= 122 {
            8000 / m
        } else {
            (2 * (m / 4)) / 3
        };
        // SELI = A  if A <  63
        //      = 63 if A >= 63
        let seli = seli_a.min(63);
        // SELR must be 0.
        let selr = 0;

        self.scg0.spllctrl().modify(|w| {
            w.set_source(variant);
            w.set_selp(selp as u8);
            w.set_seli(seli as u8);
            w.set_selr(selr);
        });

        if let Some(n) = n {
            self.scg0.spllndiv().modify(|w| w.set_ndiv(n));
        }
        if let Some(p) = p {
            self.scg0.spllpdiv().modify(|w| w.set_pdiv(p));
        }
        self.scg0.spllmdiv().modify(|w| w.set_mdiv(m));

        self.scg0.spllctrl().modify(|w| {
            w.set_bypassprediv(bp_pre);
            w.set_bypasspostdiv(bp_post);
            w.set_bypasspostdiv2(bp_post2);

            // TODO: support FRM?
            w.set_frm(false);
        });

        // Unlock
        self.scg0.spllcsr().modify(|w| w.set_lk(SpllcsrLk::WriteEnabled));

        // TODO: Support clock monitors?
        // self.scg0.spllcsr().modify(|w| w.spllcm().?);

        self.scg0.trim_lock().write(|w| {
            w.set_trim_lock_key(0x5a5a);
            w.set_trim_unlock(TrimUnlock::NotLocked)
        });

        // SPLLLOCK_CNFG: The lock time programmed in this register must be
        // equal to meet the PLL 500μs lock time plus the 300 refclk count startup.
        //
        // LOCK_TIME = 500μs/T ref + 300, F ref = F in /N (input frequency divided by pre-divider ratio).
        //
        // 500us is 1/2000th of a second, therefore Fref / 2000 is the number of cycles in 500us.
        let f_ref = if let Some(n) = n { f_in / (n as u32) } else { f_in };
        let lock_time = f_ref.div_ceil(2000) + 300;
        self.scg0.splllock_cnfg().write(|w| w.set_lock_time(lock_time));

        // TODO: Support Spread spectrum?

        let (bg_good, spllsten) = match cfg.power {
            PoweredClock::NormalEnabledDeepSleepDisabled => (self.clocks.bandgap_active, Spllsten::DisabledInStop),
            PoweredClock::AlwaysEnabled => (
                self.clocks.bandgap_active && self.clocks.bandgap_lowpower,
                Spllsten::EnabledInStop,
            ),
        };
        if !bg_good {
            return Err(ClockError::BadConfig {
                clock: "spll",
                reason: "bandgap required when active",
            });
        }

        self.scg0.spllcsr().modify(|w| {
            w.set_spllclken(true);
            w.set_spllpwren(true);
            w.set_spllsten(spllsten);
        });

        // Wait for SPLL to set up
        loop {
            let csr = self.scg0.spllcsr().read();
            if csr.spll_lock() == SpllLock::EnabledAndValid {
                if csr.spllerr() == Spllerr::EnabledAndError {
                    return Err(ClockError::BadConfig {
                        clock: "spll",
                        reason: "spllerr is set",
                    });
                }
                break;
            }
        }

        // Re-lock SPLL CSR
        self.scg0.spllcsr().modify(|w| w.set_lk(SpllcsrLk::WriteDisabled));

        // Store clock state
        self.clocks.pll1_clk = Some(Clock {
            frequency: fout,
            power: cfg.power,
        });

        // Do we enable the `pll1_clk_div` output?
        if let Some(d) = cfg.pll1_clk_div.as_ref() {
            let exp_freq = fout / d.into_divisor();
            if exp_freq > limits.pll1_clk_div {
                return Err(ClockError::BadConfig {
                    clock: "pll1_clk_div",
                    reason: "exceeds max frequency",
                });
            }

            // Halt and reset the div; then set our desired div.
            self.syscon.pll1clkdiv().write(|w| {
                w.set_halt(Pll1clkdivHalt::Halt);
                w.set_reset(Pll1clkdivReset::Asserted);
                w.set_div(d.into_bits());
            });
            // Then unhalt it, and reset it
            self.syscon.pll1clkdiv().write(|w| {
                w.set_halt(Pll1clkdivHalt::Run);
                w.set_reset(Pll1clkdivReset::Released);
            });

            // Wait for clock to stabilize
            while self.syscon.pll1clkdiv().read().unstab() == Pll1clkdivUnstab::Ongoing {}

            // Store off the clock info
            self.clocks.pll1_clk_div = Some(Clock {
                frequency: exp_freq,
                power: cfg.power,
            });
        }

        Ok(())
    }

    pub(super) fn configure_main_clk(&mut self) -> Result<(), ClockError> {
        let (var, name, clk) = match self.config.main_clock.source {
            #[cfg(not(feature = "sosc-as-gpio"))]
            MainClockSource::SoscClkIn => (Scs::Sosc, "clk_in", self.clocks.clk_in.as_ref()),
            MainClockSource::SircFro12M => (Scs::Sirc, "fro_12m", self.clocks.fro_12m.as_ref()),
            MainClockSource::FircHfRoot => (Scs::Firc, "fro_hf_root", self.clocks.fro_hf_root.as_ref()),
            #[cfg(feature = "mcxa2xx")]
            MainClockSource::RoscFro16K => (Scs::Rosc, "fro16k", self.clocks.clk_16k_vdd_core.as_ref()),
            #[cfg(all(feature = "mcxa5xx", not(feature = "rosc-32k-as-gpio")))]
            MainClockSource::RoscOsc32K => (Scs::Rosc, "osc32k", self.clocks.clk_32k_vdd_core.as_ref()),
            MainClockSource::SPll1 => (Scs::Spll, "pll1_clk", self.clocks.pll1_clk.as_ref()),
        };
        let Some(main_clk_src) = clk else {
            return Err(ClockError::BadConfig {
                clock: name,
                reason: "Needed for main_clock but not enabled",
            });
        };

        if !main_clk_src.power.meets_requirement_of(&self.config.main_clock.power) {
            return Err(ClockError::BadConfig {
                clock: name,
                reason: "Needed for main_clock but not low power",
            });
        }

        let lowest_limits = self.lowest_relevant_limits(&self.config.main_clock.power);
        let active_limits = self.active_limits();

        let (levels, wsmax) = match self.config.vdd_power.active_mode.level {
            VddLevel::MidDriveMode => (
                clock_limits::VDD_CORE_MID_DRIVE_WAIT_STATE_LIMITS,
                clock_limits::VDD_CORE_MID_DRIVE_MAX_WAIT_STATES,
            ),
            #[cfg(feature = "mcxa5xx")]
            VddLevel::NormalMode => (
                clock_limits::VDD_CORE_NORMAL_DRIVE_WAIT_STATE_LIMITS,
                clock_limits::VDD_CORE_NORMAL_DRIVE_MAX_WAIT_STATES,
            ),
            VddLevel::OverDriveMode => (
                clock_limits::VDD_CORE_OVER_DRIVE_WAIT_STATE_LIMITS,
                clock_limits::VDD_CORE_OVER_DRIVE_MAX_WAIT_STATES,
            ),
        };

        // Is the main_clk source in range for main_clk?
        if main_clk_src.frequency > lowest_limits.main_clk {
            return Err(ClockError::BadConfig {
                clock: name,
                reason: "Exceeds main_clock frequency",
            });
        }

        // Calculate expected CPU frequency based on main_clk and AHB div
        let ahb_div = self.config.main_clock.ahb_clk_div;
        let cpu_freq = main_clk_src.frequency / ahb_div.into_divisor();

        // Is the expected CPU frequency in range for cpu_clk? Note: the CPU
        // is never running in deep sleep, so we directly use the active limits here
        if cpu_freq > active_limits.cpu_clk {
            return Err(ClockError::BadConfig {
                clock: name,
                reason: "Exceeds ahb max frequency",
            });
        }

        // BEFORE we switch, update the flash wait states to the appropriate levels
        //
        // NOTE: "cpu_clk" is the same as "system_clk". Table 22 is not clear exactly
        // WHICH source clock the limits apply to, but system/ahb/cpu is a fair bet.
        //
        // TODO: This calculation doesn't consider low power mode yet!
        let wait_states = levels
            .iter()
            .find(|(fmax, _ws)| cpu_freq <= *fmax)
            .map(|t| t.1)
            .unwrap_or(wsmax);
        self.fmu0.fctrl().modify(|w| w.set_rwsc(wait_states));

        // TODO: (Double) check if clock is actually valid before switching?
        // Are we already on the right clock?
        let now = self.scg0.csr().read().scs();
        if now != var {
            // Set RCCR
            self.scg0.rccr().modify(|w| w.set_scs(var));

            // Wait for match
            while self.scg0.csr().read().scs() != var {}
        }

        // The main_clk is now set to the selected input clock
        self.clocks.main_clk = Some(main_clk_src.clone());

        // Update AHB clock division, if necessary
        if ahb_div.into_bits() != 0 {
            // AHB has no halt/reset fields - it's different to other DIV8s!
            self.syscon.ahbclkdiv().modify(|w| w.set_div(ahb_div.into_bits()));
            // Wait for clock to stabilize
            while self.syscon.ahbclkdiv().read().unstab() == AhbclkdivUnstab::Ongoing {}
        }

        // Store off the clock info
        self.clocks.cpu_system_clk = Some(Clock {
            frequency: cpu_freq,
            power: main_clk_src.power,
        });

        Ok(())
    }

    pub(super) fn configure_voltages(&mut self) -> Result<(), ClockError> {
        // Determine if we need to change the active mode voltage levels
        let to_change = match self.config.vdd_power.active_mode.level {
            VddLevel::MidDriveMode => {
                // This is the default mode, I don't believe we need to do anything.
                //
                // "The LVDE and HVDE fields reset only with a POR.
                // All other fields reset only with a system reset."
                None
            }
            #[cfg(feature = "mcxa5xx")]
            VddLevel::NormalMode => Some((ActiveCfgCoreldoVddLvl::Normal, Vsm::Sram1v1)),
            VddLevel::OverDriveMode => Some((ActiveCfgCoreldoVddLvl::Over, Vsm::Sram1v2)),
        };

        if let Some((vdd, vsm)) = to_change {
            // You can change the core VDD levels for the LDO_CORE low power regulator only
            // when CORELDO_VDD_DS=1.
            //
            // When switching CORELDO_VDD_DS from low to normal drive strength, ensure the LDO_CORE high
            // VDD LVL setting is set to the same level that was set prior to switching to the LDO_CORE drive strength
            // (CORELDO_VDD_DS). Otherwise, if the LVDs are enabled, an unexpected LVD can occur.
            //
            // Ensure drive strength is normal (BEFORE shifting level)
            self.spc0
                .active_cfg()
                .modify(|w| w.set_coreldo_vdd_ds(ActiveCfgCoreldoVddDs::Normal));

            // ## DS 26.3.2:
            //
            // When increasing voltage and frequency in Active mode, you must perform the following steps:
            //
            // 1. Increase voltage to a new level (ACTIVE_CFG[CORELDO_VDD_LVL]).
            self.spc0.active_cfg().modify(|w| w.set_coreldo_vdd_lvl(vdd));

            // 2. Wait for voltage change to complete (SC[BUSY] = 0).
            while self.spc0.sc().read().busy() {}

            // 3. Configure flash memory to support higher voltage level and frequency (FMU_FCTRL[RWSC].
            //
            // NOTE: This step skipped - we will update RWSC when we later apply main cpu clock
            // frequency changes.

            // 4. Configure SRAM to support higher voltage levels (SRAMCTL[VSM]).
            self.spc0.sramctl().modify(|w| w.set_vsm(vsm));

            // 5. Request SRAM voltage update (write 1 to SRAMCTL[REQ]).
            self.spc0.sramctl().modify(|w| w.set_req(true));

            // 6. Wait for SRAM voltage change to complete (SRAMCTL[ACK] = 1).
            while !self.spc0.sramctl().read().ack() {}

            // 7. Clear request for SRAM voltage change (write 0 to SRAMCTL[REQ]).
            self.spc0.sramctl().modify(|w| w.set_req(false));

            // 8. Increase frequency to a new level (for example, SCG_RCCR).
            //
            // NOTE: This step skipped - we will update RCCR when we later apply main cpu clock
            // frequency changes.

            // 9. You can continue execution.
            // :)
        }

        // If the CORELDO_VDD_DS fields are set to the same value in both the ACTIVE_CFG and LP_CFG registers,
        // the CORELDO_VDD_LVL's in the ACTIVE_CFG and LP_CFG register must be set to the same voltage
        // level settings.
        //
        // TODO(AJM): I don't really understand this! Enforce it literally for now I guess.
        const BAD_ASCENDING: Result<(), ClockError> = Err(ClockError::BadConfig {
            clock: "vdd_power",
            reason: "Deep sleep can't have higher level than active mode",
        });
        let ds_match = self.config.vdd_power.active_mode.drive == self.config.vdd_power.low_power_mode.drive;
        let (vdd_match, lpwkup) = match (
            self.config.vdd_power.active_mode.level,
            self.config.vdd_power.low_power_mode.level,
        ) {
            //
            // Correct "descending" options
            //
            // When voltage levels are not the same between ACTIVE mode and Low Power mode, you must write a
            // nonzero value to SPC->LPWKUP_DELAY.
            //
            // This SHOULD be covered by table 165. LPWKUP Delay, but it doesn't actually have
            // a value for the 1.0v-1.2v transition we need. For now, the C SDK always uses 0x5B.
            #[cfg(feature = "mcxa5xx")]
            (VddLevel::OverDriveMode, VddLevel::NormalMode) => (false, 0x005b),
            (VddLevel::OverDriveMode, VddLevel::MidDriveMode) => (false, 0x005b),
            #[cfg(feature = "mcxa5xx")]
            (VddLevel::NormalMode, VddLevel::MidDriveMode) => (false, 0x005b),

            //
            // Incorrect "ascending" options
            //
            // For now, enforce that active is always >= voltage to low power. I don't know if this
            // is required, but there's probably also no reason to support it?
            #[cfg(feature = "mcxa5xx")]
            (VddLevel::MidDriveMode, VddLevel::NormalMode) => return BAD_ASCENDING,
            (VddLevel::MidDriveMode, VddLevel::OverDriveMode) => return BAD_ASCENDING,
            #[cfg(feature = "mcxa5xx")]
            (VddLevel::NormalMode, VddLevel::OverDriveMode) => return BAD_ASCENDING,

            // Correct "matching" options
            (VddLevel::MidDriveMode, VddLevel::MidDriveMode) => (true, 0x0000),
            #[cfg(feature = "mcxa5xx")]
            (VddLevel::NormalMode, VddLevel::NormalMode) => (true, 0x0000),
            (VddLevel::OverDriveMode, VddLevel::OverDriveMode) => (true, 0x0000),
        };
        self.spc0.lpwkup_delay().write(|w| w.set_lpwkup_delay(lpwkup));

        if ds_match && !vdd_match {
            return Err(ClockError::BadConfig {
                clock: "vdd_power",
                reason: "DS matches but LVL mismatches!",
            });
        }

        // You can change the core VDD levels for the LDO_CORE low power regulator only when
        // ACTIVE_CFG[CORELDO_VDD_DS] = 1. So, before entering any of the low-power states (DSLEEP,
        // PDOWN, DPDOWN) with LDO_CORE low power regulator selected (LP_CFG[CORELDO_VDD_DS] = 0),
        // you must use CORELDO_VDD_LVL to select the correct regulation level during ACTIVE run mode.
        //
        // NOTE(AJM): We've set drive strength to "normal" above, and do not (potentially) set it to
        // "low" until later below.

        // NOTE(AJM): The reference manual doesn't have any similar configuration requirements
        // for low power mode. We'll just configure it, I guess?
        //
        // NOTE(AJM): "LP_CFG: This register resets only after a POR or LVD event."
        let (ds, bgap) = match self.config.vdd_power.low_power_mode.drive {
            VddDriveStrength::Low { enable_bandgap } => {
                // If the bandgap is enabled, also enable the high/low voltage
                // detectors. if it is disabled, these must also be disabled.
                self.spc0.lp_cfg().modify(|w| {
                    w.set_sys_hvde(enable_bandgap);
                    w.set_sys_lvde(enable_bandgap);
                    w.set_core_lvde(enable_bandgap);
                });

                (pac::spc::LpCfgCoreldoVddDs::Low, enable_bandgap)
            }
            VddDriveStrength::Normal => {
                // "If you specify normal drive strength, you must write a value to LP[BGMODE] that enables the bandgap."
                (pac::spc::LpCfgCoreldoVddDs::Normal, true)
            }
        };
        let lvl = match self.config.vdd_power.low_power_mode.level {
            VddLevel::MidDriveMode => LpCfgCoreldoVddLvl::Mid,
            #[cfg(feature = "mcxa5xx")]
            VddLevel::NormalMode => LpCfgCoreldoVddLvl::Normal,
            VddLevel::OverDriveMode => LpCfgCoreldoVddLvl::Over,
        };
        self.spc0.lp_cfg().modify(|w| w.set_coreldo_vdd_ds(ds));

        // If we're enabling the bandgap, ensure we do it BEFORE changing the VDD level
        // If we're disabling the bandgap, ensure we do it AFTER changing the VDD level
        if bgap {
            self.spc0.lp_cfg().modify(|w| w.set_bgmode(LpCfgBgmode::Bgmode01));
            self.spc0.lp_cfg().modify(|w| w.set_coreldo_vdd_lvl(lvl));
        } else {
            self.spc0.lp_cfg().modify(|w| w.set_coreldo_vdd_lvl(lvl));
            self.spc0.lp_cfg().modify(|w| w.set_bgmode(LpCfgBgmode::Bgmode0));
        }
        self.clocks.bandgap_lowpower = bgap;

        // Updating CORELDO_VDD_LVL sets the SC[BUSY] flag. That flag remains set for at least the total time
        // delay that Active Voltage Trim Delay (ACTIVE_VDELAY) specifies.
        //
        // Before changing CORELDO_VDD_LVL, you must wait until the SC[BUSY] flag clears before entering the
        // selected low-power sleep
        //
        // NOTE(AJM): Let's just proactively wait now so we don't have to worry about it on subsequent sleeps
        while self.spc0.sc().read().busy() {}

        // NOTE(AJM): I don't really know if this is valid! I'm guessing in most cases you would want to
        // use the low drive strength for lp mode, and high drive strength for active mode?
        match self.config.vdd_power.active_mode.drive {
            VddDriveStrength::Low { enable_bandgap } => {
                // If the bandgap is enabled, also enable the high/low voltage
                // detectors. if it is disabled, these must also be disabled.
                self.spc0.active_cfg().modify(|w| {
                    w.set_sys_hvde(enable_bandgap);
                    w.set_sys_lvde(enable_bandgap);
                    w.set_core_lvde(enable_bandgap);
                });

                // optionally disable bandgap AFTER setting vdd strength to low
                self.spc0
                    .active_cfg()
                    .modify(|w| w.set_coreldo_vdd_ds(ActiveCfgCoreldoVddDs::Low));
                self.spc0.active_cfg().modify(|w| {
                    if enable_bandgap {
                        w.set_bgmode(ActiveCfgBgmode::Bgmode01)
                    } else {
                        w.set_bgmode(ActiveCfgBgmode::Bgmode0)
                    }
                });

                self.clocks.bandgap_active = enable_bandgap;
            }
            VddDriveStrength::Normal => {
                // Already set to normal above
                self.clocks.bandgap_active = true;
            }
        }

        // NOTE: calling `cortex_m::Peripherals::steal()` still marks the core peripherals as taken. See
        // https://github.com/embassy-rs/embassy/issues/5563 for discussion. Since this
        // is a ZST, transmuting from `()` is reasonable.
        let mut scb: SCB = unsafe { core::mem::transmute(()) };

        // Apply sleep settings
        match self.config.vdd_power.core_sleep {
            CoreSleep::WfeUngated => {
                // Do not gate
                self.cmc.ckctrl().modify(|w| w.set_ckmode(Ckmode::Ckmode0000));

                // Debug is enabled when core sleeps
                self.cmc.dbgctl().modify(|w| w.set_sod(false));

                // Don't allow the core to be gated to avoid killing the debugging session
                scb.clear_sleepdeep();
            }
            CoreSleep::WfeGated => {
                // Allow automatic gating of the core when in LIGHT sleep
                self.cmc.ckctrl().modify(|w| w.set_ckmode(Ckmode::Ckmode0001));

                // Debug is disabled when core sleeps
                self.cmc.dbgctl().modify(|w| w.set_sod(true));

                // Allow the core to be gated - this WILL kill the debugging session!
                scb.set_sleepdeep();
            }
            CoreSleep::DeepSleep => {
                // We can only support deep sleep with a custom executor which properly
                // handles going to sleep and returning
                #[cfg(all(not(feature = "executor-platform"), feature = "defmt"))]
                defmt::warn!("deep sleep enabled without custom executor");

                // For now, just enable light sleep. The executor will set deep sleep when
                // appropriate
                self.cmc.ckctrl().modify(|w| w.set_ckmode(Ckmode::Ckmode0001));

                // Debug is disabled when core sleeps
                self.cmc.dbgctl().modify(|w| w.set_sod(true));

                // Allow the core to be gated - this WILL kill the debugging session!
                scb.set_sleepdeep();

                // Enable sevonpend, to allow us to wake from WFE sleep with interrupts disabled
                unsafe {
                    // TODO: wait for https://github.com/rust-embedded/cortex-m/commit/1be630fdd06990bd14251eabe4cca9307bde549d
                    // to be released, until then, manual version of SCB.set_sevonpend();
                    scb.scr.modify(|w| w | (1 << 4));
                }
            }
        }
        self.clocks.core_sleep = self.config.vdd_power.core_sleep;

        // Allow automatic gating of the flash memory
        let (wake, doze) = match self.config.vdd_power.flash_sleep {
            config::FlashSleep::Never => (false, false),
            config::FlashSleep::FlashDoze => (false, true),
            config::FlashSleep::FlashDozeWithFlashWake => (true, true),
        };

        self.cmc.flashcr().modify(|w| {
            w.set_flashdoze(doze);
            w.set_flashwake(wake);
        });

        // At init, disable all analog peripherals. These can be re-enabled
        // if necessary for HAL drivers.
        self.spc0.active_cfg1().write(|w| w.0 = 0);
        self.spc0.lp_cfg1().write(|w| w.0 = 0);

        // Update status
        self.clocks.active_power = self.config.vdd_power.active_mode.level;
        self.clocks.lp_power = self.config.vdd_power.low_power_mode.level;

        Ok(())
    }
}
