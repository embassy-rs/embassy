use crate::pac::flash::vals::Latency;
use crate::pac::rcc::vals::Sw;
pub use crate::pac::rcc::vals::{
    Hpre as AHBPrescaler, Pllm as PLLPreDiv, Plln as PLLMul, Pllp as PLLPDiv, Pllq as PLLQDiv, Pllsrc as PLLSrc,
    Ppre as APBPrescaler,
};
use crate::pac::{FLASH, RCC};
use crate::rcc::{set_freqs, Clocks};
use crate::time::Hertz;

/// HSI speed
pub const HSI_FREQ: Hertz = Hertz(16_000_000);

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
    pub p_div: PLLPDiv,
    pub q_div: PLLQDiv,
}

impl Default for PLLConfig {
    fn default() -> Self {
        PLLConfig {
            pre_div: PLLPreDiv::DIV16,
            mul: PLLMul::MUL192,
            p_div: PLLPDiv::DIV2,
            q_div: PLLQDiv::DIV4,
        }
    }
}

impl PLLConfig {
    pub fn clocks(&self, src_freq: Hertz) -> PLLClocks {
        let in_freq = src_freq / self.pre_div;
        let vco_freq = src_freq / self.pre_div * self.mul;
        let main_freq = vco_freq / self.p_div;
        let pll48_freq = vco_freq / self.q_div;
        PLLClocks {
            in_freq,
            vco_freq,
            main_freq,
            pll48_freq,
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

/// Voltage range of the power supply used.
///
/// Used to calculate flash waitstates. See
/// RM0033 - Table 3. Number of wait states according to Cortex®-M3 clock frequency
pub enum VoltageScale {
    /// 2.7 to 3.6 V
    Range0,
    /// 2.4 to 2.7 V
    Range1,
    /// 2.1 to 2.4 V
    Range2,
    /// 1.8 to 2.1 V
    Range3,
}

impl VoltageScale {
    const fn wait_states(&self, ahb_freq: Hertz) -> Option<Latency> {
        let ahb_freq = ahb_freq.0;
        // Reference: RM0033 - Table 3. Number of wait states according to Cortex®-M3 clock
        // frequency
        match self {
            VoltageScale::Range3 => {
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
            VoltageScale::Range2 => {
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
            VoltageScale::Range1 => {
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
            VoltageScale::Range0 => {
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
    pub voltage: VoltageScale,
    pub ahb_pre: AHBPrescaler,
    pub apb1_pre: APBPrescaler,
    pub apb2_pre: APBPrescaler,
    pub ls: super::LsConfig,
}

impl Default for Config {
    #[inline]
    fn default() -> Config {
        Config {
            hse: None,
            hsi: true,
            pll_mux: PLLSrc::HSI,
            pll: PLLConfig::default(),
            voltage: VoltageScale::Range3,
            mux: ClockSrc::HSI,
            ahb_pre: AHBPrescaler::DIV1,
            apb1_pre: APBPrescaler::DIV1,
            apb2_pre: APBPrescaler::DIV1,
            ls: Default::default(),
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
        w.set_pllsrc(config.pll_mux);
        w.set_pllm(config.pll.pre_div);
        w.set_plln(config.pll.mul);
        w.set_pllp(config.pll.p_div);
        w.set_pllq(config.pll.q_div);
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
            (pll_clocks.main_freq, Sw::PLL1_P)
        }
    };
    // RM0033 Figure 9. Clock tree suggests max SYSCLK/HCLK is 168 MHz, but datasheet specifies PLL
    // max output to be 120 MHz, so there's no way to get higher frequencies
    assert!(sys_clk <= Hertz(120_000_000));

    let ahb_freq = sys_clk / config.ahb_pre;
    // Reference: STM32F215xx/217xx datasheet Table 13. General operating conditions
    assert!(ahb_freq <= Hertz(120_000_000));

    let (apb1_freq, apb1_tim_freq) = match config.apb1_pre {
        APBPrescaler::DIV1 => (ahb_freq, ahb_freq),
        pre => {
            let freq = ahb_freq / pre;
            (freq, Hertz(freq.0 * 2))
        }
    };
    // Reference: STM32F215xx/217xx datasheet Table 13. General operating conditions
    assert!(apb1_freq <= Hertz(30_000_000));

    let (apb2_freq, apb2_tim_freq) = match config.apb2_pre {
        APBPrescaler::DIV1 => (ahb_freq, ahb_freq),
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
        w.set_hpre(config.ahb_pre);
        w.set_ppre1(config.apb1_pre);
        w.set_ppre2(config.apb2_pre);
    });
    while RCC.cfgr().read().sws().to_bits() != sw.to_bits() {}

    // Turn off HSI to save power if we don't need it
    if !config.hsi {
        RCC.cr().modify(|w| w.set_hsion(false));
    }

    let rtc = config.ls.init();

    set_freqs(Clocks {
        sys: sys_clk,
        hclk1: ahb_freq,
        hclk2: ahb_freq,
        hclk3: ahb_freq,
        pclk1: apb1_freq,
        pclk1_tim: apb1_tim_freq,
        pclk2: apb2_freq,
        pclk2_tim: apb2_tim_freq,
        pll1_q: Some(pll_clocks.pll48_freq),
        rtc,
    });
}
