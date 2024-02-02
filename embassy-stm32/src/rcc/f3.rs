#[cfg(rcc_f3)]
use crate::pac::adccommon::vals::Ckmode;
use crate::pac::flash::vals::Latency;
pub use crate::pac::rcc::vals::Adcpres;
use crate::pac::rcc::vals::{Hpre, Pllmul, Pllsrc, Ppre, Prediv, Sw, Usbpre};
use crate::pac::{FLASH, RCC};
use crate::time::Hertz;

/// HSI speed
pub const HSI_FREQ: Hertz = Hertz(8_000_000);

#[cfg(rcc_f3)]
impl From<AdcClockSource> for Ckmode {
    fn from(value: AdcClockSource) -> Self {
        match value {
            AdcClockSource::BusDiv1 => Ckmode::SYNCDIV1,
            AdcClockSource::BusDiv2 => Ckmode::SYNCDIV2,
            AdcClockSource::BusDiv4 => Ckmode::SYNCDIV4,
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Copy)]
pub enum AdcClockSource {
    Pll(Adcpres),
    BusDiv1,
    BusDiv2,
    BusDiv4,
}

impl AdcClockSource {
    pub fn bus_div(&self) -> u32 {
        match self {
            Self::BusDiv1 => 1,
            Self::BusDiv2 => 2,
            Self::BusDiv4 => 4,
            _ => unreachable!(),
        }
    }
}

#[derive(Default)]
pub enum HrtimClockSource {
    #[default]
    BusClk,
    PllClk,
}

/// Clocks configutation
#[non_exhaustive]
#[derive(Default)]
pub struct Config {
    /// Frequency of HSE oscillator
    /// 4MHz to 32MHz
    pub hse: Option<Hertz>,
    /// Bypass HSE for an external clock
    pub bypass_hse: bool,
    /// Frequency of the System Clock
    pub sysclk: Option<Hertz>,
    /// Frequency of AHB bus
    pub hclk: Option<Hertz>,
    /// Frequency of APB1 bus
    /// - Max frequency 36MHz
    pub pclk1: Option<Hertz>,
    /// Frequency of APB2 bus
    /// - Max frequency with HSE is 72MHz
    /// - Max frequency without HSE is 64MHz
    pub pclk2: Option<Hertz>,
    /// USB clock setup
    /// It is valid only when,
    /// - HSE is enabled,
    /// - The System clock frequency is either 48MHz or 72MHz
    /// - APB1 clock has a minimum frequency of 10MHz
    pub pll48: bool,
    #[cfg(rcc_f3)]
    /// ADC clock setup
    /// - For AHB, a psc of 4 or less must be used
    pub adc: Option<AdcClockSource>,
    #[cfg(rcc_f3)]
    /// ADC clock setup
    /// - For AHB, a psc of 4 or less must be used
    pub adc34: Option<AdcClockSource>,
    #[cfg(stm32f334)]
    pub hrtim: HrtimClockSource,
    pub ls: super::LsConfig,
}

// Information required to setup the PLL clock
#[derive(Clone, Copy)]
struct PllConfig {
    pll_src: Pllsrc,
    pll_mul: Pllmul,
    pll_div: Option<Prediv>,
}

/// Initialize and Set the clock frequencies
pub(crate) unsafe fn init(config: Config) {
    // Calculate the real System clock, and PLL configuration if applicable
    let (sysclk, pll_config) = get_sysclk(&config);
    assert!(sysclk.0 <= 72_000_000);

    // Calculate real AHB clock
    let hclk = config.hclk.map(|h| h).unwrap_or(sysclk);
    let hpre = match sysclk.0 / hclk.0 {
        0 => unreachable!(),
        1 => Hpre::DIV1,
        2 => Hpre::DIV2,
        3..=5 => Hpre::DIV4,
        6..=11 => Hpre::DIV8,
        12..=39 => Hpre::DIV16,
        40..=95 => Hpre::DIV64,
        96..=191 => Hpre::DIV128,
        192..=383 => Hpre::DIV256,
        _ => Hpre::DIV512,
    };
    let hclk = sysclk / hpre;
    assert!(hclk <= Hertz(72_000_000));

    // Calculate real APB1 clock
    let pclk1 = config.pclk1.unwrap_or(hclk);
    let ppre1 = match hclk / pclk1 {
        0 => unreachable!(),
        1 => Ppre::DIV1,
        2 => Ppre::DIV2,
        3..=5 => Ppre::DIV4,
        6..=11 => Ppre::DIV8,
        _ => Ppre::DIV16,
    };
    let timer_mul1 = if ppre1 == Ppre::DIV1 { 1u32 } else { 2 };
    let pclk1 = hclk / ppre1;
    assert!(pclk1 <= Hertz(36_000_000));

    // Calculate real APB2 clock
    let pclk2 = config.pclk2.unwrap_or(hclk);
    let ppre2 = match hclk / pclk2 {
        0 => unreachable!(),
        1 => Ppre::DIV1,
        2 => Ppre::DIV2,
        3..=5 => Ppre::DIV4,
        6..=11 => Ppre::DIV8,
        _ => Ppre::DIV16,
    };
    let timer_mul2 = if ppre2 == Ppre::DIV1 { 1u32 } else { 2 };
    let pclk2 = hclk / ppre2;
    assert!(pclk2 <= Hertz(72_000_000));

    // Set latency based on HCLK frquency
    // RM0316: "The prefetch buffer must be kept on when using a prescaler
    // different from 1 on the AHB clock.", "Half-cycle access cannot be
    // used when there is a prescaler different from 1 on the AHB clock"
    FLASH.acr().modify(|w| {
        w.set_latency(if hclk <= Hertz(24_000_000) {
            Latency::WS0
        } else if hclk <= Hertz(48_000_000) {
            Latency::WS1
        } else {
            Latency::WS2
        });
        if hpre != Hpre::DIV1 {
            w.set_hlfcya(false);
            w.set_prftbe(true);
        }
    });

    // Enable HSE
    // RM0316: "Bits 31:26 Reserved, must be kept at reset value."
    if config.hse.is_some() {
        RCC.cr().modify(|w| {
            w.set_hsebyp(config.bypass_hse);
            // We turn on clock security to switch to HSI when HSE fails
            w.set_csson(true);
            w.set_hseon(true);
        });
        while !RCC.cr().read().hserdy() {}
    }

    // Enable PLL
    // RM0316: "Reserved, must be kept at reset value."
    if let Some(ref pll_config) = pll_config {
        RCC.cfgr().modify(|w| {
            w.set_pllmul(pll_config.pll_mul);
            w.set_pllsrc(pll_config.pll_src);
        });
        if let Some(pll_div) = pll_config.pll_div {
            RCC.cfgr2().modify(|w| w.set_prediv(pll_div));
        }
        RCC.cr().modify(|w| w.set_pllon(true));
        while !RCC.cr().read().pllrdy() {}
    }

    // CFGR has been written before (PLL) don't overwrite these settings
    if config.pll48 {
        let usb_pre = get_usb_pre(&config, sysclk, pclk1, &pll_config);
        RCC.cfgr().modify(|w| {
            w.set_usbpre(usb_pre);
        });
    }

    // Set prescalers
    // CFGR has been written before (PLL, PLL48) don't overwrite these settings
    RCC.cfgr().modify(|w| {
        w.set_ppre2(ppre2);
        w.set_ppre1(ppre1);
        w.set_hpre(hpre);
    });

    // Wait for the new prescalers to kick in
    // "The clocks are divided with the new prescaler factor from
    //  1 to 16 AHB cycles after write"
    cortex_m::asm::delay(16);

    // CFGR has been written before (PLL, PLL48, clock divider) don't overwrite these settings
    RCC.cfgr().modify(|w| {
        w.set_sw(match (pll_config, config.hse) {
            (Some(_), _) => Sw::PLL1_P,
            (None, Some(_)) => Sw::HSE,
            (None, None) => Sw::HSI,
        })
    });

    #[cfg(rcc_f3)]
    let adc = config.adc.map(|adc| match adc {
        AdcClockSource::Pll(adcpres) => {
            RCC.cfgr2().modify(|w| {
                // Make sure that we're using the PLL
                pll_config.unwrap();
                w.set_adc12pres(adcpres);

                sysclk / adcpres
            })
        }
        _ => crate::pac::ADC_COMMON.ccr().modify(|w| {
            assert!(!(adc.bus_div() == 1 && hpre != Hpre::DIV1));

            w.set_ckmode(adc.into());

            sysclk / adc.bus_div()
        }),
    });

    #[cfg(all(rcc_f3, adc3_common))]
    let adc34 = config.adc34.map(|adc| match adc {
        AdcClockSource::Pll(adcpres) => {
            RCC.cfgr2().modify(|w| {
                // Make sure that we're using the PLL
                pll_config.unwrap();
                w.set_adc34pres(adcpres);

                sysclk / adcpres
            })
        }
        _ => crate::pac::ADC_COMMON.ccr().modify(|w| {
            assert!(!(adc.bus_div() == 1 && hpre != Hpre::DIV1));

            w.set_ckmode(adc.into());

            sysclk / adc.bus_div()
        }),
    });

    #[cfg(stm32f334)]
    let hrtim = match config.hrtim {
        // Must be configured after the bus is ready, otherwise it won't work
        HrtimClockSource::BusClk => None,
        HrtimClockSource::PllClk => {
            use crate::pac::rcc::vals::Timsw;

            // Make sure that we're using the PLL
            pll_config.unwrap();
            assert!((pclk2 == sysclk) || (pclk2 * 2u32 == sysclk));

            RCC.cfgr3().modify(|w| w.set_hrtim1sw(Timsw::PLL1_P));

            Some(sysclk * 2u32)
        }
    };

    let rtc = config.ls.init();

    set_clocks!(
        hsi: None,
        lse: None,
        pll1_p: None,
        sys: Some(sysclk),
        pclk1: Some(pclk1),
        pclk2: Some(pclk2),
        pclk1_tim: Some(pclk1 * timer_mul1),
        pclk2_tim: Some(pclk2 * timer_mul2),
        hclk1: Some(hclk),
        #[cfg(rcc_f3)]
        adc: adc,
        #[cfg(all(rcc_f3, adc3_common))]
        adc34: adc34,
        #[cfg(all(rcc_f3, not(adc3_common)))]
        adc34: None,
        #[cfg(stm32f334)]
        hrtim: hrtim,
        rtc: rtc,
    );
}

#[inline]
fn get_sysclk(config: &Config) -> (Hertz, Option<PllConfig>) {
    match (config.sysclk, config.hse) {
        (Some(sysclk), Some(hse)) if sysclk == hse => (hse, None),
        (Some(sysclk), None) if sysclk == HSI_FREQ => (HSI_FREQ, None),
        // If the user selected System clock is different from HSI or HSE
        // we will have to setup PLL clock source
        (Some(sysclk), _) => {
            let (sysclk, pll_config) = calc_pll(config, sysclk);
            (sysclk, Some(pll_config))
        }
        (None, Some(hse)) => (hse, None),
        (None, None) => (HSI_FREQ, None),
    }
}

#[inline]
fn calc_pll(config: &Config, Hertz(sysclk): Hertz) -> (Hertz, PllConfig) {
    // Calculates the Multiplier and the Divisor to arrive at
    // the required System clock from PLL source frequency
    let get_mul_div = |sysclk, pllsrcclk| {
        let bus_div = gcd(sysclk, pllsrcclk);
        let mut multiplier = sysclk / bus_div;
        let mut divisor = pllsrcclk / bus_div;
        // Minimum PLL multiplier is two
        if multiplier == 1 {
            multiplier *= 2;
            divisor *= 2;
        }
        assert!(multiplier <= 16);
        assert!(divisor <= 16);
        (multiplier, divisor)
    };
    // Based on the source of Pll, we calculate the actual system clock
    // frequency, PLL's source identifier, multiplier and divisor
    let (act_sysclk, pll_src, pll_mul, pll_div) = match config.hse {
        Some(Hertz(hse)) => {
            let (multiplier, divisor) = get_mul_div(sysclk, hse);
            (
                Hertz((hse / divisor) * multiplier),
                Pllsrc::HSE_DIV_PREDIV,
                into_pll_mul(multiplier),
                Some(into_pre_div(divisor)),
            )
        }
        None => {
            cfg_if::cfg_if! {
                // For some chips PREDIV is always two, and cannot be changed
                if #[cfg(any(flashsize_d, flashsize_e))] {
                    let (multiplier, divisor) = get_mul_div(sysclk, HSI_FREQ.0);
                    (
                        Hertz((HSI_FREQ.0 / divisor) * multiplier),
                        Pllsrc::HSI_DIV_PREDIV,
                        into_pll_mul(multiplier),
                        Some(into_pre_div(divisor)),
                    )
                } else {
                    let pllsrcclk = HSI_FREQ.0 / 2;
                    let multiplier = sysclk / pllsrcclk;
                    assert!(multiplier <= 16);
                    (
                        Hertz(pllsrcclk * multiplier),
                        Pllsrc::HSI_DIV2,
                        into_pll_mul(multiplier),
                        None,
                    )
                }
            }
        }
    };
    (
        act_sysclk,
        PllConfig {
            pll_src,
            pll_mul,
            pll_div,
        },
    )
}

#[inline]
#[allow(unused_variables)]
fn get_usb_pre(config: &Config, sysclk: Hertz, pclk1: Hertz, pll_config: &Option<PllConfig>) -> Usbpre {
    cfg_if::cfg_if! {
        // Some chips do not have USB
        if #[cfg(any(stm32f301, stm32f318, stm32f334))] {
            panic!("USB clock not supported by the chip");
        } else {
            let usb_ok = config.hse.is_some() && pll_config.is_some() && (pclk1 >= Hertz(10_000_000));
            match (usb_ok, sysclk) {
                (true, Hertz(72_000_000)) => Usbpre::DIV1_5,
                (true, Hertz(48_000_000)) => Usbpre::DIV1,
                _ => panic!(
                    "USB clock is only valid if the PLL output frequency is either 48MHz or 72MHz"
                ),
            }
        }
    }
}

// This function assumes cases when multiplier is one and it
// being greater than 16 is made impossible
#[inline]
fn into_pll_mul(multiplier: u32) -> Pllmul {
    match multiplier {
        2 => Pllmul::MUL2,
        3 => Pllmul::MUL3,
        4 => Pllmul::MUL4,
        5 => Pllmul::MUL5,
        6 => Pllmul::MUL6,
        7 => Pllmul::MUL7,
        8 => Pllmul::MUL8,
        9 => Pllmul::MUL9,
        10 => Pllmul::MUL10,
        11 => Pllmul::MUL11,
        12 => Pllmul::MUL12,
        13 => Pllmul::MUL13,
        14 => Pllmul::MUL14,
        15 => Pllmul::MUL15,
        16 => Pllmul::MUL16,
        _ => unreachable!(),
    }
}

// This function assumes the incoming divisor cannot be greater
// than 16
#[inline]
fn into_pre_div(divisor: u32) -> Prediv {
    match divisor {
        1 => Prediv::DIV1,
        2 => Prediv::DIV2,
        3 => Prediv::DIV3,
        4 => Prediv::DIV4,
        5 => Prediv::DIV5,
        6 => Prediv::DIV6,
        7 => Prediv::DIV7,
        8 => Prediv::DIV8,
        9 => Prediv::DIV9,
        10 => Prediv::DIV10,
        11 => Prediv::DIV11,
        12 => Prediv::DIV12,
        13 => Prediv::DIV13,
        14 => Prediv::DIV14,
        15 => Prediv::DIV15,
        16 => Prediv::DIV16,
        _ => unreachable!(),
    }
}

// Determine GCD using Euclidean algorithm
#[inline]
fn gcd(mut a: u32, mut b: u32) -> u32 {
    while b != 0 {
        let r = a % b;
        a = b;
        b = r;
    }
    a
}
