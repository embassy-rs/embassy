use stm32_metapac::flash::vals::Latency;

pub use crate::pac::rcc::vals::{
    Hpre as AHBPrescaler, Pllm as PllPreDiv, Plln as PllMul, Pllp as PllPDiv, Pllq as PllQDiv, Pllr as PllRDiv,
    Pllsrc as PllSource, Ppre as APBPrescaler, Sw as Sysclk,
};
#[cfg(any(stm32f4, stm32f7))]
use crate::pac::PWR;
use crate::pac::{FLASH, RCC};
use crate::time::Hertz;

// TODO: on some F4s, PLLM is shared between all PLLs. Enforce that.
// TODO: on some F4s, add support for plli2s_src
//
//             plli2s  plli2s_m     plli2s_src   pllsai   pllsai_m
// f401        y       shared
// f410
// f411        y       individual
// f412        y       individual   y
// f4[12]3     y       individual   y
// f446        y       individual                y        individual
// f4[67]9     y       shared                    y        shared
// f4[23][79]  y       shared                    y        shared
// f4[01][57]  y       shared

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
    pub divp: Option<PllPDiv>,
    /// PLL Q division factor. If None, PLL Q output is disabled.
    pub divq: Option<PllQDiv>,
    /// PLL R division factor. If None, PLL R output is disabled.
    pub divr: Option<PllRDiv>,
}

/// Voltage range of the power supply used.
///
/// Used to calculate flash waitstates. See
/// RM0033 - Table 3. Number of wait states according to Cortex®-M3 clock frequency
#[cfg(stm32f2)]
pub enum VoltageScale {
    /// 2.7 to 3.6 V
    Range0,
    /// 2.4 to 2.7 V
    Range1,
    /// 2.1 to 2.4 V
    Range2,
    /// 1.8 to 2.1 V
    Range3,
}

/// Configuration of the core clocks
#[non_exhaustive]
pub struct Config {
    pub hsi: bool,
    pub hse: Option<Hse>,
    pub sys: Sysclk,

    pub pll_src: PllSource,

    pub pll: Option<Pll>,
    #[cfg(any(stm32f2, all(stm32f4, not(stm32f410)), stm32f7))]
    pub plli2s: Option<Pll>,
    #[cfg(any(stm32f446, stm32f427, stm32f437, stm32f4x9, stm32f7))]
    pub pllsai: Option<Pll>,

    pub ahb_pre: AHBPrescaler,
    pub apb1_pre: APBPrescaler,
    pub apb2_pre: APBPrescaler,

    pub ls: super::LsConfig,

    #[cfg(stm32f2)]
    pub voltage: VoltageScale,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            hsi: true,
            hse: None,
            sys: Sysclk::HSI,
            pll_src: PllSource::HSI,
            pll: None,
            #[cfg(any(stm32f2, all(stm32f4, not(stm32f410)), stm32f7))]
            plli2s: None,
            #[cfg(any(stm32f446, stm32f427, stm32f437, stm32f4x9, stm32f7))]
            pllsai: None,

            ahb_pre: AHBPrescaler::DIV1,
            apb1_pre: APBPrescaler::DIV1,
            apb2_pre: APBPrescaler::DIV1,

            ls: Default::default(),

            #[cfg(stm32f2)]
            voltage: VoltageScale::Range3,
        }
    }
}

pub(crate) unsafe fn init(config: Config) {
    // set VOS to SCALE1, if use PLL
    // TODO: check real clock speed before set VOS
    #[cfg(any(stm32f4, stm32f7))]
    if config.pll.is_some() {
        PWR.cr1().modify(|w| w.set_vos(crate::pac::pwr::vals::Vos::SCALE1));
    }

    // always enable overdrive for now. Make it configurable in the future.
    #[cfg(any(stm32f446, stm32f4x9, stm32f427, stm32f437, stm32f7))]
    {
        PWR.cr1().modify(|w| w.set_oden(true));
        while !PWR.csr1().read().odrdy() {}

        PWR.cr1().modify(|w| w.set_odswen(true));
        while !PWR.csr1().read().odswrdy() {}
    }

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
    #[cfg(any(stm32f2, all(stm32f4, not(stm32f410)), stm32f7))]
    let plli2s = init_pll(PllInstance::Plli2s, config.plli2s, &pll_input);
    #[cfg(any(stm32f446, stm32f427, stm32f437, stm32f4x9, stm32f7))]
    let pllsai = init_pll(PllInstance::Pllsai, config.pllsai, &pll_input);

    // Configure sysclk
    let sys = match config.sys {
        Sysclk::HSI => unwrap!(hsi),
        Sysclk::HSE => unwrap!(hse),
        Sysclk::PLL1_P => unwrap!(pll.p),
        _ => unreachable!(),
    };

    let hclk = sys / config.ahb_pre;
    let (pclk1, pclk1_tim) = super::util::calc_pclk(hclk, config.apb1_pre);
    let (pclk2, pclk2_tim) = super::util::calc_pclk(hclk, config.apb2_pre);

    assert!(max::SYSCLK.contains(&sys));
    assert!(max::HCLK.contains(&hclk));
    assert!(max::PCLK1.contains(&pclk1));
    assert!(max::PCLK2.contains(&pclk2));

    let rtc = config.ls.init();

    #[cfg(stm32f2)]
    let latency = match (config.voltage, hclk.0) {
        (VoltageScale::Range3, ..=16_000_000) => Latency::WS0,
        (VoltageScale::Range3, ..=32_000_000) => Latency::WS1,
        (VoltageScale::Range3, ..=48_000_000) => Latency::WS2,
        (VoltageScale::Range3, ..=64_000_000) => Latency::WS3,
        (VoltageScale::Range3, ..=80_000_000) => Latency::WS4,
        (VoltageScale::Range3, ..=96_000_000) => Latency::WS5,
        (VoltageScale::Range3, ..=112_000_000) => Latency::WS6,
        (VoltageScale::Range3, ..=120_000_000) => Latency::WS7,
        (VoltageScale::Range2, ..=18_000_000) => Latency::WS0,
        (VoltageScale::Range2, ..=36_000_000) => Latency::WS1,
        (VoltageScale::Range2, ..=54_000_000) => Latency::WS2,
        (VoltageScale::Range2, ..=72_000_000) => Latency::WS3,
        (VoltageScale::Range2, ..=90_000_000) => Latency::WS4,
        (VoltageScale::Range2, ..=108_000_000) => Latency::WS5,
        (VoltageScale::Range2, ..=120_000_000) => Latency::WS6,
        (VoltageScale::Range1, ..=24_000_000) => Latency::WS0,
        (VoltageScale::Range1, ..=48_000_000) => Latency::WS1,
        (VoltageScale::Range1, ..=72_000_000) => Latency::WS2,
        (VoltageScale::Range1, ..=96_000_000) => Latency::WS3,
        (VoltageScale::Range1, ..=120_000_000) => Latency::WS4,
        (VoltageScale::Range0, ..=30_000_000) => Latency::WS0,
        (VoltageScale::Range0, ..=60_000_000) => Latency::WS1,
        (VoltageScale::Range0, ..=90_000_000) => Latency::WS2,
        (VoltageScale::Range0, ..=120_000_000) => Latency::WS3,
        _ => unreachable!(),
    };

    #[cfg(any(stm32f4, stm32f7))]
    let latency = {
        // Be conservative with voltage ranges
        const FLASH_LATENCY_STEP: u32 = 30_000_000;

        let latency = (hclk.0 - 1) / FLASH_LATENCY_STEP;
        debug!("flash: latency={}", latency);

        Latency::from_bits(latency as u8)
    };

    FLASH.acr().write(|w| w.set_latency(latency));
    while FLASH.acr().read().latency() != latency {}

    RCC.cfgr().modify(|w| {
        w.set_sw(config.sys);
        w.set_hpre(config.ahb_pre);
        w.set_ppre1(config.apb1_pre);
        w.set_ppre2(config.apb2_pre);
    });
    while RCC.cfgr().read().sws() != config.sys {}

    set_clocks!(
        hsi: hsi,
        hse: hse,
        lse: None, // TODO
        lsi: None, // TODO
        sys: Some(sys),
        hclk1: Some(hclk),
        hclk2: Some(hclk),
        hclk3: Some(hclk),
        pclk1: Some(pclk1),
        pclk2: Some(pclk2),
        pclk1_tim: Some(pclk1_tim),
        pclk2_tim: Some(pclk2_tim),
        rtc: rtc,
        pll1_q: pll.q,

        #[cfg(any(stm32f2, all(stm32f4, not(stm32f410)), stm32f7))]
        plli2s1_p: plli2s.p,
        #[cfg(any(stm32f2, all(stm32f4, not(stm32f410)), stm32f7))]
        plli2s1_q: plli2s.q,
        #[cfg(any(stm32f2, all(stm32f4, not(stm32f410)), stm32f7))]
        plli2s1_r: plli2s.r,

        #[cfg(any(stm32f446, stm32f427, stm32f437, stm32f4x9, stm32f7))]
        pllsai1_p: pllsai.p,
        #[cfg(any(stm32f446, stm32f427, stm32f437, stm32f4x9, stm32f7))]
        pllsai1_q: pllsai.q,
        #[cfg(any(stm32f446, stm32f427, stm32f437, stm32f4x9, stm32f7))]
        pllsai1_r: pllsai.r,

        clk48: pll.q,

        hsi_hse: None,
        afif: None,
    );
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
    #[cfg(any(stm32f2, all(stm32f4, not(stm32f410)), stm32f7))]
    Plli2s,
    #[cfg(any(stm32f446, stm32f427, stm32f437, stm32f4x9, stm32f7))]
    Pllsai,
}

fn pll_enable(instance: PllInstance, enabled: bool) {
    match instance {
        PllInstance::Pll => {
            RCC.cr().modify(|w| w.set_pllon(enabled));
            while RCC.cr().read().pllrdy() != enabled {}
        }
        #[cfg(any(stm32f2, all(stm32f4, not(stm32f410)), stm32f7))]
        PllInstance::Plli2s => {
            RCC.cr().modify(|w| w.set_plli2son(enabled));
            while RCC.cr().read().plli2srdy() != enabled {}
        }
        #[cfg(any(stm32f446, stm32f427, stm32f437, stm32f4x9, stm32f7))]
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

    // stm32f2 plls are like swiss cheese
    #[cfg(stm32f2)]
    match instance {
        PllInstance::Pll => {
            assert!(pll.divr.is_none());
        }
        PllInstance::Plli2s => {
            assert!(pll.divp.is_none());
            assert!(pll.divq.is_none());
        }
    }

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
            #[cfg(any(stm32f4, stm32f7))]
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
        #[cfg(any(all(stm32f4, not(stm32f410)), stm32f7))]
        PllInstance::Plli2s => RCC.plli2scfgr().write(|w| {
            write_fields!(w);
        }),
        #[cfg(stm32f2)]
        PllInstance::Plli2s => RCC.plli2scfgr().write(|w| {
            if let Some(divr) = pll.divr {
                w.set_pllr(divr);
            }
        }),
        #[cfg(any(stm32f446, stm32f427, stm32f437, stm32f4x9, stm32f7))]
        PllInstance::Pllsai => RCC.pllsaicfgr().write(|w| {
            write_fields!(w);
        }),
    }

    // Enable PLL
    pll_enable(instance, true);

    PllOutput { p, q, r }
}

#[cfg(stm32f7)]
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

#[cfg(stm32f4)]
mod max {
    use core::ops::RangeInclusive;

    use crate::time::Hertz;

    pub(crate) const HSE_OSC: RangeInclusive<Hertz> = Hertz(4_000_000)..=Hertz(26_000_000);
    pub(crate) const HSE_BYP: RangeInclusive<Hertz> = Hertz(1_000_000)..=Hertz(50_000_000);

    #[cfg(stm32f401)]
    pub(crate) const SYSCLK: RangeInclusive<Hertz> = Hertz(0)..=Hertz(84_000_000);
    #[cfg(any(stm32f405, stm32f407, stm32f415, stm32f417,))]
    pub(crate) const SYSCLK: RangeInclusive<Hertz> = Hertz(0)..=Hertz(168_000_000);
    #[cfg(any(stm32f410, stm32f411, stm32f412, stm32f413, stm32f423,))]
    pub(crate) const SYSCLK: RangeInclusive<Hertz> = Hertz(0)..=Hertz(100_000_000);
    #[cfg(any(stm32f427, stm32f429, stm32f437, stm32f439, stm32f446, stm32f469, stm32f479,))]
    pub(crate) const SYSCLK: RangeInclusive<Hertz> = Hertz(0)..=Hertz(180_000_000);

    pub(crate) const HCLK: RangeInclusive<Hertz> = Hertz(0)..=Hertz(SYSCLK.end().0);

    pub(crate) const PCLK1: RangeInclusive<Hertz> = Hertz(0)..=Hertz(PCLK2.end().0 / 2);

    #[cfg(any(stm32f401, stm32f410, stm32f411, stm32f412, stm32f413, stm32f423,))]
    pub(crate) const PCLK2: RangeInclusive<Hertz> = Hertz(0)..=Hertz(HCLK.end().0);
    #[cfg(not(any(stm32f401, stm32f410, stm32f411, stm32f412, stm32f413, stm32f423,)))]
    pub(crate) const PCLK2: RangeInclusive<Hertz> = Hertz(0)..=Hertz(HCLK.end().0 / 2);

    pub(crate) const PLL_IN: RangeInclusive<Hertz> = Hertz(1_000_000)..=Hertz(2_100_000);
    pub(crate) const PLL_VCO: RangeInclusive<Hertz> = Hertz(100_000_000)..=Hertz(432_000_000);
}

#[cfg(stm32f2)]
mod max {
    use core::ops::RangeInclusive;

    use crate::time::Hertz;

    pub(crate) const HSE_OSC: RangeInclusive<Hertz> = Hertz(4_000_000)..=Hertz(26_000_000);
    pub(crate) const HSE_BYP: RangeInclusive<Hertz> = Hertz(1_000_000)..=Hertz(26_000_000);

    pub(crate) const SYSCLK: RangeInclusive<Hertz> = Hertz(0)..=Hertz(120_000_000);

    pub(crate) const HCLK: RangeInclusive<Hertz> = Hertz(0)..=Hertz(SYSCLK.end().0);
    pub(crate) const PCLK1: RangeInclusive<Hertz> = Hertz(0)..=Hertz(SYSCLK.end().0 / 4);
    pub(crate) const PCLK2: RangeInclusive<Hertz> = Hertz(0)..=Hertz(SYSCLK.end().0 / 2);

    pub(crate) const PLL_IN: RangeInclusive<Hertz> = Hertz(0_950_000)..=Hertz(2_100_000);
    pub(crate) const PLL_VCO: RangeInclusive<Hertz> = Hertz(192_000_000)..=Hertz(432_000_000);
}
