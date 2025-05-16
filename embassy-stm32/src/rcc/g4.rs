use crate::pac::flash::vals::Latency;
pub use crate::pac::rcc::vals::{
    Hpre as AHBPrescaler, Pllm as PllPreDiv, Plln as PllMul, Pllp as PllPDiv, Pllq as PllQDiv, Pllr as PllRDiv,
    Pllsrc as PllSource, Ppre as APBPrescaler, Sw as Sysclk,
};
use crate::pac::{FLASH, PWR, RCC};
use crate::time::Hertz;

/// HSI speed
pub const HSI_FREQ: Hertz = Hertz(16_000_000);

/// HSE Mode
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum HseMode {
    /// crystal/ceramic oscillator (HSEBYP=0)
    Oscillator,
    /// external analog clock (low swing) (HSEBYP=1)
    Bypass,
}

/// HSE Configuration
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
#[derive(Clone, Copy)]
pub struct Pll {
    /// PLL Source clock selection.
    pub source: PllSource,

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
#[derive(Clone, Copy)]
pub struct Config {
    /// HSI Enable
    pub hsi: bool,

    /// HSE Configuration
    pub hse: Option<Hse>,

    /// System Clock Configuration
    pub sys: Sysclk,

    /// HSI48 Configuration
    pub hsi48: Option<super::Hsi48Config>,

    /// PLL Configuration
    pub pll: Option<Pll>,

    /// If PLL is requested as the main clock source in the `sys` field then the PLL configuration
    /// MUST turn on the PLLR output.
    pub ahb_pre: AHBPrescaler,
    pub apb1_pre: APBPrescaler,
    pub apb2_pre: APBPrescaler,

    pub low_power_run: bool,

    /// Low-Speed Clock Configuration
    pub ls: super::LsConfig,

    /// Enable range1 boost mode
    /// Recommended when the SYSCLK frequency is greater than 150MHz.
    pub boost: bool,

    /// Per-peripheral kernel clock selection muxes
    pub mux: super::mux::ClockMux,
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
            ls: Default::default(),
            boost: false,
            mux: Default::default(),
        }
    }
}

#[derive(Default)]
pub struct PllFreq {
    pub pll_p: Option<Hertz>,
    pub pll_q: Option<Hertz>,
    pub pll_r: Option<Hertz>,
}

pub(crate) unsafe fn init(config: Config) {
    // Turn on the HSI
    RCC.cr().modify(|w| w.set_hsion(true));
    while !RCC.cr().read().hsirdy() {}

    // Use the HSI clock as system clock during the actual clock setup
    RCC.cfgr().modify(|w| w.set_sw(Sysclk::HSI));
    while RCC.cfgr().read().sws() != Sysclk::HSI {}

    // Configure HSI
    let hsi = match config.hsi {
        false => None,
        true => Some(HSI_FREQ),
    };

    // Configure HSE
    let hse = match config.hse {
        None => {
            RCC.cr().modify(|w| w.set_hseon(false));
            None
        }
        Some(hse) => {
            match hse.mode {
                HseMode::Bypass => rcc_assert!(max::HSE_BYP.contains(&hse.freq)),
                HseMode::Oscillator => rcc_assert!(max::HSE_OSC.contains(&hse.freq)),
            }

            RCC.cr().modify(|w| w.set_hsebyp(hse.mode != HseMode::Oscillator));
            RCC.cr().modify(|w| w.set_hseon(true));
            while !RCC.cr().read().hserdy() {}
            Some(hse.freq)
        }
    };

    // Configure HSI48 if required
    let hsi48 = config.hsi48.map(super::init_hsi48);

    let pll = config
        .pll
        .map(|pll_config| {
            let src_freq = match pll_config.source {
                PllSource::HSI => unwrap!(hsi),
                PllSource::HSE => unwrap!(hse),
                _ => unreachable!(),
            };

            // Disable PLL before configuration
            RCC.cr().modify(|w| w.set_pllon(false));
            while RCC.cr().read().pllrdy() {}

            let in_freq = src_freq / pll_config.prediv;
            rcc_assert!(max::PLL_IN.contains(&in_freq));
            let internal_freq = in_freq * pll_config.mul;

            rcc_assert!(max::PLL_VCO.contains(&internal_freq));

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
                let freq = internal_freq / div_p;
                rcc_assert!(max::PLL_P.contains(&freq));
                freq
            });

            let pll_q_freq = pll_config.divq.map(|div_q| {
                RCC.pllcfgr().modify(|w| {
                    w.set_pllq(div_q);
                    w.set_pllqen(true);
                });
                let freq = internal_freq / div_q;
                rcc_assert!(max::PLL_Q.contains(&freq));
                freq
            });

            let pll_r_freq = pll_config.divr.map(|div_r| {
                RCC.pllcfgr().modify(|w| {
                    w.set_pllr(div_r);
                    w.set_pllren(true);
                });
                let freq = internal_freq / div_r;
                rcc_assert!(max::PLL_R.contains(&freq));
                freq
            });

            // Enable the PLL
            RCC.cr().modify(|w| w.set_pllon(true));
            while !RCC.cr().read().pllrdy() {}

            PllFreq {
                pll_p: pll_p_freq,
                pll_q: pll_q_freq,
                pll_r: pll_r_freq,
            }
        })
        .unwrap_or_default();

    let sys = match config.sys {
        Sysclk::HSI => unwrap!(hsi),
        Sysclk::HSE => unwrap!(hse),
        Sysclk::PLL1_R => unwrap!(pll.pll_r),
        _ => unreachable!(),
    };

    rcc_assert!(max::SYSCLK.contains(&sys));

    // Calculate the AHB frequency (HCLK), among other things so we can calculate the correct flash read latency.
    let hclk = sys / config.ahb_pre;
    rcc_assert!(max::HCLK.contains(&hclk));

    let (pclk1, pclk1_tim) = super::util::calc_pclk(hclk, config.apb1_pre);
    let (pclk2, pclk2_tim) = super::util::calc_pclk(hclk, config.apb2_pre);
    rcc_assert!(max::PCLK.contains(&pclk1));
    rcc_assert!(max::PCLK.contains(&pclk2));

    // Configure Core Boost mode ([RM0440] p234 – inverted because setting r1mode to 0 enables boost mode!)
    if config.boost {
        // RM0440 p235
        // “The sequence to switch from Range1 normal mode to Range1 boost mode is:
        // 1. The system clock must be divided by 2 using the AHB prescaler before switching to a higher system frequency.
        RCC.cfgr().modify(|w| w.set_hpre(AHBPrescaler::DIV2));
        // 2. Clear the R1MODE bit in the PWR_CR5 register. (enables boost mode)
        PWR.cr5().modify(|w| w.set_r1mode(false));

        // Below:
        // 3. Adjust wait states according to new freq target
        // 4. Configure and switch to new frequency
    }

    let latency = match (config.boost, hclk.0) {
        (true, ..=34_000_000) => Latency::WS0,
        (true, ..=68_000_000) => Latency::WS1,
        (true, ..=102_000_000) => Latency::WS2,
        (true, ..=136_000_000) => Latency::WS3,
        (true, _) => Latency::WS4,

        (false, ..=36_000_000) => Latency::WS0,
        (false, ..=60_000_000) => Latency::WS1,
        (false, ..=90_000_000) => Latency::WS2,
        (false, ..=120_000_000) => Latency::WS3,
        (false, _) => Latency::WS4,
    };

    // Configure flash read access latency based on boost mode and frequency (RM0440 p98)
    FLASH.acr().modify(|w| {
        w.set_latency(latency);
    });

    // Spin until the effective flash latency is set.
    while FLASH.acr().read().latency() != latency {}

    if config.boost {
        // 5. Wait for at least 1us and then reconfigure the AHB prescaler to get the needed HCLK clock frequency.
        cortex_m::asm::delay(16);
    }

    // Now that boost mode and flash read access latency are configured, set up SYSCLK
    RCC.cfgr().modify(|w| {
        w.set_sw(config.sys);
        w.set_hpre(config.ahb_pre);
        w.set_ppre1(config.apb1_pre);
        w.set_ppre2(config.apb2_pre);
    });
    while RCC.cfgr().read().sws() != config.sys {}

    // Disable HSI if not used
    if !config.hsi {
        RCC.cr().modify(|w| w.set_hsion(false));
    }

    if config.low_power_run {
        assert!(sys <= Hertz(2_000_000));
        PWR.cr1().modify(|w| w.set_lpr(true));
    }

    let rtc = config.ls.init();

    config.mux.init();

    set_clocks!(
        sys: Some(sys),
        hclk1: Some(hclk),
        hclk2: Some(hclk),
        hclk3: Some(hclk),
        pclk1: Some(pclk1),
        pclk1_tim: Some(pclk1_tim),
        pclk2: Some(pclk2),
        pclk2_tim: Some(pclk2_tim),
        pll1_p: pll.pll_p,
        pll1_q: pll.pll_q,
        pll1_r: pll.pll_r,
        hsi: hsi,
        hse: hse,
        hsi48: hsi48,
        rtc: rtc,
        lsi: None,
        lse: None,
    );
}

/// Acceptable Frequency Ranges
/// Currently assuming voltage scaling range 1 boost mode.
/// Where not specified in the generic G4 reference manual (RM0440), values taken from the STM32G474 datasheet.
/// If acceptable ranges for other G4-family chips differ, make additional max modules gated behind cfg attrs.
mod max {
    use core::ops::RangeInclusive;

    use crate::time::Hertz;

    /// HSE Frequency Range (RM0440 p280)
    pub(crate) const HSE_OSC: RangeInclusive<Hertz> = Hertz(4_000_000)..=Hertz(48_000_000);

    /// External Clock Frequency Range (RM0440 p280)
    pub(crate) const HSE_BYP: RangeInclusive<Hertz> = Hertz(0)..=Hertz(48_000_000);

    /// SYSCLK Frequency Range (RM0440 p282)
    pub(crate) const SYSCLK: RangeInclusive<Hertz> = Hertz(0)..=Hertz(170_000_000);

    /// PLL Output Frequency Range (RM0440 p281, STM32G474 Datasheet p123, Table 46)
    pub(crate) const PCLK: RangeInclusive<Hertz> = Hertz(8)..=Hertz(170_000_000);

    /// HCLK (AHB) Clock Frequency Range (STM32G474 Datasheet)
    pub(crate) const HCLK: RangeInclusive<Hertz> = Hertz(0)..=Hertz(170_000_000);

    /// PLL Source Frequency Range (STM32G474 Datasheet p123, Table 46)
    pub(crate) const PLL_IN: RangeInclusive<Hertz> = Hertz(2_660_000)..=Hertz(16_000_000);

    /// PLL VCO (internal) Frequency Range (STM32G474 Datasheet p123, Table 46)
    pub(crate) const PLL_VCO: RangeInclusive<Hertz> = Hertz(96_000_000)..=Hertz(344_000_000);
    pub(crate) const PLL_P: RangeInclusive<Hertz> = Hertz(2_064_500)..=Hertz(170_000_000);
    pub(crate) const PLL_Q: RangeInclusive<Hertz> = Hertz(8_000_000)..=Hertz(170_000_000);
    pub(crate) const PLL_R: RangeInclusive<Hertz> = Hertz(8_000_000)..=Hertz(170_000_000);
}
