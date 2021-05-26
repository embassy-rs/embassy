use crate::clock::Clock;
use crate::interrupt;
use crate::pac;
use crate::pac::peripherals::{self, RCC, TIM2};
use crate::time::Hertz;
use crate::time::U32Ext;
use pac::rcc::vals;
use vals::{Hpre, Lptimen, Msirange, Plldiv, Pllmul, Pllon, Pllsrc, Ppre, Sw};

/// Most of clock setup is copied from stm32l0xx-hal, and adopted to the generated PAC,
/// and with the addition of the init function to configure a system clock.

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
#[derive(Clone, Copy)]
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

/// HSI speed
pub const HSI_FREQ: u32 = 16_000_000;

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

    #[inline]
    pub fn hsi16() -> Config {
        Config {
            mux: ClockSrc::HSI16,
            ahb_pre: AHBPrescaler::NotDivided,
            apb1_pre: APBPrescaler::NotDivided,
            apb2_pre: APBPrescaler::NotDivided,
        }
    }

    #[inline]
    pub fn msi(range: MSIRange) -> Config {
        Config {
            mux: ClockSrc::MSI(range),
            ahb_pre: AHBPrescaler::NotDivided,
            apb1_pre: APBPrescaler::NotDivided,
            apb2_pre: APBPrescaler::NotDivided,
        }
    }

    #[inline]
    pub fn pll(pll_src: PLLSource, pll_mul: PLLMul, pll_div: PLLDiv) -> Config {
        Config {
            mux: ClockSrc::PLL(pll_src, pll_mul, pll_div),
            ahb_pre: AHBPrescaler::NotDivided,
            apb1_pre: APBPrescaler::NotDivided,
            apb2_pre: APBPrescaler::NotDivided,
        }
    }

    #[inline]
    pub fn hse<T>(freq: T) -> Config
    where
        T: Into<Hertz>,
    {
        Config {
            mux: ClockSrc::HSE(freq.into()),
            ahb_pre: AHBPrescaler::NotDivided,
            apb1_pre: APBPrescaler::NotDivided,
            apb2_pre: APBPrescaler::NotDivided,
        }
    }
}

/// RCC peripheral
pub struct Rcc {
    clocks: Clocks,
}

/*
impl Rcc {
    pub fn enable_lse(&mut self, _: &PWR) -> LSE {
        self.rb.csr.modify(|_, w| {
            // Enable LSE clock
            w.lseon().set_bit()
        });
        while self.rb.csr.read().lserdy().bit_is_clear() {}
        LSE(())
    }
}
impl Rcc {
    pub fn enable_hsi48(&mut self, syscfg: &mut SYSCFG, crs: CRS) -> HSI48 {
        // Reset CRS peripheral
        self.rb.apb1rstr.modify(|_, w| w.crsrst().set_bit());
        self.rb.apb1rstr.modify(|_, w| w.crsrst().clear_bit());

        // Enable CRS peripheral
        self.rb.apb1enr.modify(|_, w| w.crsen().set_bit());

        // Initialize CRS
        crs.cfgr.write(|w|
            // Select LSE as synchronization source
            unsafe { w.syncsrc().bits(0b01) });
        crs.cr
            .modify(|_, w| w.autotrimen().set_bit().cen().set_bit());

        // Enable VREFINT reference for HSI48 oscillator
        syscfg
            .syscfg
            .cfgr3
            .modify(|_, w| w.enref_hsi48().set_bit().en_vrefint().set_bit());

        // Select HSI48 as USB clock
        self.rb.ccipr.modify(|_, w| w.hsi48msel().set_bit());

        // Enable dedicated USB clock
        self.rb.crrcr.modify(|_, w| w.hsi48on().set_bit());
        while self.rb.crrcr.read().hsi48rdy().bit_is_clear() {}

        HSI48(())
    }
}

impl Rcc {
    /// Configure MCO (Microcontroller Clock Output).
    pub fn configure_mco<P>(
        &mut self,
        source: MCOSEL_A,
        prescaler: MCOPRE_A,
        output_pin: P,
    ) -> MCOEnabled
    where
        P: mco::Pin,
    {
        output_pin.into_mco();

        self.rb.cfgr.modify(|_, w| {
            w.mcosel().variant(source);
            w.mcopre().variant(prescaler)
        });

        MCOEnabled(())
    }
}
*/

/// Extension trait that freezes the `RCC` peripheral with provided clocks configuration
pub trait RccExt {
    fn freeze(self, config: Config) -> Rcc;
}

impl RccExt for RCC {
    // `cfgr` is almost always a constant, so make sure it can be constant-propagated properly by
    // marking this function and all `Config` constructors and setters as `#[inline]`.
    // This saves ~900 Bytes for the `pwr.rs` example.
    #[inline]
    fn freeze(self, cfgr: Config) -> Rcc {
        let rcc = pac::RCC;
        let (sys_clk, sw) = match cfgr.mux {
            ClockSrc::MSI(range) => {
                // Set MSI range
                unsafe {
                    rcc.icscr().write(|w| w.set_msirange(range.into()));
                }

                // Enable MSI
                unsafe {
                    rcc.cr().write(|w| w.set_msion(Pllon::ENABLED));
                    while !rcc.cr().read().msirdy() {}
                }

                let freq = 32_768 * (1 << (range as u8 + 1));
                (freq, Sw::MSI)
            }
            ClockSrc::HSI16 => {
                // Enable HSI16
                unsafe {
                    rcc.cr().write(|w| w.set_hsi16on(Pllon::ENABLED));
                    while !rcc.cr().read().hsi16rdyf() {}
                }

                (HSI_FREQ, Sw::HSI16)
            }
            ClockSrc::HSE(freq) => {
                // Enable HSE
                unsafe {
                    rcc.cr().write(|w| w.set_hseon(Pllon::ENABLED));
                    while !rcc.cr().read().hserdy() {}
                }

                (freq.0, Sw::HSE)
            }
            ClockSrc::PLL(src, mul, div) => {
                let freq = match src {
                    PLLSource::HSE(freq) => {
                        // Enable HSE
                        unsafe {
                            rcc.cr().write(|w| w.set_hseon(Pllon::ENABLED));
                            while !rcc.cr().read().hserdy() {}
                        }
                        freq.0
                    }
                    PLLSource::HSI16 => {
                        // Enable HSI
                        unsafe {
                            rcc.cr().write(|w| w.set_hsi16on(Pllon::ENABLED));
                            while !rcc.cr().read().hsi16rdyf() {}
                        }
                        HSI_FREQ
                    }
                };

                // Disable PLL
                unsafe {
                    rcc.cr().modify(|w| w.set_pllon(Pllon::DISABLED));
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
                    rcc.cr().modify(|w| w.set_pllon(Pllon::ENABLED));
                    while !rcc.cr().read().pllrdy() {}
                }

                (freq, Sw::PLL)
            }
        };

        unsafe {
            rcc.cfgr().modify(|w| {
                w.set_sw(sw.into());
                w.set_hpre(cfgr.ahb_pre.into());
                w.set_ppre(0, cfgr.apb1_pre.into());
                w.set_ppre(1, cfgr.apb2_pre.into());
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

        let (apb1_freq, apb1_tim_freq, apb1_pre) = match cfgr.apb1_pre {
            APBPrescaler::NotDivided => (ahb_freq, ahb_freq, 1),
            pre => {
                let pre: Ppre = pre.into();
                let pre: u8 = 1 << (pre.0 - 3);
                let freq = ahb_freq / pre as u32;
                (freq, freq * 2, pre as u8)
            }
        };

        let (apb2_freq, apb2_tim_freq, apb2_pre) = match cfgr.apb2_pre {
            APBPrescaler::NotDivided => (ahb_freq, ahb_freq, 1),
            pre => {
                let pre: Ppre = pre.into();
                let pre: u8 = 1 << (pre.0 - 3);
                let freq = ahb_freq / (1 << (pre as u8 - 3));
                (freq, freq * 2, pre as u8)
            }
        };

        let clocks = Clocks {
            source: cfgr.mux,
            sys_clk: sys_clk.hz(),
            ahb_clk: ahb_freq.hz(),
            apb1_clk: apb1_freq.hz(),
            apb2_clk: apb2_freq.hz(),
            apb1_tim_clk: apb1_tim_freq.hz(),
            apb2_tim_clk: apb2_tim_freq.hz(),
            apb1_pre,
            apb2_pre,
        };

        Rcc { clocks }
    }
}

/// Frozen clock frequencies
///
/// The existence of this value indicates that the clock configuration can no longer be changed
#[derive(Clone, Copy)]
pub struct Clocks {
    source: ClockSrc,
    sys_clk: Hertz,
    ahb_clk: Hertz,
    apb1_clk: Hertz,
    apb1_tim_clk: Hertz,
    apb2_clk: Hertz,
    apb2_tim_clk: Hertz,
    apb1_pre: u8,
    apb2_pre: u8,
}

impl Clocks {
    /// Returns the clock source
    pub fn source(&self) -> &ClockSrc {
        &self.source
    }

    /// Returns the system (core) frequency
    pub fn sys_clk(&self) -> Hertz {
        self.sys_clk
    }

    /// Returns the frequency of the AHB
    pub fn ahb_clk(&self) -> Hertz {
        self.ahb_clk
    }

    /// Returns the frequency of the APB1
    pub fn apb1_clk(&self) -> Hertz {
        self.apb1_clk
    }

    /// Returns the frequency of the APB1 timers
    pub fn apb1_tim_clk(&self) -> Hertz {
        self.apb1_tim_clk
    }

    /// Returns the prescaler of the APB1
    pub fn apb1_pre(&self) -> u8 {
        self.apb1_pre
    }

    /// Returns the frequency of the APB2
    pub fn apb2_clk(&self) -> Hertz {
        self.apb2_clk
    }

    /// Returns the frequency of the APB2 timers
    pub fn apb2_tim_clk(&self) -> Hertz {
        self.apb2_tim_clk
    }

    /// Returns the prescaler of the APB2
    pub fn apb2_pre(&self) -> u8 {
        self.apb2_pre
    }
}

/// Token that exists only, if the HSI48 clock has been enabled
///
/// You can get an instance of this struct by calling [`Rcc::enable_hsi48`].
#[derive(Clone, Copy)]
pub struct HSI48(());

/// Token that exists only if MCO (Microcontroller Clock Out) has been enabled.
///
/// You can get an instance of this struct by calling [`Rcc::configure_mco`].
#[derive(Clone, Copy)]
pub struct MCOEnabled(());

/// Token that exists only, if the LSE clock has been enabled
///
/// You can get an instance of this struct by calling [`Rcc::enable_lse`].
#[derive(Clone, Copy)]
pub struct LSE(());

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

// We use TIM2 as SystemClock
pub type SystemClock = Clock<TIM2>;

pub unsafe fn init(config: Config) -> SystemClock {
    let rcc = pac::RCC;
    let enabled = vals::Iophen::ENABLED;
    rcc.iopenr().write(|w| {
        w.set_iopaen(enabled);
        w.set_iopben(enabled);
        w.set_iopcen(enabled);
        w.set_iopden(enabled);
        w.set_iopeen(enabled);
        w.set_iophen(enabled);
    });

    let r = <peripherals::RCC as embassy::util::Steal>::steal();
    let r = r.freeze(config);

    rcc.apb1enr().modify(|w| w.set_tim2en(Lptimen::ENABLED));
    rcc.apb1rstr().modify(|w| w.set_tim2rst(true));
    rcc.apb1rstr().modify(|w| w.set_tim2rst(false));

    Clock::new(
        <peripherals::TIM2 as embassy::util::Steal>::steal(),
        interrupt::take!(TIM2),
        r.clocks.apb1_clk(),
    )
}
