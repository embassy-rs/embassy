use crate::pac::rcc::regs::Cfgr;
pub use crate::pac::rcc::vals::{
    Hpre as AHBPrescaler, Msirange as MSIRange, Pllm as PllPreDiv, Plln as PllMul, Pllp as PllPDiv, Pllq as PllQDiv,
    Pllr as PllRDiv, Ppre as APBPrescaler,
};
use crate::pac::rcc::vals::{Msirange, Pllsrc, Sw};
use crate::pac::{FLASH, RCC};
use crate::rcc::bd::{BackupDomain, RtcClockSource};
use crate::rcc::{set_freqs, Clocks};
use crate::time::Hertz;

/// HSI speed
pub const HSI_FREQ: Hertz = Hertz(16_000_000);

/// LSI speed
pub const LSI_FREQ: Hertz = Hertz(32_000);

/// System clock mux source
#[derive(Clone, Copy)]
pub enum ClockSrc {
    MSI(MSIRange),
    PLL(PLLSource, PllRDiv, PllPreDiv, PllMul, Option<PllQDiv>),
    HSE(Hertz),
    HSI16,
}

/// PLL clock input source
#[derive(Clone, Copy)]
pub enum PLLSource {
    HSI16,
    HSE(Hertz),
    MSI(MSIRange),
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

/// Clocks configutation
pub struct Config {
    pub mux: ClockSrc,
    pub ahb_pre: AHBPrescaler,
    pub apb1_pre: APBPrescaler,
    pub apb2_pre: APBPrescaler,
    pub pllsai1: Option<(PllMul, PllPreDiv, Option<PllRDiv>, Option<PllQDiv>, Option<PllPDiv>)>,
    #[cfg(not(any(stm32l471, stm32l475, stm32l476, stm32l486)))]
    pub hsi48: bool,
    pub rtc_mux: RtcClockSource,
    pub lse: Option<Hertz>,
    pub lsi: bool,
}

impl Default for Config {
    #[inline]
    fn default() -> Config {
        Config {
            mux: ClockSrc::MSI(MSIRange::RANGE4M),
            ahb_pre: AHBPrescaler::DIV1,
            apb1_pre: APBPrescaler::DIV1,
            apb2_pre: APBPrescaler::DIV1,
            pllsai1: None,
            #[cfg(not(any(stm32l471, stm32l475, stm32l476, stm32l486)))]
            hsi48: false,
            rtc_mux: RtcClockSource::LSI,
            lsi: true,
            lse: None,
        }
    }
}

pub(crate) unsafe fn init(config: Config) {
    // Switch to MSI to prevent problems with PLL configuration.
    if !RCC.cr().read().msion() {
        // Turn on MSI and configure it to 4MHz.
        RCC.cr().modify(|w| {
            w.set_msirgsel(true); // MSI Range is provided by MSIRANGE[3:0].
            w.set_msirange(MSIRange::RANGE4M);
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

    BackupDomain::configure_ls(config.rtc_mux, config.lsi, config.lse.map(|_| Default::default()));

    let (sys_clk, sw) = match config.mux {
        ClockSrc::MSI(range) => {
            // Enable MSI
            RCC.cr().write(|w| {
                w.set_msirange(range);
                w.set_msirgsel(true);
                w.set_msion(true);

                if config.rtc_mux == RtcClockSource::LSE {
                    // If LSE is enabled, enable calibration of MSI
                    w.set_msipllen(true);
                } else {
                    w.set_msipllen(false);
                }
            });
            while !RCC.cr().read().msirdy() {}

            // Enable as clock source for USB, RNG if running at 48 MHz
            if range == MSIRange::RANGE48M {
                RCC.ccipr().modify(|w| {
                    w.set_clk48sel(0b11);
                });
            }
            (msirange_to_hertz(range), Sw::MSI)
        }
        ClockSrc::HSI16 => {
            // Enable HSI16
            RCC.cr().write(|w| w.set_hsion(true));
            while !RCC.cr().read().hsirdy() {}

            (HSI_FREQ, Sw::HSI16)
        }
        ClockSrc::HSE(freq) => {
            // Enable HSE
            RCC.cr().write(|w| w.set_hseon(true));
            while !RCC.cr().read().hserdy() {}

            (freq, Sw::HSE)
        }
        ClockSrc::PLL(src, divr, prediv, mul, divq) => {
            let src_freq = match src {
                PLLSource::HSE(freq) => {
                    // Enable HSE
                    RCC.cr().write(|w| w.set_hseon(true));
                    while !RCC.cr().read().hserdy() {}
                    freq
                }
                PLLSource::HSI16 => {
                    // Enable HSI
                    RCC.cr().write(|w| w.set_hsion(true));
                    while !RCC.cr().read().hsirdy() {}
                    HSI_FREQ
                }
                PLLSource::MSI(range) => {
                    // Enable MSI
                    RCC.cr().write(|w| {
                        w.set_msirange(range);
                        w.set_msipllen(false); // should be turned on if LSE is started
                        w.set_msirgsel(true);
                        w.set_msion(true);
                    });
                    while !RCC.cr().read().msirdy() {}

                    msirange_to_hertz(range)
                }
            };

            // Disable PLL
            RCC.cr().modify(|w| w.set_pllon(false));
            while RCC.cr().read().pllrdy() {}

            let freq = src_freq / prediv * mul / divr;

            #[cfg(any(stm32l4px, stm32l4qx, stm32l4rx, stm32l4sx))]
            assert!(freq.0 <= 120_000_000);
            #[cfg(not(any(stm32l4px, stm32l4qx, stm32l4rx, stm32l4sx)))]
            assert!(freq.0 <= 80_000_000);

            RCC.pllcfgr().write(move |w| {
                w.set_plln(mul);
                w.set_pllm(prediv);
                w.set_pllr(divr);
                if let Some(divq) = divq {
                    w.set_pllq(divq);
                    w.set_pllqen(true);
                }
                w.set_pllsrc(src.into());
            });

            // Enable as clock source for USB, RNG if PLL48 divisor is provided
            if let Some(divq) = divq {
                let freq = src_freq / prediv * mul / divq;
                assert!(freq.0 == 48_000_000);
                RCC.ccipr().modify(|w| {
                    w.set_clk48sel(0b10);
                });
            }

            if let Some((mul, prediv, r_div, q_div, p_div)) = config.pllsai1 {
                RCC.pllsai1cfgr().write(move |w| {
                    w.set_plln(mul);
                    w.set_pllm(prediv);
                    if let Some(r_div) = r_div {
                        w.set_pllr(r_div);
                        w.set_pllren(true);
                    }
                    if let Some(q_div) = q_div {
                        w.set_pllq(q_div);
                        w.set_pllqen(true);
                        let freq = src_freq / prediv * mul / q_div;
                        if freq.0 == 48_000_000 {
                            RCC.ccipr().modify(|w| {
                                w.set_clk48sel(0b1);
                            });
                        }
                    }
                    if let Some(p_div) = p_div {
                        w.set_pllp(p_div);
                        w.set_pllpen(true);
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
        w.set_latency(match sys_clk.0 {
            0..=16_000_000 => 0,
            0..=32_000_000 => 1,
            0..=48_000_000 => 2,
            0..=64_000_000 => 3,
            _ => 4,
        })
    });

    RCC.cfgr().modify(|w| {
        w.set_sw(sw);
        w.set_hpre(config.ahb_pre);
        w.set_ppre1(config.apb1_pre);
        w.set_ppre2(config.apb2_pre);
    });

    let ahb_freq = sys_clk / config.ahb_pre;

    let (apb1_freq, apb1_tim_freq) = match config.apb1_pre {
        APBPrescaler::DIV1 => (ahb_freq, ahb_freq),
        pre => {
            let freq = ahb_freq / pre;
            (freq, freq * 2u32)
        }
    };

    let (apb2_freq, apb2_tim_freq) = match config.apb2_pre {
        APBPrescaler::DIV1 => (ahb_freq, ahb_freq),
        pre => {
            let freq = ahb_freq / pre;
            (freq, freq * 2u32)
        }
    };

    set_freqs(Clocks {
        sys: sys_clk,
        ahb1: ahb_freq,
        ahb2: ahb_freq,
        ahb3: ahb_freq,
        apb1: apb1_freq,
        apb2: apb2_freq,
        apb1_tim: apb1_tim_freq,
        apb2_tim: apb2_tim_freq,
    });
}

fn msirange_to_hertz(range: Msirange) -> Hertz {
    match range {
        MSIRange::RANGE100K => Hertz(100_000),
        MSIRange::RANGE200K => Hertz(200_000),
        MSIRange::RANGE400K => Hertz(400_000),
        MSIRange::RANGE800K => Hertz(800_000),
        MSIRange::RANGE1M => Hertz(1_000_000),
        MSIRange::RANGE2M => Hertz(2_000_000),
        MSIRange::RANGE4M => Hertz(4_000_000),
        MSIRange::RANGE8M => Hertz(8_000_000),
        MSIRange::RANGE16M => Hertz(16_000_000),
        MSIRange::RANGE24M => Hertz(24_000_000),
        MSIRange::RANGE32M => Hertz(32_000_000),
        MSIRange::RANGE48M => Hertz(48_000_000),
        _ => unreachable!(),
    }
}
