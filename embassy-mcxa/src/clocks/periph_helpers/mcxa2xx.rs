use nxp_pac::mrcc::vals::{AdcClkselMux, ClkdivHalt, ClkdivReset, ClkdivUnstab, CtimerClkselMux, FclkClkselMux, Lpi2cClkselMux, LpspiClkselMux, LpuartClkselMux, OstimerClkselMux};

use crate::clocks::config::VddLevel;
use crate::clocks::{ClockError, Clocks, PoweredClock, WakeGuard};
use crate::{apply_div4, pac};
use super::{Div4, PreEnableParts, SPConfHelper};

//
// I3C
//


/// Selectable clocks for `I3c` peripherals
#[derive(Debug, Clone, Copy)]
pub enum I3cClockSel {
    /// FRO12M/FRO_LF/SIRC clock source, passed through divider
    /// "fro_lf_div"
    FroLfDiv,
    /// FRO180M/FRO_HF/FIRC clock source, passed through divider
    /// "fro_hf_div"
    FroHfDiv,
    /// SOSC/XTAL/EXTAL clock source
    #[cfg(not(feature = "sosc-as-gpio"))]
    ClkIn,
    /// clk_1m/FRO_LF divided by 12
    Clk1M,
    /// Disabled
    None,
}

/// Top level configuration for `I3c` instances.
pub struct I3cConfig {
    /// Power state required for this peripheral
    pub power: PoweredClock,
    /// Clock source
    pub source: I3cClockSel,
    /// Clock divisor
    pub div: Div4,
}

impl SPConfHelper for I3cConfig {
    fn pre_enable_config(&self, clocks: &Clocks) -> Result<PreEnableParts, ClockError> {
        // Always 25MHz maximum frequency.
        const I3C_FCLK_MAX: u32 = 25_000_000;
        // check that source is suitable
        let mrcc0 = pac::MRCC0;

        let (clkdiv, clksel) = (mrcc0.mrcc_i3c0_fclk_clkdiv(), mrcc0.mrcc_i3c0_fclk_clksel());

        let (freq, variant) = match self.source {
            I3cClockSel::FroLfDiv => {
                let freq = clocks.ensure_fro_lf_div_active(&self.power)?;
                (freq, FclkClkselMux::CLKROOT_FUNC_0)
            }
            I3cClockSel::FroHfDiv => {
                let freq = clocks.ensure_fro_hf_div_active(&self.power)?;
                (freq, FclkClkselMux::CLKROOT_FUNC_2)
            }
            #[cfg(not(feature = "sosc-as-gpio"))]
            I3cClockSel::ClkIn => {
                let freq = clocks.ensure_clk_in_active(&self.power)?;
                (freq, FclkClkselMux::CLKROOT_FUNC_3)
            }
            I3cClockSel::Clk1M => {
                let freq = clocks.ensure_clk_1m_active(&self.power)?;
                (freq, FclkClkselMux::CLKROOT_FUNC_5)
            }
            I3cClockSel::None => {
                // no ClkrootFunc7, just write manually for now
                clksel.write(|w| w.0 = 0b111);
                clkdiv.modify(|w| {
                    w.set_reset(ClkdivReset::OFF);
                    w.set_halt(ClkdivHalt::OFF);
                });
                return Ok(PreEnableParts::empty());
            }
        };

        if freq > I3C_FCLK_MAX {
            return Err(ClockError::BadConfig {
                clock: "i3c fclk",
                reason: "exceeds max rating",
            });
        }

        apply_div4!(self, clksel, clkdiv, variant, freq)
    }
}

//
// LPSPI
//

/// Selectable clocks for `Lpspi` peripherals
#[derive(Debug, Clone, Copy)]
pub enum LpspiClockSel {
    /// FRO12M/FRO_LF/SIRC clock source, passed through divider
    /// "fro_lf_div"
    FroLfDiv,
    /// FRO180M/FRO_HF/FIRC clock source, passed through divider
    /// "fro_hf_div"
    FroHfDiv,
    /// SOSC/XTAL/EXTAL clock source
    ClkIn,
    /// clk_1m/FRO_LF divided by 12
    Clk1M,
    /// Output of PLL1, passed through clock divider,
    /// "pll1_clk_div", maybe "pll1_lf_div"?
    Pll1ClkDiv,
    /// Disabled
    None,
}

/// Which instance of the `Lpspi` is this?
///
/// Should not be directly selectable by end-users.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LpspiInstance {
    /// Instance 0
    Lpspi0,
    /// Instance 1
    Lpspi1,
}

/// Top level configuration for `Lpspi` instances.
pub struct LpspiConfig {
    /// Power state required for this peripheral
    pub power: PoweredClock,
    /// Clock source
    pub source: LpspiClockSel,
    /// Clock divisor
    pub div: Div4,
    /// Which instance is this?
    // NOTE: should not be user settable
    pub(crate) instance: LpspiInstance,
}

impl SPConfHelper for LpspiConfig {
    fn pre_enable_config(&self, clocks: &Clocks) -> Result<PreEnableParts, ClockError> {
        // check that source is suitable
        let mrcc0 = pac::MRCC0;

        let (clkdiv, clksel) = match self.instance {
            LpspiInstance::Lpspi0 => (mrcc0.mrcc_lpspi0_clkdiv(), mrcc0.mrcc_lpspi0_clksel()),
            LpspiInstance::Lpspi1 => (mrcc0.mrcc_lpspi1_clkdiv(), mrcc0.mrcc_lpspi1_clksel()),
        };

        let (freq, variant) = match self.source {
            LpspiClockSel::FroLfDiv => {
                let freq = clocks.ensure_fro_lf_div_active(&self.power)?;
                (freq, LpspiClkselMux::CLKROOT_FUNC_0)
            }
            LpspiClockSel::FroHfDiv => {
                let freq = clocks.ensure_fro_hf_div_active(&self.power)?;
                (freq, LpspiClkselMux::CLKROOT_FUNC_2)
            }
            LpspiClockSel::ClkIn => {
                let freq = clocks.ensure_clk_in_active(&self.power)?;
                (freq, LpspiClkselMux::CLKROOT_FUNC_3)
            }
            LpspiClockSel::Clk1M => {
                let freq = clocks.ensure_clk_1m_active(&self.power)?;
                (freq, LpspiClkselMux::CLKROOT_FUNC_5)
            }
            LpspiClockSel::Pll1ClkDiv => {
                let freq = clocks.ensure_pll1_clk_div_active(&self.power)?;
                (freq, LpspiClkselMux::CLKROOT_FUNC_6)
            }
            LpspiClockSel::None => {
                // no ClkrootFunc7, just write manually for now
                clksel.write(|w| w.0 = 0b111);
                clkdiv.modify(|w| {
                    w.set_reset(ClkdivReset::OFF);
                    w.set_halt(ClkdivHalt::OFF);
                });
                return Ok(PreEnableParts::empty());
            }
        };

        let div = self.div.into_divisor();
        let expected = freq / div;
        // 21.3.2 peripheral clock max functional clock limits
        let power = match self.power {
            PoweredClock::NormalEnabledDeepSleepDisabled => clocks.active_power,
            PoweredClock::AlwaysEnabled => clocks.lp_power,
        };
        let fmax = match power {
            VddLevel::MidDriveMode => 50_000_000,
            VddLevel::OverDriveMode => 100_000_000,
        };
        if expected > fmax {
            return Err(ClockError::BadConfig {
                clock: "lpspi fclk",
                reason: "exceeds max rating",
            });
        }

        apply_div4!(self, clksel, clkdiv, variant, freq)
    }
}

//
// LPI2c
//

/// Selectable clocks for `Lpi2c` peripherals
#[derive(Debug, Clone, Copy)]
pub enum Lpi2cClockSel {
    /// FRO12M/FRO_LF/SIRC clock source, passed through divider
    /// "fro_lf_div"
    FroLfDiv,
    /// FRO180M/FRO_HF/FIRC clock source, passed through divider
    /// "fro_hf_div"
    FroHfDiv,
    /// SOSC/XTAL/EXTAL clock source
    #[cfg(not(feature = "sosc-as-gpio"))]
    ClkIn,
    /// clk_1m/FRO_LF divided by 12
    Clk1M,
    /// Output of PLL1, passed through clock divider,
    /// "pll1_clk_div", maybe "pll1_lf_div"?
    Pll1ClkDiv,
    /// Disabled
    None,
}

/// Which instance of the `Lpi2c` is this?
///
/// Should not be directly selectable by end-users.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Lpi2cInstance {
    /// Instance 0
    Lpi2c0,
    /// Instance 1
    Lpi2c1,
    /// Instance 2
    Lpi2c2,
    /// Instance 3
    Lpi2c3,
}

/// Top level configuration for `Lpi2c` instances.
pub struct Lpi2cConfig {
    /// Power state required for this peripheral
    pub power: PoweredClock,
    /// Clock source
    pub source: Lpi2cClockSel,
    /// Clock divisor
    pub div: Div4,
    /// Which instance is this?
    // NOTE: should not be user settable
    pub(crate) instance: Lpi2cInstance,
}

impl SPConfHelper for Lpi2cConfig {
    fn pre_enable_config(&self, clocks: &Clocks) -> Result<PreEnableParts, ClockError> {
        // check that source is suitable
        let mrcc0 = pac::MRCC0;

        let (clkdiv, clksel) = match self.instance {
            Lpi2cInstance::Lpi2c0 => (mrcc0.mrcc_lpi2c0_clkdiv(), mrcc0.mrcc_lpi2c0_clksel()),
            Lpi2cInstance::Lpi2c1 => (mrcc0.mrcc_lpi2c1_clkdiv(), mrcc0.mrcc_lpi2c1_clksel()),
            Lpi2cInstance::Lpi2c2 => (mrcc0.mrcc_lpi2c2_clkdiv(), mrcc0.mrcc_lpi2c2_clksel()),
            Lpi2cInstance::Lpi2c3 => (mrcc0.mrcc_lpi2c3_clkdiv(), mrcc0.mrcc_lpi2c3_clksel()),
        };

        let (freq, variant) = match self.source {
            Lpi2cClockSel::FroLfDiv => {
                let freq = clocks.ensure_fro_lf_div_active(&self.power)?;
                (freq, Lpi2cClkselMux::CLKROOT_FUNC_0)
            }
            Lpi2cClockSel::FroHfDiv => {
                let freq = clocks.ensure_fro_hf_div_active(&self.power)?;
                (freq, Lpi2cClkselMux::CLKROOT_FUNC_2)
            }
            #[cfg(not(feature = "sosc-as-gpio"))]
            Lpi2cClockSel::ClkIn => {
                let freq = clocks.ensure_clk_in_active(&self.power)?;
                (freq, Lpi2cClkselMux::CLKROOT_FUNC_3)
            }
            Lpi2cClockSel::Clk1M => {
                let freq = clocks.ensure_clk_1m_active(&self.power)?;
                (freq, Lpi2cClkselMux::CLKROOT_FUNC_5)
            }
            Lpi2cClockSel::Pll1ClkDiv => {
                let freq = clocks.ensure_pll1_clk_div_active(&self.power)?;
                (freq, Lpi2cClkselMux::CLKROOT_FUNC_6)
            }
            Lpi2cClockSel::None => {
                // no ClkrootFunc7, just write manually for now
                clksel.write(|w| w.0 = 0b111);
                clkdiv.modify(|w| {
                    w.set_reset(ClkdivReset::OFF);
                    w.set_halt(ClkdivHalt::OFF);
                });
                return Ok(PreEnableParts::empty());
            }
        };
        let div = self.div.into_divisor();
        let expected = freq / div;
        // 22.3.2 peripheral clock max functional clock limits
        let power = match self.power {
            PoweredClock::NormalEnabledDeepSleepDisabled => clocks.active_power,
            PoweredClock::AlwaysEnabled => clocks.lp_power,
        };
        let fmax = match power {
            VddLevel::MidDriveMode => 25_000_000,
            VddLevel::OverDriveMode => 60_000_000,
        };
        if expected > fmax {
            return Err(ClockError::BadConfig {
                clock: "lpi2c fclk",
                reason: "exceeds max rating",
            });
        }

        apply_div4!(self, clksel, clkdiv, variant, freq)
    }
}

//
// LPUart
//

/// Selectable clocks for Lpuart peripherals
#[derive(Debug, Clone, Copy)]
pub enum LpuartClockSel {
    /// FRO12M/FRO_LF/SIRC clock source, passed through divider
    /// "fro_lf_div"
    FroLfDiv,
    /// FRO180M/FRO_HF/FIRC clock source, passed through divider
    /// "fro_hf_div"
    FroHfDiv,
    /// SOSC/XTAL/EXTAL clock source
    #[cfg(not(feature = "sosc-as-gpio"))]
    ClkIn,
    /// FRO16K/clk_16k source
    Clk16K,
    /// clk_1m/FRO_LF divided by 12
    Clk1M,
    /// Output of PLL1, passed through clock divider,
    /// "pll1_clk_div", maybe "pll1_lf_div"?
    Pll1ClkDiv,
    /// Disabled
    None,
}

/// Which instance of the Lpuart is this?
///
/// Should not be directly selectable by end-users.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LpuartInstance {
    /// Instance 0
    Lpuart0,
    /// Instance 1
    Lpuart1,
    /// Instance 2
    Lpuart2,
    /// Instance 3
    Lpuart3,
    /// Instance 4
    Lpuart4,
    /// Instance 5
    Lpuart5,
}

/// Top level configuration for `Lpuart` instances.
pub struct LpuartConfig {
    /// Power state required for this peripheral
    pub power: PoweredClock,
    /// Clock source
    pub source: LpuartClockSel,
    /// Clock divisor
    pub div: Div4,
    /// Which instance is this?
    // NOTE: should not be user settable
    pub(crate) instance: LpuartInstance,
}

impl SPConfHelper for LpuartConfig {
    fn pre_enable_config(&self, clocks: &Clocks) -> Result<PreEnableParts, ClockError> {
        // check that source is suitable
        let mrcc0 = pac::MRCC0;

        let (clkdiv, clksel) = match self.instance {
            LpuartInstance::Lpuart0 => (mrcc0.mrcc_lpuart0_clkdiv(), mrcc0.mrcc_lpuart0_clksel()),
            LpuartInstance::Lpuart1 => (mrcc0.mrcc_lpuart1_clkdiv(), mrcc0.mrcc_lpuart1_clksel()),
            LpuartInstance::Lpuart2 => (mrcc0.mrcc_lpuart2_clkdiv(), mrcc0.mrcc_lpuart2_clksel()),
            LpuartInstance::Lpuart3 => (mrcc0.mrcc_lpuart3_clkdiv(), mrcc0.mrcc_lpuart3_clksel()),
            LpuartInstance::Lpuart4 => (mrcc0.mrcc_lpuart4_clkdiv(), mrcc0.mrcc_lpuart4_clksel()),
            LpuartInstance::Lpuart5 => (mrcc0.mrcc_lpuart5_clkdiv(), mrcc0.mrcc_lpuart5_clksel()),
        };

        let (freq, variant) = match self.source {
            LpuartClockSel::FroLfDiv => {
                let freq = clocks.ensure_fro_lf_div_active(&self.power)?;
                (freq, LpuartClkselMux::CLKROOT_FUNC_0)
            }
            LpuartClockSel::FroHfDiv => {
                let freq = clocks.ensure_fro_hf_div_active(&self.power)?;
                (freq, LpuartClkselMux::CLKROOT_FUNC_2)
            }
            #[cfg(not(feature = "sosc-as-gpio"))]
            LpuartClockSel::ClkIn => {
                let freq = clocks.ensure_clk_in_active(&self.power)?;
                (freq, LpuartClkselMux::CLKROOT_FUNC_3)
            }
            LpuartClockSel::Clk16K => {
                let freq = clocks.ensure_clk_16k_vdd_core_active(&self.power)?;
                (freq, LpuartClkselMux::CLKROOT_FUNC_4)
            }
            LpuartClockSel::Clk1M => {
                let freq = clocks.ensure_clk_1m_active(&self.power)?;
                (freq, LpuartClkselMux::CLKROOT_FUNC_5)
            }
            LpuartClockSel::Pll1ClkDiv => {
                let freq = clocks.ensure_pll1_clk_div_active(&self.power)?;
                (freq, LpuartClkselMux::CLKROOT_FUNC_6)
            }
            LpuartClockSel::None => {
                // no ClkrootFunc7, just write manually for now
                clksel.write(|w| w.set_mux(LpuartClkselMux::_RESERVED_7));
                clkdiv.modify(|w| {
                    w.set_reset(ClkdivReset::ON);
                    w.set_halt(ClkdivHalt::ON);
                });
                return Ok(PreEnableParts::empty());
            }
        };

        // Check clock speed is reasonable
        let div = self.div.into_divisor();
        let expected = freq / div;
        // 22.3.2 peripheral clock max functional clock limits
        let power = match self.power {
            PoweredClock::NormalEnabledDeepSleepDisabled => clocks.active_power,
            PoweredClock::AlwaysEnabled => clocks.lp_power,
        };
        let fmax = match power {
            VddLevel::MidDriveMode => 45_000_000,
            VddLevel::OverDriveMode => 180_000_000,
        };
        if expected > fmax {
            return Err(ClockError::BadConfig {
                clock: "lpuart fclk",
                reason: "exceeds max rating",
            });
        }

        // set clksel
        apply_div4!(self, clksel, clkdiv, variant, freq)
    }
}

//
// CTimer
//

/// Selectable clocks for `CTimer` peripherals
#[derive(Debug, Clone, Copy)]
pub enum CTimerClockSel {
    /// FRO12M/FRO_LF/SIRC clock source, passed through divider
    /// "fro_lf_div"
    FroLfDiv,
    /// FRO180M/FRO_HF/FIRC clock source, passed through divider
    /// "fro_hf_div"
    FroHfDiv,
    /// SOSC/XTAL/EXTAL clock source
    #[cfg(not(feature = "sosc-as-gpio"))]
    ClkIn,
    /// FRO16K/clk_16k source
    Clk16K,
    /// clk_1m/FRO_LF divided by 12
    Clk1M,
    /// Internal PLL output, with configurable divisor
    Pll1ClkDiv,
    /// Disabled
    None,
}

/// Which instance of the `CTimer` is this?
///
/// Should not be directly selectable by end-users.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CTimerInstance {
    /// Instance 0
    CTimer0,
    /// Instance 1
    CTimer1,
    /// Instance 2
    CTimer2,
    /// Instance 3
    CTimer3,
    /// Instance 4
    CTimer4,
}

/// Top level configuration for `CTimer` instances.
pub struct CTimerConfig {
    /// Power state required for this peripheral
    pub power: PoweredClock,
    /// Clock source
    pub source: CTimerClockSel,
    /// Clock divisor
    pub div: Div4,
    /// Which instance is this?
    // NOTE: should not be user settable
    pub(crate) instance: CTimerInstance,
}

impl SPConfHelper for CTimerConfig {
    fn pre_enable_config(&self, clocks: &Clocks) -> Result<PreEnableParts, ClockError> {
        // check that source is suitable
        let mrcc0 = pac::MRCC0;

        let (clkdiv, clksel) = match self.instance {
            CTimerInstance::CTimer0 => (mrcc0.mrcc_ctimer0_clkdiv(), mrcc0.mrcc_ctimer0_clksel()),
            CTimerInstance::CTimer1 => (mrcc0.mrcc_ctimer1_clkdiv(), mrcc0.mrcc_ctimer1_clksel()),
            CTimerInstance::CTimer2 => (mrcc0.mrcc_ctimer2_clkdiv(), mrcc0.mrcc_ctimer2_clksel()),
            CTimerInstance::CTimer3 => (mrcc0.mrcc_ctimer3_clkdiv(), mrcc0.mrcc_ctimer3_clksel()),
            CTimerInstance::CTimer4 => (mrcc0.mrcc_ctimer4_clkdiv(), mrcc0.mrcc_ctimer4_clksel()),
        };

        let (freq, variant) = match self.source {
            CTimerClockSel::FroLfDiv => {
                let freq = clocks.ensure_fro_lf_div_active(&self.power)?;
                (freq, CtimerClkselMux::CLKROOT_FUNC_0)
            }
            CTimerClockSel::FroHfDiv => {
                let freq = clocks.ensure_fro_hf_div_active(&self.power)?;
                (freq, CtimerClkselMux::CLKROOT_FUNC_1)
            }
            #[cfg(not(feature = "sosc-as-gpio"))]
            CTimerClockSel::ClkIn => {
                let freq = clocks.ensure_clk_in_active(&self.power)?;
                (freq, CtimerClkselMux::CLKROOT_FUNC_3)
            }
            CTimerClockSel::Clk16K => {
                let freq = clocks.ensure_clk_16k_vdd_core_active(&self.power)?;
                (freq, CtimerClkselMux::CLKROOT_FUNC_4)
            }
            CTimerClockSel::Clk1M => {
                let freq = clocks.ensure_clk_1m_active(&self.power)?;
                (freq, CtimerClkselMux::CLKROOT_FUNC_5)
            }
            CTimerClockSel::Pll1ClkDiv => {
                let freq = clocks.ensure_pll1_clk_div_active(&self.power)?;
                (freq, CtimerClkselMux::CLKROOT_FUNC_6)
            }
            CTimerClockSel::None => {
                // no ClkrootFunc7, just write manually for now
                clksel.write(|w| w.set_mux(CtimerClkselMux::_RESERVED_7));
                clkdiv.modify(|w| {
                    w.set_reset(ClkdivReset::ON);
                    w.set_halt(ClkdivHalt::ON)
                });
                return Ok(PreEnableParts::empty());
            }
        };

        let div = self.div.into_divisor();
        let expected = freq / div;

        // 22.3.2 peripheral clock max functional clock limits
        let power = match self.power {
            PoweredClock::NormalEnabledDeepSleepDisabled => clocks.active_power,
            PoweredClock::AlwaysEnabled => clocks.lp_power,
        };
        let fmax = match power {
            VddLevel::MidDriveMode => 25_000_000,
            VddLevel::OverDriveMode => 60_000_000,
        };

        if expected > fmax {
            return Err(ClockError::BadConfig {
                clock: "ctimer fclk",
                reason: "exceeds max rating",
            });
        }

        apply_div4!(self, clksel, clkdiv, variant, freq)
    }
}

//
// OSTimer
//

/// Selectable clocks for the OSTimer peripheral
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum OstimerClockSel {
    /// 16k clock, sourced from FRO16K (Vdd Core)
    Clk16kVddCore,
    /// 1 MHz Clock sourced from FRO12M
    Clk1M,
    /// Disabled
    None,
}

/// Top level configuration for the `OSTimer` peripheral
pub struct OsTimerConfig {
    /// Power state required for this peripheral
    pub power: PoweredClock,
    /// Selected clock source for this peripheral
    pub source: OstimerClockSel,
}

impl SPConfHelper for OsTimerConfig {
    fn pre_enable_config(&self, clocks: &Clocks) -> Result<PreEnableParts, ClockError> {
        let mrcc0 = pac::MRCC0;
        // NOTE: complies with 22.3.2 peripheral clock max functional clock limits
        // which is 1MHz, and we can only select 1mhz/16khz.
        Ok(match self.source {
            OstimerClockSel::Clk16kVddCore => {
                let freq = clocks.ensure_clk_16k_vdd_core_active(&self.power)?;
                mrcc0
                    .mrcc_ostimer0_clksel()
                    .write(|w| w.set_mux(OstimerClkselMux::CLKROOT_16K));
                PreEnableParts {
                    freq,
                    wake_guard: WakeGuard::for_power(&self.power),
                }
            }
            OstimerClockSel::Clk1M => {
                let freq = clocks.ensure_clk_1m_active(&self.power)?;
                mrcc0
                    .mrcc_ostimer0_clksel()
                    .write(|w| w.set_mux(OstimerClkselMux::CLKROOT_1M));
                PreEnableParts {
                    freq,
                    wake_guard: WakeGuard::for_power(&self.power),
                }
            }
            OstimerClockSel::None => {
                mrcc0
                    .mrcc_ostimer0_clksel()
                    .write(|w| w.set_mux(OstimerClkselMux::_RESERVED_3));
                PreEnableParts::empty()
            }
        })
    }
}

//
// Adc
//

/// Selectable clocks for the ADC peripheral
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum AdcClockSel {
    /// Divided `fro_lf`/`clk_12m`/FRO12M source
    FroLfDiv,
    /// Gated `fro_hf`/`FRO180M` source
    FroHf,
    /// External Clock Source
    #[cfg(not(feature = "sosc-as-gpio"))]
    ClkIn,
    /// 1MHz clock sourced by a divided `fro_lf`/`clk_12m`
    Clk1M,
    /// Internal PLL output, with configurable divisor
    Pll1ClkDiv,
    /// No clock/disabled
    None,
}

/// Top level configuration for the ADC peripheral
pub struct AdcConfig {
    /// Power state required for this peripheral
    pub power: PoweredClock,
    /// Selected clock-source for this peripheral
    pub source: AdcClockSel,
    /// Pre-divisor, applied to the upstream clock output
    pub div: Div4,
}

impl SPConfHelper for AdcConfig {
    fn pre_enable_config(&self, clocks: &Clocks) -> Result<PreEnableParts, ClockError> {
        let mrcc0 = pac::MRCC0;
        let (freq, variant) = match self.source {
            AdcClockSel::FroLfDiv => {
                let freq = clocks.ensure_fro_lf_div_active(&self.power)?;
                (freq, AdcClkselMux::CLKROOT_FUNC_0)
            }
            AdcClockSel::FroHf => {
                let freq = clocks.ensure_fro_hf_active(&self.power)?;
                (freq, AdcClkselMux::CLKROOT_FUNC_1)
            }
            #[cfg(not(feature = "sosc-as-gpio"))]
            AdcClockSel::ClkIn => {
                let freq = clocks.ensure_clk_in_active(&self.power)?;
                (freq, AdcClkselMux::CLKROOT_FUNC_3)
            }
            AdcClockSel::Clk1M => {
                let freq = clocks.ensure_clk_1m_active(&self.power)?;
                (freq, AdcClkselMux::CLKROOT_FUNC_5)
            }
            AdcClockSel::Pll1ClkDiv => {
                let freq = clocks.ensure_pll1_clk_div_active(&self.power)?;
                (freq, AdcClkselMux::CLKROOT_FUNC_6)
            }
            AdcClockSel::None => {
                mrcc0.mrcc_adc_clksel().write(|w| {
                    // no ClkrootFunc7, just write manually for now
                    w.set_mux(AdcClkselMux::_RESERVED_7)
                });
                mrcc0.mrcc_adc_clkdiv().modify(|w| {
                    w.set_reset(ClkdivReset::ON);
                    w.set_halt(ClkdivHalt::ON);
                });
                return Ok(PreEnableParts::empty());
            }
        };
        let clksel = mrcc0.mrcc_adc_clksel();
        let clkdiv = mrcc0.mrcc_adc_clkdiv();

        // Check clock speed is reasonable
        let div = self.div.into_divisor();
        let expected = freq / div;
        // 22.3.2 peripheral clock max functional clock limits
        let power = match self.power {
            PoweredClock::NormalEnabledDeepSleepDisabled => clocks.active_power,
            PoweredClock::AlwaysEnabled => clocks.lp_power,
        };
        let fmax = match power {
            VddLevel::MidDriveMode => 24_000_000,
            VddLevel::OverDriveMode => 64_000_000,
        };
        if expected > fmax {
            return Err(ClockError::BadConfig {
                clock: "adc fclk",
                reason: "exceeds max rating",
            });
        }

        apply_div4!(self, clksel, clkdiv, variant, freq)
    }
}
