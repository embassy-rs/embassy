use crate::pac::flash::vals::Latency;
use crate::pac::rcc::vals::Pllsrc;
pub use crate::pac::rcc::vals::{
    Hpre as AHBPrescaler, Pllmul as PllMul, Ppre as APBPrescaler, Prediv as PllPreDiv, Sw as Sysclk,
};
use crate::pac::{FLASH, RCC};
use crate::time::Hertz;

/// HSI speed
pub const HSI_FREQ: Hertz = Hertz(8_000_000);

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

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum PllSource {
    HSE,
    HSI,
    #[cfg(rcc_f0v4)]
    HSI48,
}

#[derive(Clone, Copy)]
pub struct Pll {
    pub src: PllSource,

    /// PLL pre-divider.
    ///
    /// On some chips, this must be 2 if `src == HSI`. Init will panic if this is not the case.
    pub prediv: PllPreDiv,

    /// PLL multiplication factor.
    pub mul: PllMul,
}

/// Clocks configutation
#[non_exhaustive]
pub struct Config {
    pub hsi: bool,
    pub hse: Option<Hse>,
    #[cfg(crs)]
    pub hsi48: Option<super::Hsi48Config>,
    pub sys: Sysclk,

    pub pll: Option<Pll>,

    pub ahb_pre: AHBPrescaler,
    pub apb1_pre: APBPrescaler,

    pub ls: super::LsConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            hsi: true,
            hse: None,
            #[cfg(crs)]
            hsi48: Some(Default::default()),
            sys: Sysclk::HSI,
            pll: None,
            ahb_pre: AHBPrescaler::DIV1,
            apb1_pre: APBPrescaler::DIV1,
            ls: Default::default(),
        }
    }
}

/// Initialize and Set the clock frequencies
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

    // configure HSI48
    #[cfg(crs)]
    let hsi48 = config.hsi48.map(|config| super::init_hsi48(config));
    #[cfg(not(crs))]
    let hsi48: Option<Hertz> = None;

    // Enable PLL
    let pll = config.pll.map(|pll| {
        let (src_val, src_freq) = match pll.src {
            #[cfg(not(any(rcc_f0v1, rcc_f0v2)))]
            PllSource::HSI => (Pllsrc::HSI_DIV_PREDIV, unwrap!(hsi)),
            #[cfg(any(rcc_f0v1, rcc_f0v2))]
            PllSource::HSI => {
                if pll.prediv != PllPreDiv::DIV2 {
                    panic!("if PLL source is HSI, PLL prediv must be 2.");
                }
                (Pllsrc::HSI_DIV2, unwrap!(hsi))
            }
            PllSource::HSE => (Pllsrc::HSE_DIV_PREDIV, unwrap!(hse)),
            #[cfg(rcc_f0v4)]
            PllSource::HSI48 => (Pllsrc::HSI48_DIV_PREDIV, unwrap!(hsi48)),
        };
        let in_freq = src_freq / pll.prediv;
        assert!(max::PLL_IN.contains(&in_freq));
        let out_freq = in_freq * pll.mul;
        assert!(max::PLL_OUT.contains(&out_freq));

        RCC.cfgr2().modify(|w| w.set_prediv(pll.prediv));
        RCC.cfgr().modify(|w| {
            w.set_pllmul(pll.mul);
            w.set_pllsrc(src_val);
        });
        RCC.cr().modify(|w| w.set_pllon(true));
        while !RCC.cr().read().pllrdy() {}

        out_freq
    });

    // Configure sysclk
    let sys = match config.sys {
        Sysclk::HSI => unwrap!(hsi),
        Sysclk::HSE => unwrap!(hse),
        Sysclk::PLL1_P => unwrap!(pll),
        #[cfg(rcc_f0v4)]
        Sysclk::HSI48 => unwrap!(hsi48),
        #[allow(unreachable_patterns)]
        _ => unreachable!(),
    };

    let hclk = sys / config.ahb_pre;
    let (pclk1, pclk1_tim) = super::util::calc_pclk(hclk, config.apb1_pre);

    assert!(max::HCLK.contains(&hclk));
    assert!(max::PCLK1.contains(&pclk1));

    // Set latency based on HCLK frquency
    let latency = match hclk.0 {
        ..=24_000_000 => Latency::WS0,
        _ => Latency::WS1,
    };
    FLASH.acr().modify(|w| {
        w.set_latency(latency);
        w.set_prftbe(true);
    });

    // Set prescalers
    // CFGR has been written before (PLL, PLL48) don't overwrite these settings
    RCC.cfgr().modify(|w| {
        w.set_ppre(config.apb1_pre);
        w.set_hpre(config.ahb_pre);
    });

    // Wait for the new prescalers to kick in
    // "The clocks are divided with the new prescaler factor from
    //  1 to 16 AHB cycles after write"
    cortex_m::asm::delay(16);

    // CFGR has been written before (PLL, PLL48, clock divider) don't overwrite these settings
    RCC.cfgr().modify(|w| w.set_sw(config.sys));
    while RCC.cfgr().read().sws() != config.sys {}

    let rtc = config.ls.init();

    set_clocks!(
        hsi: hsi,
        hse: hse,
        pll1_p: pll,
        sys: Some(sys),
        pclk1: Some(pclk1),
        pclk2: Some(pclk1),
        pclk1_tim: Some(pclk1_tim),
        pclk2_tim: Some(pclk1_tim),
        hclk1: Some(hclk),
        #[cfg(all(not(rcc_f37), adc3_common))]
        adc34: Some(adc34),
        #[cfg(stm32f334)]
        hrtim: hrtim,
        hsi48: hsi48,
        rtc: rtc,
        lse: None,
    );
}

mod max {
    use core::ops::RangeInclusive;

    use crate::time::Hertz;

    pub(crate) const HSE_OSC: RangeInclusive<Hertz> = Hertz(4_000_000)..=Hertz(32_000_000);
    pub(crate) const HSE_BYP: RangeInclusive<Hertz> = Hertz(1_000_000)..=Hertz(32_000_000);

    pub(crate) const HCLK: RangeInclusive<Hertz> = Hertz(0)..=Hertz(48_000_000);
    pub(crate) const PCLK1: RangeInclusive<Hertz> = Hertz(0)..=Hertz(48_000_000);

    pub(crate) const PLL_IN: RangeInclusive<Hertz> = Hertz(1_000_000)..=Hertz(24_000_000);
    pub(crate) const PLL_OUT: RangeInclusive<Hertz> = Hertz(16_000_000)..=Hertz(48_000_000);
}
