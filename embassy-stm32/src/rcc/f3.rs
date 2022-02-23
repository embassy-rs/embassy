use crate::pac::flash::vals::Latency;
use crate::pac::rcc::vals::{Hpre, Pllmul, Pllsrc, Ppre, Prediv, Sw, Usbpre};
use crate::pac::{FLASH, RCC};
use crate::rcc::{set_freqs, Clocks};
use crate::time::Hertz;

const HSI: u32 = 8_000_000;

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
}

// Information required to setup the PLL clock
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
    FLASH.acr().write(|w| {
        w.set_latency(if hclk <= 24_000_000 {
            Latency::WS0
        } else if hclk <= 48_000_000 {
            Latency::WS1
        } else {
            Latency::WS2
        });
    });

    // Enable HSE
    if config.hse.is_some() {
        RCC.cr().write(|w| {
            w.set_hsebyp(config.bypass_hse);
            // We turn on clock security to switch to HSI when HSE fails
            w.set_csson(true);
            w.set_hseon(true);
        });
        while !RCC.cr().read().hserdy() {}
    }

    // Enable PLL
    if let Some(ref pll_config) = pll_config {
        RCC.cfgr().write(|w| {
            w.set_pllmul(pll_config.pll_mul);
            w.set_pllsrc(pll_config.pll_src);
        });
        if let Some(pll_div) = pll_config.pll_div {
            RCC.cfgr2().write(|w| w.set_prediv(pll_div));
        }
        RCC.cr().modify(|w| w.set_pllon(true));
        while !RCC.cr().read().pllrdy() {}
    }

    if config.pll48 {
        let usb_pre = get_usb_pre(&config, sysclk, pclk1, &pll_config);
        RCC.cfgr().write(|w| {
            w.set_usbpre(usb_pre);
        });
    }

    // Set prescalers
    RCC.cfgr().write(|w| {
        w.set_ppre2(ppre2_bits);
        w.set_ppre1(ppre1_bits);
        w.set_hpre(hpre_bits);
    });

    // Wait for the new prescalers to kick in
    // "The clocks are divided with the new prescaler factor from
    //  1 to 16 AHB cycles after write"
    cortex_m::asm::delay(16);

    RCC.cfgr().write(|w| {
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
    });
}

#[inline]
fn get_sysclk(config: &Config) -> (Hertz, Option<PllConfig>) {
    match (config.sysclk, config.hse) {
        (Some(sysclk), Some(hse)) if sysclk == hse => (hse, None),
        (Some(sysclk), None) if sysclk.0 == HSI => (Hertz(HSI), None),
        // If the user selected System clock is different from HSI or HSE
        // we will have to setup PLL clock source
        (Some(sysclk), _) => {
            let (sysclk, pll_config) = calc_pll(config, sysclk);
            (sysclk, Some(pll_config))
        }
        (None, Some(hse)) => (hse, None),
        (None, None) => (Hertz(HSI), None),
    }
}

#[inline]
fn calc_pll(config: &Config, Hertz(sysclk): Hertz) -> (Hertz, PllConfig) {
    // Calculates the Multiplier and the Divisor to arrive at
    // the required System clock from PLL source frequency
    let get_mul_div = |sysclk, pllsrcclk| {
        let common_div = gcd(sysclk, pllsrcclk);
        let mut multiplier = sysclk / common_div;
        let mut divisor = pllsrcclk / common_div;
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
                    let (multiplier, divisor) = get_mul_div(sysclk, HSI);
                    (
                        Hertz((HSI / divisor) * multiplier),
                        Pllsrc::HSI_DIV_PREDIV,
                        into_pll_mul(multiplier),
                        Some(into_pre_div(divisor)),
                    )
                } else {
                    let pllsrcclk = HSI / 2;
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
