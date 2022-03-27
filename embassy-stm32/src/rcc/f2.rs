use core::ops::Div;

use crate::pac::flash::vals::Latency;
use crate::pac::rcc::vals::{Hpre, Ppre, Sw};
use crate::pac::{FLASH, RCC};
use crate::rcc::{set_freqs, Clocks};
use crate::time::Hertz;

/// HSI speed
pub const HSI: Hertz = Hertz(16_000_000);

/// System clock mux source
#[derive(Clone, Copy)]
pub enum ClockSrc {
    HSE(Hertz, HSESrc),
    HSI,
}

/// HSE clock source
#[derive(Clone, Copy)]
pub enum HSESrc {
    /// Crystal/ceramic resonator
    Crystal,
    /// External clock source, HSE bypassed
    Bypass,
}

/// AHB prescaler
#[derive(Clone, Copy, PartialEq)]
pub enum AHBPrescaler {
    NotDivided,
    Div2,
    Div4,
    Div8,
    Div16,
    Div64,
    Div128,
    Div256,
    Div512,
}

impl Div<AHBPrescaler> for Hertz {
    type Output = Hertz;

    fn div(self, rhs: AHBPrescaler) -> Self::Output {
        let divisor = match rhs {
            AHBPrescaler::NotDivided => 1,
            AHBPrescaler::Div2 => 2,
            AHBPrescaler::Div4 => 4,
            AHBPrescaler::Div8 => 8,
            AHBPrescaler::Div16 => 16,
            AHBPrescaler::Div64 => 64,
            AHBPrescaler::Div128 => 128,
            AHBPrescaler::Div256 => 256,
            AHBPrescaler::Div512 => 512,
        };
        Hertz(self.0 / divisor)
    }
}

/// APB prescaler
#[derive(Clone, Copy)]
pub enum APBPrescaler {
    NotDivided,
    Div2,
    Div4,
    Div8,
    Div16,
}

impl Div<APBPrescaler> for Hertz {
    type Output = Hertz;

    fn div(self, rhs: APBPrescaler) -> Self::Output {
        let divisor = match rhs {
            APBPrescaler::NotDivided => 1,
            APBPrescaler::Div2 => 2,
            APBPrescaler::Div4 => 4,
            APBPrescaler::Div8 => 8,
            APBPrescaler::Div16 => 16,
        };
        Hertz(self.0 / divisor)
    }
}

impl Into<Ppre> for APBPrescaler {
    fn into(self) -> Ppre {
        match self {
            APBPrescaler::NotDivided => Ppre::DIV1,
            APBPrescaler::Div2 => Ppre::DIV2,
            APBPrescaler::Div4 => Ppre::DIV4,
            APBPrescaler::Div8 => Ppre::DIV8,
            APBPrescaler::Div16 => Ppre::DIV16,
        }
    }
}

impl Into<Hpre> for AHBPrescaler {
    fn into(self) -> Hpre {
        match self {
            AHBPrescaler::NotDivided => Hpre::DIV1,
            AHBPrescaler::Div2 => Hpre::DIV2,
            AHBPrescaler::Div4 => Hpre::DIV4,
            AHBPrescaler::Div8 => Hpre::DIV8,
            AHBPrescaler::Div16 => Hpre::DIV16,
            AHBPrescaler::Div64 => Hpre::DIV64,
            AHBPrescaler::Div128 => Hpre::DIV128,
            AHBPrescaler::Div256 => Hpre::DIV256,
            AHBPrescaler::Div512 => Hpre::DIV512,
        }
    }
}

/// Voltage Range
///
/// Represents the system supply voltage range
#[derive(Copy, Clone, PartialEq)]
pub enum VoltageRange {
    /// 1.8 to 3.6 V
    Min1V8,
    /// 2.1 to 3.6 V
    Min2V1,
    /// 2.4 to 3.6 V
    Min2V4,
    /// 2.7 to 3.6 V
    Min2V7,
}

impl VoltageRange {
    const fn wait_states(&self, ahb_freq: Hertz) -> Option<Latency> {
        let ahb_freq = ahb_freq.0;
        // Reference: RM0033 - Table 3. Number of wait states according to Cortex®-M3 clock
        // frequency
        match self {
            VoltageRange::Min1V8 => {
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
            VoltageRange::Min2V1 => {
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
            VoltageRange::Min2V4 => {
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
            VoltageRange::Min2V7 => {
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
    pub mux: ClockSrc,
    pub voltage: VoltageRange,
    pub ahb_pre: AHBPrescaler,
    pub apb1_pre: APBPrescaler,
    pub apb2_pre: APBPrescaler,
}

impl Default for Config {
    #[inline]
    fn default() -> Config {
        Config {
            voltage: VoltageRange::Min1V8,
            mux: ClockSrc::HSI,
            ahb_pre: AHBPrescaler::NotDivided,
            apb1_pre: APBPrescaler::NotDivided,
            apb2_pre: APBPrescaler::NotDivided,
        }
    }
}

#[inline]
unsafe fn enable_hse(source: HSESrc) {
    RCC.cr().write(|w| {
        w.set_hsebyp(match source {
            HSESrc::Bypass => true,
            HSESrc::Crystal => false,
        });
        w.set_hseon(true)
    });
    while !RCC.cr().read().hserdy() {}
}

pub(crate) unsafe fn init(config: Config) {
    let (sys_clk, sw) = match config.mux {
        ClockSrc::HSI => {
            // Enable HSI
            RCC.cr().write(|w| w.set_hsion(true));
            while !RCC.cr().read().hsirdy() {}

            (HSI, Sw::HSI)
        }
        ClockSrc::HSE(freq, source) => {
            enable_hse(source);
            (freq, Sw::HSE)
        }
    };
    // RM0033 Figure 9. Clock tree suggests max SYSCLK/HCLK is 168 MHz, but datasheet specifies PLL
    // max output to be 120 MHz, so there's no way to get higher frequencies
    assert!(sys_clk <= Hertz(120_000_000));

    let ahb_freq = sys_clk / config.ahb_pre;
    // Reference: STM32F215xx/217xx datasheet Table 13. General operating conditions
    assert!(ahb_freq <= Hertz(120_000_000));

    let flash_ws = config.voltage.wait_states(ahb_freq).expect("Invalid HCLK");
    FLASH.acr().modify(|w| w.set_latency(flash_ws));

    RCC.cfgr().modify(|w| {
        w.set_sw(sw.into());
        w.set_hpre(config.ahb_pre.into());
        w.set_ppre1(config.apb1_pre.into());
        w.set_ppre2(config.apb2_pre.into());
    });

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

    set_freqs(Clocks {
        sys: sys_clk,
        ahb1: ahb_freq,
        ahb2: ahb_freq,
        ahb3: ahb_freq,
        apb1: apb1_freq,
        apb1_tim: apb1_tim_freq,
        apb2: apb2_freq,
        apb2_tim: apb2_tim_freq,
    });
}
