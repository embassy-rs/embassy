use crate::pac;
use crate::peripherals::{self, RCC};
use crate::rcc::{get_freqs, set_freqs, Clocks};
use crate::time::Hertz;
use crate::time::U32Ext;
use core::marker::PhantomData;
use embassy::util::Unborrow;
use embassy_hal_common::unborrow;
use stm32_metapac::rcc::vals::Msirange;

/// Most of clock setup is copied from stm32l0xx-hal, and adopted to the generated PAC,
/// and with the addition of the init function to configure a system clock.

/// Only the basic setup using the HSE and HSI clocks are supported as of now.

/// HSI16 speed
pub const HSI16_FREQ: u32 = 16_000_000;

/// System clock mux source
#[derive(Clone, Copy)]
pub enum ClockSrc {
    PLL(PLLSource, PLLClkDiv, PLLSrcDiv, PLLMul, Option<PLL48Div>),
    MSI(MSIRange),
    HSE(Hertz),
    HSI16,
}

/// MSI Clock Range
///
/// These ranges control the frequency of the MSI. Internally, these ranges map
/// to the `MSIRANGE` bits in the `RCC_ICSCR` register.
#[derive(Clone, Copy)]
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

impl Into<u32> for MSIRange {
    fn into(self) -> u32 {
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
}

impl Default for MSIRange {
    fn default() -> MSIRange {
        MSIRange::Range6
    }
}

pub type PLL48Div = PLLClkDiv;

/// PLL divider
#[derive(Clone, Copy)]
pub enum PLLDiv {
    Div2,
    Div3,
    Div4,
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

seq_macro::seq!(N in 8..=86 {
    #[derive(Clone, Copy)]
    pub enum PLLMul {
        #(
            Mul#N,
        )*
    }

    impl Into<u8> for PLLMul {
        fn into(self) -> u8 {
            match self {
                #(
                    PLLMul::Mul#N => N,
                )*
            }
        }
    }

    impl PLLMul {
        pub fn to_mul(self) -> u32 {
            match self {
                #(
                    PLLMul::Mul#N => N,
                )*
            }
        }
    }
});

#[derive(Clone, Copy)]
pub enum PLLClkDiv {
    Div2,
    Div4,
    Div6,
    Div8,
}

impl PLLClkDiv {
    pub fn to_div(self) -> u32 {
        let val: u8 = self.into();
        val as u32 + 1 * 2
    }
}

impl Into<u8> for PLLClkDiv {
    fn into(self) -> u8 {
        match self {
            PLLClkDiv::Div2 => 0b00,
            PLLClkDiv::Div4 => 0b01,
            PLLClkDiv::Div6 => 0b10,
            PLLClkDiv::Div8 => 0b11,
        }
    }
}

#[derive(Clone, Copy)]
pub enum PLLSrcDiv {
    Div1,
    Div2,
    Div3,
    Div4,
    Div5,
    Div6,
    Div7,
    Div8,
}

impl PLLSrcDiv {
    pub fn to_div(self) -> u32 {
        let val: u8 = self.into();
        val as u32 + 1
    }
}

impl Into<u8> for PLLSrcDiv {
    fn into(self) -> u8 {
        match self {
            PLLSrcDiv::Div1 => 0b000,
            PLLSrcDiv::Div2 => 0b001,
            PLLSrcDiv::Div3 => 0b010,
            PLLSrcDiv::Div4 => 0b011,
            PLLSrcDiv::Div5 => 0b100,
            PLLSrcDiv::Div6 => 0b101,
            PLLSrcDiv::Div7 => 0b110,
            PLLSrcDiv::Div8 => 0b111,
        }
    }
}

impl Into<u8> for PLLSource {
    fn into(self) -> u8 {
        match self {
            PLLSource::HSI16 => 0b10,
            PLLSource::HSE(_) => 0b11,
        }
    }
}

impl Into<Msirange> for MSIRange {
    fn into(self) -> Msirange {
        match self {
            MSIRange::Range0 => Msirange::RANGE100K,
            MSIRange::Range1 => Msirange::RANGE200K,
            MSIRange::Range2 => Msirange::RANGE400K,
            MSIRange::Range3 => Msirange::RANGE800K,
            MSIRange::Range4 => Msirange::RANGE1M,
            MSIRange::Range5 => Msirange::RANGE2M,
            MSIRange::Range6 => Msirange::RANGE4M,
            MSIRange::Range7 => Msirange::RANGE8M,
            MSIRange::Range8 => Msirange::RANGE16M,
            MSIRange::Range9 => Msirange::RANGE24M,
            MSIRange::Range10 => Msirange::RANGE32M,
            MSIRange::Range11 => Msirange::RANGE48M,
        }
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
    mux: ClockSrc,
    ahb_pre: AHBPrescaler,
    apb1_pre: APBPrescaler,
    apb2_pre: APBPrescaler,
}

impl Default for Config {
    #[inline]
    fn default() -> Config {
        Config {
            mux: ClockSrc::MSI(MSIRange::Range6),
            ahb_pre: AHBPrescaler::NotDivided,
            apb1_pre: APBPrescaler::NotDivided,
            apb2_pre: APBPrescaler::NotDivided,
        }
    }
}

impl Config {
    #[inline]
    pub fn clock_src(mut self, mux: ClockSrc) -> Self {
        self.mux = mux;
        self
    }

    #[inline]
    pub fn ahb_pre(mut self, pre: AHBPrescaler) -> Self {
        self.ahb_pre = pre;
        self
    }

    #[inline]
    pub fn apb1_pre(mut self, pre: APBPrescaler) -> Self {
        self.apb1_pre = pre;
        self
    }

    #[inline]
    pub fn apb2_pre(mut self, pre: APBPrescaler) -> Self {
        self.apb2_pre = pre;
        self
    }
}

/// RCC peripheral
pub struct Rcc<'d> {
    _rb: peripherals::RCC,
    phantom: PhantomData<&'d mut peripherals::RCC>,
}

impl<'d> Rcc<'d> {
    pub fn new(rcc: impl Unborrow<Target = peripherals::RCC> + 'd) -> Self {
        unborrow!(rcc);
        Self {
            _rb: rcc,
            phantom: PhantomData,
        }
    }

    // Safety: RCC init must have been called
    pub fn clocks(&self) -> &'static Clocks {
        unsafe { get_freqs() }
    }
}

/// Extension trait that freezes the `RCC` peripheral with provided clocks configuration
pub trait RccExt {
    fn freeze(self, config: Config) -> Clocks;
}

impl RccExt for RCC {
    #[inline]
    fn freeze(self, cfgr: Config) -> Clocks {
        let rcc = pac::RCC;
        let (sys_clk, sw) = match cfgr.mux {
            ClockSrc::HSI16 => {
                // Enable HSI16
                unsafe {
                    rcc.cr().write(|w| w.set_hsion(true));
                    while !rcc.cr().read().hsirdy() {}
                }

                (HSI16_FREQ, 0b01)
            }
            ClockSrc::HSE(freq) => {
                // Enable HSE
                unsafe {
                    rcc.cr().write(|w| w.set_hseon(true));
                    while !rcc.cr().read().hserdy() {}
                }

                (freq.0, 0b10)
            }
            ClockSrc::MSI(range) => {
                // Enable MSI
                unsafe {
                    rcc.cr().write(|w| {
                        let bits: Msirange = range.into();
                        w.set_msirange(bits);
                        w.set_msipllen(false);
                        w.set_msirgsel(true);
                        w.set_msion(true);
                    });
                    while !rcc.cr().read().msirdy() {}

                    // Enable as clock source for USB, RNG if running at 48 MHz
                    if let MSIRange::Range11 = range {
                        rcc.ccipr().modify(|w| {
                            w.set_clk48sel(0b11);
                        });
                    }
                }
                (range.into(), 0b00)
            }
            ClockSrc::PLL(src, div, prediv, mul, pll48div) => {
                let freq = match src {
                    PLLSource::HSE(freq) => {
                        // Enable HSE
                        unsafe {
                            rcc.cr().write(|w| w.set_hseon(true));
                            while !rcc.cr().read().hserdy() {}
                        }
                        freq.0
                    }
                    PLLSource::HSI16 => {
                        // Enable HSI
                        unsafe {
                            rcc.cr().write(|w| w.set_hsion(true));
                            while !rcc.cr().read().hsirdy() {}
                        }
                        HSI16_FREQ
                    }
                };

                // Disable PLL
                unsafe {
                    rcc.cr().modify(|w| w.set_pllon(false));
                    while rcc.cr().read().pllrdy() {}
                }

                let freq = (freq / prediv.to_div() * mul.to_mul()) / div.to_div();

                assert!(freq <= 80_000_000);

                unsafe {
                    rcc.pllcfgr().write(move |w| {
                        w.set_plln(mul.into());
                        w.set_pllm(prediv.into());
                        w.set_pllr(div.into());
                        if let Some(pll48div) = pll48div {
                            w.set_pllq(pll48div.into());
                            w.set_pllqen(true);
                        }
                        w.set_pllsrc(src.into());
                    });

                    // Enable as clock source for USB, RNG if PLL48 divisor is provided
                    if pll48div.is_some() {
                        rcc.ccipr().modify(|w| {
                            w.set_clk48sel(0b10);
                        });
                    }

                    // Enable PLL
                    rcc.cr().modify(|w| w.set_pllon(true));
                    while !rcc.cr().read().pllrdy() {}
                    rcc.pllcfgr().modify(|w| w.set_pllren(true));
                }
                (freq, 0b11)
            }
        };

        unsafe {
            // Set flash wait states
            pac::FLASH.acr().modify(|w| {
                w.set_latency(if sys_clk <= 16_000_000 {
                    0b000
                } else if sys_clk <= 32_000_000 {
                    0b001
                } else if sys_clk <= 48_000_000 {
                    0b010
                } else if sys_clk <= 64_000_000 {
                    0b011
                } else {
                    0b100
                });
            });

            // Switch active clocks to new clock source
            rcc.cfgr().modify(|w| {
                w.set_sw(sw.into());
                w.set_hpre(cfgr.ahb_pre.into());
                w.set_ppre1(cfgr.apb1_pre.into());
                w.set_ppre2(cfgr.apb2_pre.into());
            });
        }

        let ahb_freq: u32 = match cfgr.ahb_pre {
            AHBPrescaler::NotDivided => sys_clk,
            pre => {
                let pre: u8 = pre.into();
                let pre = 1 << (pre as u32 - 7);
                sys_clk / pre
            }
        };

        let (apb1_freq, apb1_tim_freq) = match cfgr.apb1_pre {
            APBPrescaler::NotDivided => (ahb_freq, ahb_freq),
            pre => {
                let pre: u8 = pre.into();
                let pre: u8 = 1 << (pre - 3);
                let freq = ahb_freq / pre as u32;
                (freq, freq * 2)
            }
        };

        let (apb2_freq, apb2_tim_freq) = match cfgr.apb2_pre {
            APBPrescaler::NotDivided => (ahb_freq, ahb_freq),
            pre => {
                let pre: u8 = pre.into();
                let pre: u8 = 1 << (pre - 3);
                let freq = ahb_freq / (1 << (pre as u8 - 3));
                (freq, freq * 2)
            }
        };

        Clocks {
            sys: sys_clk.hz(),
            ahb1: ahb_freq.hz(),
            ahb2: ahb_freq.hz(),
            ahb3: ahb_freq.hz(),
            apb1: apb1_freq.hz(),
            apb2: apb2_freq.hz(),
            apb1_tim: apb1_tim_freq.hz(),
            apb2_tim: apb2_tim_freq.hz(),
        }
    }
}

pub unsafe fn init(config: Config) {
    let r = <peripherals::RCC as embassy::util::Steal>::steal();
    let clocks = r.freeze(config);
    set_freqs(clocks);
}
