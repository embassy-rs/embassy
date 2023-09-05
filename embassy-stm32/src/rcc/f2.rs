use core::convert::TryFrom;
use core::ops::{Div, Mul};

pub use super::bus::{AHBPrescaler, APBPrescaler};
use crate::pac::flash::vals::Latency;
use crate::pac::rcc::vals::{Pllp, Pllsrc, Sw};
use crate::pac::{FLASH, PWR, RCC};
use crate::rcc::bd::BackupDomain;
use crate::rcc::{set_freqs, Clocks};
use crate::rtc::RtcClockSource;
use crate::time::Hertz;

/// HSI speed
pub const HSI_FREQ: Hertz = Hertz(16_000_000);

/// LSI speed
pub const LSI_FREQ: Hertz = Hertz(32_000);

#[derive(Clone, Copy)]
pub struct HSEConfig {
    pub frequency: Hertz,
    pub source: HSESrc,
}

/// System clock mux source
#[derive(Clone, Copy)]
pub enum ClockSrc {
    HSE,
    HSI,
    PLL,
}

/// HSE clock source
#[derive(Clone, Copy)]
pub enum HSESrc {
    /// Crystal/ceramic resonator
    Crystal,
    /// External clock source, HSE bypassed
    Bypass,
}

#[derive(Clone, Copy)]
pub struct PLLConfig {
    pub pre_div: PLLPreDiv,
    pub mul: PLLMul,
    pub main_div: PLLMainDiv,
    pub pll48_div: PLL48Div,
}

impl Default for PLLConfig {
    fn default() -> Self {
        PLLConfig {
            pre_div: PLLPreDiv(16),
            mul: PLLMul(192),
            main_div: PLLMainDiv::Div2,
            pll48_div: PLL48Div(4),
        }
    }
}

impl PLLConfig {
    pub fn clocks(&self, src_freq: Hertz) -> PLLClocks {
        let in_freq = src_freq / self.pre_div;
        let vco_freq = Hertz((src_freq.0 as u64 * self.mul.0 as u64 / self.pre_div.0 as u64) as u32);
        let main_freq = vco_freq / self.main_div;
        let pll48_freq = vco_freq / self.pll48_div;
        PLLClocks {
            in_freq,
            vco_freq,
            main_freq,
            pll48_freq,
        }
    }
}

/// Clock source for both main PLL and PLLI2S
#[derive(Clone, Copy, PartialEq)]
pub enum PLLSrc {
    HSE,
    HSI,
}

impl Into<Pllsrc> for PLLSrc {
    fn into(self) -> Pllsrc {
        match self {
            PLLSrc::HSE => Pllsrc::HSE,
            PLLSrc::HSI => Pllsrc::HSI,
        }
    }
}

/// Division factor for both main PLL and PLLI2S
#[derive(Clone, Copy, PartialEq)]
#[repr(transparent)]
pub struct PLLPreDiv(u8);

impl TryFrom<u8> for PLLPreDiv {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            2..=63 => Ok(PLLPreDiv(value)),
            _ => Err("PLLPreDiv must be within range 2..=63"),
        }
    }
}

impl Div<PLLPreDiv> for Hertz {
    type Output = Hertz;

    fn div(self, rhs: PLLPreDiv) -> Self::Output {
        Hertz(self.0 / u32::from(rhs.0))
    }
}

/// Multiplication factor for main PLL
#[derive(Clone, Copy, PartialEq)]
#[repr(transparent)]
pub struct PLLMul(u16);

impl Mul<PLLMul> for Hertz {
    type Output = Hertz;

    fn mul(self, rhs: PLLMul) -> Self::Output {
        Hertz(self.0 * u32::from(rhs.0))
    }
}

impl TryFrom<u16> for PLLMul {
    type Error = &'static str;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            192..=432 => Ok(PLLMul(value)),
            _ => Err("PLLMul must be within range 192..=432"),
        }
    }
}

/// PLL division factor for the main system clock
#[derive(Clone, Copy, PartialEq)]
pub enum PLLMainDiv {
    Div2,
    Div4,
    Div6,
    Div8,
}

impl Into<Pllp> for PLLMainDiv {
    fn into(self) -> Pllp {
        match self {
            PLLMainDiv::Div2 => Pllp::DIV2,
            PLLMainDiv::Div4 => Pllp::DIV4,
            PLLMainDiv::Div6 => Pllp::DIV6,
            PLLMainDiv::Div8 => Pllp::DIV8,
        }
    }
}

impl Div<PLLMainDiv> for Hertz {
    type Output = Hertz;

    fn div(self, rhs: PLLMainDiv) -> Self::Output {
        let divisor = match rhs {
            PLLMainDiv::Div2 => 2,
            PLLMainDiv::Div4 => 4,
            PLLMainDiv::Div6 => 6,
            PLLMainDiv::Div8 => 8,
        };
        Hertz(self.0 / divisor)
    }
}

/// PLL division factor for USB OTG FS / SDIO / RNG
#[derive(Clone, Copy, PartialEq)]
#[repr(transparent)]
pub struct PLL48Div(u8);

impl Div<PLL48Div> for Hertz {
    type Output = Hertz;

    fn div(self, rhs: PLL48Div) -> Self::Output {
        Hertz(self.0 / u32::from(rhs.0))
    }
}

impl TryFrom<u8> for PLL48Div {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            2..=15 => Ok(PLL48Div(value)),
            _ => Err("PLL48Div must be within range 2..=15"),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct PLLClocks {
    pub in_freq: Hertz,
    pub vco_freq: Hertz,
    pub main_freq: Hertz,
    pub pll48_freq: Hertz,
}

pub use super::bus::VoltageScale;

impl VoltageScale {
    const fn wait_states(&self, ahb_freq: Hertz) -> Option<Latency> {
        let ahb_freq = ahb_freq.0;
        // Reference: RM0033 - Table 3. Number of wait states according to Cortex®-M3 clock
        // frequency
        match self {
            VoltageScale::Scale3 => {
                if ahb_freq <= 16_000_000 {
                    Some(Latency::WS0)
                } else if ahb_freq <= 32_000_000 {
                    Some(Latency::WS1)
                } else if ahb_freq <= 48_000_000 {
                    Some(Latency::WS2)
                } else if ahb_freq <= 64_000_000 {
                    Some(Latency::WS3)
                } else if ahb_freq <= 80_000_000 {
                    Some(Latency::WS4)
                } else if ahb_freq <= 96_000_000 {
                    Some(Latency::WS5)
                } else if ahb_freq <= 112_000_000 {
                    Some(Latency::WS6)
                } else if ahb_freq <= 120_000_000 {
                    Some(Latency::WS7)
                } else {
                    None
                }
            }
            VoltageScale::Scale2 => {
                if ahb_freq <= 18_000_000 {
                    Some(Latency::WS0)
                } else if ahb_freq <= 36_000_000 {
                    Some(Latency::WS1)
                } else if ahb_freq <= 54_000_000 {
                    Some(Latency::WS2)
                } else if ahb_freq <= 72_000_000 {
                    Some(Latency::WS3)
                } else if ahb_freq <= 90_000_000 {
                    Some(Latency::WS4)
                } else if ahb_freq <= 108_000_000 {
                    Some(Latency::WS5)
                } else if ahb_freq <= 120_000_000 {
                    Some(Latency::WS6)
                } else {
                    None
                }
            }
            VoltageScale::Scale1 => {
                if ahb_freq <= 24_000_000 {
                    Some(Latency::WS0)
                } else if ahb_freq <= 48_000_000 {
                    Some(Latency::WS1)
                } else if ahb_freq <= 72_000_000 {
                    Some(Latency::WS2)
                } else if ahb_freq <= 96_000_000 {
                    Some(Latency::WS3)
                } else if ahb_freq <= 120_000_000 {
                    Some(Latency::WS4)
                } else {
                    None
                }
            }
            VoltageScale::Scale0 => {
                if ahb_freq <= 30_000_000 {
                    Some(Latency::WS0)
                } else if ahb_freq <= 60_000_000 {
                    Some(Latency::WS1)
                } else if ahb_freq <= 90_000_000 {
                    Some(Latency::WS2)
                } else if ahb_freq <= 120_000_000 {
                    Some(Latency::WS3)
                } else {
                    None
                }
            }
        }
    }
}

/// Clocks configuration
pub struct Config {
    pub hse: Option<HSEConfig>,
    pub hsi: bool,
    pub pll_mux: PLLSrc,
    pub pll: PLLConfig,
    pub mux: ClockSrc,
    pub rtc: Option<RtcClockSource>,
    pub voltage: VoltageScale,
    pub ahb_pre: AHBPrescaler,
    pub apb1_pre: APBPrescaler,
    pub apb2_pre: APBPrescaler,
}

impl Default for Config {
    #[inline]
    fn default() -> Config {
        Config {
            hse: None,
            hsi: true,
            pll_mux: PLLSrc::HSI,
            pll: PLLConfig::default(),
            voltage: VoltageScale::Scale3,
            mux: ClockSrc::HSI,
            rtc: None,
            ahb_pre: AHBPrescaler::NotDivided,
            apb1_pre: APBPrescaler::NotDivided,
            apb2_pre: APBPrescaler::NotDivided,
        }
    }
}

pub(crate) unsafe fn init(config: Config) {
    // Make sure HSI is enabled
    RCC.cr().write(|w| w.set_hsion(true));
    while !RCC.cr().read().hsirdy() {}

    if let Some(hse_config) = config.hse {
        RCC.cr().modify(|w| {
            w.set_hsebyp(match hse_config.source {
                HSESrc::Bypass => true,
                HSESrc::Crystal => false,
            });
            w.set_hseon(true)
        });
        while !RCC.cr().read().hserdy() {}
    }

    let pll_src_freq = match config.pll_mux {
        PLLSrc::HSE => {
            let hse_config = config
                .hse
                .unwrap_or_else(|| panic!("HSE must be configured to be used as PLL input"));
            hse_config.frequency
        }
        PLLSrc::HSI => HSI_FREQ,
    };

    // Reference: STM32F215xx/217xx datasheet Table 33. Main PLL characteristics
    let pll_clocks = config.pll.clocks(pll_src_freq);
    assert!(Hertz(950_000) <= pll_clocks.in_freq && pll_clocks.in_freq <= Hertz(2_100_000));
    assert!(Hertz(192_000_000) <= pll_clocks.vco_freq && pll_clocks.vco_freq <= Hertz(432_000_000));
    assert!(Hertz(24_000_000) <= pll_clocks.main_freq && pll_clocks.main_freq <= Hertz(120_000_000));
    // USB actually requires == 48 MHz, but other PLL48 peripherals are fine with <= 48MHz
    assert!(pll_clocks.pll48_freq <= Hertz(48_000_000));

    RCC.pllcfgr().write(|w| {
        w.set_pllsrc(config.pll_mux.into());
        w.set_pllm(config.pll.pre_div.0);
        w.set_plln(config.pll.mul.0);
        w.set_pllp(config.pll.main_div.into());
        w.set_pllq(config.pll.pll48_div.0);
    });

    let (sys_clk, sw) = match config.mux {
        ClockSrc::HSI => {
            assert!(config.hsi, "HSI must be enabled to be used as system clock");
            (HSI_FREQ, Sw::HSI)
        }
        ClockSrc::HSE => {
            let hse_config = config
                .hse
                .unwrap_or_else(|| panic!("HSE must be configured to be used as PLL input"));
            (hse_config.frequency, Sw::HSE)
        }
        ClockSrc::PLL => {
            RCC.cr().modify(|w| w.set_pllon(true));
            while !RCC.cr().read().pllrdy() {}
            (pll_clocks.main_freq, Sw::PLL)
        }
    };
    // RM0033 Figure 9. Clock tree suggests max SYSCLK/HCLK is 168 MHz, but datasheet specifies PLL
    // max output to be 120 MHz, so there's no way to get higher frequencies
    assert!(sys_clk <= Hertz(120_000_000));

    let ahb_freq = sys_clk / config.ahb_pre;
    // Reference: STM32F215xx/217xx datasheet Table 13. General operating conditions
    assert!(ahb_freq <= Hertz(120_000_000));

    let (apb1_freq, apb1_tim_freq) = match config.apb1_pre {
        APBPrescaler::NotDivided => (ahb_freq, ahb_freq),
        pre => {
            let freq = ahb_freq / pre;
            (freq, Hertz(freq.0 * 2))
        }
    };
    // Reference: STM32F215xx/217xx datasheet Table 13. General operating conditions
    assert!(apb1_freq <= Hertz(30_000_000));

    let (apb2_freq, apb2_tim_freq) = match config.apb2_pre {
        APBPrescaler::NotDivided => (ahb_freq, ahb_freq),
        pre => {
            let freq = ahb_freq / pre;
            (freq, Hertz(freq.0 * 2))
        }
    };
    // Reference: STM32F215xx/217xx datasheet Table 13. General operating conditions
    assert!(apb2_freq <= Hertz(60_000_000));

    let flash_ws = unwrap!(config.voltage.wait_states(ahb_freq));
    FLASH.acr().modify(|w| w.set_latency(flash_ws));

    RCC.cfgr().modify(|w| {
        w.set_sw(sw.into());
        w.set_hpre(config.ahb_pre.into());
        w.set_ppre1(config.apb1_pre.into());
        w.set_ppre2(config.apb2_pre.into());
    });
    while RCC.cfgr().read().sws().to_bits() != sw.to_bits() {}

    // Turn off HSI to save power if we don't need it
    if !config.hsi {
        RCC.cr().modify(|w| w.set_hsion(false));
    }

    RCC.apb1enr().modify(|w| w.set_pwren(true));
    PWR.cr().read();

    match config.rtc {
        Some(RtcClockSource::LSE) => {
            // 1. Unlock the backup domain
            PWR.cr().modify(|w| w.set_dbp(true));

            // 2. Setup the LSE
            RCC.bdcr().modify(|w| {
                // Enable LSE
                w.set_lseon(true);
            });

            // Wait until LSE is running
            while !RCC.bdcr().read().lserdy() {}

            BackupDomain::set_rtc_clock_source(RtcClockSource::LSE);
        }
        Some(RtcClockSource::LSI) => {
            // Turn on the internal 32 kHz LSI oscillator
            RCC.csr().modify(|w| w.set_lsion(true));

            // Wait until LSI is running
            while !RCC.csr().read().lsirdy() {}

            BackupDomain::set_rtc_clock_source(RtcClockSource::LSI);
        }
        _ => todo!(),
    }

    set_freqs(Clocks {
        sys: sys_clk,
        ahb1: ahb_freq,
        ahb2: ahb_freq,
        ahb3: ahb_freq,
        apb1: apb1_freq,
        apb1_tim: apb1_tim_freq,
        apb2: apb2_freq,
        apb2_tim: apb2_tim_freq,
        pll48: Some(pll_clocks.pll48_freq),
    });
}
