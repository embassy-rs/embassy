use core::marker::PhantomData;

use stm32_metapac::rcc::vals::Timpre;

use crate::pac::rcc::vals::{Hseext, Hsidiv, Mco1, Mco2, Pllrge, Pllsrc, Pllvcosel, Sw};
use crate::pac::{FLASH, PWR, RCC};
use crate::rcc::{set_freqs, Clocks};
use crate::time::Hertz;
use crate::{peripherals, Peripheral};

/// HSI speed
pub const HSI_FREQ: Hertz = Hertz(64_000_000);

/// CSI speed
pub const CSI_FREQ: Hertz = Hertz(4_000_000);

/// HSI48 speed
pub const HSI48_FREQ: Hertz = Hertz(48_000_000);

/// LSI speed
pub const LSI_FREQ: Hertz = Hertz(32_000);

const VCO_MIN: u32 = 150_000_000;
const VCO_MAX: u32 = 420_000_000;
const VCO_WIDE_MIN: u32 = 128_000_000;
const VCO_WIDE_MAX: u32 = 560_000_000;

pub use super::bus::{AHBPrescaler, APBPrescaler};
pub use crate::pac::pwr::vals::Vos as VoltageScale;

pub enum HseMode {
    /// crystal/ceramic oscillator (HSEBYP=0)
    Oscillator,
    ///  external analog clock (low swing) (HSEBYP=1, HSEEXT=0)
    BypassAnalog,
    ///  external digital clock (full swing) (HSEBYP=1, HSEEXT=1)
    BypassDigital,
}

pub struct Hse {
    /// HSE frequency.
    pub freq: Hertz,
    /// HSE mode.
    pub mode: HseMode,
}

pub enum Hsi {
    /// 64Mhz
    Mhz64,
    /// 32Mhz (divided by 2)
    Mhz32,
    /// 16Mhz (divided by 4)
    Mhz16,
    /// 8Mhz (divided by 8)
    Mhz8,
}

pub enum Sysclk {
    /// HSI selected as sysclk
    HSI,
    /// HSE selected as sysclk
    HSE,
    /// CSI selected as sysclk
    CSI,
    /// PLL1_P selected as sysclk
    Pll1P,
}

pub enum PllSource {
    Hsi,
    Csi,
    Hse,
}

pub struct Pll {
    /// Source clock selection.
    pub source: PllSource,

    /// PLL pre-divider (DIVM). Must be between 1 and 63.
    pub prediv: u8,

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

fn apb_div_tim(apb: &APBPrescaler, clk: Hertz, tim: TimerPrescaler) -> Hertz {
    match (tim, apb) {
        // The timers kernel clock is equal to rcc_hclk1 if PPRE1 or PPRE2 corresponds to a
        // division by 1 or 2, else it is equal to 2 x Frcc_pclk1 or 2 x Frcc_pclk2
        (TimerPrescaler::DefaultX2, APBPrescaler::DIV1) => clk,
        (TimerPrescaler::DefaultX2, APBPrescaler::DIV2) => clk,
        (TimerPrescaler::DefaultX2, APBPrescaler::DIV4) => clk / 2u32,
        (TimerPrescaler::DefaultX2, APBPrescaler::DIV8) => clk / 4u32,
        (TimerPrescaler::DefaultX2, APBPrescaler::DIV16) => clk / 8u32,
        // The timers kernel clock is equal to 2 x Frcc_pclk1 or 2 x Frcc_pclk2 if PPRE1 or PPRE2
        // corresponds to a division by 1, 2 or 4, else it is equal to 4 x Frcc_pclk1 or 4 x Frcc_pclk2
        // this makes NO SENSE and is different than in the H7. Mistake in the RM??
        (TimerPrescaler::DefaultX4, APBPrescaler::DIV1) => clk * 2u32,
        (TimerPrescaler::DefaultX4, APBPrescaler::DIV2) => clk,
        (TimerPrescaler::DefaultX4, APBPrescaler::DIV4) => clk / 2u32,
        (TimerPrescaler::DefaultX4, APBPrescaler::DIV8) => clk / 2u32,
        (TimerPrescaler::DefaultX4, APBPrescaler::DIV16) => clk / 4u32,

        _ => unreachable!(),
    }
}

/// APB prescaler
#[derive(Clone, Copy)]
pub enum TimerPrescaler {
    DefaultX2,
    DefaultX4,
}

impl From<TimerPrescaler> for Timpre {
    fn from(value: TimerPrescaler) -> Self {
        match value {
            TimerPrescaler::DefaultX2 => Timpre::DEFAULTX2,
            TimerPrescaler::DefaultX4 => Timpre::DEFAULTX4,
        }
    }
}

/// Configuration of the core clocks
#[non_exhaustive]
pub struct Config {
    pub hsi: Option<Hsi>,
    pub hse: Option<Hse>,
    pub csi: bool,
    pub hsi48: bool,
    pub sys: Sysclk,

    pub pll1: Option<Pll>,
    pub pll2: Option<Pll>,
    #[cfg(rcc_h5)]
    pub pll3: Option<Pll>,

    pub ahb_pre: AHBPrescaler,
    pub apb1_pre: APBPrescaler,
    pub apb2_pre: APBPrescaler,
    pub apb3_pre: APBPrescaler,
    pub timer_prescaler: TimerPrescaler,

    pub voltage_scale: VoltageScale,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            hsi: Some(Hsi::Mhz64),
            hse: None,
            csi: false,
            hsi48: false,
            sys: Sysclk::HSI,
            pll1: None,
            pll2: None,
            #[cfg(rcc_h5)]
            pll3: None,

            ahb_pre: AHBPrescaler::DIV1,
            apb1_pre: APBPrescaler::DIV1,
            apb2_pre: APBPrescaler::DIV1,
            apb3_pre: APBPrescaler::DIV1,
            timer_prescaler: TimerPrescaler::DefaultX2,

            voltage_scale: VoltageScale::SCALE3,
        }
    }
}

pub(crate) mod sealed {
    pub trait McoInstance {
        type Source;
        unsafe fn apply_clock_settings(source: Self::Source, prescaler: u8);
    }
}

pub trait McoInstance: sealed::McoInstance + 'static {}

pin_trait!(McoPin, McoInstance);

macro_rules! impl_peri {
    ($peri:ident, $source:ident, $set_source:ident, $set_prescaler:ident) => {
        impl sealed::McoInstance for peripherals::$peri {
            type Source = $source;

            unsafe fn apply_clock_settings(source: Self::Source, prescaler: u8) {
                RCC.cfgr().modify(|w| {
                    w.$set_source(source);
                    w.$set_prescaler(prescaler);
                });
            }
        }

        impl McoInstance for peripherals::$peri {}
    };
}

impl_peri!(MCO1, Mco1, set_mco1, set_mco1pre);
impl_peri!(MCO2, Mco2, set_mco2, set_mco2pre);

pub struct Mco<'d, T: McoInstance> {
    phantom: PhantomData<&'d mut T>,
}

impl<'d, T: McoInstance> Mco<'d, T> {
    pub fn new(
        _peri: impl Peripheral<P = T> + 'd,
        _pin: impl Peripheral<P = impl McoPin<T>> + 'd,
        _source: T::Source,
    ) -> Self {
        todo!();
    }
}

pub(crate) unsafe fn init(config: Config) {
    let max_clk = match config.voltage_scale {
        VoltageScale::SCALE0 => Hertz(250_000_000),
        VoltageScale::SCALE1 => Hertz(200_000_000),
        VoltageScale::SCALE2 => Hertz(150_000_000),
        VoltageScale::SCALE3 => Hertz(100_000_000),
    };

    // Configure voltage scale.
    PWR.voscr().modify(|w| w.set_vos(config.voltage_scale));
    while !PWR.vossr().read().vosrdy() {}

    // Configure HSI
    let hsi = match config.hsi {
        None => {
            RCC.cr().modify(|w| w.set_hsion(false));
            None
        }
        Some(hsi) => {
            let (freq, hsidiv) = match hsi {
                Hsi::Mhz64 => (HSI_FREQ / 1u32, Hsidiv::DIV1),
                Hsi::Mhz32 => (HSI_FREQ / 2u32, Hsidiv::DIV2),
                Hsi::Mhz16 => (HSI_FREQ / 4u32, Hsidiv::DIV4),
                Hsi::Mhz8 => (HSI_FREQ / 8u32, Hsidiv::DIV8),
            };
            RCC.cr().modify(|w| {
                w.set_hsidiv(hsidiv);
                w.set_hsion(true);
            });
            while !RCC.cr().read().hsirdy() {}
            Some(freq)
        }
    };

    // Configure HSE
    let hse = match config.hse {
        None => {
            RCC.cr().modify(|w| w.set_hseon(false));
            None
        }
        Some(hse) => {
            let (byp, ext) = match hse.mode {
                HseMode::Oscillator => (false, Hseext::ANALOG),
                HseMode::BypassAnalog => (true, Hseext::ANALOG),
                HseMode::BypassDigital => (true, Hseext::DIGITAL),
            };

            RCC.cr().modify(|w| {
                w.set_hsebyp(byp);
                w.set_hseext(ext);
            });
            RCC.cr().modify(|w| w.set_hseon(true));
            while !RCC.cr().read().hserdy() {}
            Some(hse.freq)
        }
    };

    // Configure HSI48.
    RCC.cr().modify(|w| w.set_hsi48on(config.hsi48));
    let _hsi48 = match config.hsi48 {
        false => None,
        true => {
            while !RCC.cr().read().hsi48rdy() {}
            Some(CSI_FREQ)
        }
    };

    // Configure CSI.
    RCC.cr().modify(|w| w.set_csion(config.csi));
    let csi = match config.csi {
        false => None,
        true => {
            while !RCC.cr().read().csirdy() {}
            Some(CSI_FREQ)
        }
    };

    // Configure PLLs.
    let pll_input = PllInput { csi, hse, hsi };
    let pll1 = init_pll(0, config.pll1, &pll_input);
    let _pll2 = init_pll(1, config.pll2, &pll_input);
    #[cfg(rcc_h5)]
    let _pll3 = init_pll(2, config.pll3, &pll_input);

    // Configure sysclk
    let (sys, sw) = match config.sys {
        Sysclk::HSI => (unwrap!(hsi), Sw::HSI),
        Sysclk::HSE => (unwrap!(hse), Sw::HSE),
        Sysclk::CSI => (unwrap!(csi), Sw::CSI),
        Sysclk::Pll1P => (unwrap!(pll1.p), Sw::PLL1),
    };
    assert!(sys <= max_clk);

    let hclk = sys / config.ahb_pre;

    let apb1 = hclk / config.apb1_pre;
    let apb1_tim = apb_div_tim(&config.apb1_pre, hclk, config.timer_prescaler);
    let apb2 = hclk / config.apb2_pre;
    let apb2_tim = apb_div_tim(&config.apb2_pre, hclk, config.timer_prescaler);
    let apb3 = hclk / config.apb3_pre;

    flash_setup(hclk, config.voltage_scale);

    // Set hpre
    let hpre = config.ahb_pre.into();
    RCC.cfgr2().modify(|w| w.set_hpre(hpre));
    while RCC.cfgr2().read().hpre() != hpre {}

    // set ppre
    RCC.cfgr2().modify(|w| {
        w.set_ppre1(config.apb1_pre.into());
        w.set_ppre2(config.apb2_pre.into());
        w.set_ppre3(config.apb3_pre.into());
    });

    RCC.cfgr().modify(|w| w.set_timpre(config.timer_prescaler.into()));

    RCC.cfgr().modify(|w| w.set_sw(sw));
    while RCC.cfgr().read().sws() != sw {}

    set_freqs(Clocks {
        sys,
        ahb1: hclk,
        ahb2: hclk,
        ahb3: hclk,
        ahb4: hclk,
        apb1,
        apb2,
        apb3,
        apb1_tim,
        apb2_tim,
        adc: None,
    });
}

struct PllInput {
    hsi: Option<Hertz>,
    hse: Option<Hertz>,
    csi: Option<Hertz>,
}

struct PllOutput {
    p: Option<Hertz>,
    #[allow(dead_code)]
    q: Option<Hertz>,
    #[allow(dead_code)]
    r: Option<Hertz>,
}

fn init_pll(num: usize, config: Option<Pll>, input: &PllInput) -> PllOutput {
    let Some(config) = config else {
        // Stop PLL
        RCC.cr().modify(|w| w.set_pllon(num, false));
        while RCC.cr().read().pllrdy(num) {}

        // "To save power when PLL1 is not used, the value of PLL1M must be set to 0.""
        RCC.pllcfgr(num).write(|w| {
            w.set_divm(0);
        });

        return PllOutput {
            p: None,
            q: None,
            r: None,
        };
    };

    assert!(1 <= config.prediv && config.prediv <= 63);
    assert!(4 <= config.mul && config.mul <= 512);

    let (in_clk, src) = match config.source {
        PllSource::Hsi => (unwrap!(input.hsi), Pllsrc::HSI),
        PllSource::Hse => (unwrap!(input.hse), Pllsrc::HSE),
        PllSource::Csi => (unwrap!(input.csi), Pllsrc::CSI),
    };

    let ref_clk = in_clk / config.prediv as u32;

    let ref_range = match ref_clk.0 {
        ..=1_999_999 => Pllrge::RANGE1,
        ..=3_999_999 => Pllrge::RANGE2,
        ..=7_999_999 => Pllrge::RANGE4,
        ..=16_000_000 => Pllrge::RANGE8,
        x => panic!("pll ref_clk out of range: {} mhz", x),
    };

    // The smaller range (150 to 420 MHz) must
    // be chosen when the reference clock frequency is lower than 2 MHz.
    let wide_allowed = ref_range != Pllrge::RANGE1;

    let vco_clk = ref_clk * config.mul;
    let vco_range = match vco_clk.0 {
        VCO_MIN..=VCO_MAX => Pllvcosel::MEDIUMVCO,
        VCO_WIDE_MIN..=VCO_WIDE_MAX if wide_allowed => Pllvcosel::WIDEVCO,
        x => panic!("pll vco_clk out of range: {} mhz", x),
    };

    let p = config.divp.map(|div| {
        assert!(1 <= div && div <= 128);
        if num == 0 {
            // on PLL1, DIVP must be even.
            assert!(div % 2 == 0);
        }

        vco_clk / div
    });
    let q = config.divq.map(|div| {
        assert!(1 <= div && div <= 128);
        vco_clk / div
    });
    let r = config.divr.map(|div| {
        assert!(1 <= div && div <= 128);
        vco_clk / div
    });

    RCC.pllcfgr(num).write(|w| {
        w.set_pllsrc(src);
        w.set_divm(config.prediv);
        w.set_pllvcosel(vco_range);
        w.set_pllrge(ref_range);
        w.set_pllfracen(false);
        w.set_pllpen(p.is_some());
        w.set_pllqen(q.is_some());
        w.set_pllren(r.is_some());
    });
    RCC.plldivr(num).write(|w| {
        w.set_plln(config.mul - 1);
        w.set_pllp((config.divp.unwrap_or(1) - 1) as u8);
        w.set_pllq((config.divq.unwrap_or(1) - 1) as u8);
        w.set_pllr((config.divr.unwrap_or(1) - 1) as u8);
    });

    RCC.cr().modify(|w| w.set_pllon(num, true));
    while !RCC.cr().read().pllrdy(num) {}

    PllOutput { p, q, r }
}

fn flash_setup(clk: Hertz, vos: VoltageScale) {
    // RM0481 Rev 1, table 37
    // LATENCY  WRHIGHFREQ  VOS3           VOS2            VOS1            VOS0
    //      0           0   0 to 20 MHz    0 to 30 MHz     0 to 34 MHz     0 to 42 MHz
    //      1           0   20 to 40 MHz   30 to 60 MHz    34 to 68 MHz    42 to 84 MHz
    //      2           1   40 to 60 MHz   60 to 90 MHz    68 to 102 MHz   84 to 126 MHz
    //      3           1   60 to 80 MHz   90 to 120 MHz   102 to 136 MHz  126 to 168 MHz
    //      4           2   80 to 100 MHz  120 to 150 MHz  136 to 170 MHz  168 to 210 MHz
    //      5           2                                  170 to 200 MHz  210 to 250 MHz

    // See RM0433 Rev 7 Table 17. FLASH recommended number of wait
    // states and programming delay
    let (latency, wrhighfreq) = match (vos, clk.0) {
        (VoltageScale::SCALE0, ..=42_000_000) => (0, 0),
        (VoltageScale::SCALE0, ..=84_000_000) => (1, 0),
        (VoltageScale::SCALE0, ..=126_000_000) => (2, 1),
        (VoltageScale::SCALE0, ..=168_000_000) => (3, 1),
        (VoltageScale::SCALE0, ..=210_000_000) => (4, 2),
        (VoltageScale::SCALE0, ..=250_000_000) => (5, 2),

        (VoltageScale::SCALE1, ..=34_000_000) => (0, 0),
        (VoltageScale::SCALE1, ..=68_000_000) => (1, 0),
        (VoltageScale::SCALE1, ..=102_000_000) => (2, 1),
        (VoltageScale::SCALE1, ..=136_000_000) => (3, 1),
        (VoltageScale::SCALE1, ..=170_000_000) => (4, 2),
        (VoltageScale::SCALE1, ..=200_000_000) => (5, 2),

        (VoltageScale::SCALE2, ..=30_000_000) => (0, 0),
        (VoltageScale::SCALE2, ..=60_000_000) => (1, 0),
        (VoltageScale::SCALE2, ..=90_000_000) => (2, 1),
        (VoltageScale::SCALE2, ..=120_000_000) => (3, 1),
        (VoltageScale::SCALE2, ..=150_000_000) => (4, 2),

        (VoltageScale::SCALE3, ..=20_000_000) => (0, 0),
        (VoltageScale::SCALE3, ..=40_000_000) => (1, 0),
        (VoltageScale::SCALE3, ..=60_000_000) => (2, 1),
        (VoltageScale::SCALE3, ..=80_000_000) => (3, 1),
        (VoltageScale::SCALE3, ..=100_000_000) => (4, 2),

        _ => unreachable!(),
    };

    debug!("flash: latency={} wrhighfreq={}", latency, wrhighfreq);

    FLASH.acr().write(|w| {
        w.set_wrhighfreq(wrhighfreq);
        w.set_latency(latency);
    });
    while FLASH.acr().read().latency() != latency {}
}
