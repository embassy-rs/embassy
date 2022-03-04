use crate::pac::rcc::vals::{Hpre, Hsidiv, Ppre, Sw};
use crate::pac::{PWR, RCC};
use crate::rcc::{set_freqs, Clocks};
use crate::time::Hertz;
use crate::time::U32Ext;

/// HSI speed
pub const HSI_FREQ: u32 = 16_000_000;

/// LSI speed
pub const LSI_FREQ: u32 = 32_000;

/// System clock mux source
#[derive(Clone, Copy)]
pub enum ClockSrc {
    HSE(Hertz),
    HSI16(HSI16Prescaler),
    LSI,
}

#[derive(Clone, Copy)]
pub enum HSI16Prescaler {
    NotDivided,
    Div2,
    Div4,
    Div8,
    Div16,
    Div32,
    Div64,
    Div128,
}

impl Into<Hsidiv> for HSI16Prescaler {
    fn into(self) -> Hsidiv {
        match self {
            HSI16Prescaler::NotDivided => Hsidiv::DIV1,
            HSI16Prescaler::Div2 => Hsidiv::DIV2,
            HSI16Prescaler::Div4 => Hsidiv::DIV4,
            HSI16Prescaler::Div8 => Hsidiv::DIV8,
            HSI16Prescaler::Div16 => Hsidiv::DIV16,
            HSI16Prescaler::Div32 => Hsidiv::DIV32,
            HSI16Prescaler::Div64 => Hsidiv::DIV64,
            HSI16Prescaler::Div128 => Hsidiv::DIV128,
        }
    }
}

/// AHB prescaler
#[derive(Clone, Copy, PartialEq)]
pub enum AHBPrescaler {
    NotDivided,
    Div2,
    Div4,
    Div8,
    Div16,
    Div64,
    Div128,
    Div256,
    Div512,
}

/// APB prescaler
#[derive(Clone, Copy)]
pub enum APBPrescaler {
    NotDivided,
    Div2,
    Div4,
    Div8,
    Div16,
}

impl Into<Ppre> for APBPrescaler {
    fn into(self) -> Ppre {
        match self {
            APBPrescaler::NotDivided => Ppre::DIV1,
            APBPrescaler::Div2 => Ppre::DIV2,
            APBPrescaler::Div4 => Ppre::DIV4,
            APBPrescaler::Div8 => Ppre::DIV8,
            APBPrescaler::Div16 => Ppre::DIV16,
        }
    }
}

impl Into<Hpre> for AHBPrescaler {
    fn into(self) -> Hpre {
        match self {
            AHBPrescaler::NotDivided => Hpre::DIV1,
            AHBPrescaler::Div2 => Hpre::DIV2,
            AHBPrescaler::Div4 => Hpre::DIV4,
            AHBPrescaler::Div8 => Hpre::DIV8,
            AHBPrescaler::Div16 => Hpre::DIV16,
            AHBPrescaler::Div64 => Hpre::DIV64,
            AHBPrescaler::Div128 => Hpre::DIV128,
            AHBPrescaler::Div256 => Hpre::DIV256,
            AHBPrescaler::Div512 => Hpre::DIV512,
        }
    }
}

/// Clocks configutation
pub struct Config {
    pub mux: ClockSrc,
    pub ahb_pre: AHBPrescaler,
    pub apb_pre: APBPrescaler,
    pub low_power_run: bool,
}

impl Default for Config {
    #[inline]
    fn default() -> Config {
        Config {
            mux: ClockSrc::HSI16(HSI16Prescaler::NotDivided),
            ahb_pre: AHBPrescaler::NotDivided,
            apb_pre: APBPrescaler::NotDivided,
            low_power_run: false,
        }
    }
}

pub(crate) unsafe fn init(config: Config) {
    let (sys_clk, sw) = match config.mux {
        ClockSrc::HSI16(div) => {
            // Enable HSI16
            let div: Hsidiv = div.into();
            RCC.cr().write(|w| {
                w.set_hsidiv(div);
                w.set_hsion(true)
            });
            while !RCC.cr().read().hsirdy() {}

            (HSI_FREQ >> div.0, Sw::HSI)
        }
        ClockSrc::HSE(freq) => {
            // Enable HSE
            RCC.cr().write(|w| w.set_hseon(true));
            while !RCC.cr().read().hserdy() {}

            (freq.0, Sw::HSE)
        }
        ClockSrc::LSI => {
            // Enable LSI
            RCC.csr().write(|w| w.set_lsion(true));
            while !RCC.csr().read().lsirdy() {}
            (LSI_FREQ, Sw::LSI)
        }
    };

    RCC.cfgr().modify(|w| {
        w.set_sw(sw.into());
        w.set_hpre(config.ahb_pre.into());
        w.set_ppre(config.apb_pre.into());
    });

    let ahb_div = match config.ahb_pre {
        AHBPrescaler::NotDivided => 1,
        AHBPrescaler::Div2 => 2,
        AHBPrescaler::Div4 => 4,
        AHBPrescaler::Div8 => 8,
        AHBPrescaler::Div16 => 16,
        AHBPrescaler::Div64 => 64,
        AHBPrescaler::Div128 => 128,
        AHBPrescaler::Div256 => 256,
        AHBPrescaler::Div512 => 512,
    };
    let ahb_freq = sys_clk / ahb_div;

    let (apb_freq, apb_tim_freq) = match config.apb_pre {
        APBPrescaler::NotDivided => (ahb_freq, ahb_freq),
        pre => {
            let pre: Ppre = pre.into();
            let pre: u8 = 1 << (pre.0 - 3);
            let freq = ahb_freq / pre as u32;
            (freq, freq * 2)
        }
    };

    if config.low_power_run {
        assert!(sys_clk.hz() <= 2_000_000.hz());
        PWR.cr1().modify(|w| w.set_lpr(true));
    }

    set_freqs(Clocks {
        sys: sys_clk.hz(),
        ahb1: ahb_freq.hz(),
        apb1: apb_freq.hz(),
        apb1_tim: apb_tim_freq.hz(),
    });
}
