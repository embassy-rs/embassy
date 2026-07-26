//! Peripheral Helpers
//!
//! The purpose of this module is to define the per-peripheral special handling
//! required from a clocking perspective. Different peripherals have different
//! selectable source clocks, and some peripherals have additional pre-dividers
//! that can be used.
//!
//! See the docs of [`SPConfHelper`] for more details.

use super::{ClockError, Clocks, PoweredClock, WakeGuard};
use crate::clocks::VddLevel;
#[cfg(feature = "mcxa5xx")]
use crate::pac::mrcc::FlexspiClkselMux;
use crate::pac::mrcc::{
    AdcClkselMux, ClkdivHalt, ClkdivReset, ClkdivUnstab, CtimerClkselMux, FclkClkselMux, Lpi2cClkselMux,
    LpspiClkselMux, LpuartClkselMux, OstimerClkselMux,
};

#[must_use]
pub struct PreEnableParts {
    /// The frequency fed into the peripheral, taking into account the selected
    /// source clock, as well as any pre-divisors.
    pub freq: u32,
    /// The wake guard, if necessary for the selected clock source
    pub wake_guard: Option<WakeGuard>,
}

impl PreEnableParts {
    pub fn empty() -> Self {
        Self {
            freq: 0,
            wake_guard: None,
        }
    }
}

/// Sealed Peripheral Configuration Helper
///
/// NOTE: the name "sealed" doesn't *totally* make sense because its not sealed yet in the
/// embassy-mcxa project, but it derives from embassy-imxrt where it is. We should
/// fix the name, or actually do the sealing of peripherals.
///
/// This trait serves to act as a per-peripheral customization for clocking behavior.
///
/// This trait should be implemented on a configuration type for a given peripheral, and
/// provide the methods that will be called by the higher level operations like
/// `embassy_mcxa::clocks::enable_and_reset()`.
pub trait SPConfHelper {
    /// This method is called AFTER a given MRCC peripheral has been disabled, and BEFORE
    /// the peripheral is to be enabled.
    ///
    /// This function SHOULD NOT make any changes to the system clock configuration, even
    /// unsafely, as this should remain static for the duration of the program.
    ///
    /// This function should check that any relevant upstream clocks are enabled, are in a
    /// reasonable power state, and that the requested configuration can be made. If any of
    /// these checks fail, an `Err(ClockError)` should be returned, likely `ClockError::BadConfig`.
    ///
    /// This function WILL be called in a critical section, care should be taken not to delay
    /// for an unreasonable amount of time.
    ///
    /// On success, this function MUST return an `Ok(parts)`.
    fn pre_enable_config(&self, clocks: &Clocks) -> Result<PreEnableParts, ClockError>;
}

/// Copy and paste macro that:
///
/// * Sets the clocksel mux to `$selvar`
/// * Resets and halts the div, and applies the calculated div4 bits
/// * Releases reset + halt
/// * Waits for the div to stabilize
/// * Returns `Ok($freq / $conf.div.into_divisor())`
///
/// Assumes:
///
/// * self is a configuration struct that has fields called:
///   * `div`, which is a `Div4`
///   * `power`, which is a `PoweredClock`
///
/// usage:
///
/// ```rust,ignore
/// apply_div4!(self, clksel, clkdiv, variant, freq)
/// ```
///
/// In the future if we make all the clksel+clkdiv pairs into commonly derivedFrom
/// registers, or if we put some kind of simple trait around those regs, we could
/// do this with something other than a macro, but for now, this is harm-reduction
/// to avoid incorrect copy + paste
#[doc(hidden)]
#[macro_export]
macro_rules! apply_div4 {
    ($conf:ident, $selreg:ident, $divreg:ident, $selvar:ident, $freq:ident) => {{
        // set clksel
        $selreg.modify(|w| w.set_mux($selvar));

        // Set up clkdiv
        $divreg.modify(|w| {
            w.set_div($conf.div.into_bits());
            w.set_halt(ClkdivHalt::Off);
            w.set_reset(ClkdivReset::Off);
        });
        $divreg.modify(|w| {
            w.set_halt(ClkdivHalt::On);
            w.set_reset(ClkdivReset::On);
        });

        while $divreg.read().unstab() == ClkdivUnstab::Off {}

        Ok(PreEnableParts {
            freq: $freq / $conf.div.into_divisor(),
            wake_guard: WakeGuard::for_power(&$conf.power),
        })
    }};
}

// config types

/// This type represents a divider in the range 1..=16.
///
/// At a hardware level, this is an 8-bit register from 0..=15,
/// which adds one.
///
/// While the *clock* domain seems to use 8-bit dividers, the *peripheral* domain
/// seems to use 4 bit dividers!
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Div4(pub(super) u8);

impl Div4 {
    /// Divide by one, or no division
    pub const fn no_div() -> Self {
        Self(0)
    }

    /// Store a "raw" divisor value that will divide the source by
    /// `(n + 1)`, e.g. `Div4::from_raw(0)` will divide the source
    /// by 1, and `Div4::from_raw(15)` will divide the source by
    /// 16.
    pub const fn from_raw(n: u8) -> Option<Self> {
        if n > 0b1111 { None } else { Some(Self(n)) }
    }

    /// Store a specific divisor value that will divide the source
    /// by `n`. e.g. `Div4::from_divisor(1)` will divide the source
    /// by 1, and `Div4::from_divisor(16)` will divide the source
    /// by 16.
    ///
    /// Will return `None` if `n` is not in the range `1..=16`.
    /// Consider [`Self::from_raw`] for an infallible version.
    pub const fn from_divisor(n: u8) -> Option<Self> {
        let Some(n) = n.checked_sub(1) else {
            return None;
        };
        if n > 0b1111 {
            return None;
        }
        Some(Self(n))
    }

    /// Convert into "raw" bits form
    #[inline(always)]
    pub const fn into_bits(self) -> u8 {
        self.0
    }

    /// Convert into "divisor" form, as a u32 for convenient frequency math
    #[inline(always)]
    pub const fn into_divisor(self) -> u32 {
        self.0 as u32 + 1
    }
}

/// A basic type that always returns an error when `post_enable_config` is called.
///
/// Should only be used as a placeholder.
pub struct UnimplementedConfig;

impl SPConfHelper for UnimplementedConfig {
    fn pre_enable_config(&self, _clocks: &Clocks) -> Result<PreEnableParts, ClockError> {
        Err(ClockError::UnimplementedConfig)
    }
}

/// A basic type that always returns `Ok` when `PreEnableParts` is called.
///
/// This should only be used for peripherals that are "ambiently" clocked, like `PORTn`
/// peripherals, which have no selectable/configurable source clock.
pub struct NoConfig;
impl SPConfHelper for NoConfig {
    fn pre_enable_config(&self, _clocks: &Clocks) -> Result<PreEnableParts, ClockError> {
        Ok(PreEnableParts::empty())
    }
}

/// A basic type that always returns `Ok` when `PreEnableParts` is called.
///
/// This should only be used for peripherals that are clocked only by
/// the CLK1M clock and have no other selectable/configurable source
/// clock.
pub struct Clk1MConfig;
impl SPConfHelper for Clk1MConfig {
    fn pre_enable_config(&self, _clocks: &Clocks) -> Result<PreEnableParts, ClockError> {
        Ok(PreEnableParts {
            freq: 1_000_000,
            wake_guard: None,
        })
    }
}

/// Placeholder configuration for the DAC peripheral.
///
/// The DAC HAL driver is not yet implemented, but the PAC metadata
/// declares the gate config type, so we provide a stub here. Replace
/// with the real implementation when the DAC driver is added.
pub struct DacConfig;
impl SPConfHelper for DacConfig {
    fn pre_enable_config(&self, _clocks: &Clocks) -> Result<PreEnableParts, ClockError> {
        Ok(PreEnableParts::empty())
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
    // /// USB PLL Clk
    // #[cfg(feature = "mcxa5xx")]
    // UsbPllClk,
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
        let mrcc0 = crate::pac::MRCC0;

        let (freq, variant) = match self.source {
            AdcClockSel::FroLfDiv => {
                let freq = clocks.ensure_fro_lf_div_active(&self.power)?;
                // TODO: fix PAC names for consistency
                #[cfg(feature = "mcxa2xx")]
                let mux = AdcClkselMux::ClkrootFunc0;
                #[cfg(feature = "mcxa5xx")]
                let mux = AdcClkselMux::I0ClkrootSircDiv;

                (freq, mux)
            }
            AdcClockSel::FroHf => {
                let freq = clocks.ensure_fro_hf_active(&self.power)?;
                #[cfg(feature = "mcxa2xx")]
                let mux = AdcClkselMux::ClkrootFunc1;
                #[cfg(feature = "mcxa5xx")]
                let mux = AdcClkselMux::I1ClkrootFircGated;

                (freq, mux)
            }
            #[cfg(not(feature = "sosc-as-gpio"))]
            AdcClockSel::ClkIn => {
                let freq = clocks.ensure_clk_in_active(&self.power)?;
                #[cfg(feature = "mcxa2xx")]
                let mux = AdcClkselMux::ClkrootFunc3;
                #[cfg(feature = "mcxa5xx")]
                let mux = AdcClkselMux::I3ClkrootSosc;

                (freq, mux)
            }
            // #[cfg(feature = "mcxa5xx")]
            // AdcClockSel::UsbPllClk => {
            //     let freq = clocks.ensure_usb_pll_clk_active(&self.power)?;
            //     let mux = AdcClkselMux::I4_CLKROOT_USBPFD;
            //     (freq, mux)
            // }
            AdcClockSel::Clk1M => {
                let freq = clocks.ensure_clk_1m_active(&self.power)?;
                #[cfg(feature = "mcxa2xx")]
                let mux = AdcClkselMux::ClkrootFunc5;
                #[cfg(feature = "mcxa5xx")]
                let mux = AdcClkselMux::I5Clkroot1m;

                (freq, mux)
            }
            AdcClockSel::Pll1ClkDiv => {
                let freq = clocks.ensure_pll1_clk_div_active(&self.power)?;
                #[cfg(feature = "mcxa2xx")]
                let mux = AdcClkselMux::ClkrootFunc6;
                #[cfg(feature = "mcxa5xx")]
                let mux = AdcClkselMux::I6ClkrootSpllDiv;

                (freq, mux)
            }
            AdcClockSel::None => {
                mrcc0.mrcc_adc_clksel().write(|w| {
                    // no ClkrootFunc7, just write manually for now
                    w.set_mux(AdcClkselMux::_RESERVED_7)
                });
                mrcc0.mrcc_adc_clkdiv().modify(|w| {
                    w.set_reset(ClkdivReset::On);
                    w.set_halt(ClkdivHalt::On);
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

        #[cfg(feature = "mcxa2xx")]
        let fmax = match power {
            VddLevel::MidDriveMode => 24_000_000,
            VddLevel::OverDriveMode => 64_000_000,
        };

        #[cfg(feature = "mcxa5xx")]
        let fmax = match power {
            VddLevel::MidDriveMode => 24_000_000,
            VddLevel::NormalMode | VddLevel::OverDriveMode => 64_000_000,
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
        let mrcc0 = crate::pac::MRCC0;
        // NOTE: complies with 22.3.2 peripheral clock max functional clock limits
        // which is 1MHz, and we can only select 1mhz/16khz.
        Ok(match self.source {
            OstimerClockSel::Clk16kVddCore => {
                // TODO: fix PAC names for consistency
                #[cfg(feature = "mcxa2xx")]
                let mux = OstimerClkselMux::Clkroot16k;
                #[cfg(feature = "mcxa5xx")]
                let mux = OstimerClkselMux::I0Clkroot16k;

                let freq = clocks.ensure_clk_16k_vdd_core_active(&self.power)?;
                mrcc0.mrcc_ostimer0_clksel().write(|w| w.set_mux(mux));
                PreEnableParts {
                    freq,
                    wake_guard: WakeGuard::for_power(&self.power),
                }
            }
            OstimerClockSel::Clk1M => {
                let freq = clocks.ensure_clk_1m_active(&self.power)?;
                // TODO: fix PAC names for consistency
                #[cfg(feature = "mcxa2xx")]
                let mux = OstimerClkselMux::Clkroot1m;
                #[cfg(feature = "mcxa5xx")]
                let mux = OstimerClkselMux::I2Clkroot1m;

                mrcc0.mrcc_ostimer0_clksel().write(|w| w.set_mux(mux));
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
// LPSPI
//

/// Selectable clocks for `Lpspi` peripherals
#[derive(Debug, Clone, Copy)]
pub enum LpspiClockSel {
    /// FRO12M/FRO_LF/SIRC clock source, passed through divider
    /// "fro_lf_div"
    FroLfDiv,
    /// FRO180M/FRO192M/FRO_HF/FIRC clock source, passed through divider
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

/// Which instance of the `Lpspi` is this?
///
/// Should not be directly selectable by end-users.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LpspiInstance {
    /// Instance 0
    Lpspi0,
    /// Instance 1
    Lpspi1,
    /// Instance 2
    #[cfg(feature = "mcxa5xx")]
    Lpspi2,
    /// Instance 3
    #[cfg(feature = "mcxa5xx")]
    Lpspi3,
    /// Instance 4
    #[cfg(feature = "mcxa5xx")]
    Lpspi4,
    /// Instance 5
    #[cfg(feature = "mcxa5xx")]
    Lpspi5,
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
        let mrcc0 = crate::pac::MRCC0;

        let (clkdiv, clksel) = match self.instance {
            LpspiInstance::Lpspi0 => (mrcc0.mrcc_lpspi0_clkdiv(), mrcc0.mrcc_lpspi0_clksel()),
            LpspiInstance::Lpspi1 => (mrcc0.mrcc_lpspi1_clkdiv(), mrcc0.mrcc_lpspi1_clksel()),
            #[cfg(feature = "mcxa5xx")]
            LpspiInstance::Lpspi2 => (mrcc0.mrcc_lpspi2_clkdiv(), mrcc0.mrcc_lpspi2_clksel()),
            #[cfg(feature = "mcxa5xx")]
            LpspiInstance::Lpspi3 => (mrcc0.mrcc_lpspi3_clkdiv(), mrcc0.mrcc_lpspi3_clksel()),
            #[cfg(feature = "mcxa5xx")]
            LpspiInstance::Lpspi4 => (mrcc0.mrcc_lpspi4_clkdiv(), mrcc0.mrcc_lpspi4_clksel()),
            #[cfg(feature = "mcxa5xx")]
            LpspiInstance::Lpspi5 => (mrcc0.mrcc_lpspi5_clkdiv(), mrcc0.mrcc_lpspi5_clksel()),
        };

        let (freq, variant) = match self.source {
            LpspiClockSel::FroLfDiv => {
                let freq = clocks.ensure_fro_lf_div_active(&self.power)?;

                // TODO: fix PAC names for consistency
                #[cfg(feature = "mcxa2xx")]
                let mux = LpspiClkselMux::ClkrootFunc0;
                #[cfg(feature = "mcxa5xx")]
                let mux = LpspiClkselMux::I0ClkrootFunc0;

                (freq, mux)
            }
            LpspiClockSel::FroHfDiv => {
                let freq = clocks.ensure_fro_hf_div_active(&self.power)?;

                // TODO: fix PAC names for consistency
                #[cfg(feature = "mcxa2xx")]
                let mux = LpspiClkselMux::ClkrootFunc2;
                #[cfg(feature = "mcxa5xx")]
                let mux = LpspiClkselMux::I2ClkrootFunc2;

                (freq, mux)
            }
            #[cfg(not(feature = "sosc-as-gpio"))]
            LpspiClockSel::ClkIn => {
                let freq = clocks.ensure_clk_in_active(&self.power)?;

                // TODO: fix PAC names for consistency
                #[cfg(feature = "mcxa2xx")]
                let mux = LpspiClkselMux::ClkrootFunc3;
                #[cfg(feature = "mcxa5xx")]
                let mux = LpspiClkselMux::I3ClkrootFunc3;

                (freq, mux)
            }
            LpspiClockSel::Clk1M => {
                let freq = clocks.ensure_clk_1m_active(&self.power)?;

                // TODO: fix PAC names for consistency
                #[cfg(feature = "mcxa2xx")]
                let mux = LpspiClkselMux::ClkrootFunc5;
                #[cfg(feature = "mcxa5xx")]
                let mux = LpspiClkselMux::I5ClkrootFunc5;

                (freq, mux)
            }
            LpspiClockSel::Pll1ClkDiv => {
                let freq = clocks.ensure_pll1_clk_div_active(&self.power)?;

                // TODO: fix PAC names for consistency
                #[cfg(feature = "mcxa2xx")]
                let mux = LpspiClkselMux::ClkrootFunc6;
                #[cfg(feature = "mcxa5xx")]
                let mux = LpspiClkselMux::I6ClkrootFunc6;

                (freq, mux)
            }
            LpspiClockSel::None => {
                // no ClkrootFunc7, just write manually for now
                clksel.write(|w| w.0 = 0b111);
                clkdiv.modify(|w| {
                    w.set_reset(ClkdivReset::Off);
                    w.set_halt(ClkdivHalt::Off);
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

        #[cfg(feature = "mcxa2xx")]
        let fmax = match power {
            VddLevel::MidDriveMode => 25_000_000,
            VddLevel::OverDriveMode => 60_000_000,
        };

        #[cfg(feature = "mcxa5xx")]
        let fmax = match power {
            VddLevel::MidDriveMode => 50_000_000,
            VddLevel::NormalMode => 150_000_000,
            VddLevel::OverDriveMode => 200_000_000,
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
// FlexSPI
//

/// Selectable clocks for `FlexSPI` peripherals.
#[cfg(feature = "mcxa5xx")]
#[derive(Debug, Clone, Copy)]
pub enum FlexspiClockSel {
    /// Gated FRO_HF / FIRC clock.
    FroHf,
    /// PLL1 clock after its divider.
    Pll1ClkDiv,
}

/// Which instance of the `FlexSPI` peripheral is this?
#[cfg(feature = "mcxa5xx")]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FlexspiInstance {
    /// Instance 0.
    Flexspi0,
}

/// Top level configuration for `FlexSPI` instances.
#[cfg(feature = "mcxa5xx")]
pub struct FlexspiConfig {
    /// Power state required for this peripheral.
    pub power: PoweredClock,
    /// Clock source.
    pub source: FlexspiClockSel,
    /// Clock divisor.
    pub div: Div4,
    /// Which instance is this?
    pub(crate) instance: FlexspiInstance,
}

#[cfg(feature = "mcxa5xx")]
impl SPConfHelper for FlexspiConfig {
    fn pre_enable_config(&self, clocks: &Clocks) -> Result<PreEnableParts, ClockError> {
        let mrcc0 = crate::pac::MRCC0;

        let (clkdiv, clksel) = match self.instance {
            FlexspiInstance::Flexspi0 => (mrcc0.mrcc_flexspi0_clkdiv(), mrcc0.mrcc_flexspi0_clksel()),
        };

        let (freq, variant) = match self.source {
            FlexspiClockSel::FroHf => (
                clocks.ensure_fro_hf_active(&self.power)?,
                FlexspiClkselMux::I1ClkrootFircGated,
            ),
            FlexspiClockSel::Pll1ClkDiv => (
                clocks.ensure_pll1_clk_div_active(&self.power)?,
                FlexspiClkselMux::I6ClkrootSpll,
            ),
        };

        apply_div4!(self, clksel, clkdiv, variant, freq)
    }
}

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
    /// Internal PLL output, with configurable divisor
    #[cfg(feature = "mcxa5xx")]
    Pll1ClkDiv,
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
        #[cfg(feature = "mcxa2xx")]
        // Always 25MHz maximum frequency.
        const I3C_FCLK_MAX: u32 = 25_000_000;

        #[cfg(feature = "mcxa5xx")]
        // Always 100MHz maximum frequency.
        const I3C_FCLK_MAX: u32 = 100_000_000;

        // check that source is suitable
        let mrcc0 = crate::pac::MRCC0;

        let (clkdiv, clksel) = (mrcc0.mrcc_i3c0_fclk_clkdiv(), mrcc0.mrcc_i3c0_fclk_clksel());

        let (freq, variant) = match self.source {
            I3cClockSel::FroLfDiv => {
                let freq = clocks.ensure_fro_lf_div_active(&self.power)?;
                #[cfg(feature = "mcxa2xx")]
                let mux = FclkClkselMux::ClkrootFunc0;
                #[cfg(feature = "mcxa5xx")]
                let mux = FclkClkselMux::I0ClkrootFunc0;

                (freq, mux)
            }
            I3cClockSel::FroHfDiv => {
                let freq = clocks.ensure_fro_hf_div_active(&self.power)?;
                #[cfg(feature = "mcxa2xx")]
                let mux = FclkClkselMux::ClkrootFunc2;
                #[cfg(feature = "mcxa5xx")]
                let mux = FclkClkselMux::I2ClkrootFunc2;

                (freq, mux)
            }
            #[cfg(not(feature = "sosc-as-gpio"))]
            I3cClockSel::ClkIn => {
                let freq = clocks.ensure_clk_in_active(&self.power)?;
                #[cfg(feature = "mcxa2xx")]
                let mux = FclkClkselMux::ClkrootFunc3;
                #[cfg(feature = "mcxa5xx")]
                let mux = FclkClkselMux::I3ClkrootFunc3;

                (freq, mux)
            }
            I3cClockSel::Clk1M => {
                let freq = clocks.ensure_clk_1m_active(&self.power)?;
                #[cfg(feature = "mcxa2xx")]
                let mux = FclkClkselMux::ClkrootFunc5;
                #[cfg(feature = "mcxa5xx")]
                let mux = FclkClkselMux::I5ClkrootFunc5;

                (freq, mux)
            }
            #[cfg(feature = "mcxa5xx")]
            I3cClockSel::Pll1ClkDiv => {
                let freq = clocks.ensure_pll1_clk_div_active(&self.power)?;
                let mux = FclkClkselMux::I6ClkrootFunc6;

                (freq, mux)
            }
            I3cClockSel::None => {
                // no ClkrootFunc7, just write manually for now
                clksel.write(|w| w.0 = 0b111);
                clkdiv.modify(|w| {
                    w.set_reset(ClkdivReset::Off);
                    w.set_halt(ClkdivHalt::Off);
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
// LPI2c
//

/// Selectable clocks for `Lpi2c` peripherals
#[derive(Debug, Clone, Copy)]
pub enum Lpi2cClockSel {
    /// FRO12M/FRO_LF/SIRC clock source, passed through divider
    /// "fro_lf_div"
    FroLfDiv,
    /// FRO180M/FRO192M/FRO_HF/FIRC clock source, passed through divider
    /// "fro_hf_div"
    FroHfDiv,
    /// SOSC/XTAL/EXTAL clock source
    #[cfg(feature = "mcxa2xx")]
    #[cfg(not(feature = "sosc-as-gpio"))]
    ClkIn,
    /// clk_1m/FRO_LF divided by 12
    Clk1M,
    /// Output of PLL1, passed through clock divider,
    /// "pll1_clk_div", maybe "pll1_lf_div"?
    #[cfg(feature = "mcxa2xx")]
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
    #[cfg(feature = "mcxa5xx")]
    /// Instance 4
    Lpi2c4,
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
        let mrcc0 = crate::pac::MRCC0;

        let (clkdiv, clksel) = match self.instance {
            Lpi2cInstance::Lpi2c0 => (mrcc0.mrcc_lpi2c0_clkdiv(), mrcc0.mrcc_lpi2c0_clksel()),
            Lpi2cInstance::Lpi2c1 => (mrcc0.mrcc_lpi2c1_clkdiv(), mrcc0.mrcc_lpi2c1_clksel()),
            Lpi2cInstance::Lpi2c2 => (mrcc0.mrcc_lpi2c2_clkdiv(), mrcc0.mrcc_lpi2c2_clksel()),
            Lpi2cInstance::Lpi2c3 => (mrcc0.mrcc_lpi2c3_clkdiv(), mrcc0.mrcc_lpi2c3_clksel()),
            #[cfg(feature = "mcxa5xx")]
            Lpi2cInstance::Lpi2c4 => (mrcc0.mrcc_lpi2c4_clkdiv(), mrcc0.mrcc_lpi2c4_clksel()),
        };

        let (freq, variant) = match self.source {
            Lpi2cClockSel::FroLfDiv => {
                let freq = clocks.ensure_fro_lf_div_active(&self.power)?;
                #[cfg(feature = "mcxa2xx")]
                let mux = Lpi2cClkselMux::ClkrootFunc0;
                #[cfg(feature = "mcxa5xx")]
                let mux = Lpi2cClkselMux::I0ClkrootFunc0;

                (freq, mux)
            }
            Lpi2cClockSel::FroHfDiv => {
                let freq = clocks.ensure_fro_hf_div_active(&self.power)?;
                #[cfg(feature = "mcxa2xx")]
                let mux = Lpi2cClkselMux::ClkrootFunc2;
                #[cfg(feature = "mcxa5xx")]
                let mux = Lpi2cClkselMux::I2ClkrootFunc2;

                (freq, mux)
            }
            #[cfg(feature = "mcxa2xx")]
            #[cfg(not(feature = "sosc-as-gpio"))]
            Lpi2cClockSel::ClkIn => {
                let freq = clocks.ensure_clk_in_active(&self.power)?;
                (freq, Lpi2cClkselMux::ClkrootFunc3)
            }
            Lpi2cClockSel::Clk1M => {
                let freq = clocks.ensure_clk_1m_active(&self.power)?;
                #[cfg(feature = "mcxa2xx")]
                let mux = Lpi2cClkselMux::ClkrootFunc5;
                #[cfg(feature = "mcxa5xx")]
                let mux = Lpi2cClkselMux::I5ClkrootFunc5;

                (freq, mux)
            }
            #[cfg(feature = "mcxa2xx")]
            Lpi2cClockSel::Pll1ClkDiv => {
                let freq = clocks.ensure_pll1_clk_div_active(&self.power)?;
                (freq, Lpi2cClkselMux::ClkrootFunc6)
            }
            Lpi2cClockSel::None => {
                // no ClkrootFunc7, just write manually for now
                clksel.write(|w| w.0 = 0b111);
                clkdiv.modify(|w| {
                    w.set_reset(ClkdivReset::Off);
                    w.set_halt(ClkdivHalt::Off);
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

        #[cfg(feature = "mcxa2xx")]
        let fmax = match power {
            VddLevel::MidDriveMode => 25_000_000,
            VddLevel::OverDriveMode => 60_000_000,
        };

        #[cfg(feature = "mcxa5xx")]
        let fmax = match power {
            VddLevel::MidDriveMode => 25_000_000,
            VddLevel::NormalMode | VddLevel::OverDriveMode => 64_000_000,
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
    /// FRO180M/FRO192M/FRO_HF/FIRC clock source, passed through divider
    /// "fro_hf_div"
    FroHfDiv,
    /// SOSC/XTAL/EXTAL clock source
    #[cfg(not(feature = "sosc-as-gpio"))]
    ClkIn,
    /// FRO16K/clk_16k source
    #[cfg(feature = "mcxa2xx")]
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
        let mrcc0 = crate::pac::MRCC0;

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
                // TODO: fix PAC names for consistency
                #[cfg(feature = "mcxa2xx")]
                let mux = LpuartClkselMux::ClkrootFunc0;
                #[cfg(feature = "mcxa5xx")]
                let mux = LpuartClkselMux::I0ClkrootSircDiv;

                (freq, mux)
            }
            LpuartClockSel::FroHfDiv => {
                let freq = clocks.ensure_fro_hf_div_active(&self.power)?;
                // TODO: fix PAC names for consistency
                #[cfg(feature = "mcxa2xx")]
                let mux = LpuartClkselMux::ClkrootFunc2;
                #[cfg(feature = "mcxa5xx")]
                let mux = LpuartClkselMux::I2ClkrootFircDiv;

                (freq, mux)
            }
            #[cfg(not(feature = "sosc-as-gpio"))]
            LpuartClockSel::ClkIn => {
                let freq = clocks.ensure_clk_in_active(&self.power)?;
                // TODO: fix PAC names for consistency
                #[cfg(feature = "mcxa2xx")]
                let mux = LpuartClkselMux::ClkrootFunc3;
                #[cfg(feature = "mcxa5xx")]
                let mux = LpuartClkselMux::I3ClkrootSosc;

                (freq, mux)
            }
            #[cfg(feature = "mcxa2xx")]
            LpuartClockSel::Clk16K => {
                let freq = clocks.ensure_clk_16k_vdd_core_active(&self.power)?;

                // TODO: fix PAC names for consistency
                #[cfg(feature = "mcxa2xx")]
                let mux = LpuartClkselMux::ClkrootFunc4;
                // #[cfg(feature = "mcxa5xx")]
                // let mux = LpuartClkselMux::I4_CLKROOT_LPOSC;

                (freq, mux)
            }
            LpuartClockSel::Clk1M => {
                let freq = clocks.ensure_clk_1m_active(&self.power)?;
                // TODO: fix PAC names for consistency
                #[cfg(feature = "mcxa2xx")]
                let mux = LpuartClkselMux::ClkrootFunc5;
                #[cfg(feature = "mcxa5xx")]
                let mux = LpuartClkselMux::I5Clkroot1m;

                (freq, mux)
            }
            LpuartClockSel::Pll1ClkDiv => {
                let freq = clocks.ensure_pll1_clk_div_active(&self.power)?;
                // TODO: fix PAC names for consistency
                #[cfg(feature = "mcxa2xx")]
                let mux = LpuartClkselMux::ClkrootFunc6;
                #[cfg(feature = "mcxa5xx")]
                let mux = LpuartClkselMux::I6ClkrootSpllDiv;

                (freq, mux)
            }
            LpuartClockSel::None => {
                // no ClkrootFunc7, just write manually for now
                clksel.write(|w| w.set_mux(LpuartClkselMux::_RESERVED_7));
                clkdiv.modify(|w| {
                    w.set_reset(ClkdivReset::On);
                    w.set_halt(ClkdivHalt::On);
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
        #[cfg(feature = "mcxa2xx")]
        let fmax = match power {
            VddLevel::MidDriveMode => 45_000_000,
            VddLevel::OverDriveMode => 180_000_000,
        };
        #[cfg(feature = "mcxa5xx")]
        let fmax = match power {
            VddLevel::MidDriveMode => 50_000_000,
            VddLevel::NormalMode => 150_000_000,
            VddLevel::OverDriveMode => 200_000_000,
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
    /// FRO180M/FRO192M/FRO_HF/FIRC clock source, passed through divider
    /// "fro_hf_div"
    FroHfDiv,
    /// SOSC/XTAL/EXTAL clock source
    #[cfg(not(feature = "sosc-as-gpio"))]
    ClkIn,
    /// FRO16K/clk_16k source
    #[cfg(feature = "mcxa2xx")]
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
        let mrcc0 = crate::pac::MRCC0;

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
                // TODO: fix PAC names for consistency
                #[cfg(feature = "mcxa2xx")]
                let mux = CtimerClkselMux::ClkrootFunc0;
                #[cfg(feature = "mcxa5xx")]
                let mux = CtimerClkselMux::I0ClkrootSircDiv;

                (freq, mux)
            }
            CTimerClockSel::FroHfDiv => {
                let freq = clocks.ensure_fro_hf_div_active(&self.power)?;
                // TODO: fix PAC names for consistency
                #[cfg(feature = "mcxa2xx")]
                let mux = CtimerClkselMux::ClkrootFunc1;
                #[cfg(feature = "mcxa5xx")]
                let mux = CtimerClkselMux::I1ClkrootFircGated;

                (freq, mux)
            }
            #[cfg(not(feature = "sosc-as-gpio"))]
            CTimerClockSel::ClkIn => {
                let freq = clocks.ensure_clk_in_active(&self.power)?;
                // TODO: fix PAC names for consistency
                #[cfg(feature = "mcxa2xx")]
                let mux = CtimerClkselMux::ClkrootFunc3;
                #[cfg(feature = "mcxa5xx")]
                let mux = CtimerClkselMux::I3ClkrootSosc;

                (freq, mux)
            }
            #[cfg(feature = "mcxa2xx")]
            CTimerClockSel::Clk16K => {
                let freq = clocks.ensure_clk_16k_vdd_core_active(&self.power)?;

                // TODO: fix PAC names for consistency
                #[cfg(feature = "mcxa2xx")]
                let mux = CtimerClkselMux::ClkrootFunc4;
                // TODO: MCXA5xx uses "LPOSC", which can either be clk_16k or clk_32k.
                // We do not support this yet.
                // #[cfg(feature = "mcxa5xx")]
                // let mux = CtimerClkselMux::I4_CLKROOT_LPOSC;

                (freq, mux)
            }
            CTimerClockSel::Clk1M => {
                let freq = clocks.ensure_clk_1m_active(&self.power)?;
                // TODO: fix PAC names for consistency
                #[cfg(feature = "mcxa2xx")]
                let mux = CtimerClkselMux::ClkrootFunc5;
                #[cfg(feature = "mcxa5xx")]
                let mux = CtimerClkselMux::I5Clkroot1m;

                (freq, mux)
            }
            CTimerClockSel::Pll1ClkDiv => {
                let freq = clocks.ensure_pll1_clk_div_active(&self.power)?;
                // TODO: fix PAC names for consistency
                #[cfg(feature = "mcxa2xx")]
                let mux = CtimerClkselMux::ClkrootFunc6;
                #[cfg(feature = "mcxa5xx")]
                let mux = CtimerClkselMux::I6ClkrootSpllDiv;

                (freq, mux)
            }
            CTimerClockSel::None => {
                // no ClkrootFunc7, just write manually for now
                clksel.write(|w| w.set_mux(CtimerClkselMux::_RESERVED_7));
                clkdiv.modify(|w| {
                    w.set_reset(ClkdivReset::On);
                    w.set_halt(ClkdivHalt::On)
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
        #[cfg(feature = "mcxa2xx")]
        let fmax = match power {
            VddLevel::MidDriveMode => 25_000_000,
            VddLevel::OverDriveMode => 60_000_000,
        };
        #[cfg(feature = "mcxa5xx")]
        let fmax = match power {
            VddLevel::MidDriveMode => 50_000_000,
            VddLevel::NormalMode => 150_000_000,
            VddLevel::OverDriveMode => 200_000_000,
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
