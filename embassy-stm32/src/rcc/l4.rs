use crate::pac::rcc::regs::Cfgr;
use crate::pac::rcc::vals::Msirgsel;
pub use crate::pac::rcc::vals::{
    Clk48sel as Clk48Src, Hpre as AHBPrescaler, Msirange as MSIRange, Pllm as PllPreDiv, Plln as PllMul,
    Pllp as PllPDiv, Pllq as PllQDiv, Pllr as PllRDiv, Pllsrc as PLLSource, Ppre as APBPrescaler, Sw as ClockSrc,
};
use crate::pac::{FLASH, RCC};
use crate::rcc::{set_freqs, Clocks};
use crate::time::Hertz;

/// HSI speed
pub const HSI_FREQ: Hertz = Hertz(16_000_000);

#[derive(Clone, Copy)]
pub struct Pll {
    /// PLL source
    pub source: PLLSource,

    /// PLL pre-divider (DIVM).
    pub prediv: PllPreDiv,

    /// PLL multiplication factor.
    pub mul: PllMul,

    /// PLL P division factor. If None, PLL P output is disabled.
    pub divp: Option<PllPDiv>,
    /// PLL Q division factor. If None, PLL Q output is disabled.
    pub divq: Option<PllQDiv>,
    /// PLL R division factor. If None, PLL R output is disabled.
    pub divr: Option<PllRDiv>,
}

/// Clocks configutation
pub struct Config {
    // base clock sources
    pub msi: Option<MSIRange>,
    pub hsi16: bool,
    pub hse: Option<Hertz>,
    #[cfg(not(any(stm32l47x, stm32l48x)))]
    pub hsi48: bool,

    // pll
    pub pll: Option<Pll>,
    pub pllsai1: Option<Pll>,
    #[cfg(any(
        stm32l47x, stm32l48x, stm32l49x, stm32l4ax, stm32l4px, stm32l4qx, stm32l4rx, stm32l4sx
    ))]
    pub pllsai2: Option<Pll>,

    // sysclk, buses.
    pub mux: ClockSrc,
    pub ahb_pre: AHBPrescaler,
    pub apb1_pre: APBPrescaler,
    pub apb2_pre: APBPrescaler,

    // muxes
    pub clk48_src: Clk48Src,

    // low speed LSI/LSE/RTC
    pub ls: super::LsConfig,
}

impl Default for Config {
    #[inline]
    fn default() -> Config {
        Config {
            hse: None,
            hsi16: false,
            msi: Some(MSIRange::RANGE4M),
            mux: ClockSrc::MSI,
            ahb_pre: AHBPrescaler::DIV1,
            apb1_pre: APBPrescaler::DIV1,
            apb2_pre: APBPrescaler::DIV1,
            pll: None,
            pllsai1: None,
            #[cfg(any(
                stm32l47x, stm32l48x, stm32l49x, stm32l4ax, stm32l4px, stm32l4qx, stm32l4rx, stm32l4sx
            ))]
            pllsai2: None,
            #[cfg(not(any(stm32l471, stm32l475, stm32l476, stm32l486)))]
            hsi48: true,
            clk48_src: Clk48Src::HSI48,
            ls: Default::default(),
        }
    }
}

pub(crate) unsafe fn init(config: Config) {
    // Switch to MSI to prevent problems with PLL configuration.
    if !RCC.cr().read().msion() {
        // Turn on MSI and configure it to 4MHz.
        RCC.cr().modify(|w| {
            w.set_msirgsel(Msirgsel::CR);
            w.set_msirange(MSIRange::RANGE4M);
            w.set_msipllen(false);
            w.set_msion(true)
        });

        // Wait until MSI is running
        while !RCC.cr().read().msirdy() {}
    }
    if RCC.cfgr().read().sws() != ClockSrc::MSI {
        // Set MSI as a clock source, reset prescalers.
        RCC.cfgr().write_value(Cfgr::default());
        // Wait for clock switch status bits to change.
        while RCC.cfgr().read().sws() != ClockSrc::MSI {}
    }

    let rtc = config.ls.init();

    let msi = config.msi.map(|range| {
        // Enable MSI
        RCC.cr().write(|w| {
            w.set_msirange(range);
            w.set_msirgsel(Msirgsel::CR);
            w.set_msion(true);

            // If LSE is enabled, enable calibration of MSI
            w.set_msipllen(config.ls.lse.is_some());
        });
        while !RCC.cr().read().msirdy() {}

        // Enable as clock source for USB, RNG if running at 48 MHz
        if range == MSIRange::RANGE48M {}

        msirange_to_hertz(range)
    });

    let hsi16 = config.hsi16.then(|| {
        RCC.cr().write(|w| w.set_hsion(true));
        while !RCC.cr().read().hsirdy() {}

        HSI_FREQ
    });

    let hse = config.hse.map(|freq| {
        RCC.cr().write(|w| w.set_hseon(true));
        while !RCC.cr().read().hserdy() {}

        freq
    });

    #[cfg(not(any(stm32l47x, stm32l48x)))]
    let hsi48 = config.hsi48.then(|| {
        RCC.crrcr().modify(|w| w.set_hsi48on(true));
        while !RCC.crrcr().read().hsi48rdy() {}

        Hertz(48_000_000)
    });
    #[cfg(any(stm32l47x, stm32l48x))]
    let hsi48 = None;

    let _plls = [
        &config.pll,
        &config.pllsai1,
        #[cfg(any(
            stm32l47x, stm32l48x, stm32l49x, stm32l4ax, stm32l4px, stm32l4qx, stm32l4rx, stm32l4sx
        ))]
        &config.pllsai2,
    ];

    // L4 has shared PLLSRC, PLLM, check it's equal in all PLLs.
    #[cfg(all(stm32l4, not(any(stm32l4px, stm32l4qx, stm32l4rx, stm32l4sx))))]
    match get_equal(_plls.into_iter().flatten().map(|p| (p.source, p.prediv))) {
        Err(()) => panic!("Source must be equal across all enabled PLLs."),
        Ok(None) => {}
        Ok(Some((source, prediv))) => RCC.pllcfgr().write(|w| {
            w.set_pllm(prediv);
            w.set_pllsrc(source);
        }),
    };

    // L4+ has shared PLLSRC, check it's equal in all PLLs.
    #[cfg(any(stm32l4px, stm32l4qx, stm32l4rx, stm32l4sx))]
    match get_equal(_plls.into_iter().flatten().map(|p| p.source)) {
        Err(()) => panic!("Source must be equal across all enabled PLLs."),
        Ok(None) => {}
        Ok(Some(source)) => RCC.pllcfgr().write(|w| {
            w.set_pllsrc(source);
        }),
    };

    let pll_input = PllInput { hse, hsi16, msi };
    let pll = init_pll(PllInstance::Pll, config.pll, &pll_input);
    let pllsai1 = init_pll(PllInstance::Pllsai1, config.pllsai1, &pll_input);
    #[cfg(any(
        stm32l47x, stm32l48x, stm32l49x, stm32l4ax, stm32l4px, stm32l4qx, stm32l4rx, stm32l4sx
    ))]
    let _pllsai2 = init_pll(PllInstance::Pllsai2, config.pllsai2, &pll_input);

    let sys_clk = match config.mux {
        ClockSrc::HSE => hse.unwrap(),
        ClockSrc::HSI16 => hsi16.unwrap(),
        ClockSrc::MSI => msi.unwrap(),
        ClockSrc::PLL => pll._r.unwrap(),
    };

    let _clk48 = match config.clk48_src {
        Clk48Src::HSI48 => hsi48,
        Clk48Src::MSI => msi,
        Clk48Src::PLLSAI1_Q => pllsai1._q,
        Clk48Src::PLL_Q => pll._q,
    };

    #[cfg(any(stm32l4px, stm32l4qx, stm32l4rx, stm32l4sx))]
    assert!(sys_clk.0 <= 120_000_000);
    #[cfg(not(any(stm32l4px, stm32l4qx, stm32l4rx, stm32l4sx)))]
    assert!(sys_clk.0 <= 80_000_000);

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
        w.set_sw(config.mux);
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
        hclk1: ahb_freq,
        hclk2: ahb_freq,
        hclk3: ahb_freq,
        pclk1: apb1_freq,
        pclk2: apb2_freq,
        pclk1_tim: apb1_tim_freq,
        pclk2_tim: apb2_tim_freq,
        rtc,
    });
}

fn msirange_to_hertz(range: MSIRange) -> Hertz {
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

fn get_equal<T: Eq>(mut iter: impl Iterator<Item = T>) -> Result<Option<T>, ()> {
    let Some(x) = iter.next() else { return Ok(None) };
    if !iter.all(|y| y == x) {
        return Err(());
    }
    return Ok(Some(x));
}

struct PllInput {
    hsi16: Option<Hertz>,
    hse: Option<Hertz>,
    msi: Option<Hertz>,
}

#[derive(Default)]
struct PllOutput {
    _p: Option<Hertz>,
    _q: Option<Hertz>,
    _r: Option<Hertz>,
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum PllInstance {
    Pll,
    Pllsai1,
    #[cfg(any(
        stm32l47x, stm32l48x, stm32l49x, stm32l4ax, stm32l4px, stm32l4qx, stm32l4rx, stm32l4sx
    ))]
    Pllsai2,
}

fn init_pll(instance: PllInstance, config: Option<Pll>, input: &PllInput) -> PllOutput {
    // Disable PLL
    match instance {
        PllInstance::Pll => {
            RCC.cr().modify(|w| w.set_pllon(false));
            while RCC.cr().read().pllrdy() {}
        }
        PllInstance::Pllsai1 => {
            RCC.cr().modify(|w| w.set_pllsai1on(false));
            while RCC.cr().read().pllsai1rdy() {}
        }
        #[cfg(any(
            stm32l47x, stm32l48x, stm32l49x, stm32l4ax, stm32l4px, stm32l4qx, stm32l4rx, stm32l4sx
        ))]
        PllInstance::Pllsai2 => {
            RCC.cr().modify(|w| w.set_pllsai2on(false));
            while RCC.cr().read().pllsai2rdy() {}
        }
    }

    let Some(pll) = config else { return PllOutput::default() };

    let pll_src = match pll.source {
        PLLSource::NONE => panic!("must not select PLL source as NONE"),
        PLLSource::HSE => input.hse,
        PLLSource::HSI16 => input.hsi16,
        PLLSource::MSI => input.msi,
    };

    let pll_src = pll_src.unwrap();

    let vco_freq = pll_src / pll.prediv * pll.mul;

    let p = pll.divp.map(|div| vco_freq / div);
    let q = pll.divq.map(|div| vco_freq / div);
    let r = pll.divr.map(|div| vco_freq / div);

    macro_rules! write_fields {
        ($w:ident) => {
            $w.set_plln(pll.mul);
            if let Some(divp) = pll.divp {
                $w.set_pllp(divp);
                $w.set_pllpen(true);
            }
            if let Some(divq) = pll.divq {
                $w.set_pllq(divq);
                $w.set_pllqen(true);
            }
            if let Some(divr) = pll.divr {
                $w.set_pllr(divr);
                $w.set_pllren(true);
            }
        };
    }

    match instance {
        PllInstance::Pll => RCC.pllcfgr().write(|w| {
            w.set_pllm(pll.prediv);
            w.set_pllsrc(pll.source);
            write_fields!(w);
        }),
        PllInstance::Pllsai1 => RCC.pllsai1cfgr().write(|w| {
            #[cfg(any(stm32l4px, stm32l4qx, stm32l4rx, stm32l4sx, stm32l5))]
            w.set_pllm(pll.prediv);
            #[cfg(stm32l5)]
            w.set_pllsrc(pll.source);
            write_fields!(w);
        }),
        #[cfg(any(
            stm32l47x, stm32l48x, stm32l49x, stm32l4ax, stm32l4px, stm32l4qx, stm32l4rx, stm32l4sx
        ))]
        PllInstance::Pllsai2 => RCC.pllsai2cfgr().write(|w| {
            #[cfg(any(stm32l4px, stm32l4qx, stm32l4rx, stm32l4sx, stm32l5))]
            w.set_pllm(pll.prediv);
            #[cfg(stm32l5)]
            w.set_pllsrc(pll.source);
            write_fields!(w);
        }),
    }

    // Enable PLL
    match instance {
        PllInstance::Pll => {
            RCC.cr().modify(|w| w.set_pllon(true));
            while !RCC.cr().read().pllrdy() {}
        }
        PllInstance::Pllsai1 => {
            RCC.cr().modify(|w| w.set_pllsai1on(true));
            while !RCC.cr().read().pllsai1rdy() {}
        }
        #[cfg(any(
            stm32l47x, stm32l48x, stm32l49x, stm32l4ax, stm32l4px, stm32l4qx, stm32l4rx, stm32l4sx
        ))]
        PllInstance::Pllsai2 => {
            RCC.cr().modify(|w| w.set_pllsai2on(true));
            while !RCC.cr().read().pllsai2rdy() {}
        }
    }

    PllOutput { _p: p, _q: q, _r: r }
}
