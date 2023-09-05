use crate::pac::flash::vals::Latency;
use crate::pac::rcc::vals::{Adcpres, Hpre, Pllmul, Pllsrc, Ppre, Prediv, Sw, Usbpre};
use crate::pac::{FLASH, RCC};
use crate::rcc::{set_freqs, Clocks};
use crate::time::Hertz;

/// HSI speed
pub const HSI_FREQ: Hertz = Hertz(8_000_000);

/// LSI speed
pub const LSI_FREQ: Hertz = Hertz(40_000);

#[repr(u16)]
#[derive(Clone, Copy)]
pub enum ADCPrescaler {
    Div1 = 1,
    Div2 = 2,
    Div4 = 4,
    Div6 = 6,
    Div8 = 8,
    Div12 = 12,
    Div16 = 16,
    Div32 = 32,
    Div64 = 64,
    Div128 = 128,
    Div256 = 256,
}

impl From<ADCPrescaler> for Adcpres {
    fn from(value: ADCPrescaler) -> Self {
        match value {
            ADCPrescaler::Div1 => Adcpres::DIV1,
            ADCPrescaler::Div2 => Adcpres::DIV2,
            ADCPrescaler::Div4 => Adcpres::DIV4,
            ADCPrescaler::Div6 => Adcpres::DIV6,
            ADCPrescaler::Div8 => Adcpres::DIV8,
            ADCPrescaler::Div12 => Adcpres::DIV12,
            ADCPrescaler::Div16 => Adcpres::DIV16,
            ADCPrescaler::Div32 => Adcpres::DIV32,
            ADCPrescaler::Div64 => Adcpres::DIV64,
            ADCPrescaler::Div128 => Adcpres::DIV128,
            ADCPrescaler::Div256 => Adcpres::DIV256,
        }
    }
}

#[derive(Clone, Copy)]
pub enum ADCClock {
    AHB(ADCPrescaler),
    PLL(ADCPrescaler),
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
    pub adc: Option<ADCClock>,
    #[cfg(rcc_f3)]
    /// ADC clock setup
    /// - For AHB, a psc of 4 or less must be used
    pub adc34: Option<ADCClock>,
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
    let (Hertz(sysclk), pll_config) = get_sysclk(&config);
    assert!(sysclk <= 72_000_000);

    // Calculate real AHB clock
    let hclk = config.hclk.map(|h| h.0).unwrap_or(sysclk);
    let (hpre_bits, hpre_div) = match sysclk / hclk {
        0 => unreachable!(),
        1 => (Hpre::DIV1, 1),
        2 => (Hpre::DIV2, 2),
        3..=5 => (Hpre::DIV4, 4),
        6..=11 => (Hpre::DIV8, 8),
        12..=39 => (Hpre::DIV16, 16),
        40..=95 => (Hpre::DIV64, 64),
        96..=191 => (Hpre::DIV128, 128),
        192..=383 => (Hpre::DIV256, 256),
        _ => (Hpre::DIV512, 512),
    };
    let hclk = sysclk / hpre_div;
    assert!(hclk <= 72_000_000);

    // Calculate real APB1 clock
    let pclk1 = config.pclk1.map(|p| p.0).unwrap_or(hclk);
    let (ppre1_bits, ppre1) = match hclk / pclk1 {
        0 => unreachable!(),
        1 => (Ppre::DIV1, 1),
        2 => (Ppre::DIV2, 2),
        3..=5 => (Ppre::DIV4, 4),
        6..=11 => (Ppre::DIV8, 8),
        _ => (Ppre::DIV16, 16),
    };
    let timer_mul1 = if ppre1 == 1 { 1 } else { 2 };
    let pclk1 = hclk / ppre1;
    assert!(pclk1 <= 36_000_000);

    // Calculate real APB2 clock
    let pclk2 = config.pclk2.map(|p| p.0).unwrap_or(hclk);
    let (ppre2_bits, ppre2) = match hclk / pclk2 {
        0 => unreachable!(),
        1 => (Ppre::DIV1, 1),
        2 => (Ppre::DIV2, 2),
        3..=5 => (Ppre::DIV4, 4),
        6..=11 => (Ppre::DIV8, 8),
        _ => (Ppre::DIV16, 16),
    };
    let timer_mul2 = if ppre2 == 1 { 1 } else { 2 };
    let pclk2 = hclk / ppre2;
    assert!(pclk2 <= 72_000_000);

    // Set latency based on HCLK frquency
    // RM0316: "The prefetch buffer must be kept on when using a prescaler
    // different from 1 on the AHB clock.", "Half-cycle access cannot be
    // used when there is a prescaler different from 1 on the AHB clock"
    FLASH.acr().modify(|w| {
        w.set_latency(if hclk <= 24_000_000 {
            Latency::WS0
        } else if hclk <= 48_000_000 {
            Latency::WS1
        } else {
            Latency::WS2
        });
        if hpre_div != 1 {
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

    #[cfg(rcc_f3)]
    let adc = config.adc.map(|adc| match adc {
        ADCClock::PLL(psc) => RCC.cfgr2().modify(|w| {
            // Make sure that we're using the PLL
            pll_config.unwrap();
            w.set_adc12pres(psc.into());

            Hertz(sysclk / psc as u32)
        }),
        ADCClock::AHB(psc) => {
            assert!(psc as u16 <= 4);
            assert!(!(psc as u16 == 1 && hpre_bits != Hpre::DIV1));

            // To select this scheme, bits CKMODE[1:0] of the ADCx_CCR register must be
            // different from “00”.
            todo!();
        }
    });

    #[cfg(rcc_f3)]
    let adc34 = config.adc34.map(|adc| match adc {
        ADCClock::PLL(psc) => RCC.cfgr2().modify(|w| {
            // Make sure that we're using the PLL
            pll_config.unwrap();
            w.set_adc34pres(psc.into());

            Hertz(sysclk / psc as u32)
        }),
        ADCClock::AHB(psc) => {
            assert!(psc as u16 <= 4);
            assert!(!(psc as u16 == 1 && hpre_bits != Hpre::DIV1));

            // To select this scheme, bits CKMODE[1:0] of the ADCx_CCR register must be
            // different from “00”.
            todo!();
        }
    });

    // Set prescalers
    // CFGR has been written before (PLL, PLL48) don't overwrite these settings
    RCC.cfgr().modify(|w| {
        w.set_ppre2(ppre2_bits);
        w.set_ppre1(ppre1_bits);
        w.set_hpre(hpre_bits);
    });

    // Wait for the new prescalers to kick in
    // "The clocks are divided with the new prescaler factor from
    //  1 to 16 AHB cycles after write"
    cortex_m::asm::delay(16);

    // CFGR has been written before (PLL, PLL48, clock divider) don't overwrite these settings
    RCC.cfgr().modify(|w| {
        w.set_sw(match (pll_config, config.hse) {
            (Some(_), _) => Sw::PLL,
            (None, Some(_)) => Sw::HSE,
            (None, None) => Sw::HSI,
        })
    });

    set_freqs(Clocks {
        sys: Hertz(sysclk),
        apb1: Hertz(pclk1),
        apb2: Hertz(pclk2),
        apb1_tim: Hertz(pclk1 * timer_mul1),
        apb2_tim: Hertz(pclk2 * timer_mul2),
        ahb1: Hertz(hclk),
        #[cfg(rcc_f3)]
        adc: adc,
        #[cfg(rcc_f3)]
        adc34: adc34,
    });
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
                if #[cfg(any(
                        stm32f302xd, stm32f302xe, stm32f303xd,
                        stm32f303xe, stm32f398xe
                    ))] {
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
fn get_usb_pre(config: &Config, sysclk: u32, pclk1: u32, pll_config: &Option<PllConfig>) -> Usbpre {
    cfg_if::cfg_if! {
        // Some chips do not have USB
        if #[cfg(any(stm32f301, stm32f318, stm32f334))] {
            panic!("USB clock not supported by the chip");
        } else {
            let usb_ok = config.hse.is_some() && pll_config.is_some() && (pclk1 >= 10_000_000);
            match (usb_ok, sysclk) {
                (true, 72_000_000) => Usbpre::DIV1_5,
                (true, 48_000_000) => Usbpre::DIV1,
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
