use stm32_metapac::flash::vals::Latency;
use stm32_metapac::rcc::vals::{Adcsel, Sw};
use stm32_metapac::FLASH;

pub use crate::pac::rcc::vals::{
    Adcsel as AdcClockSource, Clk48sel as Clk48Src, Fdcansel as FdCanClockSource, Hpre as AHBPrescaler,
    Pllm as PllPreDiv, Plln as PllMul, Pllp as PllPDiv, Pllq as PllQDiv, Pllr as PllRDiv, Pllsrc, Ppre as APBPrescaler,
    Sw as Sysclk,
};
use crate::pac::{PWR, RCC};
use crate::time::Hertz;

/// HSI speed
pub const HSI_FREQ: Hertz = Hertz(16_000_000);

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum HseMode {
    /// crystal/ceramic oscillator (HSEBYP=0)
    Oscillator,
    /// external analog clock (low swing) (HSEBYP=1)
    Bypass,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Hse {
    /// HSE frequency.
    pub freq: Hertz,
    /// HSE mode.
    pub mode: HseMode,
}

/// PLL Configuration
///
/// Use this struct to configure the PLL source, input frequency, multiplication factor, and output
/// dividers. Be sure to keep check the datasheet for your specific part for the appropriate
/// frequency ranges for each of these settings.
pub struct Pll {
    /// PLL Source clock selection.
    pub source: Pllsrc,

    /// PLL pre-divider
    pub prediv: PllPreDiv,

    /// PLL multiplication factor for VCO
    pub mul: PllMul,

    /// PLL division factor for P clock (ADC Clock)
    pub divp: Option<PllPDiv>,

    /// PLL division factor for Q clock (USB, I2S23, SAI1, FDCAN, QSPI)
    pub divq: Option<PllQDiv>,

    /// PLL division factor for R clock (SYSCLK)
    pub divr: Option<PllRDiv>,
}

/// Clocks configutation
#[non_exhaustive]
pub struct Config {
    pub hsi: bool,
    pub hse: Option<Hse>,
    pub sys: Sysclk,
    pub hsi48: Option<super::Hsi48Config>,

    pub pll: Option<Pll>,

    /// Iff PLL is requested as the main clock source in the `mux` field then the PLL configuration
    /// MUST turn on the PLLR output.
    pub ahb_pre: AHBPrescaler,
    pub apb1_pre: APBPrescaler,
    pub apb2_pre: APBPrescaler,
    pub low_power_run: bool,

    /// Sets the clock source for the 48MHz clock used by the USB and RNG peripherals.
    pub clk48_src: Clk48Src,

    pub ls: super::LsConfig,

    pub adc12_clock_source: AdcClockSource,
    pub adc345_clock_source: AdcClockSource,
    pub fdcan_clock_source: FdCanClockSource,

    pub boost: bool,
}

impl Default for Config {
    #[inline]
    fn default() -> Config {
        Config {
            hsi: true,
            hse: None,
            sys: Sysclk::HSI,
            hsi48: Some(Default::default()),
            pll: None,
            ahb_pre: AHBPrescaler::DIV1,
            apb1_pre: APBPrescaler::DIV1,
            apb2_pre: APBPrescaler::DIV1,
            low_power_run: false,
            clk48_src: Clk48Src::HSI48,
            ls: Default::default(),
            adc12_clock_source: Adcsel::DISABLE,
            adc345_clock_source: Adcsel::DISABLE,
            fdcan_clock_source: FdCanClockSource::PCLK1,
            boost: false,
        }
    }
}

pub struct PllFreq {
    pub pll_p: Option<Hertz>,
    pub pll_q: Option<Hertz>,
    pub pll_r: Option<Hertz>,
}

pub(crate) unsafe fn init(config: Config) {
    // Configure HSI
    let hsi = match config.hsi {
        false => {
            RCC.cr().modify(|w| w.set_hsion(false));
            None
        }
        true => {
            RCC.cr().modify(|w| w.set_hsion(true));
            while !RCC.cr().read().hsirdy() {}
            Some(HSI_FREQ)
        }
    };

    // Configure HSE
    let hse = match config.hse {
        None => {
            RCC.cr().modify(|w| w.set_hseon(false));
            None
        }
        Some(hse) => {
            match hse.mode {
                HseMode::Bypass => assert!(max::HSE_BYP.contains(&hse.freq)),
                HseMode::Oscillator => assert!(max::HSE_OSC.contains(&hse.freq)),
            }

            RCC.cr().modify(|w| w.set_hsebyp(hse.mode != HseMode::Oscillator));
            RCC.cr().modify(|w| w.set_hseon(true));
            while !RCC.cr().read().hserdy() {}
            Some(hse.freq)
        }
    };

    // Configure HSI48 if required
    if let Some(hsi48_config) = config.hsi48 {
        super::init_hsi48(hsi48_config);
    }

    let pll_freq = config.pll.map(|pll_config| {
        let src_freq = match pll_config.source {
            Pllsrc::HSI => unwrap!(hsi),
            Pllsrc::HSE => unwrap!(hse),
            _ => unreachable!(),
        };

        // TODO: check PLL input, internal and output frequencies for validity

        // Disable PLL before configuration
        RCC.cr().modify(|w| w.set_pllon(false));
        while RCC.cr().read().pllrdy() {}

        let internal_freq = src_freq / pll_config.prediv * pll_config.mul;

        RCC.pllcfgr().write(|w| {
            w.set_plln(pll_config.mul);
            w.set_pllm(pll_config.prediv);
            w.set_pllsrc(pll_config.source.into());
        });

        let pll_p_freq = pll_config.divp.map(|div_p| {
            RCC.pllcfgr().modify(|w| {
                w.set_pllp(div_p);
                w.set_pllpen(true);
            });
            internal_freq / div_p
        });

        let pll_q_freq = pll_config.divq.map(|div_q| {
            RCC.pllcfgr().modify(|w| {
                w.set_pllq(div_q);
                w.set_pllqen(true);
            });
            internal_freq / div_q
        });

        let pll_r_freq = pll_config.divr.map(|div_r| {
            RCC.pllcfgr().modify(|w| {
                w.set_pllr(div_r);
                w.set_pllren(true);
            });
            internal_freq / div_r
        });

        // Enable the PLL
        RCC.cr().modify(|w| w.set_pllon(true));
        while !RCC.cr().read().pllrdy() {}

        PllFreq {
            pll_p: pll_p_freq,
            pll_q: pll_q_freq,
            pll_r: pll_r_freq,
        }
    });

    let (sys_clk, sw) = match config.sys {
        Sysclk::HSI => (HSI_FREQ, Sw::HSI),
        Sysclk::HSE => (unwrap!(hse), Sw::HSE),
        Sysclk::PLL1_R => {
            assert!(pll_freq.is_some());
            assert!(pll_freq.as_ref().unwrap().pll_r.is_some());

            let freq = pll_freq.as_ref().unwrap().pll_r.unwrap().0;

            assert!(freq <= 170_000_000);

            if config.boost {
                // Enable Core Boost mode ([RM0440] p234)
                PWR.cr5().modify(|w| w.set_r1mode(false));
                if freq <= 36_000_000 {
                    FLASH.acr().modify(|w| w.set_latency(Latency::WS0));
                } else if freq <= 68_000_000 {
                    FLASH.acr().modify(|w| w.set_latency(Latency::WS1));
                } else if freq <= 102_000_000 {
                    FLASH.acr().modify(|w| w.set_latency(Latency::WS2));
                } else if freq <= 136_000_000 {
                    FLASH.acr().modify(|w| w.set_latency(Latency::WS3));
                } else {
                    FLASH.acr().modify(|w| w.set_latency(Latency::WS4));
                }
            } else {
                // Enable Core Boost mode ([RM0440] p234)
                PWR.cr5().modify(|w| w.set_r1mode(true));
                if freq <= 30_000_000 {
                    FLASH.acr().modify(|w| w.set_latency(Latency::WS0));
                } else if freq <= 60_000_000 {
                    FLASH.acr().modify(|w| w.set_latency(Latency::WS1));
                } else if freq <= 80_000_000 {
                    FLASH.acr().modify(|w| w.set_latency(Latency::WS2));
                } else if freq <= 120_000_000 {
                    FLASH.acr().modify(|w| w.set_latency(Latency::WS3));
                } else {
                    FLASH.acr().modify(|w| w.set_latency(Latency::WS4));
                }
            }

            (Hertz(freq), Sw::PLL1_R)
        }
        _ => unreachable!(),
    };

    RCC.cfgr().modify(|w| {
        w.set_sw(sw);
        w.set_hpre(config.ahb_pre);
        w.set_ppre1(config.apb1_pre);
        w.set_ppre2(config.apb2_pre);
    });

    let ahb_freq = sys_clk / config.ahb_pre;

    let (apb1_freq, apb1_tim_freq) = match config.apb1_pre {
        APBPrescaler::DIV1 => (ahb_freq, ahb_freq),
        pre => {
            let freq = ahb_freq / pre;
            (freq, freq * 2u32)
        }
    };

    let (apb2_freq, apb2_tim_freq) = match config.apb2_pre {
        APBPrescaler::DIV1 => (ahb_freq, ahb_freq),
        pre => {
            let freq = ahb_freq / pre;
            (freq, freq * 2u32)
        }
    };

    // Configure the 48MHz clock source for USB and RNG peripherals.
    {
        let source = match config.clk48_src {
            Clk48Src::PLL1_Q => {
                // Make sure the PLLQ is enabled and running at 48Mhz
                let pllq_freq = pll_freq.as_ref().and_then(|f| f.pll_q);
                assert!(pllq_freq.is_some() && pllq_freq.unwrap().0 == 48_000_000);

                crate::pac::rcc::vals::Clk48sel::PLL1_Q
            }
            Clk48Src::HSI48 => {
                // Make sure HSI48 is enabled
                assert!(config.hsi48.is_some());
                crate::pac::rcc::vals::Clk48sel::HSI48
            }
            _ => unreachable!(),
        };

        RCC.ccipr().modify(|w| w.set_clk48sel(source));
    }

    RCC.ccipr().modify(|w| w.set_adc12sel(config.adc12_clock_source));
    RCC.ccipr().modify(|w| w.set_adc345sel(config.adc345_clock_source));
    RCC.ccipr().modify(|w| w.set_fdcansel(config.fdcan_clock_source));

    let adc12_ck = match config.adc12_clock_source {
        AdcClockSource::DISABLE => None,
        AdcClockSource::PLL1_P => pll_freq.as_ref().unwrap().pll_p,
        AdcClockSource::SYS => Some(sys_clk),
        _ => unreachable!(),
    };

    let adc345_ck = match config.adc345_clock_source {
        AdcClockSource::DISABLE => None,
        AdcClockSource::PLL1_P => pll_freq.as_ref().unwrap().pll_p,
        AdcClockSource::SYS => Some(sys_clk),
        _ => unreachable!(),
    };

    if config.low_power_run {
        assert!(sys_clk <= Hertz(2_000_000));
        PWR.cr1().modify(|w| w.set_lpr(true));
    }

    let rtc = config.ls.init();

    set_clocks!(
        sys: Some(sys_clk),
        hclk1: Some(ahb_freq),
        hclk2: Some(ahb_freq),
        hclk3: Some(ahb_freq),
        pclk1: Some(apb1_freq),
        pclk1_tim: Some(apb1_tim_freq),
        pclk2: Some(apb2_freq),
        pclk2_tim: Some(apb2_tim_freq),
        adc: adc12_ck,
        adc34: adc345_ck,
        pll1_p: pll_freq.as_ref().and_then(|pll| pll.pll_p),
        pll1_q: pll_freq.as_ref().and_then(|pll| pll.pll_p),
        hse: hse,
        rtc: rtc,
    );
}

// TODO: if necessary, make more of these gated behind cfg attrs
mod max {
    use core::ops::RangeInclusive;

    use crate::time::Hertz;

    /// HSE 4-48MHz (RM0440 p280)
    pub(crate) const HSE_OSC: RangeInclusive<Hertz> = Hertz(4_000_000)..=Hertz(48_000_000);

    /// External Clock ?-48MHz (RM0440 p280)
    pub(crate) const HSE_BYP: RangeInclusive<Hertz> = Hertz(0)..=Hertz(48_000_000);

    // SYSCLK ?-170MHz (RM0440 p282)
    //pub(crate) const SYSCLK: RangeInclusive<Hertz> = Hertz(0)..=Hertz(170_000_000);

    // PLL Output frequency ?-170MHz (RM0440 p281)
    //pub(crate) const PCLK: RangeInclusive<Hertz> = Hertz(0)..=Hertz(170_000_000);

    // Left over from f.rs, remove if not necessary
    //pub(crate) const HCLK: RangeInclusive<Hertz> = Hertz(12_500_000)..=Hertz(216_000_000);
    //pub(crate) const PLL_IN: RangeInclusive<Hertz> = Hertz(1_000_000)..=Hertz(2_100_000);
    //pub(crate) const PLL_VCO: RangeInclusive<Hertz> = Hertz(100_000_000)..=Hertz(432_000_000);
}
