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

impl Into<u8> for HSI16Prescaler {
    fn into(self) -> u8 {
        match self {
            HSI16Prescaler::NotDivided => 0x00,
            HSI16Prescaler::Div2 => 0x01,
            HSI16Prescaler::Div4 => 0x02,
            HSI16Prescaler::Div8 => 0x03,
            HSI16Prescaler::Div16 => 0x04,
            HSI16Prescaler::Div32 => 0x05,
            HSI16Prescaler::Div64 => 0x06,
            HSI16Prescaler::Div128 => 0x07,
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

impl Into<u8> for APBPrescaler {
    fn into(self) -> u8 {
        match self {
            APBPrescaler::NotDivided => 1,
            APBPrescaler::Div2 => 0x04,
            APBPrescaler::Div4 => 0x05,
            APBPrescaler::Div8 => 0x06,
            APBPrescaler::Div16 => 0x07,
        }
    }
}

impl Into<u8> for AHBPrescaler {
    fn into(self) -> u8 {
        match self {
            AHBPrescaler::NotDivided => 1,
            AHBPrescaler::Div2 => 0x08,
            AHBPrescaler::Div4 => 0x09,
            AHBPrescaler::Div8 => 0x0a,
            AHBPrescaler::Div16 => 0x0b,
            AHBPrescaler::Div64 => 0x0c,
            AHBPrescaler::Div128 => 0x0d,
            AHBPrescaler::Div256 => 0x0e,
            AHBPrescaler::Div512 => 0x0f,
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
            let div: u8 = div.into();
            RCC.cr().write(|w| {
                w.set_hsidiv(div);
                w.set_hsion(true)
            });
            while !RCC.cr().read().hsirdy() {}

            (HSI_FREQ >> div, 0x00)
        }
        ClockSrc::HSE(freq) => {
            // Enable HSE
            RCC.cr().write(|w| w.set_hseon(true));
            while !RCC.cr().read().hserdy() {}

            (freq.0, 0x01)
        }
        ClockSrc::LSI => {
            // Enable LSI
            RCC.csr().write(|w| w.set_lsion(true));
            while !RCC.csr().read().lsirdy() {}
            (LSI_FREQ, 0x03)
        }
    };

    RCC.cfgr().modify(|w| {
        w.set_sw(sw.into());
        w.set_hpre(config.ahb_pre.into());
        w.set_ppre(config.apb_pre.into());
    });

    let ahb_freq: u32 = match config.ahb_pre {
        AHBPrescaler::NotDivided => sys_clk,
        pre => {
            let pre: u8 = pre.into();
            let pre = 1 << (pre as u32 - 7);
            sys_clk / pre
        }
    };

    let (apb_freq, apb_tim_freq) = match config.apb_pre {
        APBPrescaler::NotDivided => (ahb_freq, ahb_freq),
        pre => {
            let pre: u8 = pre.into();
            let pre: u8 = 1 << (pre - 3);
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
