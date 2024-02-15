use stm32_metapac::flash::vals::Latency;
use stm32_metapac::rcc::vals::{Adcsel, Pllsrc, Sw};
use stm32_metapac::FLASH;

pub use crate::pac::rcc::vals::{
    Adcsel as AdcClockSource, Fdcansel as FdCanClockSource, Hpre as AHBPrescaler, Pllm as PllM, Plln as PllN,
    Pllp as PllP, Pllq as PllQ, Pllr as PllR, Ppre as APBPrescaler,
};
use crate::pac::{PWR, RCC};
use crate::time::Hertz;

/// HSI speed
pub const HSI_FREQ: Hertz = Hertz(16_000_000);

/// System clock mux source
#[derive(Clone, Copy)]
pub enum ClockSrc {
    HSE(Hertz),
    HSI,
    PLL,
}

/// PLL clock input source
#[derive(Clone, Copy, Debug)]
pub enum PllSource {
    HSI,
    HSE(Hertz),
}

impl Into<Pllsrc> for PllSource {
    fn into(self) -> Pllsrc {
        match self {
            PllSource::HSE(..) => Pllsrc::HSE,
            PllSource::HSI => Pllsrc::HSI,
        }
    }
}

/// PLL Configuration
///
/// Use this struct to configure the PLL source, input frequency, multiplication factor, and output
/// dividers. Be sure to keep check the datasheet for your specific part for the appropriate
/// frequency ranges for each of these settings.
pub struct Pll {
    /// PLL Source clock selection.
    pub source: PllSource,

    /// PLL pre-divider
    pub prediv_m: PllM,

    /// PLL multiplication factor for VCO
    pub mul_n: PllN,

    /// PLL division factor for P clock (ADC Clock)
    pub div_p: Option<PllP>,

    /// PLL division factor for Q clock (USB, I2S23, SAI1, FDCAN, QSPI)
    pub div_q: Option<PllQ>,

    /// PLL division factor for R clock (SYSCLK)
    pub div_r: Option<PllR>,
}

/// Sets the source for the 48MHz clock to the USB and RNG peripherals.
pub enum Clock48MhzSrc {
    /// Use the High Speed Internal Oscillator. For USB usage, the CRS must be used to calibrate the
    /// oscillator to comply with the USB specification for oscillator tolerance.
    Hsi48(super::Hsi48Config),
    /// Use the PLLQ output. The PLL must be configured to output a 48MHz clock. For USB usage the
    /// PLL needs to be using the HSE source to comply with the USB specification for oscillator
    /// tolerance.
    PllQ,
}

/// Clocks configutation
pub struct Config {
    pub mux: ClockSrc,
    pub ahb_pre: AHBPrescaler,
    pub apb1_pre: APBPrescaler,
    pub apb2_pre: APBPrescaler,
    pub low_power_run: bool,
    /// Iff PLL is requested as the main clock source in the `mux` field then the PLL configuration
    /// MUST turn on the PLLR output.
    pub pll: Option<Pll>,
    /// Sets the clock source for the 48MHz clock used by the USB and RNG peripherals.
    pub clock_48mhz_src: Option<Clock48MhzSrc>,
    pub adc12_clock_source: AdcClockSource,
    pub adc345_clock_source: AdcClockSource,
    pub fdcan_clock_source: FdCanClockSource,

    pub ls: super::LsConfig,
}

impl Default for Config {
    #[inline]
    fn default() -> Config {
        Config {
            mux: ClockSrc::HSI,
            ahb_pre: AHBPrescaler::DIV1,
            apb1_pre: APBPrescaler::DIV1,
            apb2_pre: APBPrescaler::DIV1,
            low_power_run: false,
            pll: None,
            clock_48mhz_src: Some(Clock48MhzSrc::Hsi48(Default::default())),
            adc12_clock_source: Adcsel::DISABLE,
            adc345_clock_source: Adcsel::DISABLE,
            fdcan_clock_source: FdCanClockSource::PCLK1,
            ls: Default::default(),
        }
    }
}

pub struct PllFreq {
    pub pll_p: Option<Hertz>,
    pub pll_q: Option<Hertz>,
    pub pll_r: Option<Hertz>,
}

pub(crate) unsafe fn init(config: Config) {
    let pll_freq = config.pll.map(|pll_config| {
        let src_freq = match pll_config.source {
            PllSource::HSI => {
                RCC.cr().write(|w| w.set_hsion(true));
                while !RCC.cr().read().hsirdy() {}

                HSI_FREQ
            }
            PllSource::HSE(freq) => {
                RCC.cr().write(|w| w.set_hseon(true));
                while !RCC.cr().read().hserdy() {}
                freq
            }
        };

        // Disable PLL before configuration
        RCC.cr().modify(|w| w.set_pllon(false));
        while RCC.cr().read().pllrdy() {}

        let internal_freq = src_freq / pll_config.prediv_m * pll_config.mul_n;

        RCC.pllcfgr().write(|w| {
            w.set_plln(pll_config.mul_n);
            w.set_pllm(pll_config.prediv_m);
            w.set_pllsrc(pll_config.source.into());
        });

        let pll_p_freq = pll_config.div_p.map(|div_p| {
            RCC.pllcfgr().modify(|w| {
                w.set_pllp(div_p);
                w.set_pllpen(true);
            });
            internal_freq / div_p
        });

        let pll_q_freq = pll_config.div_q.map(|div_q| {
            RCC.pllcfgr().modify(|w| {
                w.set_pllq(div_q);
                w.set_pllqen(true);
            });
            internal_freq / div_q
        });

        let pll_r_freq = pll_config.div_r.map(|div_r| {
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

    let (sys_clk, sw) = match config.mux {
        ClockSrc::HSI => {
            // Enable HSI
            RCC.cr().write(|w| w.set_hsion(true));
            while !RCC.cr().read().hsirdy() {}

            (HSI_FREQ, Sw::HSI)
        }
        ClockSrc::HSE(freq) => {
            // Enable HSE
            RCC.cr().write(|w| w.set_hseon(true));
            while !RCC.cr().read().hserdy() {}

            (freq, Sw::HSE)
        }
        ClockSrc::PLL => {
            assert!(pll_freq.is_some());
            assert!(pll_freq.as_ref().unwrap().pll_r.is_some());

            let freq = pll_freq.as_ref().unwrap().pll_r.unwrap().0;

            assert!(freq <= 170_000_000);

            if freq >= 150_000_000 {
                // Enable Core Boost mode on freq >= 150Mhz ([RM0440] p234)
                PWR.cr5().modify(|w| w.set_r1mode(false));
                // Set flash wait state in boost mode based on frequency ([RM0440] p191)
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
                PWR.cr5().modify(|w| w.set_r1mode(true));
                // Set flash wait state in normal mode based on frequency ([RM0440] p191)
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

    // Setup the 48 MHz clock if needed
    if let Some(clock_48mhz_src) = config.clock_48mhz_src {
        let source = match clock_48mhz_src {
            Clock48MhzSrc::PllQ => {
                // Make sure the PLLQ is enabled and running at 48Mhz
                let pllq_freq = pll_freq.as_ref().and_then(|f| f.pll_q);
                assert!(pllq_freq.is_some() && pllq_freq.unwrap().0 == 48_000_000);

                crate::pac::rcc::vals::Clk48sel::PLL1_Q
            }
            Clock48MhzSrc::Hsi48(config) => {
                super::init_hsi48(config);
                crate::pac::rcc::vals::Clk48sel::HSI48
            }
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
        pll1_p: None,
        pll1_q: None, // TODO
        hse: None,    // TODO
        rtc: rtc,
    );
}
