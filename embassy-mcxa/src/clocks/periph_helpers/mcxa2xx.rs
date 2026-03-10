use nxp_pac::mrcc::vals::{AdcClkselMux, ClkdivHalt, ClkdivReset, ClkdivUnstab};

use super::{Div4, PreEnableParts, SPConfHelper};
use crate::clocks::config::VddLevel;
use crate::clocks::{ClockError, Clocks, PoweredClock, WakeGuard};
use crate::{apply_div4, pac};

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
