//! # Clock configuration for the RP2040 and RP235x microcontrollers.
//!
//! # Clock Configuration API
//!
//! This module provides both high-level convenience functions and low-level manual
//! configuration options for the RP2040 clock system.
//!
//! ## High-Level Convenience Functions
//!
//! For most users, these functions provide an easy way to configure clocks:
//!
//! - `ClockConfig::system_freq(125_000_000)` - Set system clock to a specific frequency with automatic voltage scaling
//! - `ClockConfig::crystal(12_000_000)` - Default configuration with 12MHz crystal giving 125MHz system clock
//!
//! ## Manual Configuration
//!
//! For advanced users who need precise control:
//!
//! ```rust,ignore
//! // Start with default configuration and customize it
//! let mut config = ClockConfig::default();
//!
//! // Set custom PLL parameters
//! config.xosc = Some(XoscConfig {
//!     hz: 12_000_000,
//!     sys_pll: Some(PllConfig {
//!         refdiv: 1,
//!         fbdiv: 200,
//!         post_div1: 6,
//!         post_div2: 2,
//!     }),
//!     // ... other fields
//! });
//!
//! // Set voltage for overclocking
//! config.core_voltage = CoreVoltage::V1_15;
//! ```
//!
//! ## Examples
//!
//! ### Standard 125MHz (rp2040) or 150Mhz (rp235x) configuration
//! ```rust,ignore
//! let config = ClockConfig::crystal(12_000_000);
//! ```
//!
//! Or using the default configuration:
//! ```rust,ignore
//! let config = ClockConfig::default();
//! ```
//!
//! ### Overclock to 200MHz
//! ```rust,ignore
//! let config = ClockConfig::system_freq(200_000_000);
//! ```
//!
//! ### Manual configuration for advanced scenarios
//! ```rust,ignore
//! use embassy_rp::clocks::{ClockConfig, XoscConfig, PllConfig, CoreVoltage};
//!
//! // Start with defaults and customize
//! let mut config = ClockConfig::default();
//! config.core_voltage = CoreVoltage::V1_15;
//! // Set other parameters as needed...
//! ```

use core::arch::asm;
use core::marker::PhantomData;
#[cfg(feature = "rp2040")]
use core::sync::atomic::AtomicU16;
use core::sync::atomic::{AtomicU32, Ordering};

use pac::clocks::vals::*;

use crate::gpio::{AnyPin, SealedPin};
use crate::pac::common::{Reg, RW};
use crate::{pac, reset, Peri};

// NOTE: all gpin handling is commented out for future reference.
// gpin is not usually safe to use during the boot init() call, so it won't
// be very useful until we have runtime clock reconfiguration. once this
// happens we can resurrect the commented-out gpin bits.

/// Clock error types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ClockError {
    /// PLL failed to lock within the timeout period.
    PllLockTimedOut,
    /// Could not find valid PLL parameters for system clock.
    InvalidPllParameters,
    /// Reading the core voltage failed due to an unexpected value in the register.
    UnexpectedCoreVoltageRead,
}

struct Clocks {
    xosc: AtomicU32,
    sys: AtomicU32,
    reference: AtomicU32,
    pll_sys: AtomicU32,
    pll_usb: AtomicU32,
    usb: AtomicU32,
    adc: AtomicU32,
    // See above re gpin handling being commented out
    // gpin0: AtomicU32,
    // gpin1: AtomicU32,
    rosc: AtomicU32,
    peri: AtomicU32,
    #[cfg(feature = "rp2040")]
    rtc: AtomicU16,
}

static CLOCKS: Clocks = Clocks {
    xosc: AtomicU32::new(0),
    sys: AtomicU32::new(0),
    reference: AtomicU32::new(0),
    pll_sys: AtomicU32::new(0),
    pll_usb: AtomicU32::new(0),
    usb: AtomicU32::new(0),
    adc: AtomicU32::new(0),
    // See above re gpin handling being commented out
    // gpin0: AtomicU32::new(0),
    // gpin1: AtomicU32::new(0),
    rosc: AtomicU32::new(0),
    peri: AtomicU32::new(0),
    #[cfg(feature = "rp2040")]
    rtc: AtomicU16::new(0),
};

/// Peripheral clock sources.
#[repr(u8)]
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PeriClkSrc {
    /// SYS.
    Sys = ClkPeriCtrlAuxsrc::CLK_SYS as _,
    /// PLL SYS.
    PllSys = ClkPeriCtrlAuxsrc::CLKSRC_PLL_SYS as _,
    /// PLL USB.
    PllUsb = ClkPeriCtrlAuxsrc::CLKSRC_PLL_USB as _,
    /// ROSC.
    Rosc = ClkPeriCtrlAuxsrc::ROSC_CLKSRC_PH as _,
    /// XOSC.
    Xosc = ClkPeriCtrlAuxsrc::XOSC_CLKSRC as _,
    // See above re gpin handling being commented out
    // Gpin0 = ClkPeriCtrlAuxsrc::CLKSRC_GPIN0 as _ ,
    // Gpin1 = ClkPeriCtrlAuxsrc::CLKSRC_GPIN1 as _ ,
}

/// Core voltage regulator settings.
///
/// The voltage regulator can be configured for different output voltages.
/// Higher voltages allow for higher clock frequencies but increase power consumption and heat.
#[cfg(feature = "rp2040")]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CoreVoltage {
    /// 0.80V
    V0_80 = 0b0000,
    /// 0.85V
    V0_85 = 0b0110,
    /// 0.90V
    V0_90 = 0b0111,
    /// 0.95V
    V0_95 = 0b1000,
    /// 1.00V
    V1_00 = 0b1001,
    /// 1.05V
    V1_05 = 0b1010,
    /// 1.10V - Default voltage level
    V1_10 = 0b1011,
    /// 1.15V - Required for overclocking to 133-200MHz
    V1_15 = 0b1100,
    /// 1.20V
    V1_20 = 0b1101,
    /// 1.25V
    V1_25 = 0b1110,
    /// 1.30V
    V1_30 = 0b1111,
}

/// Core voltage regulator settings.
///
/// The voltage regulator can be configured for different output voltages.
/// Higher voltages allow for higher clock frequencies but increase power consumption and heat.
///
/// **Note**: The maximum voltage is 1.30V, unless unlocked by setting unless the voltage limit
/// is disabled using the disable_voltage_limit field in the vreg_ctrl register. For lack of practical use at this
/// point in time, this is not implemented here. So the maximum voltage in this enum is 1.30V for now.
#[cfg(feature = "_rp235x")]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CoreVoltage {
    /// 0.55V
    V0_55 = 0b00000,
    /// 0.60V
    V0_60 = 0b00001,
    /// 0.65V
    V0_65 = 0b00010,
    /// 0.70V
    V0_70 = 0b00011,
    /// 0.75V
    V0_75 = 0b00100,
    /// 0.80V
    V0_80 = 0b00101,
    /// 0.85V
    V0_85 = 0b00110,
    /// 0.90V
    V0_90 = 0b00111,
    /// 0.95V
    V0_95 = 0b01000,
    /// 1.00V
    V1_00 = 0b01001,
    /// 1.05V
    V1_05 = 0b01010,
    /// 1.10V - Default voltage level
    V1_10 = 0b01011,
    /// 1.15V
    V1_15 = 0b01100,
    /// 1.20V
    V1_20 = 0b01101,
    /// 1.25V
    V1_25 = 0b01110,
    /// 1.30V
    V1_30 = 0b01111,
}

impl CoreVoltage {
    /// Get the recommended Brown-Out Detection (BOD) setting for this voltage.
    /// Sets the BOD threshold to approximately 80% of the core voltage.
    fn recommended_bod(self) -> u8 {
        #[cfg(feature = "rp2040")]
        match self {
            CoreVoltage::V0_80 => 0b0100, // 0.645V (~81% of 0.80V)
            CoreVoltage::V0_85 => 0b0101, // 0.688V (~81% of 0.85V)
            CoreVoltage::V0_90 => 0b0110, // 0.731V (~81% of 0.90V)
            CoreVoltage::V0_95 => 0b0111, // 0.774V (~81% of 0.95V)
            CoreVoltage::V1_00 => 0b1000, // 0.817V (~82% of 1.00V)
            CoreVoltage::V1_05 => 0b1000, // 0.817V (~78% of 1.05V)
            CoreVoltage::V1_10 => 0b1001, // 0.860V (~78% of 1.10V), the default
            CoreVoltage::V1_15 => 0b1010, // 0.903V (~79% of 1.15V)
            CoreVoltage::V1_20 => 0b1011, // 0.946V (~79% of 1.20V)
            CoreVoltage::V1_25 => 0b1100, // 0.989V (~79% of 1.25V)
            CoreVoltage::V1_30 => 0b1101, // 1.032V (~79% of 1.30V)
        }
        #[cfg(feature = "_rp235x")]
        match self {
            CoreVoltage::V0_55 => 0b00001, // 0.516V (~94% of 0.55V)
            CoreVoltage::V0_60 => 0b00010, // 0.559V (~93% of 0.60V)
            CoreVoltage::V0_65 => 0b00011, // 0.602V (~93% of 0.65V)
            CoreVoltage::V0_70 => 0b00011, // 0.602V (~86% of 0.70V)
            CoreVoltage::V0_75 => 0b00100, // 0.645V (~86% of 0.75V)
            CoreVoltage::V0_80 => 0b00101, // 0.688V (~86% of 0.80V)
            CoreVoltage::V0_85 => 0b00110, // 0.731V (~86% of 0.85V)
            CoreVoltage::V0_90 => 0b00110, // 0.731V (~81% of 0.90V)
            CoreVoltage::V0_95 => 0b00111, // 0.774V (~81% of 0.95V)
            CoreVoltage::V1_00 => 0b01000, // 0.817V (~82% of 1.00V)
            CoreVoltage::V1_05 => 0b01000, // 0.817V (~78% of 1.05V)
            CoreVoltage::V1_10 => 0b01001, // 0.860V (~78% of 1.10V), the default
            CoreVoltage::V1_15 => 0b01001, // 0.860V (~75% of 1.15V)
            CoreVoltage::V1_20 => 0b01010, // 0.903V (~75% of 1.20V)
            CoreVoltage::V1_25 => 0b01010, // 0.903V (~72% of 1.25V)
            CoreVoltage::V1_30 => 0b01011, // 0.946V (~73% of 1.30V)
                                            // all others: 0.946V (see CoreVoltage: we do not support setting Voltages higher than 1.30V at this point)
        }
    }
}

/// CLock configuration.
#[non_exhaustive]
pub struct ClockConfig {
    /// Ring oscillator configuration.
    pub rosc: Option<RoscConfig>,
    /// External oscillator configuration.
    pub xosc: Option<XoscConfig>,
    /// Reference clock configuration.
    pub ref_clk: RefClkConfig,
    /// System clock configuration.
    pub sys_clk: SysClkConfig,
    /// Peripheral clock source configuration.
    pub peri_clk_src: Option<PeriClkSrc>,
    /// USB clock configuration.
    pub usb_clk: Option<UsbClkConfig>,
    /// ADC clock configuration.
    pub adc_clk: Option<AdcClkConfig>,
    /// RTC clock configuration.
    #[cfg(feature = "rp2040")]
    pub rtc_clk: Option<RtcClkConfig>,
    /// Core voltage scaling. Defaults to 1.10V.
    pub core_voltage: CoreVoltage,
    /// Voltage stabilization delay in microseconds.
    /// If not set, defaults will be used based on voltage level.
    pub voltage_stabilization_delay_us: Option<u32>,
    // See above re gpin handling being commented out
    // gpin0: Option<(u32, Gpin<'static, AnyPin>)>,
    // gpin1: Option<(u32, Gpin<'static, AnyPin>)>,
}

impl Default for ClockConfig {
    /// Creates a minimal default configuration with safe values.
    ///
    /// This configuration uses the ring oscillator (ROSC) as the clock source
    /// and sets minimal defaults that guarantee a working system. It's intended
    /// as a starting point for manual configuration.
    ///
    /// Most users should use one of the more specific configuration functions:
    /// - `ClockConfig::crystal()` - Standard configuration with external crystal
    /// - `ClockConfig::rosc()` - Configuration using only the internal oscillator
    /// - `ClockConfig::system_freq()` - Configuration for a specific system frequency
    fn default() -> Self {
        Self {
            rosc: None,
            xosc: None,
            ref_clk: RefClkConfig {
                src: RefClkSrc::Rosc,
                div: 1,
            },
            sys_clk: SysClkConfig {
                src: SysClkSrc::Rosc,
                div_int: 1,
                div_frac: 0,
            },
            peri_clk_src: None,
            usb_clk: None,
            adc_clk: None,
            #[cfg(feature = "rp2040")]
            rtc_clk: None,
            core_voltage: CoreVoltage::V1_10,
            voltage_stabilization_delay_us: None,
            // See above re gpin handling being commented out
            // gpin0: None,
            // gpin1: None,
        }
    }
}

impl ClockConfig {
    /// Clock configuration derived from external crystal.
    ///
    /// This uses default settings for most parameters, suitable for typical use cases.
    /// For manual control of PLL parameters, use `new_manual()` or modify the struct fields directly.
    pub fn crystal(crystal_hz: u32) -> Self {
        Self {
            rosc: Some(RoscConfig {
                hz: 6_500_000,
                range: RoscRange::Medium,
                drive_strength: [0; 8],
                div: 16,
            }),
            xosc: Some(XoscConfig {
                hz: crystal_hz,
                sys_pll: Some(PllConfig {
                    refdiv: 1,
                    fbdiv: 125,
                    #[cfg(feature = "rp2040")]
                    post_div1: 6,
                    #[cfg(feature = "_rp235x")]
                    post_div1: 5,
                    post_div2: 2,
                }),
                usb_pll: Some(PllConfig {
                    refdiv: 1,
                    fbdiv: 120,
                    post_div1: 6,
                    post_div2: 5,
                }),
                delay_multiplier: 128,
            }),
            ref_clk: RefClkConfig {
                src: RefClkSrc::Xosc,
                div: 1,
            },
            sys_clk: SysClkConfig {
                src: SysClkSrc::PllSys,
                div_int: 1,
                div_frac: 0,
            },
            peri_clk_src: Some(PeriClkSrc::Sys),
            // CLK USB = PLL USB (48MHz) / 1 = 48MHz
            usb_clk: Some(UsbClkConfig {
                src: UsbClkSrc::PllUsb,
                div: 1,
                phase: 0,
            }),
            // CLK ADC = PLL USB (48MHZ) / 1 = 48MHz
            adc_clk: Some(AdcClkConfig {
                src: AdcClkSrc::PllUsb,
                div: 1,
                phase: 0,
            }),
            // CLK RTC = PLL USB (48MHz) / 1024 = 46875Hz
            #[cfg(feature = "rp2040")]
            rtc_clk: Some(RtcClkConfig {
                src: RtcClkSrc::PllUsb,
                div_int: 1024,
                div_frac: 0,
                phase: 0,
            }),
            core_voltage: CoreVoltage::V1_10, // Use hardware default (1.10V)
            voltage_stabilization_delay_us: None,
            // See above re gpin handling being commented out
            // gpin0: None,
            // gpin1: None,
        }
    }

    /// Clock configuration from internal oscillator.
    pub fn rosc() -> Self {
        Self {
            rosc: Some(RoscConfig {
                hz: 140_000_000,
                range: RoscRange::High,
                drive_strength: [0; 8],
                div: 1,
            }),
            xosc: None,
            ref_clk: RefClkConfig {
                src: RefClkSrc::Rosc,
                div: 1,
            },
            sys_clk: SysClkConfig {
                src: SysClkSrc::Rosc,
                div_int: 1,
                div_frac: 0,
            },
            peri_clk_src: Some(PeriClkSrc::Rosc),
            usb_clk: None,
            // CLK ADC = ROSC (140MHz) / 3 ≅ 48MHz
            adc_clk: Some(AdcClkConfig {
                src: AdcClkSrc::Rosc,
                div: 3,
                phase: 0,
            }),
            // CLK RTC = ROSC (140MHz) / 2986.667969 ≅ 46875Hz
            #[cfg(feature = "rp2040")]
            rtc_clk: Some(RtcClkConfig {
                src: RtcClkSrc::Rosc,
                div_int: 2986,
                div_frac: 171,
                phase: 0,
            }),
            core_voltage: CoreVoltage::V1_10, // Use hardware default (1.10V)
            voltage_stabilization_delay_us: None,
            // See above re gpin handling being commented out
            // gpin0: None,
            // gpin1: None,
        }
    }

    /// Configure clocks derived from an external crystal with specific system frequency.
    ///
    /// This function calculates optimal PLL parameters to achieve the requested system
    /// frequency. This only works for the usual 12MHz crystal. In case a different crystal is used,
    /// You will have to set the PLL parameters manually.
    ///
    /// # Arguments
    ///
    /// * `sys_freq_hz` - The desired system clock frequency in Hz
    ///
    /// # Returns
    ///
    /// A ClockConfig configured to achieve the requested system frequency using the
    /// the usual 12Mhz crystal, or an error if no valid parameters can be found.
    ///
    /// # Note on core voltage:
    ///
    /// **For RP2040**:
    /// To date the only officially documented core voltages (see Datasheet section 2.15.3.1. Instances) are:
    /// - Up to 133MHz: V1_10 (default)
    /// - Above 133MHz: V1_15, but in the context of the datasheet covering reaching up to 200Mhz
    /// That way all other frequencies below 133MHz or above 200MHz are not explicitly documented and not covered here.
    /// In case You want to go below 133MHz or above 200MHz and want a different voltage, You will have to set that manually and with caution.
    ///
    /// **For RP235x**:
    /// At this point in time there is no official manufacturer endorsement for running the chip on other core voltages and/or other clock speeds than the defaults.
    /// Using this function is experimental and may not work as expected or even damage the chip.
    ///
    /// # Returns
    ///
    /// A Result containing either the configured ClockConfig or a ClockError.
    pub fn system_freq(hz: u32) -> Result<Self, ClockError> {
        // Start with the standard configuration from crystal()
        const DEFAULT_CRYSTAL_HZ: u32 = 12_000_000;
        let mut config = Self::crystal(DEFAULT_CRYSTAL_HZ);

        // No need to modify anything if target frequency is already 125MHz
        // (which is what crystal() configures by default)
        #[cfg(feature = "rp2040")]
        if hz == 125_000_000 {
            return Ok(config);
        }
        #[cfg(feature = "_rp235x")]
        if hz == 150_000_000 {
            return Ok(config);
        }

        // Find optimal PLL parameters for the requested frequency
        let sys_pll_params = find_pll_params(DEFAULT_CRYSTAL_HZ, hz).ok_or(ClockError::InvalidPllParameters)?;

        // Replace the sys_pll configuration with our custom parameters
        if let Some(xosc) = &mut config.xosc {
            xosc.sys_pll = Some(sys_pll_params);
        }

        // Set the voltage scale based on the target frequency
        // Higher frequencies require higher voltage
        #[cfg(feature = "rp2040")]
        {
            config.core_voltage = match hz {
                freq if freq > 133_000_000 => CoreVoltage::V1_15,
                _ => CoreVoltage::V1_10, // Use default voltage (V1_10)
            };
        }
        #[cfg(feature = "_rp235x")]
        {
            config.core_voltage = match hz {
                // There is no official support for running the chip on other core voltages and/or other clock speeds than the defaults.
                // So for now we have not way of knowing what the voltage should be. Change this if the manufacturer provides more information.
                _ => CoreVoltage::V1_10, // Use default voltage (V1_10)
            };
        }

        Ok(config)
    }

    /// Configure with manual PLL settings for full control over system clock
    ///
    /// This method provides a simple way to configure the system with custom PLL parameters
    /// without needing to understand the full nested configuration structure.
    ///
    /// # Arguments
    ///
    /// * `xosc_hz` - The frequency of the external crystal in Hz
    /// * `pll_config` - The PLL configuration parameters to achieve desired frequency
    /// * `core_voltage` - Voltage scaling for overclocking (required for >133MHz)
    ///
    /// # Returns
    ///
    /// A ClockConfig configured with the specified PLL parameters
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // Configure for 200MHz operation
    /// let config = Config::default();
    /// config.clocks = ClockConfig::manual_pll(
    ///     12_000_000,
    ///     PllConfig {
    ///         refdiv: 1,    // Reference divider (12 MHz / 1 = 12 MHz)
    ///         fbdiv: 100,   // Feedback divider (12 MHz * 100 = 1200 MHz VCO)
    ///         post_div1: 3, // First post divider (1200 MHz / 3 = 400 MHz)
    ///         post_div2: 2, // Second post divider (400 MHz / 2 = 200 MHz)
    ///     },
    ///     CoreVoltage::V1_15
    /// );
    /// ```
    #[cfg(feature = "rp2040")]
    pub fn manual_pll(xosc_hz: u32, pll_config: PllConfig, core_voltage: CoreVoltage) -> Self {
        // Validate PLL parameters
        assert!(pll_config.is_valid(xosc_hz), "Invalid PLL parameters");

        let mut config = Self::default();

        config.xosc = Some(XoscConfig {
            hz: xosc_hz,
            sys_pll: Some(pll_config),
            usb_pll: Some(PllConfig {
                refdiv: 1,
                fbdiv: 120,
                post_div1: 6,
                post_div2: 5,
            }),
            delay_multiplier: 128,
        });

        config.ref_clk = RefClkConfig {
            src: RefClkSrc::Xosc,
            div: 1,
        };

        config.sys_clk = SysClkConfig {
            src: SysClkSrc::PllSys,
            div_int: 1,
            div_frac: 0,
        };

        config.core_voltage = core_voltage;
        config.peri_clk_src = Some(PeriClkSrc::Sys);

        // Set reasonable defaults for other clocks
        config.usb_clk = Some(UsbClkConfig {
            src: UsbClkSrc::PllUsb,
            div: 1,
            phase: 0,
        });

        config.adc_clk = Some(AdcClkConfig {
            src: AdcClkSrc::PllUsb,
            div: 1,
            phase: 0,
        });

        config.rtc_clk = Some(RtcClkConfig {
            src: RtcClkSrc::PllUsb,
            div_int: 1024,
            div_frac: 0,
            phase: 0,
        });

        config
    }
}

/// ROSC freq range.
#[repr(u16)]
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum RoscRange {
    /// Low range.
    Low = pac::rosc::vals::FreqRange::LOW.0,
    /// Medium range (1.33x low)
    Medium = pac::rosc::vals::FreqRange::MEDIUM.0,
    /// High range (2x low)
    High = pac::rosc::vals::FreqRange::HIGH.0,
    /// Too high. Should not be used.
    TooHigh = pac::rosc::vals::FreqRange::TOOHIGH.0,
}

/// On-chip ring oscillator configuration.
pub struct RoscConfig {
    /// Final frequency of the oscillator, after the divider has been applied.
    /// The oscillator has a nominal frequency of 6.5MHz at medium range with
    /// divider 16 and all drive strengths set to 0, other values should be
    /// measured in situ.
    pub hz: u32,
    /// Oscillator range.
    pub range: RoscRange,
    /// Drive strength for oscillator.
    pub drive_strength: [u8; 8],
    /// Output divider.
    pub div: u16,
}

/// Crystal oscillator configuration.
pub struct XoscConfig {
    /// Final frequency of the oscillator.
    pub hz: u32,
    /// Configuring PLL for the system clock.
    pub sys_pll: Option<PllConfig>,
    /// Configuring PLL for the USB clock.
    pub usb_pll: Option<PllConfig>,
    /// Multiplier for the startup delay.
    pub delay_multiplier: u32,
}

/// PLL configuration.
#[derive(Clone, Copy, Debug)]
pub struct PllConfig {
    /// Reference divisor.
    pub refdiv: u8,
    /// Feedback divisor.
    pub fbdiv: u16,
    /// Output divisor 1.
    pub post_div1: u8,
    /// Output divisor 2.
    pub post_div2: u8,
}

impl PllConfig {
    /// Calculate the output frequency for this PLL configuration
    /// given an input frequency.
    pub fn output_frequency(&self, input_hz: u32) -> u32 {
        let ref_freq = input_hz / self.refdiv as u32;
        let vco_freq = ref_freq * self.fbdiv as u32;
        vco_freq / ((self.post_div1 * self.post_div2) as u32)
    }

    /// Check if this PLL configuration is valid for the given input frequency.
    pub fn is_valid(&self, input_hz: u32) -> bool {
        // Check divisor constraints
        if self.refdiv < 1 || self.refdiv > 63 {
            return false;
        }
        if self.fbdiv < 16 || self.fbdiv > 320 {
            return false;
        }
        if self.post_div1 < 1 || self.post_div1 > 7 {
            return false;
        }
        if self.post_div2 < 1 || self.post_div2 > 7 {
            return false;
        }
        if self.post_div2 > self.post_div1 {
            return false;
        }

        // Calculate reference frequency
        let ref_freq = input_hz / self.refdiv as u32;

        // Check reference frequency range
        if ref_freq < 5_000_000 || ref_freq > 800_000_000 {
            return false;
        }

        // Calculate VCO frequency
        let vco_freq = ref_freq * self.fbdiv as u32;

        // Check VCO frequency range
        vco_freq >= 750_000_000 && vco_freq <= 1_800_000_000
    }
}

/// Reference clock config.
pub struct RefClkConfig {
    /// Reference clock source.
    pub src: RefClkSrc,
    /// Reference clock divider.
    pub div: u8,
}

/// Reference clock source.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum RefClkSrc {
    /// XOSC.
    Xosc,
    /// ROSC.
    Rosc,
    /// PLL USB.
    PllUsb,
    // See above re gpin handling being commented out
    // Gpin0,
    // Gpin1,
}

/// SYS clock source.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SysClkSrc {
    /// REF.
    Ref,
    /// PLL SYS.
    PllSys,
    /// PLL USB.
    PllUsb,
    /// ROSC.
    Rosc,
    /// XOSC.
    Xosc,
    // See above re gpin handling being commented out
    // Gpin0,
    // Gpin1,
}

/// SYS clock config.
pub struct SysClkConfig {
    /// SYS clock source.
    pub src: SysClkSrc,
    /// SYS clock divider.
    #[cfg(feature = "rp2040")]
    pub div_int: u32,
    /// SYS clock fraction.
    #[cfg(feature = "rp2040")]
    pub div_frac: u8,
    /// SYS clock divider.
    #[cfg(feature = "_rp235x")]
    pub div_int: u16,
    /// SYS clock fraction.
    #[cfg(feature = "_rp235x")]
    pub div_frac: u16,
}

/// USB clock source.
#[repr(u8)]
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum UsbClkSrc {
    /// PLL USB.
    PllUsb = ClkUsbCtrlAuxsrc::CLKSRC_PLL_USB as _,
    /// PLL SYS.
    PllSys = ClkUsbCtrlAuxsrc::CLKSRC_PLL_SYS as _,
    /// ROSC.
    Rosc = ClkUsbCtrlAuxsrc::ROSC_CLKSRC_PH as _,
    /// XOSC.
    Xosc = ClkUsbCtrlAuxsrc::XOSC_CLKSRC as _,
    // See above re gpin handling being commented out
    // Gpin0 = ClkUsbCtrlAuxsrc::CLKSRC_GPIN0 as _ ,
    // Gpin1 = ClkUsbCtrlAuxsrc::CLKSRC_GPIN1 as _ ,
}

/// USB clock config.
pub struct UsbClkConfig {
    /// USB clock source.
    pub src: UsbClkSrc,
    /// USB clock divider.
    pub div: u8,
    /// USB clock phase.
    pub phase: u8,
}

/// ADC clock source.
#[repr(u8)]
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum AdcClkSrc {
    /// PLL USB.
    PllUsb = ClkAdcCtrlAuxsrc::CLKSRC_PLL_USB as _,
    /// PLL SYS.
    PllSys = ClkAdcCtrlAuxsrc::CLKSRC_PLL_SYS as _,
    /// ROSC.
    Rosc = ClkAdcCtrlAuxsrc::ROSC_CLKSRC_PH as _,
    /// XOSC.
    Xosc = ClkAdcCtrlAuxsrc::XOSC_CLKSRC as _,
    // See above re gpin handling being commented out
    // Gpin0 = ClkAdcCtrlAuxsrc::CLKSRC_GPIN0 as _ ,
    // Gpin1 = ClkAdcCtrlAuxsrc::CLKSRC_GPIN1 as _ ,
}

/// ADC clock config.
pub struct AdcClkConfig {
    /// ADC clock source.
    pub src: AdcClkSrc,
    /// ADC clock divider.
    pub div: u8,
    /// ADC clock phase.
    pub phase: u8,
}

/// RTC clock source.
#[repr(u8)]
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg(feature = "rp2040")]
pub enum RtcClkSrc {
    /// PLL USB.
    PllUsb = ClkRtcCtrlAuxsrc::CLKSRC_PLL_USB as _,
    /// PLL SYS.
    PllSys = ClkRtcCtrlAuxsrc::CLKSRC_PLL_SYS as _,
    /// ROSC.
    Rosc = ClkRtcCtrlAuxsrc::ROSC_CLKSRC_PH as _,
    /// XOSC.
    Xosc = ClkRtcCtrlAuxsrc::XOSC_CLKSRC as _,
    // See above re gpin handling being commented out
    // Gpin0 = ClkRtcCtrlAuxsrc::CLKSRC_GPIN0 as _ ,
    // Gpin1 = ClkRtcCtrlAuxsrc::CLKSRC_GPIN1 as _ ,
}

/// RTC clock config.
#[cfg(feature = "rp2040")]
pub struct RtcClkConfig {
    /// RTC clock source.
    pub src: RtcClkSrc,
    /// RTC clock divider.
    pub div_int: u32,
    /// RTC clock divider fraction.
    pub div_frac: u8,
    /// RTC clock phase.
    pub phase: u8,
}

/// Find valid PLL parameters (refdiv, fbdiv, post_div1, post_div2) for a target output frequency
/// based on the input frequency.
///
/// This function searches for the best PLL configuration to achieve the requested target frequency
/// while staying within the VCO frequency range of 750MHz to 1800MHz. It prioritizes stability
/// over exact frequency matching by using larger divisors where possible.
///
/// # Parameters
///
/// * `input_hz`: The input frequency in Hz (typically the crystal frequency, e.g. 12MHz for th most common one used on rp2040 boards)
/// * `target_hz`: The desired output frequency in Hz (e.g. 125MHz for standard RP2040 operation)
///
/// # Returns
///
/// * `Some(PllConfig)` if valid parameters were found
/// * `None` if no valid parameters could be found for the requested combination
///
/// # Example
///
/// ```rust,ignore
/// // Find parameters for 133MHz system clock from 12MHz crystal
/// let pll_params = find_pll_params(12_000_000, 133_000_000).unwrap();
/// ```
fn find_pll_params(input_hz: u32, target_hz: u32) -> Option<PllConfig> {
    // Fixed reference divider for system PLL
    const PLL_SYS_REFDIV: u8 = 1;

    // Calculate reference frequency
    let reference_freq = input_hz as u64 / PLL_SYS_REFDIV as u64;

    // Start from highest fbdiv for better stability (like SDK does)
    for fbdiv in (16..=320).rev() {
        let vco_freq = reference_freq * fbdiv;

        // Check VCO frequency is within valid range
        if vco_freq < 750_000_000 || vco_freq > 1_800_000_000 {
            continue;
        }

        // Try all possible postdiv combinations starting from larger values
        // (more conservative/stable approach)
        for post_div1 in (1..=7).rev() {
            for post_div2 in (1..=post_div1).rev() {
                let out_freq = vco_freq / (post_div1 * post_div2);

                // Check if we get the exact target frequency without remainder
                if out_freq == target_hz as u64 && (vco_freq % (post_div1 * post_div2) == 0) {
                    return Some(PllConfig {
                        refdiv: PLL_SYS_REFDIV,
                        fbdiv: fbdiv as u16,
                        post_div1: post_div1 as u8,
                        post_div2: post_div2 as u8,
                    });
                }
            }
        }
    }

    // If we couldn't find an exact match, find the closest match
    let mut best_config = None;
    let mut min_diff = u32::MAX;

    for fbdiv in (16..=320).rev() {
        let vco_freq = reference_freq * fbdiv;

        if vco_freq < 750_000_000 || vco_freq > 1_800_000_000 {
            continue;
        }

        for post_div1 in (1..=7).rev() {
            for post_div2 in (1..=post_div1).rev() {
                let out_freq = (vco_freq / (post_div1 * post_div2) as u64) as u32;
                let diff = if out_freq > target_hz {
                    out_freq - target_hz
                } else {
                    target_hz - out_freq
                };

                // If this is closer to the target, save it
                if diff < min_diff {
                    min_diff = diff;
                    best_config = Some(PllConfig {
                        refdiv: PLL_SYS_REFDIV,
                        fbdiv: fbdiv as u16,
                        post_div1: post_div1 as u8,
                        post_div2: post_div2 as u8,
                    });
                }
            }
        }
    }

    // Return the closest match if we found one
    best_config
}

/// safety: must be called exactly once at bootup
pub(crate) unsafe fn init(config: ClockConfig) {
    // Reset everything except:
    // - QSPI (we're using it to run this code!)
    // - PLLs (it may be suicide if that's what's clocking us)
    // - USB, SYSCFG (breaks usb-to-swd on core1)
    // - RTC (else there would be no more time...)
    let mut peris = reset::ALL_PERIPHERALS;
    peris.set_io_qspi(false);
    // peris.set_io_bank0(false); // might be suicide if we're clocked from gpin
    peris.set_pads_qspi(false);
    peris.set_pll_sys(false);
    peris.set_pll_usb(false);
    peris.set_usbctrl(false);
    peris.set_syscfg(false);
    //peris.set_rtc(false);
    reset::reset(peris);

    // Disable resus that may be enabled from previous software
    let c = pac::CLOCKS;
    c.clk_sys_resus_ctrl()
        .write_value(pac::clocks::regs::ClkSysResusCtrl(0));

    // Before we touch PLLs, switch sys and ref cleanly away from their aux sources.
    c.clk_sys_ctrl().modify(|w| w.set_src(ClkSysCtrlSrc::CLK_REF));
    #[cfg(feature = "rp2040")]
    while c.clk_sys_selected().read() != 1 {}
    #[cfg(feature = "_rp235x")]
    while c.clk_sys_selected().read() != pac::clocks::regs::ClkSysSelected(1) {}
    c.clk_ref_ctrl().modify(|w| w.set_src(ClkRefCtrlSrc::ROSC_CLKSRC_PH));
    #[cfg(feature = "rp2040")]
    while c.clk_ref_selected().read() != 1 {}
    #[cfg(feature = "_rp235x")]
    while c.clk_ref_selected().read() != pac::clocks::regs::ClkRefSelected(1) {}

    // Reset the PLLs
    let mut peris = reset::Peripherals(0);
    peris.set_pll_sys(true);
    peris.set_pll_usb(true);
    reset::reset(peris);
    reset::unreset_wait(peris);

    // See above re gpin handling being commented out
    // let gpin0_freq = config.gpin0.map_or(0, |p| {
    //     core::mem::forget(p.1);
    //     p.0
    // });
    // CLOCKS.gpin0.store(gpin0_freq, Ordering::Relaxed);
    // let gpin1_freq = config.gpin1.map_or(0, |p| {
    //     core::mem::forget(p.1);
    //     p.0
    // });
    // CLOCKS.gpin1.store(gpin1_freq, Ordering::Relaxed);

    let rosc_freq = match config.rosc {
        Some(config) => configure_rosc(config),
        None => 0,
    };
    CLOCKS.rosc.store(rosc_freq, Ordering::Relaxed);

    // Set Core Voltage, if we have config for it and we're not using the default
    {
        let voltage = config.core_voltage;

        #[cfg(feature = "rp2040")]
        let vreg = pac::VREG_AND_CHIP_RESET;
        #[cfg(feature = "_rp235x")]
        let vreg = pac::POWMAN;

        let current_vsel = vreg.vreg().read().vsel();
        let target_vsel = voltage as u8;

        // If the target voltage is different from the current one, we need to change it
        if target_vsel != current_vsel {
            // Set the voltage regulator to the target voltage
            #[cfg(feature = "rp2040")]
            vreg.vreg().modify(|w| w.set_vsel(target_vsel));
            #[cfg(feature = "_rp235x")]
            // For rp235x changes to the voltage regulator are protected by a password, see datasheet section 6.4 Power Management (POWMAN) Registers
            // The password is "5AFE" (0x5AFE), it must be set in the top 16 bits of the register
            vreg.vreg().modify(|w| {
                w.0 = (w.0 & 0x0000FFFF) | (0x5AFE << 16); // Set the password
                w.set_vsel(target_vsel);
                *w
            });

            // Wait for the voltage to stabilize. Use the provided delay or default based on voltage
            let settling_time_us = config.voltage_stabilization_delay_us.unwrap_or_else(|| {
                match voltage {
                    CoreVoltage::V1_15 => 1000,                                           // 1ms for 1.15V
                    CoreVoltage::V1_20 | CoreVoltage::V1_25 | CoreVoltage::V1_30 => 2000, // 2ms for higher voltages
                    _ => 0,                                                               // no delay for all others
                }
            });

            if settling_time_us != 0 {
                // Delay in microseconds, using the ROSC frequency to calculate cycles
                let cycles_per_us = rosc_freq / 1_000_000;
                let delay_cycles = settling_time_us * cycles_per_us;
                cortex_m::asm::delay(delay_cycles);
            }

            // Only now set the BOD level. At this point the voltage is considered stable.
            #[cfg(feature = "rp2040")]
            vreg.bod().write(|w| {
                w.set_vsel(voltage.recommended_bod());
                w.set_en(true); // Enable brownout detection
            });
            #[cfg(feature = "_rp235x")]
            vreg.bod().write(|w| {
                w.0 = (w.0 & 0x0000FFFF) | (0x5AFE << 16); // Set the password
                w.set_vsel(voltage.recommended_bod());
                w.set_en(true); // Enable brownout detection
            });
        }
    }

    let (xosc_freq, pll_sys_freq, pll_usb_freq) = match config.xosc {
        Some(config) => {
            // start XOSC
            start_xosc(config.hz, config.delay_multiplier);

            let pll_sys_freq = match config.sys_pll {
                Some(sys_pll_config) => match configure_pll(pac::PLL_SYS, config.hz, sys_pll_config) {
                    Ok(freq) => freq,
                    Err(e) => panic!("Failed to configure PLL_SYS: {:?}", e),
                },
                None => 0,
            };
            let pll_usb_freq = match config.usb_pll {
                Some(usb_pll_config) => match configure_pll(pac::PLL_USB, config.hz, usb_pll_config) {
                    Ok(freq) => freq,
                    Err(e) => panic!("Failed to configure PLL_USB: {:?}", e),
                },
                None => 0,
            };

            (config.hz, pll_sys_freq, pll_usb_freq)
        }
        None => (0, 0, 0),
    };

    CLOCKS.xosc.store(xosc_freq, Ordering::Relaxed);
    CLOCKS.pll_sys.store(pll_sys_freq, Ordering::Relaxed);
    CLOCKS.pll_usb.store(pll_usb_freq, Ordering::Relaxed);

    let (ref_src, ref_aux, clk_ref_freq) = {
        use {ClkRefCtrlAuxsrc as Aux, ClkRefCtrlSrc as Src};
        let div = config.ref_clk.div as u32;
        assert!(div >= 1 && div <= 4);
        match config.ref_clk.src {
            RefClkSrc::Xosc => (Src::XOSC_CLKSRC, Aux::CLKSRC_PLL_USB, xosc_freq / div),
            RefClkSrc::Rosc => (Src::ROSC_CLKSRC_PH, Aux::CLKSRC_PLL_USB, rosc_freq / div),
            RefClkSrc::PllUsb => (Src::CLKSRC_CLK_REF_AUX, Aux::CLKSRC_PLL_USB, pll_usb_freq / div),
            // See above re gpin handling being commented out
            // RefClkSrc::Gpin0 => (Src::CLKSRC_CLK_REF_AUX, Aux::CLKSRC_GPIN0, gpin0_freq / div),
            // RefClkSrc::Gpin1 => (Src::CLKSRC_CLK_REF_AUX, Aux::CLKSRC_GPIN1, gpin1_freq / div),
        }
    };
    assert!(clk_ref_freq != 0);
    CLOCKS.reference.store(clk_ref_freq, Ordering::Relaxed);
    c.clk_ref_ctrl().write(|w| {
        w.set_src(ref_src);
        w.set_auxsrc(ref_aux);
    });
    #[cfg(feature = "rp2040")]
    while c.clk_ref_selected().read() != (1 << ref_src as u32) {}
    #[cfg(feature = "_rp235x")]
    while c.clk_ref_selected().read() != pac::clocks::regs::ClkRefSelected(1 << ref_src as u32) {}
    c.clk_ref_div().write(|w| {
        w.set_int(config.ref_clk.div);
    });

    // Configure tick generation on the 2040.
    #[cfg(feature = "rp2040")]
    pac::WATCHDOG.tick().write(|w| {
        w.set_cycles((clk_ref_freq / 1_000_000) as u16);
        w.set_enable(true);
    });
    // Configure tick generator on the 2350
    #[cfg(feature = "_rp235x")]
    {
        let cycle_count = clk_ref_freq / 1_000_000;

        pac::TICKS.timer0_cycles().write(|w| w.0 = cycle_count);
        pac::TICKS.timer0_ctrl().write(|w| w.set_enable(true));

        pac::TICKS.watchdog_cycles().write(|w| w.0 = cycle_count);
        pac::TICKS.watchdog_ctrl().write(|w| w.set_enable(true));
    }

    let (sys_src, sys_aux, clk_sys_freq) = {
        use {ClkSysCtrlAuxsrc as Aux, ClkSysCtrlSrc as Src};
        let (src, aux, freq) = match config.sys_clk.src {
            SysClkSrc::Ref => (Src::CLK_REF, Aux::CLKSRC_PLL_SYS, clk_ref_freq),
            SysClkSrc::PllSys => (Src::CLKSRC_CLK_SYS_AUX, Aux::CLKSRC_PLL_SYS, pll_sys_freq),
            SysClkSrc::PllUsb => (Src::CLKSRC_CLK_SYS_AUX, Aux::CLKSRC_PLL_USB, pll_usb_freq),
            SysClkSrc::Rosc => (Src::CLKSRC_CLK_SYS_AUX, Aux::ROSC_CLKSRC, rosc_freq),
            SysClkSrc::Xosc => (Src::CLKSRC_CLK_SYS_AUX, Aux::XOSC_CLKSRC, xosc_freq),
            // See above re gpin handling being commented out
            // SysClkSrc::Gpin0 => (Src::CLKSRC_CLK_SYS_AUX, Aux::CLKSRC_GPIN0, gpin0_freq),
            // SysClkSrc::Gpin1 => (Src::CLKSRC_CLK_SYS_AUX, Aux::CLKSRC_GPIN1, gpin1_freq),
        };
        let div = config.sys_clk.div_int as u64 * 256 + config.sys_clk.div_frac as u64;
        (src, aux, ((freq as u64 * 256) / div) as u32)
    };
    assert!(clk_sys_freq != 0);
    CLOCKS.sys.store(clk_sys_freq, Ordering::Relaxed);
    if sys_src != ClkSysCtrlSrc::CLK_REF {
        c.clk_sys_ctrl().write(|w| w.set_src(ClkSysCtrlSrc::CLK_REF));
        #[cfg(feature = "rp2040")]
        while c.clk_sys_selected().read() != (1 << ClkSysCtrlSrc::CLK_REF as u32) {}
        #[cfg(feature = "_rp235x")]
        while c.clk_sys_selected().read() != pac::clocks::regs::ClkSysSelected(1 << ClkSysCtrlSrc::CLK_REF as u32) {}
    }
    c.clk_sys_ctrl().write(|w| {
        w.set_auxsrc(sys_aux);
        w.set_src(sys_src);
    });

    #[cfg(feature = "rp2040")]
    while c.clk_sys_selected().read() != (1 << sys_src as u32) {}
    #[cfg(feature = "_rp235x")]
    while c.clk_sys_selected().read() != pac::clocks::regs::ClkSysSelected(1 << sys_src as u32) {}

    c.clk_sys_div().write(|w| {
        w.set_int(config.sys_clk.div_int);
        w.set_frac(config.sys_clk.div_frac);
    });

    let mut peris = reset::ALL_PERIPHERALS;

    if let Some(src) = config.peri_clk_src {
        c.clk_peri_ctrl().write(|w| {
            w.set_enable(true);
            w.set_auxsrc(ClkPeriCtrlAuxsrc::from_bits(src as _));
        });
        let peri_freq = match src {
            PeriClkSrc::Sys => clk_sys_freq,
            PeriClkSrc::PllSys => pll_sys_freq,
            PeriClkSrc::PllUsb => pll_usb_freq,
            PeriClkSrc::Rosc => rosc_freq,
            PeriClkSrc::Xosc => xosc_freq,
            // See above re gpin handling being commented out
            // PeriClkSrc::Gpin0 => gpin0_freq,
            // PeriClkSrc::Gpin1 => gpin1_freq,
        };
        assert!(peri_freq != 0);
        CLOCKS.peri.store(peri_freq, Ordering::Relaxed);
    } else {
        peris.set_spi0(false);
        peris.set_spi1(false);
        peris.set_uart0(false);
        peris.set_uart1(false);
        CLOCKS.peri.store(0, Ordering::Relaxed);
    }

    if let Some(conf) = config.usb_clk {
        c.clk_usb_div().write(|w| w.set_int(conf.div));
        c.clk_usb_ctrl().write(|w| {
            w.set_phase(conf.phase);
            w.set_enable(true);
            w.set_auxsrc(ClkUsbCtrlAuxsrc::from_bits(conf.src as _));
        });
        let usb_freq = match conf.src {
            UsbClkSrc::PllUsb => pll_usb_freq,
            UsbClkSrc::PllSys => pll_sys_freq,
            UsbClkSrc::Rosc => rosc_freq,
            UsbClkSrc::Xosc => xosc_freq,
            // See above re gpin handling being commented out
            // UsbClkSrc::Gpin0 => gpin0_freq,
            // UsbClkSrc::Gpin1 => gpin1_freq,
        };
        assert!(usb_freq != 0);
        assert!(conf.div >= 1 && conf.div <= 4);
        CLOCKS.usb.store(usb_freq / conf.div as u32, Ordering::Relaxed);
    } else {
        peris.set_usbctrl(false);
        CLOCKS.usb.store(0, Ordering::Relaxed);
    }

    if let Some(conf) = config.adc_clk {
        c.clk_adc_div().write(|w| w.set_int(conf.div));
        c.clk_adc_ctrl().write(|w| {
            w.set_phase(conf.phase);
            w.set_enable(true);
            w.set_auxsrc(ClkAdcCtrlAuxsrc::from_bits(conf.src as _));
        });
        let adc_in_freq = match conf.src {
            AdcClkSrc::PllUsb => pll_usb_freq,
            AdcClkSrc::PllSys => pll_sys_freq,
            AdcClkSrc::Rosc => rosc_freq,
            AdcClkSrc::Xosc => xosc_freq,
            // See above re gpin handling being commented out
            // AdcClkSrc::Gpin0 => gpin0_freq,
            // AdcClkSrc::Gpin1 => gpin1_freq,
        };
        assert!(adc_in_freq != 0);
        assert!(conf.div >= 1 && conf.div <= 4);
        CLOCKS.adc.store(adc_in_freq / conf.div as u32, Ordering::Relaxed);
    } else {
        peris.set_adc(false);
        CLOCKS.adc.store(0, Ordering::Relaxed);
    }

    // rp2040 specific clocks
    #[cfg(feature = "rp2040")]
    if let Some(conf) = config.rtc_clk {
        c.clk_rtc_div().write(|w| {
            w.set_int(conf.div_int);
            w.set_frac(conf.div_frac);
        });
        c.clk_rtc_ctrl().write(|w| {
            w.set_phase(conf.phase);
            w.set_enable(true);
            w.set_auxsrc(ClkRtcCtrlAuxsrc::from_bits(conf.src as _));
        });
        let rtc_in_freq = match conf.src {
            RtcClkSrc::PllUsb => pll_usb_freq,
            RtcClkSrc::PllSys => pll_sys_freq,
            RtcClkSrc::Rosc => rosc_freq,
            RtcClkSrc::Xosc => xosc_freq,
            // See above re gpin handling being commented out
            // RtcClkSrc::Gpin0 => gpin0_freq,
            // RtcClkSrc::Gpin1 => gpin1_freq,
        };
        assert!(rtc_in_freq != 0);
        assert!(config.sys_clk.div_int <= 0x1000000);
        CLOCKS.rtc.store(
            ((rtc_in_freq as u64 * 256) / (conf.div_int as u64 * 256 + conf.div_frac as u64)) as u16,
            Ordering::Relaxed,
        );
    } else {
        peris.set_rtc(false);
        CLOCKS.rtc.store(0, Ordering::Relaxed);
    }

    // rp235x specific clocks
    #[cfg(feature = "_rp235x")]
    {
        // TODO hstx clock
        peris.set_hstx(false);
    }

    // Peripheral clocks should now all be running
    reset::unreset_wait(peris);
}

fn configure_rosc(config: RoscConfig) -> u32 {
    let p = pac::ROSC;

    p.freqa().write(|w| {
        w.set_passwd(pac::rosc::vals::Passwd::PASS);
        w.set_ds0(config.drive_strength[0]);
        w.set_ds1(config.drive_strength[1]);
        w.set_ds2(config.drive_strength[2]);
        w.set_ds3(config.drive_strength[3]);
    });

    p.freqb().write(|w| {
        w.set_passwd(pac::rosc::vals::Passwd::PASS);
        w.set_ds4(config.drive_strength[4]);
        w.set_ds5(config.drive_strength[5]);
        w.set_ds6(config.drive_strength[6]);
        w.set_ds7(config.drive_strength[7]);
    });

    p.div().write(|w| {
        w.set_div(pac::rosc::vals::Div(config.div + pac::rosc::vals::Div::PASS.0));
    });

    p.ctrl().write(|w| {
        w.set_enable(pac::rosc::vals::Enable::ENABLE);
        w.set_freq_range(pac::rosc::vals::FreqRange(config.range as u16));
    });

    config.hz
}

/// ROSC clock frequency.
pub fn rosc_freq() -> u32 {
    CLOCKS.rosc.load(Ordering::Relaxed)
}

/// XOSC clock frequency.
pub fn xosc_freq() -> u32 {
    CLOCKS.xosc.load(Ordering::Relaxed)
}

// See above re gpin handling being commented out
// pub fn gpin0_freq() -> u32 {
//     CLOCKS.gpin0.load(Ordering::Relaxed)
// }
// pub fn gpin1_freq() -> u32 {
//     CLOCKS.gpin1.load(Ordering::Relaxed)
// }

/// PLL SYS clock frequency.
pub fn pll_sys_freq() -> u32 {
    CLOCKS.pll_sys.load(Ordering::Relaxed)
}

/// PLL USB clock frequency.
pub fn pll_usb_freq() -> u32 {
    CLOCKS.pll_usb.load(Ordering::Relaxed)
}

/// SYS clock frequency.
pub fn clk_sys_freq() -> u32 {
    CLOCKS.sys.load(Ordering::Relaxed)
}

/// REF clock frequency.
pub fn clk_ref_freq() -> u32 {
    CLOCKS.reference.load(Ordering::Relaxed)
}

/// Peripheral clock frequency.
pub fn clk_peri_freq() -> u32 {
    CLOCKS.peri.load(Ordering::Relaxed)
}

/// USB clock frequency.
pub fn clk_usb_freq() -> u32 {
    CLOCKS.usb.load(Ordering::Relaxed)
}

/// ADC clock frequency.
pub fn clk_adc_freq() -> u32 {
    CLOCKS.adc.load(Ordering::Relaxed)
}

/// RTC clock frequency.
#[cfg(feature = "rp2040")]
pub fn clk_rtc_freq() -> u16 {
    CLOCKS.rtc.load(Ordering::Relaxed)
}

/// The core voltage of the chip.
///
/// Returns the current core voltage or an error if the voltage register
/// contains an unknown value.
pub fn core_voltage() -> Result<CoreVoltage, ClockError> {
    #[cfg(feature = "rp2040")]
    {
        let vreg = pac::VREG_AND_CHIP_RESET;
        let vsel = vreg.vreg().read().vsel();
        match vsel {
            0b0000 => Ok(CoreVoltage::V0_80),
            0b0110 => Ok(CoreVoltage::V0_85),
            0b0111 => Ok(CoreVoltage::V0_90),
            0b1000 => Ok(CoreVoltage::V0_95),
            0b1001 => Ok(CoreVoltage::V1_00),
            0b1010 => Ok(CoreVoltage::V1_05),
            0b1011 => Ok(CoreVoltage::V1_10),
            0b1100 => Ok(CoreVoltage::V1_15),
            0b1101 => Ok(CoreVoltage::V1_20),
            0b1110 => Ok(CoreVoltage::V1_25),
            0b1111 => Ok(CoreVoltage::V1_30),
            _ => Err(ClockError::UnexpectedCoreVoltageRead),
        }
    }

    #[cfg(feature = "_rp235x")]
    {
        let vreg = pac::POWMAN;
        let vsel = vreg.vreg().read().vsel();
        match vsel {
            0b00000 => Ok(CoreVoltage::V0_55),
            0b00001 => Ok(CoreVoltage::V0_60),
            0b00010 => Ok(CoreVoltage::V0_65),
            0b00011 => Ok(CoreVoltage::V0_70),
            0b00100 => Ok(CoreVoltage::V0_75),
            0b00101 => Ok(CoreVoltage::V0_80),
            0b00110 => Ok(CoreVoltage::V0_85),
            0b00111 => Ok(CoreVoltage::V0_90),
            0b01000 => Ok(CoreVoltage::V0_95),
            0b01001 => Ok(CoreVoltage::V1_00),
            0b01010 => Ok(CoreVoltage::V1_05),
            0b01011 => Ok(CoreVoltage::V1_10),
            0b01100 => Ok(CoreVoltage::V1_15),
            0b01101 => Ok(CoreVoltage::V1_20),
            0b01110 => Ok(CoreVoltage::V1_25),
            0b01111 => Ok(CoreVoltage::V1_30),
            _ => Err(ClockError::UnexpectedCoreVoltageRead),
            // see CoreVoltage: we do not support setting Voltages higher than 1.30V at this point
        }
    }
}

fn start_xosc(crystal_hz: u32, delay_multiplier: u32) {
    let startup_delay = (((crystal_hz / 1000) * delay_multiplier) + 128) / 256;
    pac::XOSC.startup().write(|w| w.set_delay(startup_delay as u16));
    pac::XOSC.ctrl().write(|w| {
        w.set_freq_range(pac::xosc::vals::CtrlFreqRange::_1_15MHZ);
        w.set_enable(pac::xosc::vals::Enable::ENABLE);
    });
    while !pac::XOSC.status().read().stable() {}
}

/// PLL (Phase-Locked Loop) configuration
#[inline(always)]
fn configure_pll(p: pac::pll::Pll, input_freq: u32, config: PllConfig) -> Result<u32, ClockError> {
    // Calculate reference frequency
    let ref_freq = input_freq / config.refdiv as u32;

    // Validate PLL parameters
    // Feedback divider (FBDIV) must be between 16 and 320
    assert!(config.fbdiv >= 16 && config.fbdiv <= 320);

    // Post divider 1 (POSTDIV1) must be between 1 and 7
    assert!(config.post_div1 >= 1 && config.post_div1 <= 7);

    // Post divider 2 (POSTDIV2) must be between 1 and 7
    assert!(config.post_div2 >= 1 && config.post_div2 <= 7);

    // Post divider 2 (POSTDIV2) must be less than or equal to post divider 1 (POSTDIV1)
    assert!(config.post_div2 <= config.post_div1);

    // Reference divider (REFDIV) must be between 1 and 63
    assert!(config.refdiv >= 1 && config.refdiv <= 63);

    // Reference frequency (REF_FREQ) must be between 5MHz and 800MHz
    assert!(ref_freq >= 5_000_000 && ref_freq <= 800_000_000);

    // Calculate VCO frequency
    let vco_freq = ref_freq.saturating_mul(config.fbdiv as u32);

    // VCO (Voltage Controlled Oscillator) frequency must be between 750MHz and 1800MHz
    assert!(vco_freq >= 750_000_000 && vco_freq <= 1_800_000_000);

    // We follow the SDK's approach to PLL configuration which is:
    // 1. Power down PLL
    // 2. Configure the reference divider
    // 3. Configure the feedback divider
    // 4. Power up PLL and VCO
    // 5. Wait for PLL to lock
    // 6. Configure post-dividers
    // 7. Enable post-divider output

    // 1. Power down PLL before configuration
    p.pwr().write(|w| {
        w.set_pd(true); // Power down the PLL
        w.set_vcopd(true); // Power down the VCO
        w.set_postdivpd(true); // Power down the post divider
        w.set_dsmpd(true); // Disable fractional mode
        *w
    });

    // Short delay after powering down
    cortex_m::asm::delay(10);

    // 2. Configure reference divider first
    p.cs().write(|w| w.set_refdiv(config.refdiv as _));

    // 3. Configure feedback divider
    p.fbdiv_int().write(|w| w.set_fbdiv_int(config.fbdiv));

    // 4. Power up PLL and VCO, but keep post divider powered down during initial lock
    p.pwr().write(|w| {
        w.set_pd(false); // Power up the PLL
        w.set_vcopd(false); // Power up the VCO
        w.set_postdivpd(true); // Keep post divider powered down during initial lock
        w.set_dsmpd(true); // Disable fractional mode (simpler configuration)
        *w
    });

    // 5. Wait for PLL to lock with a timeout
    let mut timeout = 1_000_000;
    while !p.cs().read().lock() {
        timeout -= 1;
        if timeout == 0 {
            // PLL failed to lock, return 0 to indicate failure
            return Err(ClockError::PllLockTimedOut);
        }
    }

    // 6. Configure post dividers after PLL is locked
    p.prim().write(|w| {
        w.set_postdiv1(config.post_div1);
        w.set_postdiv2(config.post_div2);
    });

    // 7. Enable the post divider output
    p.pwr().modify(|w| {
        w.set_postdivpd(false); // Power up post divider
        *w
    });

    // Final delay to ensure everything is stable
    cortex_m::asm::delay(100);

    // Calculate and return actual output frequency
    Ok(vco_freq / ((config.post_div1 * config.post_div2) as u32))
}

/// General purpose input clock pin.
pub trait GpinPin: crate::gpio::Pin {
    /// Pin number.
    const NR: usize;
}

macro_rules! impl_gpinpin {
    ($name:ident, $pin_num:expr, $gpin_num:expr) => {
        impl GpinPin for crate::peripherals::$name {
            const NR: usize = $gpin_num;
        }
    };
}

impl_gpinpin!(PIN_20, 20, 0);
impl_gpinpin!(PIN_22, 22, 1);

/// General purpose clock input driver.
pub struct Gpin<'d, T: GpinPin> {
    gpin: Peri<'d, AnyPin>,
    _phantom: PhantomData<T>,
}

impl<'d, T: GpinPin> Gpin<'d, T> {
    /// Create new gpin driver.
    pub fn new(gpin: Peri<'d, T>) -> Self {
        #[cfg(feature = "rp2040")]
        gpin.gpio().ctrl().write(|w| w.set_funcsel(0x08));

        // On RP2350 GPIN changed from F8 toF9
        #[cfg(feature = "_rp235x")]
        gpin.gpio().ctrl().write(|w| w.set_funcsel(0x09));

        #[cfg(feature = "_rp235x")]
        gpin.pad_ctrl().write(|w| {
            w.set_iso(false);
        });

        Gpin {
            gpin: gpin.into(),
            _phantom: PhantomData,
        }
    }
}

impl<'d, T: GpinPin> Drop for Gpin<'d, T> {
    fn drop(&mut self) {
        self.gpin.pad_ctrl().write(|_| {});
        self.gpin
            .gpio()
            .ctrl()
            .write(|w| w.set_funcsel(pac::io::vals::Gpio0ctrlFuncsel::NULL as _));
    }
}

/// General purpose clock output pin.
pub trait GpoutPin: crate::gpio::Pin {
    /// Pin number.
    fn number(&self) -> usize;
}

macro_rules! impl_gpoutpin {
    ($name:ident, $gpout_num:expr) => {
        impl GpoutPin for crate::peripherals::$name {
            fn number(&self) -> usize {
                $gpout_num
            }
        }
    };
}

impl_gpoutpin!(PIN_21, 0);
impl_gpoutpin!(PIN_23, 1);
impl_gpoutpin!(PIN_24, 2);
impl_gpoutpin!(PIN_25, 3);

/// Gpout clock source.
#[repr(u8)]
pub enum GpoutSrc {
    /// Sys PLL.
    PllSys = ClkGpoutCtrlAuxsrc::CLKSRC_PLL_SYS as _,
    // See above re gpin handling being commented out
    // Gpin0 = ClkGpoutCtrlAuxsrc::CLKSRC_GPIN0 as _ ,
    // Gpin1 = ClkGpoutCtrlAuxsrc::CLKSRC_GPIN1 as _ ,
    /// USB PLL.
    PllUsb = ClkGpoutCtrlAuxsrc::CLKSRC_PLL_USB as _,
    /// ROSC.
    Rosc = ClkGpoutCtrlAuxsrc::ROSC_CLKSRC as _,
    /// XOSC.
    Xosc = ClkGpoutCtrlAuxsrc::XOSC_CLKSRC as _,
    /// SYS.
    Sys = ClkGpoutCtrlAuxsrc::CLK_SYS as _,
    /// USB.
    Usb = ClkGpoutCtrlAuxsrc::CLK_USB as _,
    /// ADC.
    Adc = ClkGpoutCtrlAuxsrc::CLK_ADC as _,
    /// RTC.
    #[cfg(feature = "rp2040")]
    Rtc = ClkGpoutCtrlAuxsrc::CLK_RTC as _,
    /// REF.
    Ref = ClkGpoutCtrlAuxsrc::CLK_REF as _,
}

/// General purpose clock output driver.
pub struct Gpout<'d, T: GpoutPin> {
    gpout: Peri<'d, T>,
}

impl<'d, T: GpoutPin> Gpout<'d, T> {
    /// Create new general purpose clock output.
    pub fn new(gpout: Peri<'d, T>) -> Self {
        #[cfg(feature = "rp2040")]
        gpout.gpio().ctrl().write(|w| w.set_funcsel(0x08));

        // On RP2350 GPOUT changed from F8 toF9
        #[cfg(feature = "_rp235x")]
        gpout.gpio().ctrl().write(|w| w.set_funcsel(0x09));

        #[cfg(feature = "_rp235x")]
        gpout.pad_ctrl().write(|w| {
            w.set_iso(false);
        });

        Self { gpout }
    }

    /// Set clock divider.
    #[cfg(feature = "rp2040")]
    pub fn set_div(&self, int: u32, frac: u8) {
        let c = pac::CLOCKS;
        c.clk_gpout_div(self.gpout.number()).write(|w| {
            w.set_int(int);
            w.set_frac(frac);
        });
    }

    /// Set clock divider.
    #[cfg(feature = "_rp235x")]
    pub fn set_div(&self, int: u16, frac: u16) {
        let c = pac::CLOCKS;
        c.clk_gpout_div(self.gpout.number()).write(|w| {
            w.set_int(int);
            w.set_frac(frac);
        });
    }

    /// Set clock source.
    pub fn set_src(&self, src: GpoutSrc) {
        let c = pac::CLOCKS;
        c.clk_gpout_ctrl(self.gpout.number()).modify(|w| {
            w.set_auxsrc(ClkGpoutCtrlAuxsrc::from_bits(src as _));
        });
    }

    /// Enable clock.
    pub fn enable(&self) {
        let c = pac::CLOCKS;
        c.clk_gpout_ctrl(self.gpout.number()).modify(|w| {
            w.set_enable(true);
        });
    }

    /// Disable clock.
    pub fn disable(&self) {
        let c = pac::CLOCKS;
        c.clk_gpout_ctrl(self.gpout.number()).modify(|w| {
            w.set_enable(false);
        });
    }

    /// Clock frequency.
    pub fn get_freq(&self) -> u32 {
        let c = pac::CLOCKS;
        let src = c.clk_gpout_ctrl(self.gpout.number()).read().auxsrc();

        let base = match src {
            ClkGpoutCtrlAuxsrc::CLKSRC_PLL_SYS => pll_sys_freq(),
            // See above re gpin handling being commented out
            // ClkGpoutCtrlAuxsrc::CLKSRC_GPIN0 => gpin0_freq(),
            // ClkGpoutCtrlAuxsrc::CLKSRC_GPIN1 => gpin1_freq(),
            ClkGpoutCtrlAuxsrc::CLKSRC_PLL_USB => pll_usb_freq(),
            ClkGpoutCtrlAuxsrc::ROSC_CLKSRC => rosc_freq(),
            ClkGpoutCtrlAuxsrc::XOSC_CLKSRC => xosc_freq(),
            ClkGpoutCtrlAuxsrc::CLK_SYS => clk_sys_freq(),
            ClkGpoutCtrlAuxsrc::CLK_USB => clk_usb_freq(),
            ClkGpoutCtrlAuxsrc::CLK_ADC => clk_adc_freq(),
            ClkGpoutCtrlAuxsrc::CLK_REF => clk_ref_freq(),
            _ => unreachable!(),
        };

        let div = c.clk_gpout_div(self.gpout.number()).read();
        let int = if div.int() == 0 { 0xFFFF } else { div.int() } as u64;
        let frac = div.frac() as u64;

        ((base as u64 * 256) / (int * 256 + frac)) as u32
    }
}

impl<'d, T: GpoutPin> Drop for Gpout<'d, T> {
    fn drop(&mut self) {
        self.disable();
        self.gpout.pad_ctrl().write(|_| {});
        self.gpout
            .gpio()
            .ctrl()
            .write(|w| w.set_funcsel(pac::io::vals::Gpio0ctrlFuncsel::NULL as _));
    }
}

/// Random number generator based on the ROSC RANDOMBIT register.
///
/// This will not produce random values if the ROSC is stopped or run at some
/// harmonic of the bus frequency. With default clock settings these are not
/// issues.
pub struct RoscRng;

impl RoscRng {
    /// Get a random u8
    pub fn next_u8() -> u8 {
        let random_reg = pac::ROSC.randombit();
        let mut acc = 0;
        for _ in 0..u8::BITS {
            acc <<= 1;
            acc |= random_reg.read().randombit() as u8;
        }
        acc
    }

    /// Get a random u32
    pub fn next_u32(&mut self) -> u32 {
        rand_core_09::impls::next_u32_via_fill(self)
    }

    /// Get a random u64
    pub fn next_u64(&mut self) -> u64 {
        rand_core_09::impls::next_u64_via_fill(self)
    }

    /// Fill a slice with random bytes
    pub fn fill_bytes(&mut self, dest: &mut [u8]) {
        dest.fill_with(Self::next_u8)
    }
}

impl rand_core_06::RngCore for RoscRng {
    fn next_u32(&mut self) -> u32 {
        self.next_u32()
    }

    fn next_u64(&mut self) -> u64 {
        self.next_u64()
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.fill_bytes(dest);
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core_06::Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}

impl rand_core_06::CryptoRng for RoscRng {}

impl rand_core_09::RngCore for RoscRng {
    fn next_u32(&mut self) -> u32 {
        self.next_u32()
    }

    fn next_u64(&mut self) -> u64 {
        self.next_u64()
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.fill_bytes(dest);
    }
}

impl rand_core_09::CryptoRng for RoscRng {}

/// Enter the `DORMANT` sleep state. This will stop *all* internal clocks
/// and can only be exited through resets, dormant-wake GPIO interrupts,
/// and RTC interrupts. If RTC is clocked from an internal clock source
/// it will be stopped and not function as a wakeup source.
#[cfg(all(target_arch = "arm"))]
pub fn dormant_sleep() {
    struct Set<T: Copy, F: Fn()>(Reg<T, RW>, T, F);

    impl<T: Copy, F: Fn()> Drop for Set<T, F> {
        fn drop(&mut self) {
            self.0.write_value(self.1);
            self.2();
        }
    }

    fn set_with_post_restore<T: Copy, After: Fn(), F: FnOnce(&mut T) -> After>(
        reg: Reg<T, RW>,
        f: F,
    ) -> Set<T, impl Fn()> {
        reg.modify(|w| {
            let old = *w;
            let after = f(w);
            Set(reg, old, after)
        })
    }

    fn set<T: Copy, F: FnOnce(&mut T)>(reg: Reg<T, RW>, f: F) -> Set<T, impl Fn()> {
        set_with_post_restore(reg, |r| {
            f(r);
            || ()
        })
    }

    // disable all clocks that are not vital in preparation for disabling clock sources.
    // we'll keep gpout and rtc clocks untouched, gpout because we don't care about them
    // and rtc because it's a possible wakeup source. if clk_rtc is not configured for
    // gpin we'll never wake from rtc, but that's what the user asked for then.
    let _stop_adc = set(pac::CLOCKS.clk_adc_ctrl(), |w| w.set_enable(false));
    let _stop_usb = set(pac::CLOCKS.clk_usb_ctrl(), |w| w.set_enable(false));
    let _stop_peri = set(pac::CLOCKS.clk_peri_ctrl(), |w| w.set_enable(false));
    // set up rosc. we could ask the user to tell us which clock source to wake from like
    // the C SDK does, but that seems rather unfriendly. we *may* disturb rtc by changing
    // rosc configuration if it's currently the rtc clock source, so we'll configure rosc
    // to the slowest frequency to minimize that impact.
    let _configure_rosc = (
        set(pac::ROSC.ctrl(), |w| {
            w.set_enable(pac::rosc::vals::Enable::ENABLE);
            w.set_freq_range(pac::rosc::vals::FreqRange::LOW);
        }),
        // div=32
        set(pac::ROSC.div(), |w| w.set_div(pac::rosc::vals::Div(0xaa0))),
    );
    while !pac::ROSC.status().read().stable() {}
    // switch over to rosc as the system clock source. this will change clock sources for
    // watchdog and timer clocks, but timers won't be a concern and the watchdog won't
    // speed up by enough to worry about (unless it's clocked from gpin, which we don't
    // support anyway).
    let _switch_clk_ref = set(pac::CLOCKS.clk_ref_ctrl(), |w| {
        w.set_src(pac::clocks::vals::ClkRefCtrlSrc::ROSC_CLKSRC_PH);
    });
    let _switch_clk_sys = set(pac::CLOCKS.clk_sys_ctrl(), |w| {
        w.set_src(pac::clocks::vals::ClkSysCtrlSrc::CLK_REF);
    });
    // oscillator dormancy does not power down plls, we have to do that ourselves. we'll
    // restore them to their prior glory when woken though since the system may be clocked
    // from either (and usb/adc will probably need the USB PLL anyway)
    let _stop_pll_sys = set_with_post_restore(pac::PLL_SYS.pwr(), |w| {
        let wake = !w.pd() && !w.vcopd();
        w.set_pd(true);
        w.set_vcopd(true);
        move || while wake && !pac::PLL_SYS.cs().read().lock() {}
    });
    let _stop_pll_usb = set_with_post_restore(pac::PLL_USB.pwr(), |w| {
        let wake = !w.pd() && !w.vcopd();
        w.set_pd(true);
        w.set_vcopd(true);
        move || while wake && !pac::PLL_USB.cs().read().lock() {}
    });
    // dormancy only stops the oscillator we're telling to go dormant, the other remains
    // running. nothing can use xosc at this point any more. not doing this costs an 200µA.
    let _stop_xosc = set_with_post_restore(pac::XOSC.ctrl(), |w| {
        let wake = w.enable() == pac::xosc::vals::Enable::ENABLE;
        if wake {
            w.set_enable(pac::xosc::vals::Enable::DISABLE);
        }
        move || while wake && !pac::XOSC.status().read().stable() {}
    });
    let _power_down_xip_cache = set(pac::XIP_CTRL.ctrl(), |w| w.set_power_down(true));

    // only power down memory if we're running from XIP (or ROM? how?).
    // powering down memory otherwise would require a lot of exacting checks that
    // are better done by the user in a local copy of this function.
    // powering down memories saves ~100µA, so it's well worth doing.
    unsafe {
        let is_in_flash = {
            // we can't rely on the address of this function as rust sees it since linker
            // magic or even boot2 may place it into ram.
            let pc: usize;
            asm!(
                "mov {pc}, pc",
                pc = out (reg) pc
            );
            pc < 0x20000000
        };
        if is_in_flash {
            // we will be powering down memories, so we must be *absolutely*
            // certain that we're running entirely from XIP and registers until
            // memories are powered back up again. accessing memory that's powered
            // down may corrupt memory contents (see section 2.11.4 of the manual).
            // additionally a 20ns wait time is needed after powering up memories
            // again. rosc is likely to run at only a few MHz at most, so the
            // inter-instruction delay alone will be enough to satisfy this bound.
            asm!(
                "ldr {old_mem}, [{mempowerdown}]",
                "str {power_down_mems}, [{mempowerdown}]",
                "str {coma}, [{dormant}]",
                "str {old_mem}, [{mempowerdown}]",
                old_mem = out (reg) _,
                mempowerdown = in (reg) pac::SYSCFG.mempowerdown().as_ptr(),
                power_down_mems = in (reg) 0b11111111,
                dormant = in (reg) pac::ROSC.dormant().as_ptr(),
                coma = in (reg) 0x636f6d61,
            );
        } else {
            pac::ROSC.dormant().write_value(rp_pac::rosc::regs::Dormant(0x636f6d61));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "rp2040")]
    #[test]
    fn test_find_pll_params() {
        #[cfg(feature = "rp2040")]
        {
            // Test standard 125 MHz configuration with 12 MHz crystal
            let params = find_pll_params(12_000_000, 125_000_000).unwrap();
            assert_eq!(params.refdiv, 1);
            assert_eq!(params.fbdiv, 125);

            // Test USB PLL configuration for 48MHz
            // The algorithm may find different valid parameters than the SDK defaults
            // We'll check that it generates a valid configuration that produces 48MHz
            let params = find_pll_params(12_000_000, 48_000_000).unwrap();
            assert_eq!(params.refdiv, 1);

            // Calculate the actual output frequency
            let ref_freq = 12_000_000 / params.refdiv as u32;
            let vco_freq = ref_freq as u64 * params.fbdiv as u64;
            let output_freq = (vco_freq / ((params.post_div1 * params.post_div2) as u64)) as u32;

            // Verify the output frequency is correct
            assert_eq!(output_freq, 48_000_000);

            // Verify VCO frequency is in valid range
            assert!(vco_freq >= 750_000_000 && vco_freq <= 1_800_000_000);

            // Test overclocked configuration for 200 MHz
            let params = find_pll_params(12_000_000, 200_000_000).unwrap();
            assert_eq!(params.refdiv, 1);
            let vco_freq = 12_000_000 as u64 * params.fbdiv as u64;
            let output_freq = (vco_freq / ((params.post_div1 * params.post_div2) as u64)) as u32;
            assert_eq!(output_freq, 200_000_000);
            assert!(vco_freq >= 750_000_000 && vco_freq <= 1_800_000_000); // VCO in valid range

            // Test non-standard crystal with 16 MHz
            let params = find_pll_params(16_000_000, 125_000_000).unwrap();
            let vco_freq = (16_000_000 / params.refdiv as u32) as u64 * params.fbdiv as u64;
            let output_freq = (vco_freq / ((params.post_div1 * params.post_div2) as u64)) as u32;

            // Test non-standard crystal with 15 MHz
            let params = find_pll_params(15_000_000, 125_000_000).unwrap();
            let vco_freq = (15_000_000 / params.refdiv as u32) as u64 * params.fbdiv as u64;
            let output_freq = (vco_freq / ((params.post_div1 * params.post_div2) as u64)) as u32;

            // With a 15 MHz crystal, we might not get exactly 125 MHz
            // Check that it's close enough (within 0.2% margin)
            let freq_diff = if output_freq > 125_000_000 {
                output_freq - 125_000_000
            } else {
                125_000_000 - output_freq
            };
            let error_percentage = (freq_diff as f64 / 125_000_000.0) * 100.0;
            assert!(
                error_percentage < 0.2,
                "Output frequency {} is not close enough to target 125 MHz. Error: {:.2}%",
                output_freq,
                error_percentage
            );

            assert!(vco_freq >= 750_000_000 && vco_freq <= 1_800_000_000);
        }
    }

    #[cfg(feature = "rp2040")]
    #[test]
    fn test_pll_config_validation() {
        // Test PLL configuration validation logic
        let valid_config = PllConfig {
            refdiv: 1,
            fbdiv: 125,
            post_div1: 6,
            post_div2: 2,
        };

        // Valid configuration should pass validation
        assert!(valid_config.is_valid(12_000_000));

        // Test fbdiv constraints
        let mut invalid_config = valid_config;
        invalid_config.fbdiv = 15; // Below minimum of 16
        assert!(!invalid_config.is_valid(12_000_000));

        invalid_config.fbdiv = 321; // Above maximum of 320
        assert!(!invalid_config.is_valid(12_000_000));

        // Test post_div constraints
        invalid_config = valid_config;
        invalid_config.post_div1 = 0; // Below minimum of 1
        assert!(!invalid_config.is_valid(12_000_000));

        invalid_config = valid_config;
        invalid_config.post_div1 = 8; // Above maximum of 7
        assert!(!invalid_config.is_valid(12_000_000));

        // Test post_div2 must be <= post_div1
        invalid_config = valid_config;
        invalid_config.post_div2 = 7;
        invalid_config.post_div1 = 3;
        assert!(!invalid_config.is_valid(12_000_000));

        // Test reference frequency constraints
        invalid_config = valid_config;
        assert!(!invalid_config.is_valid(4_000_000)); // Below minimum of 5 MHz
        assert!(!invalid_config.is_valid(900_000_000)); // Above maximum of 800 MHz

        // Test VCO frequency constraints
        invalid_config = valid_config;
        invalid_config.fbdiv = 16;
        assert!(!invalid_config.is_valid(12_000_000)); // VCO too low: 12MHz * 16 = 192MHz

        // Test VCO frequency constraints - too high
        invalid_config = valid_config;
        invalid_config.fbdiv = 200;
        invalid_config.refdiv = 1;
        // This should be INVALID: 12MHz * 200 = 2400MHz exceeds max VCO of 1800MHz
        assert!(!invalid_config.is_valid(12_000_000));

        // Test a valid high VCO configuration
        invalid_config.fbdiv = 150; // 12MHz * 150 = 1800MHz, exactly at the limit
        assert!(invalid_config.is_valid(12_000_000));
    }

    #[cfg(feature = "rp2040")]
    #[test]
    fn test_manual_pll_helper() {
        {
            // Test the new manual_pll helper method
            let config = ClockConfig::manual_pll(
                12_000_000,
                PllConfig {
                    refdiv: 1,
                    fbdiv: 100,
                    post_div1: 3,
                    post_div2: 2,
                },
                CoreVoltage::V1_15,
            );

            // Check voltage scale was set correctly
            assert_eq!(config.core_voltage, CoreVoltage::V1_15);

            // Check PLL config was set correctly
            assert_eq!(config.xosc.as_ref().unwrap().sys_pll.as_ref().unwrap().refdiv, 1);
            assert_eq!(config.xosc.as_ref().unwrap().sys_pll.as_ref().unwrap().fbdiv, 100);
            assert_eq!(config.xosc.as_ref().unwrap().sys_pll.as_ref().unwrap().post_div1, 3);
            assert_eq!(config.xosc.as_ref().unwrap().sys_pll.as_ref().unwrap().post_div2, 2);

            // Check we get the expected frequency
            assert_eq!(
                config
                    .xosc
                    .as_ref()
                    .unwrap()
                    .sys_pll
                    .as_ref()
                    .unwrap()
                    .output_frequency(12_000_000),
                200_000_000
            );
        }
    }

    #[cfg(feature = "rp2040")]
    #[test]
    fn test_auto_voltage_scaling() {
        {
            // Test automatic voltage scaling based on frequency
            // Under 133 MHz should use default voltage (V1_10)
            let config = ClockConfig::system_freq(125_000_000).unwrap();
            assert_eq!(config.core_voltage, CoreVoltage::V1_10);

            // 133-200 MHz should use V1_15
            let config = ClockConfig::system_freq(150_000_000).unwrap();
            assert_eq!(config.core_voltage, CoreVoltage::V1_15);
            let config = ClockConfig::system_freq(200_000_000).unwrap();
            assert_eq!(config.core_voltage, CoreVoltage::V1_15);

            // Above 200 MHz should use V1_15
            let config = ClockConfig::system_freq(250_000_000).unwrap();
            assert_eq!(config.core_voltage, CoreVoltage::V1_15);

            // Below 125 MHz should use V1_10
            let config = ClockConfig::system_freq(100_000_000).unwrap();
            assert_eq!(config.core_voltage, CoreVoltage::V1_10);
        }
    }
}
