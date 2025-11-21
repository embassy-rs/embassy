//! Peripheral Helpers
//!
//! The purpose of this module is to define the per-peripheral special handling
//! required from a clocking perspective. Different peripherals have different
//! selectable source clocks, and some peripherals have additional pre-dividers
//! that can be used.
//!
//! See the docs of [`SPConfHelper`] for more details.

use super::{ClockError, Clocks, PoweredClock};
use crate::pac;

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
    /// This method is called AFTER a given MRCC peripheral has been enabled (e.g. un-gated),
    /// but BEFORE the peripheral reset line is reset.
    ///
    /// This function should check that any relevant upstream clocks are enabled, are in a
    /// reasonable power state, and that the requested configuration can be made. If any of
    /// these checks fail, an `Err(ClockError)` should be returned, likely `ClockError::BadConfig`.
    ///
    /// This function SHOULD NOT make any changes to the system clock configuration, even
    /// unsafely, as this should remain static for the duration of the program.
    ///
    /// This function WILL be called in a critical section, care should be taken not to delay
    /// for an unreasonable amount of time.
    ///
    /// On success, this function MUST return an `Ok(freq)`, where `freq` is the frequency
    /// fed into the peripheral, taking into account the selected source clock, as well as
    /// any pre-divisors.
    fn post_enable_config(&self, clocks: &Clocks) -> Result<u32, ClockError>;
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
        if n > 0b1111 {
            None
        } else {
            Some(Self(n))
        }
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
    fn post_enable_config(&self, _clocks: &Clocks) -> Result<u32, ClockError> {
        Err(ClockError::UnimplementedConfig)
    }
}

/// A basic type that always returns `Ok(0)` when `post_enable_config` is called.
///
/// This should only be used for peripherals that are "ambiently" clocked, like `PORTn`
/// peripherals, which have no selectable/configurable source clock.
pub struct NoConfig;
impl SPConfHelper for NoConfig {
    fn post_enable_config(&self, _clocks: &Clocks) -> Result<u32, ClockError> {
        Ok(0)
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
    fn post_enable_config(&self, clocks: &Clocks) -> Result<u32, ClockError> {
        // check that source is suitable
        let mrcc0 = unsafe { pac::Mrcc0::steal() };
        use mcxa_pac::mrcc0::mrcc_lpuart0_clksel::Mux;

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
                (freq, Mux::ClkrootFunc0)
            }
            LpuartClockSel::FroHfDiv => {
                let freq = clocks.ensure_fro_hf_div_active(&self.power)?;
                (freq, Mux::ClkrootFunc2)
            }
            LpuartClockSel::ClkIn => {
                let freq = clocks.ensure_clk_in_active(&self.power)?;
                (freq, Mux::ClkrootFunc3)
            }
            LpuartClockSel::Clk16K => {
                let freq = clocks.ensure_clk_16k_vdd_core_active(&self.power)?;
                (freq, Mux::ClkrootFunc4)
            }
            LpuartClockSel::Clk1M => {
                let freq = clocks.ensure_clk_1m_active(&self.power)?;
                (freq, Mux::ClkrootFunc5)
            }
            LpuartClockSel::Pll1ClkDiv => {
                let freq = clocks.ensure_pll1_clk_div_active(&self.power)?;
                (freq, Mux::ClkrootFunc6)
            }
            LpuartClockSel::None => unsafe {
                // no ClkrootFunc7, just write manually for now
                clksel.write(|w| w.bits(0b111));
                clkdiv.modify(|_r, w| {
                    w.reset().on();
                    w.halt().on();
                    w
                });
                return Ok(0);
            },
        };

        // set clksel
        clksel.modify(|_r, w| w.mux().variant(variant));

        // Set up clkdiv
        clkdiv.modify(|_r, w| {
            w.halt().on();
            w.reset().on();
            unsafe { w.div().bits(self.div.into_bits()) };
            w
        });
        clkdiv.modify(|_r, w| {
            w.halt().off();
            w.reset().off();
            w
        });

        while clkdiv.read().unstab().is_on() {}

        Ok(freq / self.div.into_divisor())
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
    fn post_enable_config(&self, clocks: &Clocks) -> Result<u32, ClockError> {
        let mrcc0 = unsafe { pac::Mrcc0::steal() };
        Ok(match self.source {
            OstimerClockSel::Clk16kVddCore => {
                let freq = clocks.ensure_clk_16k_vdd_core_active(&self.power)?;
                mrcc0.mrcc_ostimer0_clksel().write(|w| w.mux().clkroot_16k());
                freq
            }
            OstimerClockSel::Clk1M => {
                let freq = clocks.ensure_clk_1m_active(&self.power)?;
                mrcc0.mrcc_ostimer0_clksel().write(|w| w.mux().clkroot_1m());
                freq
            }
            OstimerClockSel::None => {
                mrcc0.mrcc_ostimer0_clksel().write(|w| unsafe { w.mux().bits(0b11) });
                0
            }
        })
    }
}

//
// Adc
//

/// Selectable clocks for the ADC peripheral
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AdcClockSel {
    /// Divided `fro_lf`/`clk_12m`/FRO12M source
    FroLfDiv,
    /// Gated `fro_hf`/`FRO180M` source
    FroHf,
    /// External Clock Source
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
    fn post_enable_config(&self, clocks: &Clocks) -> Result<u32, ClockError> {
        use mcxa_pac::mrcc0::mrcc_adc_clksel::Mux;
        let mrcc0 = unsafe { pac::Mrcc0::steal() };
        let (freq, variant) = match self.source {
            AdcClockSel::FroLfDiv => {
                let freq = clocks.ensure_fro_lf_div_active(&self.power)?;
                (freq, Mux::ClkrootFunc0)
            }
            AdcClockSel::FroHf => {
                let freq = clocks.ensure_fro_hf_active(&self.power)?;
                (freq, Mux::ClkrootFunc1)
            }
            AdcClockSel::ClkIn => {
                let freq = clocks.ensure_clk_in_active(&self.power)?;
                (freq, Mux::ClkrootFunc3)
            }
            AdcClockSel::Clk1M => {
                let freq = clocks.ensure_clk_1m_active(&self.power)?;
                (freq, Mux::ClkrootFunc5)
            }
            AdcClockSel::Pll1ClkDiv => {
                let freq = clocks.ensure_pll1_clk_div_active(&self.power)?;
                (freq, Mux::ClkrootFunc6)
            }
            AdcClockSel::None => {
                mrcc0.mrcc_adc_clksel().write(|w| unsafe {
                    // no ClkrootFunc7, just write manually for now
                    w.mux().bits(0b111)
                });
                mrcc0.mrcc_adc_clkdiv().modify(|_r, w| {
                    w.reset().on();
                    w.halt().on();
                    w
                });
                return Ok(0);
            }
        };

        // set clksel
        mrcc0.mrcc_adc_clksel().modify(|_r, w| w.mux().variant(variant));

        // Set up clkdiv
        mrcc0.mrcc_adc_clkdiv().modify(|_r, w| {
            w.halt().on();
            w.reset().on();
            unsafe { w.div().bits(self.div.into_bits()) };
            w
        });
        mrcc0.mrcc_adc_clkdiv().modify(|_r, w| {
            w.halt().off();
            w.reset().off();
            w
        });

        while mrcc0.mrcc_adc_clkdiv().read().unstab().is_on() {}

        Ok(freq / self.div.into_divisor())
    }
}
