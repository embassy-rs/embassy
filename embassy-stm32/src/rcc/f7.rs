pub use crate::pac::rcc::vals::{
    Hpre as AHBPrescaler, Pllm as PllPreDiv, Plln as PllMul, Pllp, Pllq, Pllr, Pllsrc as PllSource,
    Ppre as APBPrescaler, Sw as Sysclk,
};
use crate::pac::{FLASH, PWR, RCC};
use crate::rcc::{set_freqs, Clocks};
use crate::time::Hertz;

/// HSI speed
pub const HSI_FREQ: Hertz = Hertz(16_000_000);

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum HseMode {
    /// crystal/ceramic oscillator (HSEBYP=0)
    Oscillator,
    /// external analog clock (low swing) (HSEBYP=1)
    Bypass,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Hse {
    /// HSE frequency.
    pub freq: Hertz,
    /// HSE mode.
    pub mode: HseMode,
}

#[derive(Clone, Copy)]
pub struct Pll {
    /// PLL pre-divider (DIVM).
    pub prediv: PllPreDiv,

    /// PLL multiplication factor.
    pub mul: PllMul,

    /// PLL P division factor. If None, PLL P output is disabled.
    pub divp: Option<Pllp>,
    /// PLL Q division factor. If None, PLL Q output is disabled.
    pub divq: Option<Pllq>,
    /// PLL R division factor. If None, PLL R output is disabled.
    pub divr: Option<Pllr>,
}

/// Configuration of the core clocks
#[non_exhaustive]
pub struct Config {
    pub hsi: bool,
    pub hse: Option<Hse>,
    pub sys: Sysclk,

    pub pll_src: PllSource,

    pub pll: Option<Pll>,
    pub plli2s: Option<Pll>,
    pub pllsai: Option<Pll>,

    pub ahb_pre: AHBPrescaler,
    pub apb1_pre: APBPrescaler,
    pub apb2_pre: APBPrescaler,

    pub ls: super::LsConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            hsi: true,
            hse: None,
            sys: Sysclk::HSI,
            pll_src: PllSource::HSI,
            pll: None,
            plli2s: None,
            pllsai: None,

            ahb_pre: AHBPrescaler::DIV1,
            apb1_pre: APBPrescaler::DIV1,
            apb2_pre: APBPrescaler::DIV1,

            ls: Default::default(),
        }
    }
}

pub(crate) unsafe fn init(config: Config) {
    // always enable overdrive for now. Make it configurable in the future.
    PWR.cr1().modify(|w| w.set_oden(true));
    while !PWR.csr1().read().odrdy() {}

    PWR.cr1().modify(|w| w.set_odswen(true));
    while !PWR.csr1().read().odswrdy() {}

    // Configure HSI
    let hsi = match config.hsi {
        false => {
            RCC.cr().modify(|w| w.set_hsion(false));
            None
        }
        true => {
            RCC.cr().modify(|w| w.set_hsion(true));
            while !RCC.cr().read().hsirdy() {}
            Some(HSI_FREQ)
        }
    };

    // Configure HSE
    let hse = match config.hse {
        None => {
            RCC.cr().modify(|w| w.set_hseon(false));
            None
        }
        Some(hse) => {
            match hse.mode {
                HseMode::Bypass => assert!(max::HSE_BYP.contains(&hse.freq)),
                HseMode::Oscillator => assert!(max::HSE_OSC.contains(&hse.freq)),
            }

            RCC.cr().modify(|w| w.set_hsebyp(hse.mode != HseMode::Oscillator));
            RCC.cr().modify(|w| w.set_hseon(true));
            while !RCC.cr().read().hserdy() {}
            Some(hse.freq)
        }
    };

    // Configure PLLs.
    let pll_input = PllInput {
        hse,
        hsi,
        source: config.pll_src,
    };
    let pll = init_pll(PllInstance::Pll, config.pll, &pll_input);
    let _plli2s = init_pll(PllInstance::Plli2s, config.plli2s, &pll_input);
    let _pllsai = init_pll(PllInstance::Pllsai, config.pllsai, &pll_input);

    // Configure sysclk
    let sys = match config.sys {
        Sysclk::HSI => unwrap!(hsi),
        Sysclk::HSE => unwrap!(hse),
        Sysclk::PLL1_P => unwrap!(pll.p),
        _ => unreachable!(),
    };

    let hclk = sys / config.ahb_pre;
    let (pclk1, pclk1_tim) = calc_pclk(hclk, config.apb1_pre);
    let (pclk2, pclk2_tim) = calc_pclk(hclk, config.apb2_pre);

    assert!(max::SYSCLK.contains(&sys));
    assert!(max::HCLK.contains(&hclk));
    assert!(max::PCLK1.contains(&pclk1));
    assert!(max::PCLK2.contains(&pclk2));

    let rtc = config.ls.init();

    flash_setup(hclk);

    RCC.cfgr().modify(|w| {
        w.set_sw(config.sys);
        w.set_hpre(config.ahb_pre);
        w.set_ppre1(config.apb1_pre);
        w.set_ppre2(config.apb2_pre);
    });
    while RCC.cfgr().read().sws() != config.sys {}

    set_freqs(Clocks {
        sys,
        hclk1: hclk,
        hclk2: hclk,
        hclk3: hclk,
        pclk1,
        pclk2,
        pclk1_tim,
        pclk2_tim,
        rtc,
        pll1_q: pll.q,
    });
}

struct PllInput {
    source: PllSource,
    hsi: Option<Hertz>,
    hse: Option<Hertz>,
}

#[derive(Default)]
#[allow(unused)]
struct PllOutput {
    p: Option<Hertz>,
    q: Option<Hertz>,
    r: Option<Hertz>,
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum PllInstance {
    Pll,
    Plli2s,
    Pllsai,
}

fn pll_enable(instance: PllInstance, enabled: bool) {
    match instance {
        PllInstance::Pll => {
            RCC.cr().modify(|w| w.set_pllon(enabled));
            while RCC.cr().read().pllrdy() != enabled {}
        }
        PllInstance::Plli2s => {
            RCC.cr().modify(|w| w.set_plli2son(enabled));
            while RCC.cr().read().plli2srdy() != enabled {}
        }
        PllInstance::Pllsai => {
            RCC.cr().modify(|w| w.set_pllsaion(enabled));
            while RCC.cr().read().pllsairdy() != enabled {}
        }
    }
}

fn init_pll(instance: PllInstance, config: Option<Pll>, input: &PllInput) -> PllOutput {
    // Disable PLL
    pll_enable(instance, false);

    let Some(pll) = config else { return PllOutput::default() };

    let pll_src = match input.source {
        PllSource::HSE => input.hse,
        PllSource::HSI => input.hsi,
    };

    let pll_src = pll_src.unwrap();

    let in_freq = pll_src / pll.prediv;
    assert!(max::PLL_IN.contains(&in_freq));
    let vco_freq = in_freq * pll.mul;
    assert!(max::PLL_VCO.contains(&vco_freq));

    let p = pll.divp.map(|div| vco_freq / div);
    let q = pll.divq.map(|div| vco_freq / div);
    let r = pll.divr.map(|div| vco_freq / div);

    macro_rules! write_fields {
        ($w:ident) => {
            $w.set_plln(pll.mul);
            if let Some(divp) = pll.divp {
                $w.set_pllp(divp);
            }
            if let Some(divq) = pll.divq {
                $w.set_pllq(divq);
            }
            if let Some(divr) = pll.divr {
                $w.set_pllr(divr);
            }
        };
    }

    match instance {
        PllInstance::Pll => RCC.pllcfgr().write(|w| {
            w.set_pllm(pll.prediv);
            w.set_pllsrc(input.source);
            write_fields!(w);
        }),
        PllInstance::Plli2s => RCC.plli2scfgr().write(|w| {
            write_fields!(w);
        }),
        PllInstance::Pllsai => RCC.pllsaicfgr().write(|w| {
            write_fields!(w);
        }),
    }

    // Enable PLL
    pll_enable(instance, true);

    PllOutput { p, q, r }
}

fn flash_setup(clk: Hertz) {
    use crate::pac::flash::vals::Latency;

    // Be conservative with voltage ranges
    const FLASH_LATENCY_STEP: u32 = 30_000_000;

    let latency = (clk.0 - 1) / FLASH_LATENCY_STEP;
    debug!("flash: latency={}", latency);

    let latency = Latency::from_bits(latency as u8);
    FLASH.acr().write(|w| {
        w.set_latency(latency);
    });
    while FLASH.acr().read().latency() != latency {}
}

fn calc_pclk<D>(hclk: Hertz, ppre: D) -> (Hertz, Hertz)
where
    Hertz: core::ops::Div<D, Output = Hertz>,
{
    let pclk = hclk / ppre;
    let pclk_tim = if hclk == pclk { pclk } else { pclk * 2u32 };
    (pclk, pclk_tim)
}

mod max {
    use core::ops::RangeInclusive;

    use crate::time::Hertz;

    pub(crate) const HSE_OSC: RangeInclusive<Hertz> = Hertz(4_000_000)..=Hertz(26_000_000);
    pub(crate) const HSE_BYP: RangeInclusive<Hertz> = Hertz(1_000_000)..=Hertz(50_000_000);

    pub(crate) const SYSCLK: RangeInclusive<Hertz> = Hertz(12_500_000)..=Hertz(216_000_000);
    pub(crate) const HCLK: RangeInclusive<Hertz> = Hertz(12_500_000)..=Hertz(216_000_000);
    pub(crate) const PCLK1: RangeInclusive<Hertz> = Hertz(12_500_000)..=Hertz(216_000_000 / 4);
    pub(crate) const PCLK2: RangeInclusive<Hertz> = Hertz(12_500_000)..=Hertz(216_000_000 / 2);

    pub(crate) const PLL_IN: RangeInclusive<Hertz> = Hertz(1_000_000)..=Hertz(2_100_000);
    pub(crate) const PLL_VCO: RangeInclusive<Hertz> = Hertz(100_000_000)..=Hertz(432_000_000);
}
