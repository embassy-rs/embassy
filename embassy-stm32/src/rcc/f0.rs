use stm32_metapac::flash::vals::Latency;

use crate::pac::rcc::vals::{Hpre, Pllmul, Pllsrc, Ppre, Sw, Usbsw};
use crate::pac::{FLASH, RCC};
use crate::time::Hertz;

/// HSI speed
pub const HSI_FREQ: Hertz = Hertz(8_000_000);

/// Configuration of the clocks
///
/// hse takes precedence over hsi48 if both are enabled
#[non_exhaustive]
pub struct Config {
    pub hse: Option<Hertz>,
    pub bypass_hse: bool,
    pub usb_pll: bool,

    #[cfg(crs)]
    pub hsi48: Option<super::Hsi48Config>,

    pub sys_ck: Option<Hertz>,
    pub hclk: Option<Hertz>,
    pub pclk: Option<Hertz>,

    pub ls: super::LsConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            hse: Default::default(),
            bypass_hse: Default::default(),
            usb_pll: Default::default(),
            #[cfg(crs)]
            hsi48: Some(Default::default()),
            sys_ck: Default::default(),
            hclk: Default::default(),
            pclk: Default::default(),
            ls: Default::default(),
        }
    }
}

pub(crate) unsafe fn init(config: Config) {
    let sysclk = config.sys_ck.map(|v| v.0).unwrap_or(HSI_FREQ.0);

    #[cfg(crs)]
    let hsi48 = config.hsi48.map(|config| super::init_hsi48(config));
    #[cfg(not(crs))]
    let hsi48: Option<Hertz> = None;

    let (src_clk, use_hsi48) = config.hse.map(|v| (v.0, false)).unwrap_or_else(|| {
        if hsi48.is_some() {
            return (48_000_000, true);
        }
        (HSI_FREQ.0, false)
    });

    let (pllmul_bits, real_sysclk) = if sysclk == src_clk {
        (None, sysclk)
    } else {
        let prediv = if config.hse.is_some() { 1 } else { 2 };
        let pllmul = (2 * prediv * sysclk + src_clk) / src_clk / 2;
        let pllmul = pllmul.max(2).min(16);

        let pllmul_bits = pllmul as u8 - 2;
        let real_sysclk = pllmul * src_clk / prediv;
        (Some(pllmul_bits), real_sysclk)
    };

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
    let hclk = real_sysclk / (1 << (hpre_bits - 0b0111));

    let ppre_bits = config
        .pclk
        .map(|pclk| match hclk / pclk.0 {
            0 => unreachable!(),
            1 => 0b011,
            2 => 0b100,
            3..=5 => 0b101,
            6..=11 => 0b110,
            _ => 0b111,
        })
        .unwrap_or(0b011);

    let ppre: u8 = 1 << (ppre_bits - 0b011);
    let pclk = hclk / u32::from(ppre);

    let timer_mul = if ppre == 1 { 1 } else { 2 };

    FLASH.acr().write(|w| {
        w.set_latency(if real_sysclk <= 24_000_000 {
            Latency::WS0
        } else {
            Latency::WS1
        });
    });

    match (config.hse.is_some(), use_hsi48) {
        (true, _) => {
            RCC.cr().modify(|w| {
                w.set_csson(true);
                w.set_hseon(true);
                w.set_hsebyp(config.bypass_hse);
            });
            while !RCC.cr().read().hserdy() {}

            if pllmul_bits.is_some() {
                RCC.cfgr().modify(|w| w.set_pllsrc(Pllsrc::HSE_DIV_PREDIV))
            }
        }
        // use_hsi48 will always be false for stm32f0x0
        #[cfg(not(stm32f0x0))]
        (false, true) => {
            RCC.cr2().modify(|w| w.set_hsi48on(true));
            while !RCC.cr2().read().hsi48rdy() {}

            if pllmul_bits.is_some() {
                RCC.cfgr().modify(|w| w.set_pllsrc(Pllsrc::HSI48_DIV_PREDIV))
            }
        }
        _ => {
            RCC.cr().modify(|w| w.set_hsion(true));
            while !RCC.cr().read().hsirdy() {}

            if pllmul_bits.is_some() {
                RCC.cfgr().modify(|w| w.set_pllsrc(Pllsrc::HSI_DIV2))
            }
        }
    }

    if config.usb_pll {
        RCC.cfgr3().modify(|w| w.set_usbsw(Usbsw::PLL1_P));
    }
    // TODO: Option to use CRS (Clock Recovery)

    if let Some(pllmul_bits) = pllmul_bits {
        RCC.cfgr().modify(|w| w.set_pllmul(Pllmul::from_bits(pllmul_bits)));

        RCC.cr().modify(|w| w.set_pllon(true));
        while !RCC.cr().read().pllrdy() {}

        RCC.cfgr().modify(|w| {
            w.set_ppre(Ppre::from_bits(ppre_bits));
            w.set_hpre(Hpre::from_bits(hpre_bits));
            w.set_sw(Sw::PLL1_P)
        });
    } else {
        RCC.cfgr().modify(|w| {
            w.set_ppre(Ppre::from_bits(ppre_bits));
            w.set_hpre(Hpre::from_bits(hpre_bits));

            if config.hse.is_some() {
                w.set_sw(Sw::HSE);
            } else if use_hsi48 {
                #[cfg(not(stm32f0x0))]
                w.set_sw(Sw::HSI48);
            } else {
                w.set_sw(Sw::HSI)
            }
        })
    }

    let rtc = config.ls.init();

    set_clocks!(
        hsi: None,
        lse: None,
        sys: Some(Hertz(real_sysclk)),
        pclk1: Some(Hertz(pclk)),
        pclk2: Some(Hertz(pclk)),
        pclk1_tim: Some(Hertz(pclk * timer_mul)),
        pclk2_tim: Some(Hertz(pclk * timer_mul)),
        hclk1: Some(Hertz(hclk)),
        rtc: rtc,
        hsi48: hsi48,

        // TODO:
        pll1_p: None,
    );
}
