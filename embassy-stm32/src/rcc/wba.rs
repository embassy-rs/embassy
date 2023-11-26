use stm32_metapac::rcc::vals::{Pllsrc, Sw};

use crate::pac::{FLASH, RCC};
use crate::rcc::{set_freqs, Clocks};
use crate::time::Hertz;

/// HSI speed
pub const HSI_FREQ: Hertz = Hertz(16_000_000);

pub use crate::pac::pwr::vals::Vos as VoltageScale;
pub use crate::pac::rcc::vals::{Hpre as AHBPrescaler, Ppre as APBPrescaler};

#[derive(Copy, Clone)]
pub enum ClockSrc {
    HSE(Hertz),
    HSI,
}

#[derive(Clone, Copy, Debug)]
pub enum PllSource {
    HSE(Hertz),
    HSI,
}

impl Into<Pllsrc> for PllSource {
    fn into(self) -> Pllsrc {
        match self {
            PllSource::HSE(..) => Pllsrc::HSE,
            PllSource::HSI => Pllsrc::HSI,
        }
    }
}

impl Into<Sw> for ClockSrc {
    fn into(self) -> Sw {
        match self {
            ClockSrc::HSE(..) => Sw::HSE,
            ClockSrc::HSI => Sw::HSI,
        }
    }
}

pub struct Config {
    pub mux: ClockSrc,
    pub ahb_pre: AHBPrescaler,
    pub apb1_pre: APBPrescaler,
    pub apb2_pre: APBPrescaler,
    pub apb7_pre: APBPrescaler,
    pub ls: super::LsConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mux: ClockSrc::HSI,
            ahb_pre: AHBPrescaler::DIV1,
            apb1_pre: APBPrescaler::DIV1,
            apb2_pre: APBPrescaler::DIV1,
            apb7_pre: APBPrescaler::DIV1,
            ls: Default::default(),
        }
    }
}

pub(crate) unsafe fn init(config: Config) {
    let sys_clk = match config.mux {
        ClockSrc::HSE(freq) => {
            RCC.cr().write(|w| w.set_hseon(true));
            while !RCC.cr().read().hserdy() {}

            freq
        }
        ClockSrc::HSI => {
            RCC.cr().write(|w| w.set_hsion(true));
            while !RCC.cr().read().hsirdy() {}

            HSI_FREQ
        }
    };

    // TODO make configurable
    let power_vos = VoltageScale::RANGE1;

    // states and programming delay
    let wait_states = match power_vos {
        VoltageScale::RANGE1 => match sys_clk.0 {
            ..=32_000_000 => 0,
            ..=64_000_000 => 1,
            ..=96_000_000 => 2,
            ..=100_000_000 => 3,
            _ => 4,
        },
        VoltageScale::RANGE2 => match sys_clk.0 {
            ..=8_000_000 => 0,
            ..=16_000_000 => 1,
            _ => 2,
        },
    };

    FLASH.acr().modify(|w| {
        w.set_latency(wait_states);
    });

    RCC.cfgr1().modify(|w| {
        w.set_sw(config.mux.into());
    });

    RCC.cfgr2().modify(|w| {
        w.set_hpre(config.ahb_pre);
        w.set_ppre1(config.apb1_pre);
        w.set_ppre2(config.apb2_pre);
    });

    RCC.cfgr3().modify(|w| {
        w.set_ppre7(config.apb7_pre);
    });

    let ahb_freq = sys_clk / config.ahb_pre;
    let (apb1_freq, apb1_tim_freq) = match config.apb1_pre {
        APBPrescaler::DIV1 => (ahb_freq, ahb_freq),
        pre => {
            let freq = ahb_freq / pre;
            (freq, freq * 2u32)
        }
    };
    let (apb2_freq, apb2_tim_freq) = match config.apb2_pre {
        APBPrescaler::DIV1 => (ahb_freq, ahb_freq),
        pre => {
            let freq = ahb_freq / pre;
            (freq, freq * 2u32)
        }
    };
    let (apb7_freq, _apb7_tim_freq) = match config.apb7_pre {
        APBPrescaler::DIV1 => (ahb_freq, ahb_freq),
        pre => {
            let freq = ahb_freq / pre;
            (freq, freq * 2u32)
        }
    };

    let rtc = config.ls.init();

    set_freqs(Clocks {
        sys: sys_clk,
        hclk1: ahb_freq,
        hclk2: ahb_freq,
        hclk4: ahb_freq,
        pclk1: apb1_freq,
        pclk2: apb2_freq,
        pclk7: apb7_freq,
        pclk1_tim: apb1_tim_freq,
        pclk2_tim: apb2_tim_freq,
        rtc,
    });
}
