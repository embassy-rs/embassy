pub use super::types::*;
use crate::pac;
use crate::peripherals::{self, CRS, RCC, SYSCFG};
use crate::rcc::{get_freqs, set_freqs, Clocks};
use crate::time::Hertz;
use crate::time::U32Ext;
use core::marker::PhantomData;
use embassy::util::Unborrow;
use embassy_extras::unborrow;
use pac::rcc::vals::{Hpre, Msirange, Plldiv, Pllmul, Pllsrc, Ppre, Sw};

/// Most of clock setup is copied from stm32l0xx-hal, and adopted to the generated PAC,
/// and with the addition of the init function to configure a system clock.

/// HSI speed
pub const HSI_FREQ: u32 = 16_000_000;

/// System clock mux source
#[derive(Clone, Copy)]
pub enum ClockSrc {
    MSI(MSIRange),
    PLL(PLLSource, PLLMul, PLLDiv),
    HSE(Hertz),
    HSI16,
}

impl Into<Pllmul> for PLLMul {
    fn into(self) -> Pllmul {
        match self {
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

impl Into<Plldiv> for PLLDiv {
    fn into(self) -> Plldiv {
        match self {
            PLLDiv::Div2 => Plldiv::DIV2,
            PLLDiv::Div3 => Plldiv::DIV3,
            PLLDiv::Div4 => Plldiv::DIV4,
        }
    }
}

impl Into<Pllsrc> for PLLSource {
    fn into(self) -> Pllsrc {
        match self {
            PLLSource::HSI16 => Pllsrc::HSI16,
            PLLSource::HSE(_) => Pllsrc::HSE,
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

impl Into<Msirange> for MSIRange {
    fn into(self) -> Msirange {
        match self {
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
    mux: ClockSrc,
    ahb_pre: AHBPrescaler,
    apb1_pre: APBPrescaler,
    apb2_pre: APBPrescaler,
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

    pub fn enable_debug_wfe(&mut self, _dbg: &mut peripherals::DBGMCU, enable_dma: bool) {
        // NOTE(unsafe) We have exclusive access to the RCC and DBGMCU
        unsafe {
            pac::RCC.ahbenr().modify(|w| w.set_dmaen(enable_dma));

            pac::DBGMCU.cr().modify(|w| {
                w.set_dbg_sleep(true);
                w.set_dbg_standby(true);
                w.set_dbg_stop(true);
            });
        }
    }

    pub fn enable_hsi48(&mut self, _syscfg: &mut SYSCFG, _crs: CRS) -> HSI48 {
        let rcc = pac::RCC;
        unsafe {
            // Reset SYSCFG peripheral
            rcc.apb2rstr().modify(|w| w.set_syscfgrst(true));
            rcc.apb2rstr().modify(|w| w.set_syscfgrst(false));

            // Enable SYSCFG peripheral
            rcc.apb2enr().modify(|w| w.set_syscfgen(true));

            // Reset CRS peripheral
            rcc.apb1rstr().modify(|w| w.set_crsrst(true));
            rcc.apb1rstr().modify(|w| w.set_crsrst(false));

            // Enable CRS peripheral
            rcc.apb1enr().modify(|w| w.set_crsen(true));

            // Initialize CRS
            let crs = pac::CRS;
            crs.cfgr().write(|w|

            // Select LSE as synchronization source
            w.set_syncsrc(0b01));
            crs.cr().modify(|w| {
                w.set_autotrimen(true);
                w.set_cen(true);
            });

            // Enable VREFINT reference for HSI48 oscillator
            let syscfg = pac::SYSCFG;
            syscfg.cfgr3().modify(|w| {
                w.set_enref_hsi48(true);
                w.set_en_vrefint(true);
            });

            // Select HSI48 as USB clock
            rcc.ccipr().modify(|w| w.set_hsi48msel(true));

            // Enable dedicated USB clock
            rcc.crrcr().modify(|w| w.set_hsi48on(true));
            while !rcc.crrcr().read().hsi48rdy() {}
        }

        HSI48(())
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
                (freq, Sw::MSI)
            }
            ClockSrc::HSI16 => {
                // Enable HSI16
                unsafe {
                    rcc.cr().write(|w| w.set_hsi16on(true));
                    while !rcc.cr().read().hsi16rdyf() {}
                }

                (HSI_FREQ, Sw::HSI16)
            }
            ClockSrc::HSE(freq) => {
                // Enable HSE
                unsafe {
                    rcc.cr().write(|w| w.set_hseon(true));
                    while !rcc.cr().read().hserdy() {}
                }

                (freq.0, Sw::HSE)
            }
            ClockSrc::PLL(src, mul, div) => {
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
                            rcc.cr().write(|w| w.set_hsi16on(true));
                            while !rcc.cr().read().hsi16rdyf() {}
                        }
                        HSI_FREQ
                    }
                };

                // Disable PLL
                unsafe {
                    rcc.cr().modify(|w| w.set_pllon(false));
                    while rcc.cr().read().pllrdy() {}
                }

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

                unsafe {
                    rcc.cfgr().write(move |w| {
                        w.set_pllmul(mul.into());
                        w.set_plldiv(div.into());
                        w.set_pllsrc(src.into());
                    });

                    // Enable PLL
                    rcc.cr().modify(|w| w.set_pllon(true));
                    while !rcc.cr().read().pllrdy() {}
                }

                (freq, Sw::PLL)
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
                let pre = 1 << (pre.0 as u32 - 7);
                sys_clk / pre
            }
        };

        let (apb1_freq, apb1_tim_freq) = match cfgr.apb1_pre {
            APBPrescaler::NotDivided => (ahb_freq, ahb_freq),
            pre => {
                let pre: Ppre = pre.into();
                let pre: u8 = 1 << (pre.0 - 3);
                let freq = ahb_freq / pre as u32;
                (freq, freq * 2)
            }
        };

        let (apb2_freq, apb2_tim_freq) = match cfgr.apb2_pre {
            APBPrescaler::NotDivided => (ahb_freq, ahb_freq),
            pre => {
                let pre: Ppre = pre.into();
                let pre: u8 = 1 << (pre.0 - 3);
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

/// Token that exists only, if the HSI48 clock has been enabled
///
/// You can get an instance of this struct by calling [`Rcc::enable_hsi48`].
#[derive(Clone, Copy)]
pub struct HSI48(());

pub unsafe fn init(config: Config) {
    let rcc = pac::RCC;
    rcc.iopenr().write(|w| {
        w.set_iopaen(true);
        w.set_iopben(true);
        w.set_iopcen(true);
        w.set_iopden(true);
        w.set_iopeen(true);
        w.set_iophen(true);
    });

    let r = <peripherals::RCC as embassy::util::Steal>::steal();
    let clocks = r.freeze(config);
    set_freqs(clocks);
}
