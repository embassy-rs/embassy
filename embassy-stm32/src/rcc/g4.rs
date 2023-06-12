use stm32_metapac::flash::vals::Latency;
use stm32_metapac::rcc::vals::{Hpre, Pllsrc, Ppre, Sw};
use stm32_metapac::FLASH;

use crate::pac::{PWR, RCC};
use crate::rcc::{set_freqs, Clocks};
use crate::time::Hertz;

/// HSI speed
pub const HSI_FREQ: Hertz = Hertz(16_000_000);

/// LSI speed
pub const LSI_FREQ: Hertz = Hertz(32_000);

/// System clock mux source
#[derive(Clone, Copy)]
pub enum ClockSrc {
    HSE(Hertz),
    HSI16,
    PLLCLK(PllSrc, PllM, PllN, PllR),
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

/// PLL clock input source
#[derive(Clone, Copy, Debug)]
pub enum PllSrc {
    HSI16,
    HSE(Hertz),
}

impl Into<Pllsrc> for PllSrc {
    fn into(self) -> Pllsrc {
        match self {
            PllSrc::HSE(..) => Pllsrc::HSE,
            PllSrc::HSI16 => Pllsrc::HSI16,
        }
    }
}

#[derive(Clone, Copy)]
pub enum PllR {
    Div2,
    Div4,
    Div6,
    Div8,
}

impl PllR {
    pub fn to_div(self) -> u32 {
        let val: u8 = self.into();
        (val as u32 + 1) * 2
    }
}

impl From<PllR> for u8 {
    fn from(val: PllR) -> u8 {
        match val {
            PllR::Div2 => 0b00,
            PllR::Div4 => 0b01,
            PllR::Div6 => 0b10,
            PllR::Div8 => 0b11,
        }
    }
}

seq_macro::seq!(N in 8..=127 {
    #[derive(Clone, Copy)]
    pub enum PllN {
        #(
            Mul~N,
        )*
    }

    impl From<PllN> for u8 {
        fn from(val: PllN) -> u8 {
            match val {
                #(
                    PllN::Mul~N => N,
                )*
            }
        }
    }

    impl PllN {
        pub fn to_mul(self) -> u32 {
            match self {
                #(
                    PllN::Mul~N => N,
                )*
            }
        }
    }
});

// Pre-division
#[derive(Copy, Clone)]
pub enum PllM {
    Div1,
    Div2,
    Div3,
    Div4,
    Div5,
    Div6,
    Div7,
    Div8,
    Div9,
    Div10,
    Div11,
    Div12,
    Div13,
    Div14,
    Div15,
    Div16,
}

impl PllM {
    pub fn to_div(self) -> u32 {
        let val: u8 = self.into();
        val as u32 + 1
    }
}

impl From<PllM> for u8 {
    fn from(val: PllM) -> u8 {
        match val {
            PllM::Div1 => 0b0000,
            PllM::Div2 => 0b0001,
            PllM::Div3 => 0b0010,
            PllM::Div4 => 0b0011,
            PllM::Div5 => 0b0100,
            PllM::Div6 => 0b0101,
            PllM::Div7 => 0b0110,
            PllM::Div8 => 0b0111,
            PllM::Div9 => 0b1000,
            PllM::Div10 => 0b1001,
            PllM::Div11 => 0b1010,
            PllM::Div12 => 0b1011,
            PllM::Div13 => 0b1100,
            PllM::Div14 => 0b1101,
            PllM::Div15 => 0b1110,
            PllM::Div16 => 0b1111,
        }
    }
}

impl AHBPrescaler {
    const fn div(self) -> u32 {
        match self {
            AHBPrescaler::NotDivided => 1,
            AHBPrescaler::Div2 => 2,
            AHBPrescaler::Div4 => 4,
            AHBPrescaler::Div8 => 8,
            AHBPrescaler::Div16 => 16,
            AHBPrescaler::Div64 => 64,
            AHBPrescaler::Div128 => 128,
            AHBPrescaler::Div256 => 256,
            AHBPrescaler::Div512 => 512,
        }
    }
}

impl APBPrescaler {
    const fn div(self) -> u32 {
        match self {
            APBPrescaler::NotDivided => 1,
            APBPrescaler::Div2 => 2,
            APBPrescaler::Div4 => 4,
            APBPrescaler::Div8 => 8,
            APBPrescaler::Div16 => 16,
        }
    }
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
    pub apb1_pre: APBPrescaler,
    pub apb2_pre: APBPrescaler,
    pub low_power_run: bool,
}

impl Default for Config {
    #[inline]
    fn default() -> Config {
        Config {
            mux: ClockSrc::HSI16,
            ahb_pre: AHBPrescaler::NotDivided,
            apb1_pre: APBPrescaler::NotDivided,
            apb2_pre: APBPrescaler::NotDivided,
            low_power_run: false,
        }
    }
}

pub(crate) unsafe fn init(config: Config) {
    let (sys_clk, sw) = match config.mux {
        ClockSrc::HSI16 => {
            // Enable HSI16
            RCC.cr().write(|w| w.set_hsion(true));
            while !RCC.cr().read().hsirdy() {}

            (HSI_FREQ.0, Sw::HSI16)
        }
        ClockSrc::HSE(freq) => {
            // Enable HSE
            RCC.cr().write(|w| w.set_hseon(true));
            while !RCC.cr().read().hserdy() {}

            (freq.0, Sw::HSE)
        }
        ClockSrc::PLLCLK(src, prediv, mul, div) => {
            let src_freq = match src {
                PllSrc::HSI16 => {
                    // Enable HSI16 as clock source for PLL
                    RCC.cr().write(|w| w.set_hsion(true));
                    while !RCC.cr().read().hsirdy() {}

                    HSI_FREQ.0
                }
                PllSrc::HSE(freq) => {
                    // Enable HSE as clock source for PLL
                    RCC.cr().write(|w| w.set_hseon(true));
                    while !RCC.cr().read().hserdy() {}

                    freq.0
                }
            };

            // Make sure PLL is disabled while we configure it
            RCC.cr().modify(|w| w.set_pllon(false));
            while RCC.cr().read().pllrdy() {}

            let freq = src_freq / prediv.to_div() * mul.to_mul() / div.to_div();
            assert!(freq <= 170_000_000);

            if freq >= 150_000_000 {
                // Enable Core Boost mode on freq >= 150Mhz ([RM0440] p234)
                PWR.cr5().modify(|w| w.set_r1mode(false));
                // Set flash wait state in boost mode based on frequency ([RM0440] p191)
                if freq <= 36_000_000 {
                    FLASH.acr().modify(|w| w.set_latency(Latency::WS0));
                } else if freq <= 68_000_000 {
                    FLASH.acr().modify(|w| w.set_latency(Latency::WS1));
                } else if freq <= 102_000_000 {
                    FLASH.acr().modify(|w| w.set_latency(Latency::WS2));
                } else if freq <= 136_000_000 {
                    FLASH.acr().modify(|w| w.set_latency(Latency::WS3));
                } else {
                    FLASH.acr().modify(|w| w.set_latency(Latency::WS4));
                }
            } else {
                PWR.cr5().modify(|w| w.set_r1mode(true));
                // Set flash wait state in normal mode based on frequency ([RM0440] p191)
                if freq <= 30_000_000 {
                    FLASH.acr().modify(|w| w.set_latency(Latency::WS0));
                } else if freq <= 60_000_000 {
                    FLASH.acr().modify(|w| w.set_latency(Latency::WS1));
                } else if freq <= 80_000_000 {
                    FLASH.acr().modify(|w| w.set_latency(Latency::WS2));
                } else if freq <= 120_000_000 {
                    FLASH.acr().modify(|w| w.set_latency(Latency::WS3));
                } else {
                    FLASH.acr().modify(|w| w.set_latency(Latency::WS4));
                }
            }

            RCC.pllcfgr().write(move |w| {
                w.set_plln(mul.into());
                w.set_pllm(prediv.into());
                w.set_pllr(div.into());
                w.set_pllsrc(src.into());
            });

            // Enable PLL
            RCC.cr().modify(|w| w.set_pllon(true));
            while !RCC.cr().read().pllrdy() {}
            RCC.pllcfgr().modify(|w| w.set_pllren(true));

            (freq, Sw::PLLRCLK)
        }
    };

    RCC.cfgr().modify(|w| {
        w.set_sw(sw);
        w.set_hpre(config.ahb_pre.into());
        w.set_ppre1(config.apb1_pre.into());
        w.set_ppre2(config.apb2_pre.into());
    });

    let ahb_freq: u32 = match config.ahb_pre {
        AHBPrescaler::NotDivided => sys_clk,
        pre => sys_clk / pre.div(),
    };

    let (apb1_freq, apb1_tim_freq) = match config.apb1_pre {
        APBPrescaler::NotDivided => (ahb_freq, ahb_freq),
        pre => {
            let freq = ahb_freq / pre.div();
            (freq, freq * 2)
        }
    };

    let (apb2_freq, apb2_tim_freq) = match config.apb2_pre {
        APBPrescaler::NotDivided => (ahb_freq, ahb_freq),
        pre => {
            let freq = ahb_freq / pre.div();
            (freq, freq * 2)
        }
    };

    if config.low_power_run {
        assert!(sys_clk <= 2_000_000);
        PWR.cr1().modify(|w| w.set_lpr(true));
    }

    set_freqs(Clocks {
        sys: Hertz(sys_clk),
        ahb1: Hertz(ahb_freq),
        ahb2: Hertz(ahb_freq),
        apb1: Hertz(apb1_freq),
        apb1_tim: Hertz(apb1_tim_freq),
        apb2: Hertz(apb2_freq),
        apb2_tim: Hertz(apb2_tim_freq),
    });
}
