use crate::pac::flash::vals::Latency;
#[cfg(stm32f1)]
pub use crate::pac::rcc::vals::Adcpre as ADCPrescaler;
#[cfg(stm32f3)]
pub use crate::pac::rcc::vals::Adcpres as AdcPllPrescaler;
use crate::pac::rcc::vals::Pllsrc;
#[cfg(stm32f1)]
pub use crate::pac::rcc::vals::Pllxtpre as PllPreDiv;
#[cfg(any(stm32f0, stm32f3))]
pub use crate::pac::rcc::vals::Prediv as PllPreDiv;
pub use crate::pac::rcc::vals::{Hpre as AHBPrescaler, Pllmul as PllMul, Ppre as APBPrescaler, Sw as Sysclk};
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

#[cfg(all(stm32f3, not(rcc_f37)))]
#[derive(Clone, Copy)]
pub enum AdcClockSource {
    Pll(AdcPllPrescaler),
    Hclk(AdcHclkPrescaler),
}

#[cfg(all(stm32f3, not(rcc_f37)))]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AdcHclkPrescaler {
    Div1,
    Div2,
    Div4,
}

#[cfg(stm32f334)]
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
    #[cfg(crs)]
    pub hsi48: Option<super::Hsi48Config>,
    pub sys: Sysclk,

    pub pll: Option<Pll>,

    pub ahb_pre: AHBPrescaler,
    pub apb1_pre: APBPrescaler,
    #[cfg(not(stm32f0))]
    pub apb2_pre: APBPrescaler,

    #[cfg(stm32f1)]
    pub adc_pre: ADCPrescaler,

    #[cfg(all(stm32f3, not(rcc_f37)))]
    pub adc: AdcClockSource,
    #[cfg(all(stm32f3, not(rcc_f37), adc3_common))]
    pub adc34: AdcClockSource,

    /// Per-peripheral kernel clock selection muxes
    pub mux: super::mux::ClockMux,

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
            #[cfg(not(stm32f0))]
            apb2_pre: APBPrescaler::DIV1,
            ls: Default::default(),

            #[cfg(stm32f1)]
            // ensure ADC is not out of range by default even if APB2 is maxxed out (36mhz)
            adc_pre: ADCPrescaler::DIV6,

            #[cfg(all(stm32f3, not(rcc_f37)))]
            adc: AdcClockSource::Hclk(AdcHclkPrescaler::Div1),
            #[cfg(all(stm32f3, not(rcc_f37), adc3_common))]
            adc34: AdcClockSource::Hclk(AdcHclkPrescaler::Div1),

            mux: Default::default(),
        }
    }
}

/// Initialize and Set the clock frequencies
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

    // configure HSI48
    #[cfg(crs)]
    let hsi48 = config.hsi48.map(|config| super::init_hsi48(config));
    #[cfg(not(crs))]
    let hsi48: Option<Hertz> = None;

    // Enable PLL
    let pll = config.pll.map(|pll| {
        let (src_val, src_freq) = match pll.src {
            #[cfg(any(rcc_f0v3, rcc_f0v4, rcc_f3v3))]
            PllSource::HSI => (Pllsrc::HSI_DIV_PREDIV, unwrap!(hsi)),
            #[cfg(not(any(rcc_f0v3, rcc_f0v4, rcc_f3v3)))]
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

        #[cfg(not(stm32f1))]
        RCC.cfgr2().modify(|w| w.set_prediv(pll.prediv));
        RCC.cfgr().modify(|w| {
            w.set_pllmul(pll.mul);
            w.set_pllsrc(src_val);
            #[cfg(stm32f1)]
            w.set_pllxtpre(pll.prediv);
        });
        RCC.cr().modify(|w| w.set_pllon(true));
        while !RCC.cr().read().pllrdy() {}

        out_freq
    });

    #[cfg(stm32f3)]
    let pll_mul_2 = pll.map(|pll| pll * 2u32);

    #[cfg(any(rcc_f1, rcc_f1cl, stm32f3))]
    let usb = match pll {
        Some(Hertz(72_000_000)) => Some(crate::pac::rcc::vals::Usbpre::DIV1_5),
        Some(Hertz(48_000_000)) => Some(crate::pac::rcc::vals::Usbpre::DIV1),
        _ => None,
    }
    .map(|usbpre| {
        RCC.cfgr().modify(|w| w.set_usbpre(usbpre));
        Hertz(48_000_000)
    });

    // Configure sysclk
    let sys = match config.sys {
        Sysclk::HSI => unwrap!(hsi),
        Sysclk::HSE => unwrap!(hse),
        Sysclk::PLL1_P => unwrap!(pll),
        _ => unreachable!(),
    };

    let hclk = sys / config.ahb_pre;
    let (pclk1, pclk1_tim) = super::util::calc_pclk(hclk, config.apb1_pre);
    #[cfg(not(stm32f0))]
    let (pclk2, pclk2_tim) = super::util::calc_pclk(hclk, config.apb2_pre);
    #[cfg(stm32f0)]
    let (pclk2, pclk2_tim) = (pclk1, pclk1_tim);

    assert!(max::HCLK.contains(&hclk));
    assert!(max::PCLK1.contains(&pclk1));
    #[cfg(not(stm32f0))]
    assert!(max::PCLK2.contains(&pclk2));

    #[cfg(stm32f1)]
    let adc = pclk2 / config.adc_pre;
    #[cfg(stm32f1)]
    assert!(max::ADC.contains(&adc));

    // Set latency based on HCLK frquency
    #[cfg(stm32f0)]
    let latency = match hclk.0 {
        ..=24_000_000 => Latency::WS0,
        _ => Latency::WS1,
    };
    #[cfg(any(stm32f1, stm32f3))]
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
        #[cfg(stm32f3)]
        if config.ahb_pre != AHBPrescaler::DIV1 {
            w.set_hlfcya(false);
            w.set_prftbe(true);
        }
        #[cfg(not(stm32f3))]
        w.set_prftbe(true);
    });

    // Set prescalers
    // CFGR has been written before (PLL, PLL48) don't overwrite these settings
    RCC.cfgr().modify(|w: &mut stm32_metapac::rcc::regs::Cfgr| {
        #[cfg(not(stm32f0))]
        {
            w.set_ppre1(config.apb1_pre);
            w.set_ppre2(config.apb2_pre);
        }
        #[cfg(stm32f0)]
        w.set_ppre(config.apb1_pre);
        w.set_hpre(config.ahb_pre);
        #[cfg(stm32f1)]
        w.set_adcpre(config.adc_pre);
    });

    // Wait for the new prescalers to kick in
    // "The clocks are divided with the new prescaler factor from
    //  1 to 16 AHB cycles after write"
    cortex_m::asm::delay(16);

    // CFGR has been written before (PLL, PLL48, clock divider) don't overwrite these settings
    RCC.cfgr().modify(|w| w.set_sw(config.sys));
    while RCC.cfgr().read().sws() != config.sys {}

    // Disable HSI if not used
    if !config.hsi {
        RCC.cr().modify(|w| w.set_hsion(false));
    }

    let rtc = config.ls.init();

    // TODO: all this ADC stuff should probably go into the ADC module, not here.
    // Most STM32s manage ADC clocks in a similar way with ADCx_COMMON.
    #[cfg(all(stm32f3, not(rcc_f37)))]
    use crate::pac::adccommon::vals::Ckmode;

    #[cfg(all(stm32f3, not(rcc_f37)))]
    let adc = {
        #[cfg(peri_adc1_common)]
        let common = crate::pac::ADC1_COMMON;
        #[cfg(peri_adc12_common)]
        let common = crate::pac::ADC12_COMMON;

        match config.adc {
            AdcClockSource::Pll(adcpres) => {
                RCC.cfgr2().modify(|w| w.set_adc12pres(adcpres));
                common.ccr().modify(|w| w.set_ckmode(Ckmode::ASYNCHRONOUS));

                unwrap!(pll) / adcpres
            }
            AdcClockSource::Hclk(adcpres) => {
                assert!(!(adcpres == AdcHclkPrescaler::Div1 && config.ahb_pre != AHBPrescaler::DIV1));

                let (div, ckmode) = match adcpres {
                    AdcHclkPrescaler::Div1 => (1u32, Ckmode::SYNCDIV1),
                    AdcHclkPrescaler::Div2 => (2u32, Ckmode::SYNCDIV2),
                    AdcHclkPrescaler::Div4 => (4u32, Ckmode::SYNCDIV4),
                };
                common.ccr().modify(|w| w.set_ckmode(ckmode));

                hclk / div
            }
        }
    };

    #[cfg(all(stm32f3, not(rcc_f37), adc3_common))]
    let adc34 = {
        #[cfg(peri_adc3_common)]
        let common = crate::pac::ADC3_COMMON;
        #[cfg(peri_adc34_common)]
        let common = crate::pac::ADC34_COMMON;

        match config.adc34 {
            AdcClockSource::Pll(adcpres) => {
                RCC.cfgr2().modify(|w| w.set_adc34pres(adcpres));
                common.ccr().modify(|w| w.set_ckmode(Ckmode::ASYNCHRONOUS));

                unwrap!(pll) / adcpres
            }
            AdcClockSource::Hclk(adcpres) => {
                assert!(!(adcpres == AdcHclkPrescaler::Div1 && config.ahb_pre != AHBPrescaler::DIV1));

                let (div, ckmode) = match adcpres {
                    AdcHclkPrescaler::Div1 => (1u32, Ckmode::SYNCDIV1),
                    AdcHclkPrescaler::Div2 => (2u32, Ckmode::SYNCDIV2),
                    AdcHclkPrescaler::Div4 => (4u32, Ckmode::SYNCDIV4),
                };
                common.ccr().modify(|w| w.set_ckmode(ckmode));

                hclk / div
            }
        }
    };

    /*
    TODO: Maybe add something like this to clock_mux? How can we autogenerate the data for this?
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
     */

    config.mux.init();

    set_clocks!(
        hsi: hsi,
        hse: hse,
        pll1_p: pll,
        #[cfg(stm32f3)]
        pll1_p_mul_2: pll_mul_2,
        hsi_div_244: hsi.map(|h| h / 244u32),
        sys: Some(sys),
        pclk1: Some(pclk1),
        pclk2: Some(pclk2),
        pclk1_tim: Some(pclk1_tim),
        pclk2_tim: Some(pclk2_tim),
        hclk1: Some(hclk),
        #[cfg(all(stm32f3, not(rcc_f37)))]
        adc: Some(adc),
        #[cfg(all(stm32f3, not(rcc_f37), adc3_common))]
        adc34: Some(adc34),
        rtc: rtc,
        hsi48: hsi48,
        #[cfg(any(rcc_f1, rcc_f1cl, stm32f3))]
        usb: usb,
        lse: None,
    );
}

#[cfg(stm32f0)]
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

#[cfg(stm32f1)]
mod max {
    use core::ops::RangeInclusive;

    use crate::time::Hertz;

    #[cfg(not(rcc_f1cl))]
    pub(crate) const HSE_OSC: RangeInclusive<Hertz> = Hertz(4_000_000)..=Hertz(16_000_000);
    #[cfg(not(rcc_f1cl))]
    pub(crate) const HSE_BYP: RangeInclusive<Hertz> = Hertz(1_000_000)..=Hertz(25_000_000);

    #[cfg(rcc_f1cl)]
    pub(crate) const HSE_OSC: RangeInclusive<Hertz> = Hertz(3_000_000)..=Hertz(25_000_000);
    #[cfg(rcc_f1cl)]
    pub(crate) const HSE_BYP: RangeInclusive<Hertz> = Hertz(1_000_000)..=Hertz(50_000_000);

    pub(crate) const HCLK: RangeInclusive<Hertz> = Hertz(0)..=Hertz(72_000_000);
    pub(crate) const PCLK1: RangeInclusive<Hertz> = Hertz(0)..=Hertz(36_000_000);
    pub(crate) const PCLK2: RangeInclusive<Hertz> = Hertz(0)..=Hertz(72_000_000);

    pub(crate) const PLL_IN: RangeInclusive<Hertz> = Hertz(1_000_000)..=Hertz(25_000_000);
    pub(crate) const PLL_OUT: RangeInclusive<Hertz> = Hertz(16_000_000)..=Hertz(72_000_000);

    pub(crate) const ADC: RangeInclusive<Hertz> = Hertz(0)..=Hertz(14_000_000);
}

#[cfg(stm32f3)]
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
