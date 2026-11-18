use core::ops::RangeInclusive;

use crate::pac::rcc::vals::{Hseext, Pllrge, Pllvcosel};
pub use crate::pac::rcc::vals::{
    Hsidiv as HSIPrescaler, Plldiv as PllDiv, Pllm as PllPreDiv, Plln as PllMul, Pllsrc as PllSource, Syssw as Sysclk,
};
use crate::pac::{PWR, RCC};
use crate::time::Hertz;

/// HSI speed
pub const HSI_FREQ: Hertz = Hertz(64_000_000);

const VCO_RANGE: RangeInclusive<Hertz> = Hertz(150_000_000)..=Hertz(420_000_000);
const VCO_WIDE_RANGE: RangeInclusive<Hertz> = Hertz(128_000_000)..=Hertz(560_000_000);

pub use crate::pac::pwr::vals::Vos as VoltageScale;
pub use crate::pac::rcc::vals::{Hpre as AHBPrescaler, Ppre as APBPrescaler};

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum HseMode {
    /// crystal/ceramic oscillator (HSEBYP=0)
    Oscillator,
    /// external analog clock (low swing) (HSEBYP=1, HSEEXT=0)
    Bypass,
    /// external digital clock (full swing) (HSEBYP=1, HSEEXT=1)
    BypassDigital,
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
    /// Source clock selection.
    pub source: PllSource,

    /// PLL pre-divider (DIVM).
    pub prediv: PllPreDiv,

    /// PLL multiplication factor.
    pub mul: PllMul,

    /// PLL P division factor. If None, PLL P output is disabled.
    /// On PLL1, it must be even
    pub divp: Option<PllDiv>,
    /// PLL Q division factor. If None, PLL Q output is disabled.
    pub divq: Option<PllDiv>,
    /// PLL R division factor. If None, PLL R output is disabled.
    pub divr: Option<PllDiv>,
}

/// Configuration of the core clocks
#[non_exhaustive]
#[derive(Clone, Copy)]
pub struct Config {
    pub hsi: Option<HSIPrescaler>,
    pub hse: Option<Hse>,
    pub sys: Sysclk,

    pub pll1: Option<Pll>,
    pub pll2: Option<Pll>,

    pub ahb_pre: AHBPrescaler,
    pub apb1_pre: APBPrescaler,
    pub apb2_pre: APBPrescaler,
    pub apb3_pre: APBPrescaler,

    pub voltage_scale: VoltageScale,

    /// Per-peripheral kernel clock selection muxes
    pub mux: super::mux::ClockMux,
}

impl Config {
    pub const fn new() -> Self {
        Self {
            hsi: Some(HSIPrescaler::Div1),
            hse: None,
            sys: Sysclk::Hsi,
            pll1: None,
            pll2: None,

            ahb_pre: AHBPrescaler::Div2,
            apb1_pre: APBPrescaler::Div2,
            apb2_pre: APBPrescaler::Div2,
            apb3_pre: APBPrescaler::Div2,

            voltage_scale: VoltageScale::Scale0,
            mux: super::mux::ClockMux::default(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

pub(crate) unsafe fn init(config: Config) {
    // Configure voltage scale.
    PWR.voscr().modify(|w| w.set_vos(config.voltage_scale));
    while !PWR.vossr().read().vosrdy() {}

    // Turn on the HSI
    match config.hsi {
        None => RCC.cr().modify(|w| w.set_hsion(true)),
        Some(hsidiv) => RCC.cr().modify(|w| {
            w.set_hsidiv(hsidiv);
            w.set_hsion(true);
        }),
    }
    while !RCC.cr().read().hsirdy() {}

    // Use the HSI clock as system clock during the actual clock setup
    RCC.cfgr1().modify(|w| w.set_syssw(Sysclk::Hsi));
    while RCC.cfgr1().read().syssws() != Sysclk::Hsi {}

    // Configure HSI
    let hsi = config.hsi.map(|hsidiv| HSI_FREQ / hsidiv);

    // Configure HSE
    let hse = match config.hse {
        None => {
            RCC.cr().modify(|w| w.set_hseon(false));
            None
        }
        Some(hse) => {
            RCC.cr().modify(|w| {
                w.set_hsebyp(hse.mode != HseMode::Oscillator);
                w.set_hseext(match hse.mode {
                    HseMode::Oscillator | HseMode::Bypass => Hseext::Analog,
                    HseMode::BypassDigital => Hseext::Digital,
                });
            });
            RCC.cr().modify(|w| w.set_hseon(true));
            while !RCC.cr().read().hserdy() {}
            Some(hse.freq)
        }
    };

    // Configure PLLs.
    let pll_input = PllInput { hse, hsi };
    disable_pll(0);
    let pll1 = config
        .pll1
        .map_or_else(PllOutput::default, |c| init_pll(0, c, &pll_input));
    disable_pll(1);
    let pll2 = config
        .pll2
        .map_or_else(PllOutput::default, |c| init_pll(1, c, &pll_input));

    // Configure sysclk
    let sys = match config.sys {
        Sysclk::Hsi => unwrap!(hsi),
        Sysclk::Hse => unwrap!(hse),
        Sysclk::Pll1P => unwrap!(pll1.p),
        Sysclk::_RESERVED_3 => unreachable!(),
    };

    // Check limits.
    let hclk_pclk_max = match config.voltage_scale {
        VoltageScale::Scale0 => Hertz(280_000_000),
        VoltageScale::Scale1 => Hertz(240_000_000),
        VoltageScale::Scale2 => Hertz(170_000_000),
        VoltageScale::_RESERVED_0 => unreachable!(),
    };

    let hclk = sys / config.ahb_pre;
    assert!(hclk <= hclk_pclk_max);

    let apb1 = hclk / config.apb1_pre;
    assert!(apb1 <= hclk_pclk_max);
    let apb2 = hclk / config.apb2_pre;
    assert!(apb2 <= hclk_pclk_max);
    let apb3 = hclk / config.apb3_pre;
    assert!(apb3 <= hclk_pclk_max);

    flash_setup(hclk, config.voltage_scale);

    {
        // Set hpre
        RCC.cfgr2().modify(|w| w.set_hpre(config.ahb_pre));
        while RCC.cfgr2().read().hpre() != config.ahb_pre {}

        // set ppre
        RCC.cfgr2().modify(|w| {
            w.set_ppre1(config.apb1_pre);
            w.set_ppre2(config.apb2_pre);
            w.set_ppre3(config.apb3_pre);
        });
    }

    RCC.cfgr1().modify(|w| w.set_syssw(config.sys));
    while RCC.cfgr1().read().syssws() != config.sys {}

    // Disable HSI if not used
    if config.hsi.is_none() {
        RCC.cr().modify(|w| w.set_hsion(false));
    }

    config.mux.init();

    set_clocks!(
        sys: Some(sys),
        hclk1: Some(hclk),
        hclk2: Some(hclk),
        hclk3: Some(hclk),
        hclk4: Some(hclk),
        hclk5: Some(hclk),
        pclk1: Some(apb1),
        pclk2: Some(apb2),
        pclk3: Some(apb3),

        pclk1_tim: None,
        pclk2_tim: None,
        pclk3_tim: None,

        hsi: hsi,
        hsi48: None,
        hse: hse,

        ic1: None,
        ic2: None,

        rtc: None,

        lse: None,
        lsi: None,

        pll1_q: pll1.q,
        pll2_p: pll2.p,
        pll2_q: pll2.q,
        pll2_r: pll2.r,

        audioclk: None,
        i2s_ckin: None,
        usb: Some(Hertz(48_000_000)),
    );
}

struct PllInput {
    hsi: Option<Hertz>,
    hse: Option<Hertz>,
}

#[derive(Default)]
struct PllOutput {
    p: Option<Hertz>,
    q: Option<Hertz>,
    r: Option<Hertz>,
}

fn disable_pll(num: usize) {
    // Stop PLL
    RCC.cr().modify(|w| w.set_pllon(num, false));
    while RCC.cr().read().pllrdy(num) {}

    RCC.pllcfgr(num).write(|w| w.set_pllm(PllPreDiv::from_bits(0)));
}

fn init_pll(num: usize, config: Pll, input: &PllInput) -> PllOutput {
    let in_clk = match config.source {
        PllSource::Disable => panic!("must not set PllSource::Disable"),
        PllSource::Hsi => unwrap!(input.hsi),
        PllSource::Hse => unwrap!(input.hse),
        PllSource::_RESERVED_3 => unreachable!(),
    };

    let ref_clk = in_clk / config.prediv as u32;

    let ref_range = match ref_clk.0 {
        ..=1_999_999 => Pllrge::Range1,
        ..=3_999_999 => Pllrge::Range2,
        ..=7_999_999 => Pllrge::Range4,
        ..=16_000_000 => Pllrge::Range8,
        x => panic!("pll ref_clk out of range: {} hz", x),
    };

    // The smaller range (150 to 420 MHz) must
    // be chosen when the reference clock frequency is lower than 2 MHz.
    let wide_allowed = ref_range != Pllrge::Range1;
    let vco_clk = ref_clk * config.mul;

    let vco_range = if VCO_RANGE.contains(&vco_clk) {
        Pllvcosel::MediumVco
    } else if wide_allowed && VCO_WIDE_RANGE.contains(&vco_clk) {
        Pllvcosel::WideVco
    } else {
        panic!("pll vco_clk out of range: {}", vco_clk)
    };

    let p = config.divp.map(|div| {
        if num == 0 {
            // on PLL1, DIVP must be even.
            // The enum value is 1 less than the divider, so check it's odd.
            assert!(div.to_bits() % 2 == 1);
        }

        vco_clk / div
    });
    let q = config.divq.map(|div| vco_clk / div);
    let r = config.divr.map(|div| vco_clk / div);

    RCC.pllcfgr(num).write(|w| {
        w.set_pllsrc(config.source);
        w.set_pllm(config.prediv);
        w.set_pllvcosel(vco_range);
        w.set_pllrge(ref_range);
        w.set_pllfracen(false);
        w.set_pllpen(p.is_some());
        w.set_pllqen(q.is_some());
        w.set_pllren(r.is_some());
    });

    RCC.plldivr(num).write(|w| {
        w.set_plln(config.mul);
        w.set_pllp(config.divp.unwrap_or(PllDiv::Div2));
        w.set_pllq(config.divq.unwrap_or(PllDiv::Div2));
        w.set_pllr(config.divr.unwrap_or(PllDiv::Div2));
    });

    RCC.cr().modify(|w| w.set_pllon(num, true));
    while !RCC.cr().read().pllrdy(num) {}

    PllOutput { p, q, r }
}

fn flash_setup(clk: Hertz, vos: VoltageScale) {
    // RM0514 Rev 1, table 39
    // LATENCY  WRHIGHFREQ  VOS3           VOS2            VOS1            VOS0
    //      0           0   0 to 20 MHz    0 to 30 MHz     0 to 34 MHz     0 to 42 MHz
    //      1           0   20 to 40 MHz   30 to 60 MHz    34 to 68 MHz    42 to 84 MHz
    //      2           1   40 to 60 MHz   60 to 90 MHz    68 to 102 MHz   84 to 126 MHz
    //      3           1   60 to 80 MHz   90 to 120 MHz   102 to 136 MHz  126 to 168 MHz
    //      4           2   80 to 100 MHz  120 to 150 MHz  136 to 171 MHz  168 to 210 MHz
    //      5           2                                  171 to 205 MHz  210 to 252 MHz
    //      6           3                                  205 to 240 MHz  252 to 280 MHz
    let (latency, wrhighfreq) = match (vos, clk.0) {
        (VoltageScale::Scale0, ..=42_000_000) => (0, 0),
        (VoltageScale::Scale0, ..=84_000_000) => (1, 0),
        (VoltageScale::Scale0, ..=126_000_000) => (2, 1),
        (VoltageScale::Scale0, ..=168_000_000) => (3, 1),
        (VoltageScale::Scale0, ..=210_000_000) => (4, 2),
        (VoltageScale::Scale0, ..=252_000_000) => (5, 2),
        (VoltageScale::Scale0, ..=280_000_000) => (6, 3),

        (VoltageScale::Scale1, ..=34_000_000) => (0, 0),
        (VoltageScale::Scale1, ..=68_000_000) => (1, 0),
        (VoltageScale::Scale1, ..=102_000_000) => (2, 1),
        (VoltageScale::Scale1, ..=136_000_000) => (3, 1),
        (VoltageScale::Scale1, ..=171_000_000) => (4, 2),
        (VoltageScale::Scale1, ..=205_000_000) => (5, 2),
        (VoltageScale::Scale1, ..=240_000_000) => (6, 3),

        (VoltageScale::Scale2, ..=30_000_000) => (0, 0),
        (VoltageScale::Scale2, ..=60_000_000) => (1, 0),
        (VoltageScale::Scale2, ..=90_000_000) => (2, 1),
        (VoltageScale::Scale2, ..=120_000_000) => (3, 1),
        (VoltageScale::Scale2, ..=150_000_000) => (4, 2),
        (VoltageScale::Scale2, ..=170_000_000) => (5, 2),

        _ => unreachable!(),
    };

    debug!("flash: latency={} wrhighfreq={}", latency, wrhighfreq);

    /*FLASH.acr().write(|w| {
        w.set_wrhighfreq(wrhighfreq);
        w.set_latency(latency);
    });
    while FLASH.acr().read().latency() != latency {}*/
}
