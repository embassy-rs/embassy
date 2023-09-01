pub use super::bus::{AHBPrescaler, APBPrescaler};
use crate::rcc::bd::{BackupDomain, RtcClockSource};
use crate::rcc::Clocks;
use crate::time::{khz, mhz, Hertz};

/// Most of clock setup is copied from stm32l0xx-hal, and adopted to the generated PAC,
/// and with the addition of the init function to configure a system clock.

/// Only the basic setup using the HSE and HSI clocks are supported as of now.

/// HSI speed
pub const HSI_FREQ: Hertz = Hertz(16_000_000);

/// LSI speed
pub const LSI_FREQ: Hertz = Hertz(32_000);

#[derive(Clone, Copy)]
pub enum HsePrescaler {
    NotDivided,
    Div2,
}

impl From<HsePrescaler> for bool {
    fn from(value: HsePrescaler) -> Self {
        match value {
            HsePrescaler::NotDivided => false,
            HsePrescaler::Div2 => true,
        }
    }
}

pub struct Hse {
    pub prediv: HsePrescaler,

    pub frequency: Hertz,
}

/// System clock mux source
#[derive(Clone, Copy, PartialEq)]
pub enum Sysclk {
    /// MSI selected as sysclk
    MSI,
    /// HSI selected as sysclk
    HSI,
    /// HSE selected as sysclk
    HSE,
    /// PLL selected as sysclk
    Pll,
}

impl From<Sysclk> for u8 {
    fn from(value: Sysclk) -> Self {
        match value {
            Sysclk::MSI => 0b00,
            Sysclk::HSI => 0b01,
            Sysclk::HSE => 0b10,
            Sysclk::Pll => 0b11,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum PllSource {
    Hsi,
    Msi,
    Hse,
}

impl From<PllSource> for u8 {
    fn from(value: PllSource) -> Self {
        match value {
            PllSource::Msi => 0b01,
            PllSource::Hsi => 0b10,
            PllSource::Hse => 0b11,
        }
    }
}

pub enum Pll48Source {
    PllSai,
    Pll,
    Msi,
    Hsi48,
}

pub struct PllMux {
    /// Source clock selection.
    pub source: PllSource,

    /// PLL pre-divider (DIVM). Must be between 1 and 63.
    pub prediv: u8,
}

pub struct Pll {
    /// PLL multiplication factor. Must be between 4 and 512.
    pub mul: u16,

    /// PLL P division factor. If None, PLL P output is disabled. Must be between 1 and 128.
    /// On PLL1, it must be even (in particular, it cannot be 1.)
    pub divp: Option<u16>,
    /// PLL Q division factor. If None, PLL Q output is disabled. Must be between 1 and 128.
    pub divq: Option<u16>,
    /// PLL R division factor. If None, PLL R output is disabled. Must be between 1 and 128.
    pub divr: Option<u16>,
}

/// Clocks configutation
pub struct Config {
    pub hse: Option<Hse>,
    pub lse: Option<Hertz>,
    pub sys: Sysclk,
    pub mux: Option<PllMux>,
    pub pll48: Option<Pll48Source>,
    pub rtc: Option<RtcClockSource>,

    pub pll: Option<Pll>,
    pub pllsai: Option<Pll>,

    pub ahb1_pre: AHBPrescaler,
    pub ahb2_pre: AHBPrescaler,
    pub ahb3_pre: AHBPrescaler,
    pub apb1_pre: APBPrescaler,
    pub apb2_pre: APBPrescaler,
}

pub const WPAN_DEFAULT: Config = Config {
    hse: Some(Hse {
        frequency: mhz(32),
        prediv: HsePrescaler::NotDivided,
    }),
    lse: Some(khz(32)),
    sys: Sysclk::Pll,
    mux: Some(PllMux {
        source: PllSource::Hse,
        prediv: 2,
    }),
    pll48: None,
    rtc: None,

    pll: Some(Pll {
        mul: 12,
        divp: Some(3),
        divq: Some(4),
        divr: Some(3),
    }),
    pllsai: None,

    ahb1_pre: AHBPrescaler::NotDivided,
    ahb2_pre: AHBPrescaler::Div2,
    ahb3_pre: AHBPrescaler::NotDivided,
    apb1_pre: APBPrescaler::NotDivided,
    apb2_pre: APBPrescaler::NotDivided,
};

impl Default for Config {
    #[inline]
    fn default() -> Config {
        Config {
            hse: None,
            lse: None,
            sys: Sysclk::HSI,
            mux: None,
            pll48: None,
            pll: None,
            pllsai: None,
            rtc: None,

            ahb1_pre: AHBPrescaler::NotDivided,
            ahb2_pre: AHBPrescaler::NotDivided,
            ahb3_pre: AHBPrescaler::NotDivided,
            apb1_pre: APBPrescaler::NotDivided,
            apb2_pre: APBPrescaler::NotDivided,
        }
    }
}

pub(crate) fn compute_clocks(config: &Config) -> Clocks {
    let hse_clk = config.hse.as_ref().map(|hse| match hse.prediv {
        HsePrescaler::NotDivided => hse.frequency,
        HsePrescaler::Div2 => hse.frequency / 2u32,
    });

    let mux_clk = config.mux.as_ref().map(|pll_mux| {
        (match pll_mux.source {
            PllSource::Hse => hse_clk.unwrap(),
            PllSource::Hsi => HSI_FREQ,
            _ => unreachable!(),
        } / pll_mux.prediv)
    });

    let (pll_r, _pll_q, _pll_p) = match &config.pll {
        Some(pll) => {
            let pll_vco = mux_clk.unwrap() * pll.mul as u32;

            (
                pll.divr.map(|divr| pll_vco / divr),
                pll.divq.map(|divq| pll_vco / divq),
                pll.divp.map(|divp| pll_vco / divp),
            )
        }
        None => (None, None, None),
    };

    let sys_clk = match config.sys {
        Sysclk::HSE => hse_clk.unwrap(),
        Sysclk::HSI => HSI_FREQ,
        Sysclk::Pll => pll_r.unwrap(),
        _ => unreachable!(),
    };

    let ahb1_clk = match config.ahb1_pre {
        AHBPrescaler::NotDivided => sys_clk,
        pre => {
            let pre: u8 = pre.into();
            let pre = 1u32 << (pre as u32 - 7);
            sys_clk / pre
        }
    };

    let ahb2_clk = match config.ahb2_pre {
        AHBPrescaler::NotDivided => sys_clk,
        pre => {
            let pre: u8 = pre.into();
            let pre = 1u32 << (pre as u32 - 7);
            sys_clk / pre
        }
    };

    let ahb3_clk = match config.ahb3_pre {
        AHBPrescaler::NotDivided => sys_clk,
        pre => {
            let pre: u8 = pre.into();
            let pre = 1u32 << (pre as u32 - 7);
            sys_clk / pre
        }
    };

    let (apb1_clk, apb1_tim_clk) = match config.apb1_pre {
        APBPrescaler::NotDivided => (ahb1_clk, ahb1_clk),
        pre => {
            let pre: u8 = pre.into();
            let pre: u8 = 1 << (pre - 3);
            let freq = ahb1_clk / pre as u32;
            (freq, freq * 2u32)
        }
    };

    let (apb2_clk, apb2_tim_clk) = match config.apb2_pre {
        APBPrescaler::NotDivided => (ahb1_clk, ahb1_clk),
        pre => {
            let pre: u8 = pre.into();
            let pre: u8 = 1 << (pre - 3);
            let freq = ahb1_clk / pre as u32;
            (freq, freq * 2u32)
        }
    };

    let rtc_clk = match config.rtc {
        Some(RtcClockSource::LSI) => Some(LSI_FREQ),
        Some(RtcClockSource::LSE) => Some(config.lse.unwrap()),
        _ => None,
    };

    Clocks {
        sys: sys_clk,
        ahb1: ahb1_clk,
        ahb2: ahb2_clk,
        ahb3: ahb3_clk,
        apb1: apb1_clk,
        apb2: apb2_clk,
        apb1_tim: apb1_tim_clk,
        apb2_tim: apb2_tim_clk,
        rtc: rtc_clk,
        rtc_hse: None,
    }
}

pub(crate) fn configure_clocks(config: &Config) {
    let pwr = crate::pac::PWR;
    let rcc = crate::pac::RCC;

    let needs_hsi = if let Some(pll_mux) = &config.mux {
        pll_mux.source == PllSource::Hsi
    } else {
        false
    };

    if needs_hsi || config.sys == Sysclk::HSI {
        rcc.cr().modify(|w| {
            w.set_hsion(true);
        });

        while !rcc.cr().read().hsirdy() {}
    }

    let needs_lsi = if let Some(rtc_mux) = &config.rtc {
        *rtc_mux == RtcClockSource::LSI
    } else {
        false
    };

    if needs_lsi {
        rcc.csr().modify(|w| w.set_lsi1on(true));

        while !rcc.csr().read().lsi1rdy() {}
    }

    match &config.lse {
        Some(_) => {
            rcc.cfgr().modify(|w| w.set_stopwuck(true));

            pwr.cr1().modify(|w| w.set_dbp(true));
            pwr.cr1().modify(|w| w.set_dbp(true));

            rcc.bdcr().modify(|w| w.set_lseon(true));
        }
        _ => {}
    }

    match &config.hse {
        Some(hse) => {
            rcc.cr().modify(|w| {
                w.set_hsepre(hse.prediv.into());
                w.set_hseon(true);
            });

            while !rcc.cr().read().hserdy() {}
        }
        _ => {}
    }

    match &config.mux {
        Some(pll_mux) => {
            rcc.pllcfgr().modify(|w| {
                w.set_pllm(pll_mux.prediv);
                w.set_pllsrc(pll_mux.source.into());
            });
        }
        _ => {}
    };

    match &config.pll {
        Some(pll) => {
            rcc.pllcfgr().modify(|w| {
                w.set_plln(pll.mul as u8);
                pll.divp.map(|divp| {
                    w.set_pllpen(true);
                    w.set_pllp((divp - 1) as u8)
                });
                pll.divq.map(|divq| {
                    w.set_pllqen(true);
                    w.set_pllq((divq - 1) as u8)
                });
                pll.divr.map(|divr| {
                    // w.set_pllren(true);
                    w.set_pllr((divr - 1) as u8);
                });
            });

            rcc.cr().modify(|w| w.set_pllon(true));

            while !rcc.cr().read().pllrdy() {}
        }
        _ => {}
    }

    rcc.cfgr().modify(|w| {
        w.set_sw(config.sys.into());
        w.set_hpre(config.ahb1_pre.into());
        w.set_ppre1(config.apb1_pre.into());
        w.set_ppre2(config.apb2_pre.into());
    });

    rcc.extcfgr().modify(|w| {
        w.set_c2hpre(config.ahb2_pre.into());
        w.set_shdhpre(config.ahb3_pre.into());
    });

    config
        .rtc
        .map(|clock_source| BackupDomain::set_rtc_clock_source(clock_source));
}
