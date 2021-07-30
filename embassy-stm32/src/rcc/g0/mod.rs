pub use super::types::*;
use crate::pac;
use crate::peripherals::{self, RCC};
use crate::rcc::{get_freqs, set_freqs, Clocks};
use crate::time::Hertz;
use crate::time::U32Ext;
use core::marker::PhantomData;
use embassy::util::Unborrow;
use embassy_hal_common::unborrow;

/// HSI speed
pub const HSI_FREQ: u32 = 16_000_000;

/// LSI speed
pub const LSI_FREQ: u32 = 32_000;

/// System clock mux source
#[derive(Clone, Copy)]
pub enum ClockSrc {
    HSE(Hertz),
    HSI16,
    LSI,
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
    apb_pre: APBPrescaler,
}

impl Default for Config {
    #[inline]
    fn default() -> Config {
        Config {
            mux: ClockSrc::HSI16,
            ahb_pre: AHBPrescaler::NotDivided,
            apb_pre: APBPrescaler::NotDivided,
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
    pub fn apb_pre(mut self, pre: APBPrescaler) -> Self {
        self.apb_pre = pre;
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

                (HSI_FREQ, 0x00)
            }
            ClockSrc::HSE(freq) => {
                // Enable HSE
                unsafe {
                    rcc.cr().write(|w| w.set_hseon(true));
                    while !rcc.cr().read().hserdy() {}
                }

                (freq.0, 0x01)
            }
            ClockSrc::LSI => {
                // Enable LSI
                unsafe {
                    rcc.csr().write(|w| w.set_lsion(true));
                    while !rcc.csr().read().lsirdy() {}
                }
                (LSI_FREQ, 0x03)
            }
        };

        unsafe {
            rcc.cfgr().modify(|w| {
                w.set_sw(sw.into());
                w.set_hpre(cfgr.ahb_pre.into());
                w.set_ppre(cfgr.apb_pre.into());
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

        let (apb_freq, apb_tim_freq) = match cfgr.apb_pre {
            APBPrescaler::NotDivided => (ahb_freq, ahb_freq),
            pre => {
                let pre: u8 = pre.into();
                let pre: u8 = 1 << (pre - 3);
                let freq = ahb_freq / pre as u32;
                (freq, freq * 2)
            }
        };

        Clocks {
            sys: sys_clk.hz(),
            ahb: ahb_freq.hz(),
            apb: apb_freq.hz(),
            apb_tim: apb_tim_freq.hz(),
        }
    }
}

pub unsafe fn init(config: Config) {
    let r = <peripherals::RCC as embassy::util::Steal>::steal();
    let clocks = r.freeze(config);
    set_freqs(clocks);
}
