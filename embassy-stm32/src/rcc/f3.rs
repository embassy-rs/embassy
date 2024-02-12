use crate::pac::flash::vals::Latency;
pub use crate::pac::rcc::vals::{
    Adcpres as AdcPllPrescaler, Hpre as AHBPrescaler, Pllmul as PllMul, Ppre as APBPrescaler, Prediv as PllPreDiv,
    Sw as Sysclk,
};
use crate::pac::rcc::vals::{Pllsrc, Usbpre};
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
}

#[derive(Clone, Copy)]
pub struct Pll {
    pub src: PllSource,

    /// PLL pre-divider.
    ///
    /// On some F3 chips, this must be 2 if `src == HSI`. Init will panic if this is not the case.
    pub prediv: PllPreDiv,

    /// PLL multiplication factor.
    pub mul: PllMul,
}

#[derive(Clone, Copy)]
pub enum AdcClockSource {
    Pll(AdcPllPrescaler),
    Hclk(AdcHclkPrescaler),
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AdcHclkPrescaler {
    Div1,
    Div2,
    Div4,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum HrtimClockSource {
    BusClk,
    PllClk,
}

/// Clocks configutation
#[non_exhaustive]
pub struct Config {
    pub hsi: bool,
    pub hse: Option<Hse>,
    pub sys: Sysclk,

    pub pll: Option<Pll>,

    pub ahb_pre: AHBPrescaler,
    pub apb1_pre: APBPrescaler,
    pub apb2_pre: APBPrescaler,

    #[cfg(not(rcc_f37))]
    pub adc: AdcClockSource,
    #[cfg(all(not(rcc_f37), adc3_common))]
    pub adc34: AdcClockSource,
    #[cfg(stm32f334)]
    pub hrtim: HrtimClockSource,

    pub ls: super::LsConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            hsi: true,
            hse: None,
            sys: Sysclk::HSI,
            pll: None,
            ahb_pre: AHBPrescaler::DIV1,
            apb1_pre: APBPrescaler::DIV1,
            apb2_pre: APBPrescaler::DIV1,
            ls: Default::default(),

            #[cfg(not(rcc_f37))]
            adc: AdcClockSource::Hclk(AdcHclkPrescaler::Div1),
            #[cfg(all(not(rcc_f37), adc3_common))]
            adc34: AdcClockSource::Hclk(AdcHclkPrescaler::Div1),
            #[cfg(stm32f334)]
            hrtim: HrtimClockSource::BusClk,
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

    // Enable PLL
    // RM0316: "Reserved, must be kept at reset value."
    let pll = config.pll.map(|pll| {
        let (src_val, src_freq) = match pll.src {
            #[cfg(rcc_f3v3)]
            PllSource::HSI => (Pllsrc::HSI_DIV_PREDIV, unwrap!(hsi)),
            #[cfg(not(rcc_f3v3))]
            PllSource::HSI => {
                if pll.prediv != PllPreDiv::DIV2 {
                    panic!("if PLL source is HSI, PLL prediv must be 2.");
                }
                (Pllsrc::HSI_DIV2, unwrap!(hsi))
            }
            PllSource::HSE => (Pllsrc::HSE_DIV_PREDIV, unwrap!(hse)),
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

    let usb = match pll {
        Some(Hertz(72_000_000)) => {
            RCC.cfgr().modify(|w| w.set_usbpre(Usbpre::DIV1_5));
            Some(Hertz(48_000_000))
        }
        Some(Hertz(48_000_000)) => {
            RCC.cfgr().modify(|w| w.set_usbpre(Usbpre::DIV1));
            Some(Hertz(48_000_000))
        }
        _ => None,
    };

    // Configure sysclk
    let sys = match config.sys {
        Sysclk::HSI => unwrap!(hsi),
        Sysclk::HSE => unwrap!(hse),
        Sysclk::PLL1_P => unwrap!(pll),
        _ => unreachable!(),
    };

    let hclk = sys / config.ahb_pre;
    let (pclk1, pclk1_tim) = super::util::calc_pclk(hclk, config.apb1_pre);
    let (pclk2, pclk2_tim) = super::util::calc_pclk(hclk, config.apb2_pre);

    assert!(max::HCLK.contains(&hclk));
    assert!(max::PCLK1.contains(&pclk1));
    assert!(max::PCLK2.contains(&pclk2));

    // Set latency based on HCLK frquency
    let latency = match hclk.0 {
        ..=24_000_000 => Latency::WS0,
        ..=48_000_000 => Latency::WS1,
        _ => Latency::WS2,
    };
    FLASH.acr().modify(|w| {
        w.set_latency(latency);
        // RM0316: "The prefetch buffer must be kept on when using a prescaler
        // different from 1 on the AHB clock.", "Half-cycle access cannot be
        // used when there is a prescaler different from 1 on the AHB clock"
        if config.ahb_pre != AHBPrescaler::DIV1 {
            w.set_hlfcya(false);
            w.set_prftbe(true);
        }
    });

    // Set prescalers
    // CFGR has been written before (PLL, PLL48) don't overwrite these settings
    RCC.cfgr().modify(|w| {
        w.set_ppre1(config.apb1_pre);
        w.set_ppre2(config.apb2_pre);
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

    #[cfg(not(rcc_f37))]
    use crate::pac::adccommon::vals::Ckmode;

    #[cfg(not(rcc_f37))]
    let adc = match config.adc {
        AdcClockSource::Pll(adcpres) => {
            RCC.cfgr2().modify(|w| w.set_adc12pres(adcpres));
            crate::pac::ADC_COMMON
                .ccr()
                .modify(|w| w.set_ckmode(Ckmode::ASYNCHRONOUS));

            unwrap!(pll) / adcpres
        }
        AdcClockSource::Hclk(adcpres) => {
            assert!(!(adcpres == AdcHclkPrescaler::Div1 && config.ahb_pre != AHBPrescaler::DIV1));

            let (div, ckmode) = match adcpres {
                AdcHclkPrescaler::Div1 => (1u32, Ckmode::SYNCDIV1),
                AdcHclkPrescaler::Div2 => (2u32, Ckmode::SYNCDIV2),
                AdcHclkPrescaler::Div4 => (4u32, Ckmode::SYNCDIV4),
            };
            crate::pac::ADC_COMMON.ccr().modify(|w| w.set_ckmode(ckmode));

            hclk / div
        }
    };

    #[cfg(all(not(rcc_f37), adc3_common))]
    let adc34 = match config.adc34 {
        AdcClockSource::Pll(adcpres) => {
            RCC.cfgr2().modify(|w| w.set_adc34pres(adcpres));
            crate::pac::ADC3_COMMON
                .ccr()
                .modify(|w| w.set_ckmode(Ckmode::ASYNCHRONOUS));

            unwrap!(pll) / adcpres
        }
        AdcClockSource::Hclk(adcpres) => {
            assert!(!(adcpres == AdcHclkPrescaler::Div1 && config.ahb_pre != AHBPrescaler::DIV1));

            let (div, ckmode) = match adcpres {
                AdcHclkPrescaler::Div1 => (1u32, Ckmode::SYNCDIV1),
                AdcHclkPrescaler::Div2 => (2u32, Ckmode::SYNCDIV2),
                AdcHclkPrescaler::Div4 => (4u32, Ckmode::SYNCDIV4),
            };
            crate::pac::ADC3_COMMON.ccr().modify(|w| w.set_ckmode(ckmode));

            hclk / div
        }
    };

    #[cfg(stm32f334)]
    let hrtim = match config.hrtim {
        // Must be configured after the bus is ready, otherwise it won't work
        HrtimClockSource::BusClk => None,
        HrtimClockSource::PllClk => {
            use crate::pac::rcc::vals::Timsw;

            // Make sure that we're using the PLL
            let pll = unwrap!(pll);
            assert!((pclk2 == pll) || (pclk2 * 2u32 == pll));

            RCC.cfgr3().modify(|w| w.set_hrtim1sw(Timsw::PLL1_P));

            Some(pll * 2u32)
        }
    };

    set_clocks!(
        hsi: hsi,
        hse: hse,
        pll1_p: pll,
        sys: Some(sys),
        pclk1: Some(pclk1),
        pclk2: Some(pclk2),
        pclk1_tim: Some(pclk1_tim),
        pclk2_tim: Some(pclk2_tim),
        hclk1: Some(hclk),
        #[cfg(not(rcc_f37))]
        adc: Some(adc),
        #[cfg(all(not(rcc_f37), adc3_common))]
        adc34: Some(adc34),
        #[cfg(stm32f334)]
        hrtim: hrtim,
        rtc: rtc,
        usb: usb,
        lse: None,
    );
}

mod max {
    use core::ops::RangeInclusive;

    use crate::time::Hertz;

    pub(crate) const HSE_OSC: RangeInclusive<Hertz> = Hertz(4_000_000)..=Hertz(32_000_000);
    pub(crate) const HSE_BYP: RangeInclusive<Hertz> = Hertz(1_000_000)..=Hertz(32_000_000);

    pub(crate) const HCLK: RangeInclusive<Hertz> = Hertz(0)..=Hertz(72_000_000);
    pub(crate) const PCLK1: RangeInclusive<Hertz> = Hertz(0)..=Hertz(36_000_000);
    pub(crate) const PCLK2: RangeInclusive<Hertz> = Hertz(0)..=Hertz(72_000_000);

    pub(crate) const PLL_IN: RangeInclusive<Hertz> = Hertz(1_000_000)..=Hertz(24_000_000);
    pub(crate) const PLL_OUT: RangeInclusive<Hertz> = Hertz(16_000_000)..=Hertz(72_000_000);
}
