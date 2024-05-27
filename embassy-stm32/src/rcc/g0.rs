use crate::pac::flash::vals::Latency;
pub use crate::pac::pwr::vals::Vos as VoltageRange;
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
pub struct Config {
    /// HSI Enable
    pub hsi: bool,

    /// HSE Configuration
    pub hse: Option<Hse>,

    /// System Clock Configuration
    pub sys: Sysclk,

    /// HSI48 Configuration
    #[cfg(crs)]
    pub hsi48: Option<super::Hsi48Config>,

    /// PLL Configuration
    pub pll: Option<Pll>,

    /// If PLL is requested as the main clock source in the `sys` field then the PLL configuration
    /// MUST turn on the PLLR output.
    pub ahb_pre: AHBPrescaler,
    pub apb1_pre: APBPrescaler,

    /// Low-Speed Clock Configuration
    pub ls: super::LsConfig,

    pub low_power_run: bool,

    pub voltage_range: VoltageRange,

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
            #[cfg(crs)]
            hsi48: Some(Default::default()),
            pll: None,
            ahb_pre: AHBPrescaler::DIV1,
            apb1_pre: APBPrescaler::DIV1,
            low_power_run: false,
            ls: Default::default(),
            voltage_range: VoltageRange::RANGE1,
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
    #[cfg(crs)]
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
            assert!(max::PLL_IN.contains(&in_freq));
            let internal_freq = in_freq * pll_config.mul;

            assert!(max::PLL_VCO.contains(&internal_freq));

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
                assert!(max::PLL_P.contains(&freq));
                freq
            });

            let pll_q_freq = pll_config.divq.map(|div_q| {
                RCC.pllcfgr().modify(|w| {
                    w.set_pllq(div_q);
                    w.set_pllqen(true);
                });
                let freq = internal_freq / div_q;
                assert!(max::PLL_Q.contains(&freq));
                freq
            });

            let pll_r_freq = pll_config.divr.map(|div_r| {
                RCC.pllcfgr().modify(|w| {
                    w.set_pllr(div_r);
                    w.set_pllren(true);
                });
                let freq = internal_freq / div_r;
                assert!(max::PLL_R.contains(&freq));
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

    assert!(max::SYSCLK.contains(&sys));

    // Calculate the AHB frequency (HCLK), among other things so we can calculate the correct flash read latency.
    let hclk = sys / config.ahb_pre;
    assert!(max::HCLK.contains(&hclk));

    let (pclk1, pclk1_tim) = super::util::calc_pclk(hclk, config.apb1_pre);
    assert!(max::PCLK.contains(&pclk1));

    let latency = match (config.voltage_range, hclk.0) {
        (VoltageRange::RANGE1, ..=24_000_000) => Latency::WS0,
        (VoltageRange::RANGE1, ..=48_000_000) => Latency::WS1,
        (VoltageRange::RANGE1, _) => Latency::WS2,
        (VoltageRange::RANGE2, ..=8_000_000) => Latency::WS0,
        (VoltageRange::RANGE2, ..=16_000_000) => Latency::WS1,
        (VoltageRange::RANGE2, _) => Latency::WS2,
        _ => unreachable!(),
    };

    // Configure flash read access latency based on voltage scale and frequency (RM0444 3.3.4)
    FLASH.acr().modify(|w| {
        w.set_latency(latency);
    });

    // Spin until the effective flash latency is set.
    while FLASH.acr().read().latency() != latency {}

    // Now that boost mode and flash read access latency are configured, set up SYSCLK
    RCC.cfgr().modify(|w| {
        w.set_sw(config.sys);
        w.set_hpre(config.ahb_pre);
        w.set_ppre(config.apb1_pre);
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
        pclk1: Some(pclk1),
        pclk1_tim: Some(pclk1_tim),
        pll1_p: pll.pll_p,
        pll1_q: pll.pll_q,
        pll1_r: pll.pll_r,
        hsi: hsi,
        hse: hse,
        #[cfg(crs)]
        hsi48: hsi48,
        rtc: rtc,
        hsi_div_8: hsi.map(|h| h / 8u32),
        hsi_div_488: hsi.map(|h| h / 488u32),

        // TODO
        lsi: None,
        lse: None,
    );
}

mod max {
    use core::ops::RangeInclusive;

    use crate::time::Hertz;

    pub(crate) const HSE_OSC: RangeInclusive<Hertz> = Hertz(4_000_000)..=Hertz(48_000_000);
    pub(crate) const HSE_BYP: RangeInclusive<Hertz> = Hertz(0)..=Hertz(48_000_000);
    pub(crate) const SYSCLK: RangeInclusive<Hertz> = Hertz(0)..=Hertz(64_000_000);
    pub(crate) const PCLK: RangeInclusive<Hertz> = Hertz(8)..=Hertz(64_000_000);
    pub(crate) const HCLK: RangeInclusive<Hertz> = Hertz(0)..=Hertz(64_000_000);
    pub(crate) const PLL_IN: RangeInclusive<Hertz> = Hertz(2_660_000)..=Hertz(16_000_000);
    pub(crate) const PLL_VCO: RangeInclusive<Hertz> = Hertz(96_000_000)..=Hertz(344_000_000);
    pub(crate) const PLL_P: RangeInclusive<Hertz> = Hertz(3_090_000)..=Hertz(122_000_000);
    pub(crate) const PLL_Q: RangeInclusive<Hertz> = Hertz(12_000_000)..=Hertz(128_000_000);
    pub(crate) const PLL_R: RangeInclusive<Hertz> = Hertz(12_000_000)..=Hertz(64_000_000);
}
