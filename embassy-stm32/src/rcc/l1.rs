use crate::pac;
use crate::peripherals::{self, RCC};
use crate::rcc::{get_freqs, set_freqs, Clocks};
use crate::time::Hertz;
use crate::time::U32Ext;
use core::marker::PhantomData;
use embassy::util::Unborrow;
use embassy_hal_common::unborrow;

/// Most of clock setup is copied from rcc/l0

/// HSI speed
pub const HSI_FREQ: u32 = 16_000_000;

/// System clock mux source
#[derive(Clone, Copy)]
pub enum ClockSrc {
    MSI(MSIRange),
    HSE(Hertz),
    HSI,
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

type Ppre = u8;
impl Into<Ppre> for APBPrescaler {
    fn into(self) -> Ppre {
        match self {
            APBPrescaler::NotDivided => 0b000,
            APBPrescaler::Div2 => 0b100,
            APBPrescaler::Div4 => 0b101,
            APBPrescaler::Div8 => 0b110,
            APBPrescaler::Div16 => 0b111,
        }
    }
}

type Hpre = u8;
impl Into<Hpre> for AHBPrescaler {
    fn into(self) -> Hpre {
        match self {
            AHBPrescaler::NotDivided => 0b0000,
            AHBPrescaler::Div2 => 0b1000,
            AHBPrescaler::Div4 => 0b1001,
            AHBPrescaler::Div8 => 0b1010,
            AHBPrescaler::Div16 => 0b1011,
            AHBPrescaler::Div64 => 0b1100,
            AHBPrescaler::Div128 => 0b1101,
            AHBPrescaler::Div256 => 0b1110,
            AHBPrescaler::Div512 => 0b1111,
        }
    }
}

impl Into<u8> for MSIRange {
    fn into(self) -> u8 {
        match self {
            MSIRange::Range0 => 0b000,
            MSIRange::Range1 => 0b001,
            MSIRange::Range2 => 0b010,
            MSIRange::Range3 => 0b011,
            MSIRange::Range4 => 0b100,
            MSIRange::Range5 => 0b101,
            MSIRange::Range6 => 0b110,
        }
    }
}

/// Clocks configutation
pub struct Config {
    pub mux: ClockSrc,
    pub ahb_pre: AHBPrescaler,
    pub apb1_pre: APBPrescaler,
    pub apb2_pre: APBPrescaler,
}

impl Default for Config {
    #[inline]
    fn default() -> Config {
        Config {
            mux: ClockSrc::MSI(MSIRange::default()),
            ahb_pre: AHBPrescaler::NotDivided,
            apb1_pre: APBPrescaler::NotDivided,
            apb2_pre: APBPrescaler::NotDivided,
        }
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
    // `cfgr` is almost always a constant, so make sure it can be constant-propagated properly by
    // marking this function and all `Config` constructors and setters as `#[inline]`.
    // This saves ~900 Bytes for the `pwr.rs` example.
    #[inline]
    fn freeze(self, cfgr: Config) -> Clocks {
        let rcc = pac::RCC;
        let (sys_clk, sw) = match cfgr.mux {
            ClockSrc::MSI(range) => {
                // Set MSI range
                unsafe {
                    rcc.icscr().write(|w| w.set_msirange(range.into()));
                }

                // Enable MSI
                unsafe {
                    rcc.cr().write(|w| w.set_msion(true));
                    while !rcc.cr().read().msirdy() {}
                }

                let freq = 32_768 * (1 << (range as u8 + 1));
                (freq, 0b00)
            }
            ClockSrc::HSI => {
                // Enable HSI
                unsafe {
                    rcc.cr().write(|w| w.set_hsion(true));
                    while !rcc.cr().read().hsirdy() {}
                }

                (HSI_FREQ, 0b01)
            }
            ClockSrc::HSE(freq) => {
                // Enable HSE
                unsafe {
                    rcc.cr().write(|w| w.set_hseon(true));
                    while !rcc.cr().read().hserdy() {}
                }

                (freq.0, 0b10)
            }
        };

        unsafe {
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
                let pre: Hpre = pre.into();
                let pre = 1 << (pre as u32 - 7);
                sys_clk / pre
            }
        };

        let (apb1_freq, apb1_tim_freq) = match cfgr.apb1_pre {
            APBPrescaler::NotDivided => (ahb_freq, ahb_freq),
            pre => {
                let pre: Ppre = pre.into();
                let pre: u8 = 1 << (pre - 3);
                let freq = ahb_freq / pre as u32;
                (freq, freq * 2)
            }
        };

        let (apb2_freq, apb2_tim_freq) = match cfgr.apb2_pre {
            APBPrescaler::NotDivided => (ahb_freq, ahb_freq),
            pre => {
                let pre: Ppre = pre.into();
                let pre: u8 = 1 << (pre - 3);
                let freq = ahb_freq / (1 << (pre as u8 - 3));
                (freq, freq * 2)
            }
        };

        Clocks {
            sys: sys_clk.hz(),
            ahb: ahb_freq.hz(),
            apb1: apb1_freq.hz(),
            apb2: apb2_freq.hz(),
            apb1_tim: apb1_tim_freq.hz(),
            apb2_tim: apb2_tim_freq.hz(),
        }
    }
}

pub(crate) unsafe fn init(config: Config) {
    let r = <peripherals::RCC as embassy::util::Steal>::steal();
    let clocks = r.freeze(config);
    set_freqs(clocks);
}
