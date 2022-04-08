use crate::pac::RCC;
use crate::rcc::{set_freqs, Clocks};
use crate::time::U32Ext;

/// Most of clock setup is copied from stm32l0xx-hal, and adopted to the generated PAC,
/// and with the addition of the init function to configure a system clock.

/// Only the basic setup using the HSE and HSI clocks are supported as of now.

/// HSI speed
pub const HSI_FREQ: u32 = 16_000_000;

pub const HSE32_FREQ: u32 = 32_000_000;

/// System clock mux source
#[derive(Clone, Copy)]
pub enum ClockSrc {
    HSE32,
    HSI16,
}

/// AHB prescaler
#[derive(Clone, Copy, PartialEq)]
pub enum AHBPrescaler {
    NotDivided,
    Div2,
    Div3,
    Div4,
    Div5,
    Div6,
    Div8,
    Div10,
    Div16,
    Div32,
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
            AHBPrescaler::Div3 => 0x01,
            AHBPrescaler::Div4 => 0x09,
            AHBPrescaler::Div5 => 0x02,
            AHBPrescaler::Div6 => 0x05,
            AHBPrescaler::Div8 => 0x0a,
            AHBPrescaler::Div10 => 0x06,
            AHBPrescaler::Div16 => 0x0b,
            AHBPrescaler::Div32 => 0x07,
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
    pub apb1_pre: APBPrescaler,
    pub apb2_pre: APBPrescaler,
    pub enable_lsi: bool,
}

impl Default for Config {
    #[inline]
    fn default() -> Config {
        Config {
            mux: ClockSrc::HSI16,
            ahb_pre: AHBPrescaler::NotDivided,
            apb1_pre: APBPrescaler::NotDivided,
            apb2_pre: APBPrescaler::NotDivided,
            enable_lsi: false,
        }
    }
}

pub(crate) unsafe fn init(config: Config) {
    let (sys_clk, sw) = match config.mux {
        ClockSrc::HSI16 => {
            // Enable HSI16
            RCC.cr().write(|w| w.set_hsion(true));
            while !RCC.cr().read().hsirdy() {}

            (HSI_FREQ, 0x01)
        }
        ClockSrc::HSE32 => {
            // Enable HSE32
            RCC.cr().write(|w| {
                w.set_hsebyppwr(true);
                w.set_hseon(true);
            });
            while !RCC.cr().read().hserdy() {}

            (HSE32_FREQ, 0x02)
        }
    };

    RCC.cfgr().modify(|w| {
        w.set_sw(sw.into());
        if config.ahb_pre == AHBPrescaler::NotDivided {
            w.set_hpre(0);
        } else {
            w.set_hpre(config.ahb_pre.into());
        }
        w.set_ppre1(config.apb1_pre.into());
        w.set_ppre2(config.apb2_pre.into());
    });

    let ahb_freq: u32 = match config.ahb_pre {
        AHBPrescaler::NotDivided => sys_clk,
        pre => {
            let pre: u8 = pre.into();
            let pre = 1 << (pre as u32 - 7);
            sys_clk / pre
        }
    };

    let (apb1_freq, apb1_tim_freq) = match config.apb1_pre {
        APBPrescaler::NotDivided => (ahb_freq, ahb_freq),
        pre => {
            let pre: u8 = pre.into();
            let pre: u8 = 1 << (pre - 3);
            let freq = ahb_freq / pre as u32;
            (freq, freq * 2)
        }
    };

    let (apb2_freq, apb2_tim_freq) = match config.apb2_pre {
        APBPrescaler::NotDivided => (ahb_freq, ahb_freq),
        pre => {
            let pre: u8 = pre.into();
            let pre: u8 = 1 << (pre - 3);
            let freq = ahb_freq / (1 << (pre as u8 - 3));
            (freq, freq * 2)
        }
    };

    // TODO: completely untested
    let apb3_freq = ahb_freq;

    if config.enable_lsi {
        let csr = RCC.csr().read();
        if !csr.lsion() {
            RCC.csr().modify(|w| w.set_lsion(true));
            while !RCC.csr().read().lsirdy() {}
        }
    }

    set_freqs(Clocks {
        sys: sys_clk.hz(),
        ahb1: ahb_freq.hz(),
        ahb2: ahb_freq.hz(),
        ahb3: ahb_freq.hz(),
        apb1: apb1_freq.hz(),
        apb2: apb2_freq.hz(),
        apb3: apb3_freq.hz(),
        apb1_tim: apb1_tim_freq.hz(),
        apb2_tim: apb2_tim_freq.hz(),
    });
}
