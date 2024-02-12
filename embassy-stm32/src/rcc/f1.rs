use crate::pac::flash::vals::Latency;
use crate::pac::rcc::vals::Pllsrc;
#[cfg(any(rcc_f1, rcc_f1cl))]
use crate::pac::rcc::vals::Usbpre;
pub use crate::pac::rcc::vals::{
    Adcpre as ADCPrescaler, Hpre as AHBPrescaler, Pllmul as PllMul, Pllxtpre as PllPreDiv, Ppre as APBPrescaler,
    Sw as Sysclk,
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

    pub adc_pre: ADCPrescaler,

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

            // ensure ADC is not out of range by default even if APB2 is maxxed out (36mhz)
            adc_pre: ADCPrescaler::DIV6,
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
    let pll = config.pll.map(|pll| {
        let (src_val, src_freq) = match pll.src {
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

        RCC.cfgr().modify(|w| {
            w.set_pllmul(pll.mul);
            w.set_pllsrc(src_val);
            w.set_pllxtpre(pll.prediv);
        });
        RCC.cr().modify(|w| w.set_pllon(true));
        while !RCC.cr().read().pllrdy() {}

        out_freq
    });

    #[cfg(any(rcc_f1, rcc_f1cl))]
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

    let adc = pclk2 / config.adc_pre;
    assert!(max::ADC.contains(&adc));

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
        w.set_adcpre(config.adc_pre);
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
        pclk2: Some(pclk2),
        pclk1_tim: Some(pclk1_tim),
        pclk2_tim: Some(pclk2_tim),
        hclk1: Some(hclk),
        adc: Some(adc),
        rtc: rtc,
        #[cfg(any(rcc_f1, rcc_f1cl))]
        usb: usb,
        lse: None,
    );
}

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
