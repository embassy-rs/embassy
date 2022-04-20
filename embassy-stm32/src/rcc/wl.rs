use crate::pac::{FLASH, RCC};
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
    MSI(MSIRange),
    HSE32,
    HSI16,
}

#[derive(Clone, Copy, PartialOrd, PartialEq)]
pub enum MSIRange {
    /// Around 100 kHz
    Range0,
    /// Around 200 kHz
    Range1,
    /// Around 400 kHz
    Range2,
    /// Around 800 kHz
    Range3,
    /// Around 1 MHz
    Range4,
    /// Around 2 MHz
    Range5,
    /// Around 4 MHz (reset value)
    Range6,
    /// Around 8 MHz
    Range7,
    /// Around 16 MHz
    Range8,
    /// Around 24 MHz
    Range9,
    /// Around 32 MHz
    Range10,
    /// Around 48 MHz
    Range11,
}

impl MSIRange {
    fn freq(&self) -> u32 {
        match self {
            MSIRange::Range0 => 100_000,
            MSIRange::Range1 => 200_000,
            MSIRange::Range2 => 400_000,
            MSIRange::Range3 => 800_000,
            MSIRange::Range4 => 1_000_000,
            MSIRange::Range5 => 2_000_000,
            MSIRange::Range6 => 4_000_000,
            MSIRange::Range7 => 8_000_000,
            MSIRange::Range8 => 16_000_000,
            MSIRange::Range9 => 24_000_000,
            MSIRange::Range10 => 32_000_000,
            MSIRange::Range11 => 48_000_000,
        }
    }

    fn vos(&self) -> VoltageScale {
        if self > &MSIRange::Range8 {
            VoltageScale::Range1
        } else {
            VoltageScale::Range2
        }
    }
}

impl Default for MSIRange {
    fn default() -> MSIRange {
        MSIRange::Range6
    }
}

impl Into<u8> for MSIRange {
    fn into(self) -> u8 {
        match self {
            MSIRange::Range0 => 0b0000,
            MSIRange::Range1 => 0b0001,
            MSIRange::Range2 => 0b0010,
            MSIRange::Range3 => 0b0011,
            MSIRange::Range4 => 0b0100,
            MSIRange::Range5 => 0b0101,
            MSIRange::Range6 => 0b0110,
            MSIRange::Range7 => 0b0111,
            MSIRange::Range8 => 0b1000,
            MSIRange::Range9 => 0b1001,
            MSIRange::Range10 => 0b1010,
            MSIRange::Range11 => 0b1011,
        }
    }
}

/// Voltage Scale
///
/// Represents the voltage range feeding the CPU core. The maximum core
/// clock frequency depends on this value.
#[derive(Copy, Clone, PartialEq)]
pub enum VoltageScale {
    Range1,
    Range2,
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
    pub shd_ahb_pre: AHBPrescaler,
    pub apb1_pre: APBPrescaler,
    pub apb2_pre: APBPrescaler,
    pub enable_lsi: bool,
}

impl Default for Config {
    #[inline]
    fn default() -> Config {
        Config {
            mux: ClockSrc::MSI(MSIRange::default()),
            ahb_pre: AHBPrescaler::NotDivided,
            shd_ahb_pre: AHBPrescaler::NotDivided,
            apb1_pre: APBPrescaler::NotDivided,
            apb2_pre: APBPrescaler::NotDivided,
            enable_lsi: false,
        }
    }
}

pub(crate) unsafe fn init(config: Config) {
    let (sys_clk, sw, vos) = match config.mux {
        ClockSrc::HSI16 => {
            // Enable HSI16
            RCC.cr().write(|w| w.set_hsion(true));
            while !RCC.cr().read().hsirdy() {}

            (HSI_FREQ, 0x01, VoltageScale::Range2)
        }
        ClockSrc::HSE32 => {
            // Enable HSE32
            RCC.cr().write(|w| {
                w.set_hsebyppwr(true);
                w.set_hseon(true);
            });
            while !RCC.cr().read().hserdy() {}

            (HSE32_FREQ, 0x02, VoltageScale::Range1)
        }
        ClockSrc::MSI(range) => {
            RCC.cr().write(|w| {
                w.set_msirange(range.into());
                w.set_msion(true);
            });

            while !RCC.cr().read().msirdy() {}

            (range.freq(), 0x00, range.vos())
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

    RCC.extcfgr().modify(|w| {
        if config.shd_ahb_pre == AHBPrescaler::NotDivided {
            w.set_shdhpre(0);
        } else {
            w.set_shdhpre(config.shd_ahb_pre.into());
        }
    });

    let ahb_freq: u32 = match config.ahb_pre {
        AHBPrescaler::NotDivided => sys_clk,
        pre => {
            let pre: u8 = pre.into();
            let pre = 1 << (pre as u32 - 7);
            sys_clk / pre
        }
    };

    let shd_ahb_freq: u32 = match config.shd_ahb_pre {
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

    let apb3_freq = shd_ahb_freq;

    if config.enable_lsi {
        let csr = RCC.csr().read();
        if !csr.lsion() {
            RCC.csr().modify(|w| w.set_lsion(true));
            while !RCC.csr().read().lsirdy() {}
        }
    }

    // Adjust flash latency
    let flash_clk_src_freq: u32 = shd_ahb_freq;
    let ws = match vos {
        VoltageScale::Range1 => match flash_clk_src_freq {
            0..=18_000_000 => 0b000,
            18_000_001..=36_000_000 => 0b001,
            _ => 0b010,
        },
        VoltageScale::Range2 => match flash_clk_src_freq {
            0..=6_000_000 => 0b000,
            6_000_001..=12_000_000 => 0b001,
            _ => 0b010,
        },
    };

    FLASH.acr().modify(|w| {
        w.set_latency(ws);
    });

    while FLASH.acr().read().latency() != ws {}

    set_freqs(Clocks {
        sys: sys_clk.hz(),
        ahb1: ahb_freq.hz(),
        ahb2: ahb_freq.hz(),
        ahb3: shd_ahb_freq.hz(),
        apb1: apb1_freq.hz(),
        apb2: apb2_freq.hz(),
        apb3: apb3_freq.hz(),
        apb1_tim: apb1_tim_freq.hz(),
        apb2_tim: apb2_tim_freq.hz(),
    });
}
