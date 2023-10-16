pub use crate::pac::rcc::vals::{
    Hpre as AHBPrescaler, Hsepre as HsePrescaler, Pllm, Plln, Pllp, Pllq, Pllr, Pllsrc as PllSource,
    Ppre as APBPrescaler, Sw as Sysclk,
};
use crate::rcc::{set_freqs, Clocks};
use crate::time::{mhz, Hertz};

/// HSI speed
pub const HSI_FREQ: Hertz = Hertz(16_000_000);

pub struct Hse {
    pub prediv: HsePrescaler,

    pub frequency: Hertz,
}

pub struct PllMux {
    /// Source clock selection.
    pub source: PllSource,

    /// PLL pre-divider (DIVM). Must be between 1 and 63.
    pub prediv: Pllm,
}

pub struct Pll {
    /// PLL multiplication factor. Must be between 4 and 512.
    pub mul: Plln,

    /// PLL P division factor. If None, PLL P output is disabled. Must be between 1 and 128.
    /// On PLL1, it must be even (in particular, it cannot be 1.)
    pub divp: Option<Pllp>,
    /// PLL Q division factor. If None, PLL Q output is disabled. Must be between 1 and 128.
    pub divq: Option<Pllq>,
    /// PLL R division factor. If None, PLL R output is disabled. Must be between 1 and 128.
    pub divr: Option<Pllr>,
}

/// Clocks configutation
pub struct Config {
    pub hse: Option<Hse>,
    pub sys: Sysclk,
    pub mux: Option<PllMux>,

    pub pll: Option<Pll>,
    pub pllsai: Option<Pll>,

    pub ahb1_pre: AHBPrescaler,
    pub ahb2_pre: AHBPrescaler,
    pub ahb3_pre: AHBPrescaler,
    pub apb1_pre: APBPrescaler,
    pub apb2_pre: APBPrescaler,

    pub ls: super::LsConfig,
}

pub const WPAN_DEFAULT: Config = Config {
    hse: Some(Hse {
        frequency: mhz(32),
        prediv: HsePrescaler::DIV1,
    }),
    sys: Sysclk::PLL,
    mux: Some(PllMux {
        source: PllSource::HSE,
        prediv: Pllm::DIV2,
    }),

    ls: super::LsConfig::default_lse(),

    pll: Some(Pll {
        mul: Plln::MUL12,
        divp: Some(Pllp::DIV3),
        divq: Some(Pllq::DIV4),
        divr: Some(Pllr::DIV3),
    }),
    pllsai: None,

    ahb1_pre: AHBPrescaler::DIV1,
    ahb2_pre: AHBPrescaler::DIV2,
    ahb3_pre: AHBPrescaler::DIV1,
    apb1_pre: APBPrescaler::DIV1,
    apb2_pre: APBPrescaler::DIV1,
};

impl Default for Config {
    #[inline]
    fn default() -> Config {
        Config {
            hse: None,
            sys: Sysclk::HSI16,
            mux: None,
            pll: None,
            pllsai: None,

            ls: Default::default(),

            ahb1_pre: AHBPrescaler::DIV1,
            ahb2_pre: AHBPrescaler::DIV1,
            ahb3_pre: AHBPrescaler::DIV1,
            apb1_pre: APBPrescaler::DIV1,
            apb2_pre: APBPrescaler::DIV1,
        }
    }
}

#[cfg(stm32wb)]
/// RCC initialization function
pub(crate) unsafe fn init(config: Config) {
    let hse_clk = config.hse.as_ref().map(|hse| hse.frequency / hse.prediv);

    let mux_clk = config.mux.as_ref().map(|pll_mux| {
        (match pll_mux.source {
            PllSource::HSE => hse_clk.unwrap(),
            PllSource::HSI16 => HSI_FREQ,
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
        Sysclk::HSI16 => HSI_FREQ,
        Sysclk::PLL => pll_r.unwrap(),
        _ => unreachable!(),
    };

    let ahb1_clk = sys_clk / config.ahb1_pre;
    let ahb2_clk = sys_clk / config.ahb2_pre;
    let ahb3_clk = sys_clk / config.ahb3_pre;

    let (apb1_clk, apb1_tim_clk) = match config.apb1_pre {
        APBPrescaler::DIV1 => (ahb1_clk, ahb1_clk),
        pre => {
            let freq = ahb1_clk / pre;
            (freq, freq * 2u32)
        }
    };

    let (apb2_clk, apb2_tim_clk) = match config.apb2_pre {
        APBPrescaler::DIV1 => (ahb1_clk, ahb1_clk),
        pre => {
            let freq = ahb1_clk / pre;
            (freq, freq * 2u32)
        }
    };

    let rcc = crate::pac::RCC;

    let needs_hsi = if let Some(pll_mux) = &config.mux {
        pll_mux.source == PllSource::HSI16
    } else {
        false
    };

    if needs_hsi || config.sys == Sysclk::HSI16 {
        rcc.cr().modify(|w| {
            w.set_hsion(true);
        });

        while !rcc.cr().read().hsirdy() {}
    }

    rcc.cfgr().modify(|w| w.set_stopwuck(true));

    let rtc = config.ls.init();

    match &config.hse {
        Some(hse) => {
            rcc.cr().modify(|w| {
                w.set_hsepre(hse.prediv);
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
                w.set_plln(pll.mul);
                pll.divp.map(|divp| {
                    w.set_pllpen(true);
                    w.set_pllp(divp)
                });
                pll.divq.map(|divq| {
                    w.set_pllqen(true);
                    w.set_pllq(divq)
                });
                pll.divr.map(|divr| {
                    w.set_pllren(true);
                    w.set_pllr(divr);
                });
            });

            rcc.cr().modify(|w| w.set_pllon(true));

            while !rcc.cr().read().pllrdy() {}
        }
        _ => {}
    }

    rcc.cfgr().modify(|w| {
        w.set_sw(config.sys.into());
        w.set_hpre(config.ahb1_pre);
        w.set_ppre1(config.apb1_pre);
        w.set_ppre2(config.apb2_pre);
    });

    rcc.extcfgr().modify(|w| {
        w.set_c2hpre(config.ahb2_pre);
        w.set_shdhpre(config.ahb3_pre);
    });

    set_freqs(Clocks {
        sys: sys_clk,
        hclk1: ahb1_clk,
        hclk2: ahb2_clk,
        hclk3: ahb3_clk,
        pclk1: apb1_clk,
        pclk2: apb2_clk,
        pclk1_tim: apb1_tim_clk,
        pclk2_tim: apb2_tim_clk,
        rtc,
    })
}
