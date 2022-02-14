use core::convert::TryFrom;

use super::{set_freqs, Clocks};
use crate::pac::flash::vals::Latency;
use crate::pac::rcc::vals::{Adcpre, Hpre, Pllmul, Pllsrc, Ppre1, Sw, Usbpre};
use crate::pac::{FLASH, RCC};
use crate::time::Hertz;

const HSI: u32 = 8_000_000;

/// Configuration of the clocks
///
#[non_exhaustive]
#[derive(Default)]
pub struct Config {
    pub hse: Option<Hertz>,

    pub sys_ck: Option<Hertz>,
    pub hclk: Option<Hertz>,
    pub pclk1: Option<Hertz>,
    pub pclk2: Option<Hertz>,
    pub adcclk: Option<Hertz>,
}

pub(crate) unsafe fn init(config: Config) {
    let pllsrcclk = config.hse.map(|hse| hse.0).unwrap_or(HSI / 2);
    let sysclk = config.sys_ck.map(|sys| sys.0).unwrap_or(pllsrcclk);
    let pllmul = sysclk / pllsrcclk;

    let (pllmul_bits, real_sysclk) = if pllmul == 1 {
        (None, config.hse.map(|hse| hse.0).unwrap_or(HSI))
    } else {
        let pllmul = core::cmp::min(core::cmp::max(pllmul, 1), 16);
        (Some(pllmul as u8 - 2), pllsrcclk * pllmul)
    };

    assert!(real_sysclk <= 72_000_000);

    let hpre_bits = config
        .hclk
        .map(|hclk| match real_sysclk / hclk.0 {
            0 => unreachable!(),
            1 => 0b0111,
            2 => 0b1000,
            3..=5 => 0b1001,
            6..=11 => 0b1010,
            12..=39 => 0b1011,
            40..=95 => 0b1100,
            96..=191 => 0b1101,
            192..=383 => 0b1110,
            _ => 0b1111,
        })
        .unwrap_or(0b0111);

    let hclk = if hpre_bits >= 0b1100 {
        real_sysclk / (1 << (hpre_bits - 0b0110))
    } else {
        real_sysclk / (1 << (hpre_bits - 0b0111))
    };

    assert!(hclk <= 72_000_000);

    let ppre1_bits = config
        .pclk1
        .map(|pclk1| match hclk / pclk1.0 {
            0 => unreachable!(),
            1 => 0b011,
            2 => 0b100,
            3..=5 => 0b101,
            6..=11 => 0b110,
            _ => 0b111,
        })
        .unwrap_or(0b011);

    let ppre1 = 1 << (ppre1_bits - 0b011);
    let pclk1 = hclk / u32::try_from(ppre1).unwrap();
    let timer_mul1 = if ppre1 == 1 { 1 } else { 2 };

    assert!(pclk1 <= 36_000_000);

    let ppre2_bits = config
        .pclk2
        .map(|pclk2| match hclk / pclk2.0 {
            0 => unreachable!(),
            1 => 0b011,
            2 => 0b100,
            3..=5 => 0b101,
            6..=11 => 0b110,
            _ => 0b111,
        })
        .unwrap_or(0b011);

    let ppre2 = 1 << (ppre2_bits - 0b011);
    let pclk2 = hclk / u32::try_from(ppre2).unwrap();
    let timer_mul2 = if ppre2 == 1 { 1 } else { 2 };

    assert!(pclk2 <= 72_000_000);

    // Only needed for stm32f103?
    FLASH.acr().write(|w| {
        w.set_latency(if real_sysclk <= 24_000_000 {
            Latency(0b000)
        } else if real_sysclk <= 48_000_000 {
            Latency(0b001)
        } else {
            Latency(0b010)
        });
    });

    // the USB clock is only valid if an external crystal is used, the PLL is enabled, and the
    // PLL output frequency is a supported one.
    // usbpre == false: divide clock by 1.5, otherwise no division
    let (usbpre, _usbclk_valid) = match (config.hse, pllmul_bits, real_sysclk) {
        (Some(_), Some(_), 72_000_000) => (false, true),
        (Some(_), Some(_), 48_000_000) => (true, true),
        _ => (true, false),
    };

    let apre_bits: u8 = config
        .adcclk
        .map(|adcclk| match pclk2 / adcclk.0 {
            0..=2 => 0b00,
            3..=4 => 0b01,
            5..=7 => 0b10,
            _ => 0b11,
        })
        .unwrap_or(0b11);

    let apre = (apre_bits + 1) << 1;
    let adcclk = pclk2 / unwrap!(u32::try_from(apre));

    assert!(adcclk <= 14_000_000);

    if config.hse.is_some() {
        // enable HSE and wait for it to be ready
        RCC.cr().modify(|w| w.set_hseon(true));
        while !RCC.cr().read().hserdy() {}
    }

    if let Some(pllmul_bits) = pllmul_bits {
        // enable PLL and wait for it to be ready
        RCC.cfgr().modify(|w| {
            w.set_pllmul(Pllmul(pllmul_bits));
            w.set_pllsrc(Pllsrc(config.hse.is_some() as u8));
        });

        RCC.cr().modify(|w| w.set_pllon(true));
        while !RCC.cr().read().pllrdy() {}
    }

    // Only needed for stm32f103?
    RCC.cfgr().modify(|w| {
        w.set_adcpre(Adcpre(apre_bits));
        w.set_ppre2(Ppre1(ppre2_bits));
        w.set_ppre1(Ppre1(ppre1_bits));
        w.set_hpre(Hpre(hpre_bits));
        w.set_usbpre(Usbpre(usbpre as u8));
        w.set_sw(Sw(if pllmul_bits.is_some() {
            // PLL
            0b10
        } else if config.hse.is_some() {
            // HSE
            0b1
        } else {
            // HSI
            0b0
        }));
    });

    set_freqs(Clocks {
        sys: Hertz(real_sysclk),
        apb1: Hertz(pclk1),
        apb2: Hertz(pclk2),
        apb1_tim: Hertz(pclk1 * timer_mul1),
        apb2_tim: Hertz(pclk2 * timer_mul2),
        ahb1: Hertz(hclk),
        adc: Hertz(adcclk),
    });
}
