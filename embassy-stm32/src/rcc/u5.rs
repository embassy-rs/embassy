use stm32_metapac::rcc::vals::{Hpre, Msirange, Msirgsel, Pllm, Pllsrc, Ppre, Sw};

use crate::pac::{FLASH, RCC};
use crate::rcc::{set_freqs, Clocks};
use crate::time::{Hertz, U32Ext};

/// HSI16 speed
pub const HSI16_FREQ: u32 = 16_000_000;

/// Voltage Scale
///
/// Represents the voltage range feeding the CPU core. The maximum core
/// clock frequency depends on this value.
#[derive(Copy, Clone, PartialEq)]
pub enum VoltageScale {
    // Highest frequency
    Range1,
    Range2,
    Range3,
    // Lowest power
    Range4,
}

#[derive(Copy, Clone)]
pub enum ClockSrc {
    MSI(MSIRange),
    HSE(Hertz),
    HSI16,
    PLL1R(PllSrc, PllM, PllN, PllClkDiv),
}

#[derive(Clone, Copy, Debug)]
pub enum PllSrc {
    MSI(MSIRange),
    HSE(Hertz),
    HSI16,
}

impl Into<Pllsrc> for PllSrc {
    fn into(self) -> Pllsrc {
        match self {
            PllSrc::MSI(..) => Pllsrc::MSIS,
            PllSrc::HSE(..) => Pllsrc::HSE,
            PllSrc::HSI16 => Pllsrc::HSI16,
        }
    }
}

seq_macro::seq!(N in 2..=128 {
    #[derive(Copy, Clone, Debug)]
    pub enum PllClkDiv {
        NotDivided,
        #(
            Div~N = (N-1),
        )*
    }

    impl PllClkDiv {
        fn to_div(&self) -> u8 {
            match self {
                PllClkDiv::NotDivided => 1,
                #(
                    PllClkDiv::Div~N => (N + 1),
                )*
            }
        }
    }
});

impl Into<u8> for PllClkDiv {
    fn into(self) -> u8 {
        (self as u8) + 1
    }
}

seq_macro::seq!(N in 4..=512 {
    #[derive(Copy, Clone, Debug)]
    pub enum PllN {
        NotMultiplied,
        #(
            Mul~N = (N-1),
        )*
    }

    impl PllN {
        fn to_mul(&self) -> u16 {
            match self {
                PllN::NotMultiplied => 1,
                #(
                    PllN::Mul~N => (N + 1),
                )*
            }
        }
    }
});

impl Into<u16> for PllN {
    fn into(self) -> u16 {
        (self as u16) + 1
    }
}

// Pre-division
#[derive(Copy, Clone, Debug)]
pub enum PllM {
    NotDivided = 0b0000,
    Div2 = 0b0001,
    Div3 = 0b0010,
    Div4 = 0b0011,
    Div5 = 0b0100,
    Div6 = 0b0101,
    Div7 = 0b0110,
    Div8 = 0b0111,
    Div9 = 0b1000,
    Div10 = 0b1001,
    Div11 = 0b1010,
    Div12 = 0b1011,
    Div13 = 0b1100,
    Div14 = 0b1101,
    Div15 = 0b1110,
    Div16 = 0b1111,
}

impl Into<Pllm> for PllM {
    fn into(self) -> Pllm {
        Pllm(self as u8)
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

impl Into<Hpre> for AHBPrescaler {
    fn into(self) -> Hpre {
        match self {
            AHBPrescaler::NotDivided => Hpre::NONE,
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

impl Default for AHBPrescaler {
    fn default() -> Self {
        AHBPrescaler::NotDivided
    }
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
            APBPrescaler::NotDivided => Ppre::NONE,
            APBPrescaler::Div2 => Ppre::DIV2,
            APBPrescaler::Div4 => Ppre::DIV4,
            APBPrescaler::Div8 => Ppre::DIV8,
            APBPrescaler::Div16 => Ppre::DIV16,
        }
    }
}

impl Default for APBPrescaler {
    fn default() -> Self {
        APBPrescaler::NotDivided
    }
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

impl Into<Sw> for ClockSrc {
    fn into(self) -> Sw {
        match self {
            ClockSrc::MSI(..) => Sw::MSIS,
            ClockSrc::HSE(..) => Sw::HSE,
            ClockSrc::HSI16 => Sw::HSI16,
            ClockSrc::PLL1R(..) => Sw::PLL1R,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum MSIRange {
    Range48mhz = 48_000_000,
    Range24mhz = 24_000_000,
    Range16mhz = 16_000_000,
    Range12mhz = 12_000_000,
    Range4mhz = 4_000_000,
    Range2mhz = 2_000_000,
    Range1_33mhz = 1_330_000,
    Range1mhz = 1_000_000,
    Range3_072mhz = 3_072_000,
    Range1_536mhz = 1_536_000,
    Range1_024mhz = 1_024_000,
    Range768khz = 768_000,
    Range400khz = 400_000,
    Range200khz = 200_000,
    Range133khz = 133_000,
    Range100khz = 100_000,
}

impl Into<u32> for MSIRange {
    fn into(self) -> u32 {
        self as u32
    }
}

impl Into<Msirange> for MSIRange {
    fn into(self) -> Msirange {
        match self {
            MSIRange::Range48mhz => Msirange::RANGE_48MHZ,
            MSIRange::Range24mhz => Msirange::RANGE_24MHZ,
            MSIRange::Range16mhz => Msirange::RANGE_16MHZ,
            MSIRange::Range12mhz => Msirange::RANGE_12MHZ,
            MSIRange::Range4mhz => Msirange::RANGE_4MHZ,
            MSIRange::Range2mhz => Msirange::RANGE_2MHZ,
            MSIRange::Range1_33mhz => Msirange::RANGE_1_33MHZ,
            MSIRange::Range1mhz => Msirange::RANGE_1MHZ,
            MSIRange::Range3_072mhz => Msirange::RANGE_3_072MHZ,
            MSIRange::Range1_536mhz => Msirange::RANGE_1_536MHZ,
            MSIRange::Range1_024mhz => Msirange::RANGE_1_024MHZ,
            MSIRange::Range768khz => Msirange::RANGE_768KHZ,
            MSIRange::Range400khz => Msirange::RANGE_400KHZ,
            MSIRange::Range200khz => Msirange::RANGE_200KHZ,
            MSIRange::Range133khz => Msirange::RANGE_133KHZ,
            MSIRange::Range100khz => Msirange::RANGE_100KHZ,
        }
    }
}

impl Default for MSIRange {
    fn default() -> Self {
        MSIRange::Range4mhz
    }
}

#[derive(Copy, Clone)]
pub struct Config {
    pub mux: ClockSrc,
    pub ahb_pre: AHBPrescaler,
    pub apb1_pre: APBPrescaler,
    pub apb2_pre: APBPrescaler,
    pub apb3_pre: APBPrescaler,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mux: ClockSrc::MSI(MSIRange::default()),
            ahb_pre: Default::default(),
            apb1_pre: Default::default(),
            apb2_pre: Default::default(),
            apb3_pre: Default::default(),
        }
    }
}

pub(crate) unsafe fn init(config: Config) {
    let sys_clk = match config.mux {
        ClockSrc::MSI(range) => {
            RCC.icscr1().modify(|w| {
                let bits: Msirange = range.into();
                w.set_msisrange(bits);
                w.set_msirgsel(Msirgsel::RCC_ICSCR1);
            });
            RCC.cr().write(|w| {
                w.set_msipllen(false);
                w.set_msison(true);
                w.set_msison(true);
            });
            while !RCC.cr().read().msisrdy() {}

            range.into()
        }
        ClockSrc::HSE(freq) => {
            RCC.cr().write(|w| w.set_hseon(true));
            while !RCC.cr().read().hserdy() {}

            freq.0
        }
        ClockSrc::HSI16 => {
            RCC.cr().write(|w| w.set_hsion(true));
            while !RCC.cr().read().hsirdy() {}

            HSI16_FREQ
        }
        ClockSrc::PLL1R(src, m, n, div) => {
            let freq = match src {
                PllSrc::MSI(_) => MSIRange::default().into(),
                PllSrc::HSE(hertz) => hertz.0,
                PllSrc::HSI16 => HSI16_FREQ,
            };

            // disable
            RCC.cr().modify(|w| w.set_pllon(0, false));
            while RCC.cr().read().pllrdy(0) {}

            let vco = freq * n as u8 as u32;
            let pll_ck = vco / (div as u8 as u32 + 1);

            RCC.pll1cfgr().write(|w| {
                w.set_pllm(m.into());
                w.set_pllsrc(src.into());
            });

            RCC.pll1divr().modify(|w| {
                w.set_pllr(div.to_div());
                w.set_plln(n.to_mul());
            });

            // Enable PLL
            RCC.cr().modify(|w| w.set_pllon(0, true));
            while !RCC.cr().read().pllrdy(0) {}
            RCC.pll1cfgr().modify(|w| w.set_pllren(true));

            RCC.cr().write(|w| w.set_pllon(0, true));
            while !RCC.cr().read().pllrdy(0) {}

            pll_ck
        }
    };

    // TODO make configurable
    let power_vos = VoltageScale::Range4;

    // states and programming delay
    let wait_states = match power_vos {
        // VOS 0 range VCORE 1.26V - 1.40V
        VoltageScale::Range1 => {
            if sys_clk < 32_000_000 {
                0
            } else if sys_clk < 64_000_000 {
                1
            } else if sys_clk < 96_000_000 {
                2
            } else if sys_clk < 128_000_000 {
                3
            } else {
                4
            }
        }
        // VOS 1 range VCORE 1.15V - 1.26V
        VoltageScale::Range2 => {
            if sys_clk < 30_000_000 {
                0
            } else if sys_clk < 60_000_000 {
                1
            } else if sys_clk < 90_000_000 {
                2
            } else {
                3
            }
        }
        // VOS 2 range VCORE 1.05V - 1.15V
        VoltageScale::Range3 => {
            if sys_clk < 24_000_000 {
                0
            } else if sys_clk < 48_000_000 {
                1
            } else {
                2
            }
        }
        // VOS 3 range VCORE 0.95V - 1.05V
        VoltageScale::Range4 => {
            if sys_clk < 12_000_000 {
                0
            } else {
                1
            }
        }
    };

    FLASH.acr().modify(|w| {
        w.set_latency(wait_states);
    });

    RCC.cfgr1().modify(|w| {
        w.set_sw(config.mux.into());
    });

    RCC.cfgr2().modify(|w| {
        w.set_hpre(config.ahb_pre.into());
        w.set_ppre1(config.apb1_pre.into());
        w.set_ppre2(config.apb2_pre.into());
    });

    RCC.cfgr3().modify(|w| {
        w.set_ppre3(config.apb3_pre.into());
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

    let (apb3_freq, _apb3_tim_freq) = match config.apb3_pre {
        APBPrescaler::NotDivided => (ahb_freq, ahb_freq),
        pre => {
            let pre: u8 = pre.into();
            let pre: u8 = 1 << (pre - 3);
            let freq = ahb_freq / (1 << (pre as u8 - 3));
            (freq, freq * 2)
        }
    };

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
