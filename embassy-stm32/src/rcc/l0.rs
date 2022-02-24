use crate::pac::rcc::vals::{Hpre, Msirange, Plldiv, Pllmul, Pllsrc, Ppre, Sw};
use crate::pac::RCC;
#[cfg(crs)]
use crate::pac::{CRS, SYSCFG};
use crate::rcc::{set_freqs, Clocks};
use crate::time::Hertz;
use crate::time::U32Ext;

/// HSI16 speed
pub const HSI16_FREQ: u32 = 16_000_000;

/// System clock mux source
#[derive(Clone, Copy)]
pub enum ClockSrc {
    MSI(MSIRange),
    PLL(PLLSource, PLLMul, PLLDiv),
    HSE(Hertz),
    HSI16,
}

/// MSI Clock Range
///
/// These ranges control the frequency of the MSI. Internally, these ranges map
/// to the `MSIRANGE` bits in the `RCC_ICSCR` register.
#[derive(Clone, Copy)]
pub enum MSIRange {
    /// Around 65.536 kHz
    Range0,
    /// Around 131.072 kHz
    Range1,
    /// Around 262.144 kHz
    Range2,
    /// Around 524.288 kHz
    Range3,
    /// Around 1.048 MHz
    Range4,
    /// Around 2.097 MHz (reset value)
    Range5,
    /// Around 4.194 MHz
    Range6,
}

impl Default for MSIRange {
    fn default() -> MSIRange {
        MSIRange::Range5
    }
}

/// PLL divider
#[derive(Clone, Copy)]
pub enum PLLDiv {
    Div2,
    Div3,
    Div4,
}

/// PLL multiplier
#[derive(Clone, Copy)]
pub enum PLLMul {
    Mul3,
    Mul4,
    Mul6,
    Mul8,
    Mul12,
    Mul16,
    Mul24,
    Mul32,
    Mul48,
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
#[derive(Clone, Copy)]
pub enum PLLSource {
    HSI16,
    HSE(Hertz),
}

impl From<PLLMul> for Pllmul {
    fn from(val: PLLMul) -> Pllmul {
        match val {
            PLLMul::Mul3 => Pllmul::MUL3,
            PLLMul::Mul4 => Pllmul::MUL4,
            PLLMul::Mul6 => Pllmul::MUL6,
            PLLMul::Mul8 => Pllmul::MUL8,
            PLLMul::Mul12 => Pllmul::MUL12,
            PLLMul::Mul16 => Pllmul::MUL16,
            PLLMul::Mul24 => Pllmul::MUL24,
            PLLMul::Mul32 => Pllmul::MUL32,
            PLLMul::Mul48 => Pllmul::MUL48,
        }
    }
}

impl From<PLLDiv> for Plldiv {
    fn from(val: PLLDiv) -> Plldiv {
        match val {
            PLLDiv::Div2 => Plldiv::DIV2,
            PLLDiv::Div3 => Plldiv::DIV3,
            PLLDiv::Div4 => Plldiv::DIV4,
        }
    }
}

impl From<PLLSource> for Pllsrc {
    fn from(val: PLLSource) -> Pllsrc {
        match val {
            PLLSource::HSI16 => Pllsrc::HSI16,
            PLLSource::HSE(_) => Pllsrc::HSE,
        }
    }
}

impl From<APBPrescaler> for Ppre {
    fn from(val: APBPrescaler) -> Ppre {
        match val {
            APBPrescaler::NotDivided => Ppre::DIV1,
            APBPrescaler::Div2 => Ppre::DIV2,
            APBPrescaler::Div4 => Ppre::DIV4,
            APBPrescaler::Div8 => Ppre::DIV8,
            APBPrescaler::Div16 => Ppre::DIV16,
        }
    }
}

impl From<AHBPrescaler> for Hpre {
    fn from(val: AHBPrescaler) -> Hpre {
        match val {
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

impl From<MSIRange> for Msirange {
    fn from(val: MSIRange) -> Msirange {
        match val {
            MSIRange::Range0 => Msirange::RANGE0,
            MSIRange::Range1 => Msirange::RANGE1,
            MSIRange::Range2 => Msirange::RANGE2,
            MSIRange::Range3 => Msirange::RANGE3,
            MSIRange::Range4 => Msirange::RANGE4,
            MSIRange::Range5 => Msirange::RANGE5,
            MSIRange::Range6 => Msirange::RANGE6,
        }
    }
}

/// Clocks configutation
pub struct Config {
    pub mux: ClockSrc,
    pub ahb_pre: AHBPrescaler,
    pub apb1_pre: APBPrescaler,
    pub apb2_pre: APBPrescaler,
    #[cfg(crs)]
    pub enable_hsi48: bool,
}

impl Default for Config {
    #[inline]
    fn default() -> Config {
        Config {
            mux: ClockSrc::MSI(MSIRange::default()),
            ahb_pre: AHBPrescaler::NotDivided,
            apb1_pre: APBPrescaler::NotDivided,
            apb2_pre: APBPrescaler::NotDivided,
            #[cfg(crs)]
            enable_hsi48: false,
        }
    }
}

pub(crate) unsafe fn init(config: Config) {
    let (sys_clk, sw) = match config.mux {
        ClockSrc::MSI(range) => {
            // Set MSI range
            RCC.icscr().write(|w| w.set_msirange(range.into()));

            // Enable MSI
            RCC.cr().write(|w| w.set_msion(true));
            while !RCC.cr().read().msirdy() {}

            let freq = 32_768 * (1 << (range as u8 + 1));
            (freq, Sw::MSI)
        }
        ClockSrc::HSI16 => {
            // Enable HSI16
            RCC.cr().write(|w| w.set_hsi16on(true));
            while !RCC.cr().read().hsi16rdyf() {}

            (HSI16_FREQ, Sw::HSI16)
        }
        ClockSrc::HSE(freq) => {
            // Enable HSE
            RCC.cr().write(|w| w.set_hseon(true));
            while !RCC.cr().read().hserdy() {}

            (freq.0, Sw::HSE)
        }
        ClockSrc::PLL(src, mul, div) => {
            let freq = match src {
                PLLSource::HSE(freq) => {
                    // Enable HSE
                    RCC.cr().write(|w| w.set_hseon(true));
                    while !RCC.cr().read().hserdy() {}
                    freq.0
                }
                PLLSource::HSI16 => {
                    // Enable HSI
                    RCC.cr().write(|w| w.set_hsi16on(true));
                    while !RCC.cr().read().hsi16rdyf() {}
                    HSI16_FREQ
                }
            };

            // Disable PLL
            RCC.cr().modify(|w| w.set_pllon(false));
            while RCC.cr().read().pllrdy() {}

            let freq = match mul {
                PLLMul::Mul3 => freq * 3,
                PLLMul::Mul4 => freq * 4,
                PLLMul::Mul6 => freq * 6,
                PLLMul::Mul8 => freq * 8,
                PLLMul::Mul12 => freq * 12,
                PLLMul::Mul16 => freq * 16,
                PLLMul::Mul24 => freq * 24,
                PLLMul::Mul32 => freq * 32,
                PLLMul::Mul48 => freq * 48,
            };

            let freq = match div {
                PLLDiv::Div2 => freq / 2,
                PLLDiv::Div3 => freq / 3,
                PLLDiv::Div4 => freq / 4,
            };
            assert!(freq <= 32_u32.mhz().0);

            RCC.cfgr().write(move |w| {
                w.set_pllmul(mul.into());
                w.set_plldiv(div.into());
                w.set_pllsrc(src.into());
            });

            // Enable PLL
            RCC.cr().modify(|w| w.set_pllon(true));
            while !RCC.cr().read().pllrdy() {}

            (freq, Sw::PLL)
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
        pre => {
            let pre: Hpre = pre.into();
            let pre = 1 << (pre.0 as u32 - 7);
            sys_clk / pre
        }
    };

    let (apb1_freq, apb1_tim_freq) = match config.apb1_pre {
        APBPrescaler::NotDivided => (ahb_freq, ahb_freq),
        pre => {
            let pre: Ppre = pre.into();
            let pre: u8 = 1 << (pre.0 - 3);
            let freq = ahb_freq / pre as u32;
            (freq, freq * 2)
        }
    };

    let (apb2_freq, apb2_tim_freq) = match config.apb2_pre {
        APBPrescaler::NotDivided => (ahb_freq, ahb_freq),
        pre => {
            let pre: Ppre = pre.into();
            let pre: u8 = 1 << (pre.0 - 3);
            let freq = ahb_freq / (1 << (pre as u8 - 3));
            (freq, freq * 2)
        }
    };

    #[cfg(crs)]
    if config.enable_hsi48 {
        // Reset SYSCFG peripheral
        RCC.apb2rstr().modify(|w| w.set_syscfgrst(true));
        RCC.apb2rstr().modify(|w| w.set_syscfgrst(false));

        // Enable SYSCFG peripheral
        RCC.apb2enr().modify(|w| w.set_syscfgen(true));

        // Reset CRS peripheral
        RCC.apb1rstr().modify(|w| w.set_crsrst(true));
        RCC.apb1rstr().modify(|w| w.set_crsrst(false));

        // Enable CRS peripheral
        RCC.apb1enr().modify(|w| w.set_crsen(true));

        // Initialize CRS
        CRS.cfgr().write(|w|

        // Select LSE as synchronization source
        w.set_syncsrc(0b01));
        CRS.cr().modify(|w| {
            w.set_autotrimen(true);
            w.set_cen(true);
        });

        // Enable VREFINT reference for HSI48 oscillator
        SYSCFG.cfgr3().modify(|w| {
            w.set_enref_hsi48(true);
            w.set_en_vrefint(true);
        });

        // Select HSI48 as USB clock
        RCC.ccipr().modify(|w| w.set_hsi48msel(true));

        // Enable dedicated USB clock
        RCC.crrcr().modify(|w| w.set_hsi48on(true));
        while !RCC.crrcr().read().hsi48rdy() {}
    }

    set_freqs(Clocks {
        sys: sys_clk.hz(),
        ahb1: ahb_freq.hz(),
        apb1: apb1_freq.hz(),
        apb2: apb2_freq.hz(),
        apb1_tim: apb1_tim_freq.hz(),
        apb2_tim: apb2_tim_freq.hz(),
    });
}
