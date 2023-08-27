use core::marker::PhantomData;

use embassy_hal_internal::into_ref;
use stm32_metapac::rcc::regs::Cfgr;
use stm32_metapac::rcc::vals::{Lsedrv, Mcopre, Mcosel};

pub use super::bus::{AHBPrescaler, APBPrescaler};
use crate::gpio::sealed::AFType;
use crate::gpio::Speed;
use crate::pac::rcc::vals::{Hpre, Msirange, Pllsrc, Ppre, Sw};
use crate::pac::{FLASH, PWR, RCC};
use crate::rcc::bd::{BackupDomain, RtcClockSource};
use crate::rcc::{set_freqs, Clocks};
use crate::time::Hertz;
use crate::{peripherals, Peripheral};

/// HSI speed
pub const HSI_FREQ: Hertz = Hertz(16_000_000);

/// LSI speed
pub const LSI_FREQ: Hertz = Hertz(32_000);

/// System clock mux source
#[derive(Clone, Copy)]
pub enum ClockSrc {
    MSI(MSIRange),
    PLL(PLLSource, PLLClkDiv, PLLSrcDiv, PLLMul, Option<PLL48Div>),
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

impl Default for MSIRange {
    fn default() -> MSIRange {
        MSIRange::Range6
    }
}

pub type PLL48Div = PLLClkDiv;
pub type PLLSAI1RDiv = PLLClkDiv;
pub type PLLSAI1QDiv = PLLClkDiv;
pub type PLLSAI1PDiv = PLLClkDiv;

/// PLL divider
#[derive(Clone, Copy)]
pub enum PLLDiv {
    Div2,
    Div3,
    Div4,
}

/// PLL clock input source
#[derive(Clone, Copy)]
pub enum PLLSource {
    HSI16,
    HSE(Hertz),
    MSI(MSIRange),
}

seq_macro::seq!(N in 8..=86 {
    #[derive(Clone, Copy)]
    pub enum PLLMul {
        #(
            Mul~N,
        )*
    }

    impl From<PLLMul> for u8 {
        fn from(val: PLLMul) -> u8 {
            match val {
                #(
                    PLLMul::Mul~N => N,
                )*
            }
        }
    }

    impl PLLMul {
        pub fn to_mul(self) -> u32 {
            match self {
                #(
                    PLLMul::Mul~N => N,
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
        (val as u32 + 1) * 2
    }
}

impl From<PLLClkDiv> for u8 {
    fn from(val: PLLClkDiv) -> u8 {
        match val {
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

impl From<PLLSrcDiv> for u8 {
    fn from(val: PLLSrcDiv) -> u8 {
        match val {
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

impl From<PLLSource> for Pllsrc {
    fn from(val: PLLSource) -> Pllsrc {
        match val {
            PLLSource::HSI16 => Pllsrc::HSI16,
            PLLSource::HSE(_) => Pllsrc::HSE,
            PLLSource::MSI(_) => Pllsrc::MSI,
        }
    }
}

impl From<MSIRange> for Msirange {
    fn from(val: MSIRange) -> Msirange {
        match val {
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

impl From<MSIRange> for u32 {
    fn from(val: MSIRange) -> u32 {
        match val {
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

/// Clocks configutation
pub struct Config {
    pub mux: ClockSrc,
    pub ahb_pre: AHBPrescaler,
    pub apb1_pre: APBPrescaler,
    pub apb2_pre: APBPrescaler,
    pub pllsai1: Option<(
        PLLMul,
        PLLSrcDiv,
        Option<PLLSAI1RDiv>,
        Option<PLLSAI1QDiv>,
        Option<PLLSAI1PDiv>,
    )>,
    #[cfg(not(any(stm32l471, stm32l475, stm32l476, stm32l486)))]
    pub hsi48: bool,
    pub rtc_mux: RtcClockSource,
}

impl Default for Config {
    #[inline]
    fn default() -> Config {
        Config {
            mux: ClockSrc::MSI(MSIRange::Range6),
            ahb_pre: AHBPrescaler::NotDivided,
            apb1_pre: APBPrescaler::NotDivided,
            apb2_pre: APBPrescaler::NotDivided,
            pllsai1: None,
            #[cfg(not(any(stm32l471, stm32l475, stm32l476, stm32l486)))]
            hsi48: false,
            rtc_mux: RtcClockSource::LSI,
        }
    }
}

pub enum McoClock {
    DIV1,
    DIV2,
    DIV4,
    DIV8,
    DIV16,
}

impl McoClock {
    fn into_raw(&self) -> Mcopre {
        match self {
            McoClock::DIV1 => Mcopre::DIV1,
            McoClock::DIV2 => Mcopre::DIV2,
            McoClock::DIV4 => Mcopre::DIV4,
            McoClock::DIV8 => Mcopre::DIV8,
            McoClock::DIV16 => Mcopre::DIV16,
        }
    }
}

#[derive(Copy, Clone)]
pub enum Mco1Source {
    Disabled,
    Lse,
    Lsi,
    Hse,
    Hsi16,
    PllClk,
    SysClk,
    Msi,
    #[cfg(not(any(stm32l471, stm32l475, stm32l476, stm32l486)))]
    Hsi48,
}

impl Default for Mco1Source {
    fn default() -> Self {
        Self::Hsi16
    }
}

pub trait McoSource {
    type Raw;

    fn into_raw(&self) -> Self::Raw;
}

impl McoSource for Mco1Source {
    type Raw = Mcosel;
    fn into_raw(&self) -> Self::Raw {
        match self {
            Mco1Source::Disabled => Mcosel::NOCLOCK,
            Mco1Source::Lse => Mcosel::LSE,
            Mco1Source::Lsi => Mcosel::LSI,
            Mco1Source::Hse => Mcosel::HSE,
            Mco1Source::Hsi16 => Mcosel::HSI16,
            Mco1Source::PllClk => Mcosel::PLL,
            Mco1Source::SysClk => Mcosel::SYSCLK,
            Mco1Source::Msi => Mcosel::MSI,
            #[cfg(not(any(stm32l471, stm32l475, stm32l476, stm32l486)))]
            Mco1Source::Hsi48 => Mcosel::HSI48,
        }
    }
}

pub(crate) mod sealed {
    use stm32_metapac::rcc::vals::Mcopre;
    pub trait McoInstance {
        type Source;
        unsafe fn apply_clock_settings(source: Self::Source, prescaler: Mcopre);
    }
}

pub trait McoInstance: sealed::McoInstance + 'static {}

pin_trait!(McoPin, McoInstance);

impl sealed::McoInstance for peripherals::MCO {
    type Source = Mcosel;

    unsafe fn apply_clock_settings(source: Self::Source, prescaler: Mcopre) {
        RCC.cfgr().modify(|w| {
            w.set_mcosel(source);
            w.set_mcopre(prescaler);
        });

        match source {
            Mcosel::HSI16 => {
                RCC.cr().modify(|w| w.set_hsion(true));
                while !RCC.cr().read().hsirdy() {}
            }
            #[cfg(not(any(stm32l471, stm32l475, stm32l476, stm32l486)))]
            Mcosel::HSI48 => {
                RCC.crrcr().modify(|w| w.set_hsi48on(true));
                while !RCC.crrcr().read().hsi48rdy() {}
            }
            _ => {}
        }
    }
}

impl McoInstance for peripherals::MCO {}

pub struct Mco<'d, T: McoInstance> {
    phantom: PhantomData<&'d mut T>,
}

impl<'d, T: McoInstance> Mco<'d, T> {
    pub fn new(
        _peri: impl Peripheral<P = T> + 'd,
        pin: impl Peripheral<P = impl McoPin<T>> + 'd,
        source: impl McoSource<Raw = T::Source>,
        prescaler: McoClock,
    ) -> Self {
        into_ref!(pin);

        critical_section::with(|_| unsafe {
            T::apply_clock_settings(source.into_raw(), prescaler.into_raw());
            pin.set_as_af(pin.af_num(), AFType::OutputPushPull);
            pin.set_speed(Speed::VeryHigh);
        });

        Self { phantom: PhantomData }
    }
}

pub(crate) unsafe fn init(config: Config) {
    // Switch to MSI to prevent problems with PLL configuration.
    if !RCC.cr().read().msion() {
        // Turn on MSI and configure it to 4MHz.
        RCC.cr().modify(|w| {
            w.set_msirgsel(true); // MSI Range is provided by MSIRANGE[3:0].
            w.set_msirange(MSIRange::default().into());
            w.set_msipllen(false);
            w.set_msion(true)
        });

        // Wait until MSI is running
        while !RCC.cr().read().msirdy() {}
    }
    if RCC.cfgr().read().sws() != Sw::MSI {
        // Set MSI as a clock source, reset prescalers.
        RCC.cfgr().write_value(Cfgr::default());
        // Wait for clock switch status bits to change.
        while RCC.cfgr().read().sws() != Sw::MSI {}
    }

    RCC.apb1enr1().modify(|w| w.set_pwren(true));

    match config.rtc_mux {
        RtcClockSource::LSE => {
            // 1. Unlock the backup domain
            PWR.cr1().modify(|w| w.set_dbp(true));

            // 2. Setup the LSE
            RCC.bdcr().modify(|w| {
                // Enable LSE
                w.set_lseon(true);
                // Max drive strength
                // TODO: should probably be settable
                w.set_lsedrv(Lsedrv::HIGH);
            });

            // Wait until LSE is running
            while !RCC.bdcr().read().lserdy() {}

            BackupDomain::set_rtc_clock_source(RtcClockSource::LSE);
        }
        RtcClockSource::LSI => {
            // Turn on the internal 32 kHz LSI oscillator
            RCC.csr().modify(|w| w.set_lsion(true));

            // Wait until LSI is running
            while !RCC.csr().read().lsirdy() {}

            BackupDomain::set_rtc_clock_source(RtcClockSource::LSI);
        }
        _ => unreachable!(),
    }

    let (sys_clk, sw) = match config.mux {
        ClockSrc::MSI(range) => {
            // Enable MSI
            RCC.cr().write(|w| {
                let bits: Msirange = range.into();
                w.set_msirange(bits);
                w.set_msirgsel(true);
                w.set_msion(true);

                if let RtcClockSource::LSE = config.rtc_mux {
                    // If LSE is enabled, enable calibration of MSI
                    w.set_msipllen(true);
                } else {
                    w.set_msipllen(false);
                }
            });
            while !RCC.cr().read().msirdy() {}

            // Enable as clock source for USB, RNG if running at 48 MHz
            if let MSIRange::Range11 = range {
                RCC.ccipr().modify(|w| {
                    w.set_clk48sel(0b11);
                });
            }
            (range.into(), Sw::MSI)
        }
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
        ClockSrc::PLL(src, div, prediv, mul, pll48div) => {
            let src_freq = match src {
                PLLSource::HSE(freq) => {
                    // Enable HSE
                    RCC.cr().write(|w| w.set_hseon(true));
                    while !RCC.cr().read().hserdy() {}
                    freq.0
                }
                PLLSource::HSI16 => {
                    // Enable HSI
                    RCC.cr().write(|w| w.set_hsion(true));
                    while !RCC.cr().read().hsirdy() {}
                    HSI_FREQ.0
                }
                PLLSource::MSI(range) => {
                    // Enable MSI
                    RCC.cr().write(|w| {
                        let bits: Msirange = range.into();
                        w.set_msirange(bits);
                        w.set_msipllen(false); // should be turned on if LSE is started
                        w.set_msirgsel(true);
                        w.set_msion(true);
                    });
                    while !RCC.cr().read().msirdy() {}
                    range.into()
                }
            };

            // Disable PLL
            RCC.cr().modify(|w| w.set_pllon(false));
            while RCC.cr().read().pllrdy() {}

            let freq = (src_freq / prediv.to_div() * mul.to_mul()) / div.to_div();

            #[cfg(any(stm32l4px, stm32l4qx, stm32l4rx, stm32l4sx))]
            assert!(freq <= 120_000_000);
            #[cfg(not(any(stm32l4px, stm32l4qx, stm32l4rx, stm32l4sx)))]
            assert!(freq <= 80_000_000);

            RCC.pllcfgr().write(move |w| {
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
            if let Some(pll48div) = pll48div {
                let freq = (src_freq / prediv.to_div() * mul.to_mul()) / pll48div.to_div();
                assert!(freq == 48_000_000);
                RCC.ccipr().modify(|w| {
                    w.set_clk48sel(0b10);
                });
            }

            if let Some((mul, prediv, r_div, q_div, p_div)) = config.pllsai1 {
                RCC.pllsai1cfgr().write(move |w| {
                    w.set_pllsai1n(mul.into());
                    w.set_pllsai1m(prediv.into());
                    if let Some(r_div) = r_div {
                        w.set_pllsai1r(r_div.into());
                        w.set_pllsai1ren(true);
                    }
                    if let Some(q_div) = q_div {
                        w.set_pllsai1q(q_div.into());
                        w.set_pllsai1qen(true);
                        let freq = (src_freq / prediv.to_div() * mul.to_mul()) / q_div.to_div();
                        if freq == 48_000_000 {
                            RCC.ccipr().modify(|w| {
                                w.set_clk48sel(0b1);
                            });
                        }
                    }
                    if let Some(p_div) = p_div {
                        w.set_pllsai1pdiv(p_div.into());
                        w.set_pllsai1pen(true);
                    }
                });

                RCC.cr().modify(|w| w.set_pllsai1on(true));
            }

            // Enable PLL
            RCC.cr().modify(|w| w.set_pllon(true));
            while !RCC.cr().read().pllrdy() {}
            RCC.pllcfgr().modify(|w| w.set_pllren(true));

            (freq, Sw::PLL)
        }
    };

    #[cfg(not(any(stm32l471, stm32l475, stm32l476, stm32l486)))]
    if config.hsi48 {
        RCC.crrcr().modify(|w| w.set_hsi48on(true));
        while !RCC.crrcr().read().hsi48rdy() {}

        // Enable as clock source for USB, RNG and SDMMC
        RCC.ccipr().modify(|w| w.set_clk48sel(0));
    }

    // Set flash wait states
    FLASH.acr().modify(|w| {
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
            let pre = 1 << (pre.to_bits() as u32 - 7);
            sys_clk / pre
        }
    };

    let (apb1_freq, apb1_tim_freq) = match config.apb1_pre {
        APBPrescaler::NotDivided => (ahb_freq, ahb_freq),
        pre => {
            let pre: Ppre = pre.into();
            let pre: u8 = 1 << (pre.to_bits() - 3);
            let freq = ahb_freq / pre as u32;
            (freq, freq * 2)
        }
    };

    let (apb2_freq, apb2_tim_freq) = match config.apb2_pre {
        APBPrescaler::NotDivided => (ahb_freq, ahb_freq),
        pre => {
            let pre: Ppre = pre.into();
            let pre: u8 = 1 << (pre.to_bits() - 3);
            let freq = ahb_freq / pre as u32;
            (freq, freq * 2)
        }
    };

    RCC.apb1enr1().modify(|w| w.set_pwren(true));

    set_freqs(Clocks {
        sys: Hertz(sys_clk),
        ahb1: Hertz(ahb_freq),
        ahb2: Hertz(ahb_freq),
        ahb3: Hertz(ahb_freq),
        apb1: Hertz(apb1_freq),
        apb2: Hertz(apb2_freq),
        apb1_tim: Hertz(apb1_tim_freq),
        apb2_tim: Hertz(apb2_tim_freq),
    });
}
