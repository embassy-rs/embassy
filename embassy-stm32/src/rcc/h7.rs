use core::marker::PhantomData;

use embassy_hal_internal::into_ref;
pub use pll::PllConfig;
use stm32_metapac::rcc::vals::{Mco1, Mco2};

use crate::gpio::sealed::AFType;
use crate::gpio::Speed;
use crate::pac::rcc::vals::{Adcsel, Ckpersel, Dppre, Hpre, Hsidiv, Pllsrc, Sw, Timpre};
use crate::pac::{PWR, RCC, SYSCFG};
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

pub use super::bus::VoltageScale;

#[derive(Clone, Copy)]
pub enum AdcClockSource {
    Pll2PCk,
    Pll3RCk,
    PerCk,
}

impl AdcClockSource {
    pub fn adcsel(&self) -> Adcsel {
        match self {
            AdcClockSource::Pll2PCk => Adcsel::PLL2_P,
            AdcClockSource::Pll3RCk => Adcsel::PLL3_R,
            AdcClockSource::PerCk => Adcsel::PER,
        }
    }
}

impl Default for AdcClockSource {
    fn default() -> Self {
        Self::Pll2PCk
    }
}

/// Core clock frequencies
#[derive(Clone, Copy)]
pub struct CoreClocks {
    pub hclk: Hertz,
    pub pclk1: Hertz,
    pub pclk2: Hertz,
    pub pclk3: Hertz,
    pub pclk4: Hertz,
    pub ppre1: u8,
    pub ppre2: u8,
    pub ppre3: u8,
    pub ppre4: u8,
    pub csi_ck: Option<Hertz>,
    pub hsi_ck: Option<Hertz>,
    pub hsi48_ck: Option<Hertz>,
    pub lsi_ck: Option<Hertz>,
    pub per_ck: Option<Hertz>,
    pub hse_ck: Option<Hertz>,
    pub pll1_p_ck: Option<Hertz>,
    pub pll1_q_ck: Option<Hertz>,
    pub pll1_r_ck: Option<Hertz>,
    pub pll2_p_ck: Option<Hertz>,
    pub pll2_q_ck: Option<Hertz>,
    pub pll2_r_ck: Option<Hertz>,
    pub pll3_p_ck: Option<Hertz>,
    pub pll3_q_ck: Option<Hertz>,
    pub pll3_r_ck: Option<Hertz>,
    pub timx_ker_ck: Option<Hertz>,
    pub timy_ker_ck: Option<Hertz>,
    pub adc_ker_ck: Option<Hertz>,
    pub sys_ck: Hertz,
    pub c_ck: Hertz,
}

/// Configuration of the core clocks
#[non_exhaustive]
#[derive(Default)]
pub struct Config {
    pub hse: Option<Hertz>,
    pub bypass_hse: bool,
    pub sys_ck: Option<Hertz>,
    pub per_ck: Option<Hertz>,
    pub hclk: Option<Hertz>,
    pub pclk1: Option<Hertz>,
    pub pclk2: Option<Hertz>,
    pub pclk3: Option<Hertz>,
    pub pclk4: Option<Hertz>,
    pub pll1: PllConfig,
    pub pll2: PllConfig,
    pub pll3: PllConfig,
    pub adc_clock_source: AdcClockSource,
}

/// Setup traceclk
/// Returns a pll1_r_ck
fn traceclk_setup(config: &mut Config, sys_use_pll1_p: bool) {
    let pll1_r_ck = match (sys_use_pll1_p, config.pll1.r_ck) {
        // pll1_p_ck selected as system clock but pll1_r_ck not
        // set. The traceclk mux is synchronous with the system
        // clock mux, but has pll1_r_ck as an input. In order to
        // keep traceclk running, we force a pll1_r_ck.
        (true, None) => Some(Hertz(unwrap!(config.pll1.p_ck).0 / 2)),

        // Either pll1 not selected as system clock, free choice
        // of pll1_r_ck. Or pll1 is selected, assume user has set
        // a suitable pll1_r_ck frequency.
        _ => config.pll1.r_ck,
    };
    config.pll1.r_ck = pll1_r_ck;
}

/// Divider calculator for pclk 1 - 4
///
/// Returns real pclk, bits, ppre and the timer kernel clock
fn ppre_calculate(
    requested_pclk: u32,
    hclk: u32,
    max_pclk: u32,
    tim_pre: Option<Timpre>,
) -> (u32, u8, u8, Option<u32>) {
    let (bits, ppre) = match (hclk + requested_pclk - 1) / requested_pclk {
        0 => panic!(),
        1 => (0b000, 1),
        2 => (0b100, 2),
        3..=5 => (0b101, 4),
        6..=11 => (0b110, 8),
        _ => (0b111, 16),
    };
    let real_pclk = hclk / u32::from(ppre);
    assert!(real_pclk <= max_pclk);

    let tim_ker_clk = if let Some(tim_pre) = tim_pre {
        let clk = match (bits, tim_pre) {
            (0b101, Timpre::DEFAULTX2) => hclk / 2,
            (0b110, Timpre::DEFAULTX4) => hclk / 2,
            (0b110, Timpre::DEFAULTX2) => hclk / 4,
            (0b111, Timpre::DEFAULTX4) => hclk / 4,
            (0b111, Timpre::DEFAULTX2) => hclk / 8,
            _ => hclk,
        };
        Some(clk)
    } else {
        None
    };
    (real_pclk, bits, ppre, tim_ker_clk)
}

/// Setup sys_ck
/// Returns sys_ck frequency, and a pll1_p_ck
fn sys_ck_setup(config: &mut Config, srcclk: Hertz) -> (Hertz, bool) {
    // Compare available with wanted clocks
    let sys_ck = config.sys_ck.unwrap_or(srcclk);

    if sys_ck != srcclk {
        // The requested system clock is not the immediately available
        // HSE/HSI clock. Perhaps there are other ways of obtaining
        // the requested system clock (such as `HSIDIV`) but we will
        // ignore those for now.
        //
        // Therefore we must use pll1_p_ck
        let pll1_p_ck = match config.pll1.p_ck {
            Some(p_ck) => {
                assert!(
                    p_ck == sys_ck,
                    "Error: Cannot set pll1_p_ck independently as it must be used to generate sys_ck"
                );
                Some(p_ck)
            }
            None => Some(sys_ck),
        };
        config.pll1.p_ck = pll1_p_ck;

        (sys_ck, true)
    } else {
        // sys_ck is derived directly from a source clock
        // (HSE/HSI). pll1_p_ck can be as requested
        (sys_ck, false)
    }
}

fn flash_setup(rcc_aclk: u32, vos: VoltageScale) {
    use crate::pac::FLASH;

    // ACLK in MHz, round down and subtract 1 from integers. eg.
    // 61_999_999 -> 61MHz
    // 62_000_000 -> 61MHz
    // 62_000_001 -> 62MHz
    let rcc_aclk_mhz = (rcc_aclk - 1) / 1_000_000;

    // See RM0433 Rev 7 Table 17. FLASH recommended number of wait
    // states and programming delay
    #[cfg(flash_h7)]
    let (wait_states, progr_delay) = match vos {
        // VOS 0 range VCORE 1.26V - 1.40V
        VoltageScale::Scale0 => match rcc_aclk_mhz {
            0..=69 => (0, 0),
            70..=139 => (1, 1),
            140..=184 => (2, 1),
            185..=209 => (2, 2),
            210..=224 => (3, 2),
            225..=239 => (4, 2),
            _ => (7, 3),
        },
        // VOS 1 range VCORE 1.15V - 1.26V
        VoltageScale::Scale1 => match rcc_aclk_mhz {
            0..=69 => (0, 0),
            70..=139 => (1, 1),
            140..=184 => (2, 1),
            185..=209 => (2, 2),
            210..=224 => (3, 2),
            _ => (7, 3),
        },
        // VOS 2 range VCORE 1.05V - 1.15V
        VoltageScale::Scale2 => match rcc_aclk_mhz {
            0..=54 => (0, 0),
            55..=109 => (1, 1),
            110..=164 => (2, 1),
            165..=224 => (3, 2),
            _ => (7, 3),
        },
        // VOS 3 range VCORE 0.95V - 1.05V
        VoltageScale::Scale3 => match rcc_aclk_mhz {
            0..=44 => (0, 0),
            45..=89 => (1, 1),
            90..=134 => (2, 1),
            135..=179 => (3, 2),
            180..=224 => (4, 2),
            _ => (7, 3),
        },
    };

    // See RM0455 Rev 10 Table 16. FLASH recommended number of wait
    // states and programming delay
    #[cfg(flash_h7ab)]
    let (wait_states, progr_delay) = match vos {
        // VOS 0 range VCORE 1.25V - 1.35V
        VoltageScale::Scale0 => match rcc_aclk_mhz {
            0..=42 => (0, 0),
            43..=84 => (1, 0),
            85..=126 => (2, 1),
            127..=168 => (3, 1),
            169..=210 => (4, 2),
            211..=252 => (5, 2),
            253..=280 => (6, 3),
            _ => (7, 3),
        },
        // VOS 1 range VCORE 1.15V - 1.25V
        VoltageScale::Scale1 => match rcc_aclk_mhz {
            0..=38 => (0, 0),
            39..=76 => (1, 0),
            77..=114 => (2, 1),
            115..=152 => (3, 1),
            153..=190 => (4, 2),
            191..=225 => (5, 2),
            _ => (7, 3),
        },
        // VOS 2 range VCORE 1.05V - 1.15V
        VoltageScale::Scale2 => match rcc_aclk_mhz {
            0..=34 => (0, 0),
            35..=68 => (1, 0),
            69..=102 => (2, 1),
            103..=136 => (3, 1),
            137..=160 => (4, 2),
            _ => (7, 3),
        },
        // VOS 3 range VCORE 0.95V - 1.05V
        VoltageScale::Scale3 => match rcc_aclk_mhz {
            0..=22 => (0, 0),
            23..=44 => (1, 0),
            45..=66 => (2, 1),
            67..=88 => (3, 1),
            _ => (7, 3),
        },
    };

    FLASH.acr().write(|w| {
        w.set_wrhighfreq(progr_delay);
        w.set_latency(wait_states)
    });
    while FLASH.acr().read().latency() != wait_states {}
}

pub enum McoClock {
    Disabled,
    Bypassed,
    Divided(u8),
}

impl McoClock {
    fn into_raw(&self) -> u8 {
        match self {
            McoClock::Disabled => 0,
            McoClock::Bypassed => 1,
            McoClock::Divided(divisor) => {
                if *divisor > 15 {
                    panic!("Mco divisor must be less than 15. Refer to the reference manual for more information.")
                }
                *divisor
            }
        }
    }
}

#[derive(Copy, Clone)]
pub enum Mco1Source {
    Hsi,
    Lse,
    Hse,
    Pll1Q,
    Hsi48,
}

impl Default for Mco1Source {
    fn default() -> Self {
        Self::Hsi
    }
}

pub trait McoSource {
    type Raw;

    fn into_raw(&self) -> Self::Raw;
}

impl McoSource for Mco1Source {
    type Raw = Mco1;
    fn into_raw(&self) -> Self::Raw {
        match self {
            Mco1Source::Hsi => Mco1::HSI,
            Mco1Source::Lse => Mco1::LSE,
            Mco1Source::Hse => Mco1::HSE,
            Mco1Source::Pll1Q => Mco1::PLL1_Q,
            Mco1Source::Hsi48 => Mco1::HSI48,
        }
    }
}

#[derive(Copy, Clone)]
pub enum Mco2Source {
    SysClk,
    Pll2Q,
    Hse,
    Pll1Q,
    Csi,
    Lsi,
}

impl Default for Mco2Source {
    fn default() -> Self {
        Self::SysClk
    }
}

impl McoSource for Mco2Source {
    type Raw = Mco2;
    fn into_raw(&self) -> Self::Raw {
        match self {
            Mco2Source::SysClk => Mco2::SYSCLK,
            Mco2Source::Pll2Q => Mco2::PLL2_P,
            Mco2Source::Hse => Mco2::HSE,
            Mco2Source::Pll1Q => Mco2::PLL1_P,
            Mco2Source::Csi => Mco2::CSI,
            Mco2Source::Lsi => Mco2::LSI,
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

pub(crate) unsafe fn init(mut config: Config) {
    // TODO make configurable?
    let enable_overdrive = false;

    // NB. The lower bytes of CR3 can only be written once after
    // POR, and must be written with a valid combination. Refer to
    // RM0433 Rev 7 6.8.4. This is partially enforced by dropping
    // `self` at the end of this method, but of course we cannot
    // know what happened between the previous POR and here.
    #[cfg(pwr_h7)]
    PWR.cr3().modify(|w| {
        w.set_scuen(true);
        w.set_ldoen(true);
        w.set_bypass(false);
    });

    #[cfg(pwr_h7smps)]
    PWR.cr3().modify(|w| {
        // hardcode "Direct SPMS" for now, this is what works on nucleos with the
        // default solderbridge configuration.
        w.set_sden(true);
        w.set_ldoen(false);
    });

    // Validate the supply configuration. If you are stuck here, it is
    // because the voltages on your board do not match those specified
    // in the D3CR.VOS and CR3.SDLEVEL fields. By default after reset
    // VOS = Scale 3, so check that the voltage on the VCAP pins =
    // 1.0V.
    while !PWR.csr1().read().actvosrdy() {}

    // Go to Scale 1
    PWR.d3cr().modify(|w| w.set_vos(0b11));
    while !PWR.d3cr().read().vosrdy() {}

    let pwr_vos = if !enable_overdrive {
        VoltageScale::Scale1
    } else {
        critical_section::with(|_| {
            RCC.apb4enr().modify(|w| w.set_syscfgen(true));

            SYSCFG.pwrcr().modify(|w| w.set_oden(1));
        });
        while !PWR.d3cr().read().vosrdy() {}
        VoltageScale::Scale0
    };

    // Freeze the core clocks, returning a Core Clocks Distribution
    // and Reset (CCDR) structure. The actual frequency of the clocks
    // configured is returned in the `clocks` member of the CCDR
    // structure.
    //
    // Note that `freeze` will never result in a clock _faster_ than
    // that specified. It may result in a clock that is a factor of [1,
    // 2) slower.
    //
    // `syscfg` is required to enable the I/O compensation cell.
    //
    // # Panics
    //
    // If a clock specification cannot be achieved within the
    // hardware specification then this function will panic. This
    // function may also panic if a clock specification can be
    // achieved, but the mechanism for doing so is not yet
    // implemented here.

    let srcclk = config.hse.unwrap_or(HSI_FREQ); // Available clocks
    let (sys_ck, sys_use_pll1_p) = sys_ck_setup(&mut config, srcclk);

    // Configure traceclk from PLL if needed
    traceclk_setup(&mut config, sys_use_pll1_p);

    let (pll1_p_ck, pll1_q_ck, pll1_r_ck) = pll::pll_setup(srcclk.0, &config.pll1, 0);
    let (pll2_p_ck, pll2_q_ck, pll2_r_ck) = pll::pll_setup(srcclk.0, &config.pll2, 1);
    let (pll3_p_ck, pll3_q_ck, pll3_r_ck) = pll::pll_setup(srcclk.0, &config.pll3, 2);

    let sys_ck = if sys_use_pll1_p {
        Hertz(unwrap!(pll1_p_ck)) // Must have been set by sys_ck_setup
    } else {
        sys_ck
    };

    // This routine does not support HSIDIV != 1. To
    // do so it would need to ensure all PLLxON bits are clear
    // before changing the value of HSIDIV
    let cr = RCC.cr().read();
    assert!(cr.hsion());
    assert!(cr.hsidiv() == Hsidiv::DIV1);

    RCC.csr().modify(|w| w.set_lsion(true));
    while !RCC.csr().read().lsirdy() {}

    // per_ck from HSI by default
    let (per_ck, ckpersel) = match (config.per_ck == config.hse, config.per_ck) {
        (true, Some(hse)) => (hse, Ckpersel::HSE),        // HSE
        (_, Some(CSI_FREQ)) => (CSI_FREQ, Ckpersel::CSI), // CSI
        _ => (HSI_FREQ, Ckpersel::HSI),                   // HSI
    };

    // D1 Core Prescaler
    // Set to 1
    let d1cpre_bits = 0;
    let d1cpre_div = 1;
    let sys_d1cpre_ck = sys_ck.0 / d1cpre_div;

    // Refer to part datasheet "General operating conditions"
    // table for (rev V). We do not assert checks for earlier
    // revisions which may have lower limits.
    let (sys_d1cpre_ck_max, rcc_hclk_max, pclk_max) = match pwr_vos {
        VoltageScale::Scale0 => (480_000_000, 240_000_000, 120_000_000),
        VoltageScale::Scale1 => (400_000_000, 200_000_000, 100_000_000),
        VoltageScale::Scale2 => (300_000_000, 150_000_000, 75_000_000),
        _ => (200_000_000, 100_000_000, 50_000_000),
    };
    assert!(sys_d1cpre_ck <= sys_d1cpre_ck_max);

    let rcc_hclk = config.hclk.map(|v| v.0).unwrap_or(sys_d1cpre_ck / 2);
    assert!(rcc_hclk <= rcc_hclk_max);

    // Estimate divisor
    let (hpre_bits, hpre_div) = match (sys_d1cpre_ck + rcc_hclk - 1) / rcc_hclk {
        0 => panic!(),
        1 => (Hpre::DIV1, 1),
        2 => (Hpre::DIV2, 2),
        3..=5 => (Hpre::DIV4, 4),
        6..=11 => (Hpre::DIV8, 8),
        12..=39 => (Hpre::DIV16, 16),
        40..=95 => (Hpre::DIV64, 64),
        96..=191 => (Hpre::DIV128, 128),
        192..=383 => (Hpre::DIV256, 256),
        _ => (Hpre::DIV512, 512),
    };
    // Calculate real AXI and AHB clock
    let rcc_hclk = sys_d1cpre_ck / hpre_div;
    assert!(rcc_hclk <= rcc_hclk_max);
    let rcc_aclk = rcc_hclk; // AXI clock is always equal to AHB clock on H7
                             // Timer prescaler selection
    let timpre = Timpre::DEFAULTX2;

    let requested_pclk1 = config.pclk1.map(|v| v.0).unwrap_or_else(|| pclk_max.min(rcc_hclk / 2));
    let (rcc_pclk1, ppre1_bits, ppre1, rcc_timerx_ker_ck) =
        ppre_calculate(requested_pclk1, rcc_hclk, pclk_max, Some(timpre));

    let requested_pclk2 = config.pclk2.map(|v| v.0).unwrap_or_else(|| pclk_max.min(rcc_hclk / 2));
    let (rcc_pclk2, ppre2_bits, ppre2, rcc_timery_ker_ck) =
        ppre_calculate(requested_pclk2, rcc_hclk, pclk_max, Some(timpre));

    let requested_pclk3 = config.pclk3.map(|v| v.0).unwrap_or_else(|| pclk_max.min(rcc_hclk / 2));
    let (rcc_pclk3, ppre3_bits, ppre3, _) = ppre_calculate(requested_pclk3, rcc_hclk, pclk_max, None);

    let requested_pclk4 = config.pclk4.map(|v| v.0).unwrap_or_else(|| pclk_max.min(rcc_hclk / 2));
    let (rcc_pclk4, ppre4_bits, ppre4, _) = ppre_calculate(requested_pclk4, rcc_hclk, pclk_max, None);

    // Start switching clocks -------------------

    // Ensure CSI is on and stable
    RCC.cr().modify(|w| w.set_csion(true));
    while !RCC.cr().read().csirdy() {}

    // Ensure HSI48 is on and stable
    RCC.cr().modify(|w| w.set_hsi48on(true));
    while !RCC.cr().read().hsi48on() {}

    // XXX: support MCO ?

    let hse_ck = match config.hse {
        Some(hse) => {
            // Ensure HSE is on and stable
            RCC.cr().modify(|w| {
                w.set_hseon(true);
                w.set_hsebyp(config.bypass_hse);
            });
            while !RCC.cr().read().hserdy() {}
            Some(hse)
        }
        None => None,
    };

    let pllsrc = if config.hse.is_some() { Pllsrc::HSE } else { Pllsrc::HSI };
    RCC.pllckselr().modify(|w| w.set_pllsrc(pllsrc));

    let enable_pll = |pll| {
        RCC.cr().modify(|w| w.set_pllon(pll, true));
        while !RCC.cr().read().pllrdy(pll) {}
    };

    if pll1_p_ck.is_some() {
        enable_pll(0);
    }

    if pll2_p_ck.is_some() {
        enable_pll(1);
    }

    if pll3_p_ck.is_some() {
        enable_pll(2);
    }

    // Core Prescaler / AHB Prescaler / APB3 Prescaler
    RCC.d1cfgr().modify(|w| {
        w.set_d1cpre(Hpre::from_bits(d1cpre_bits));
        w.set_d1ppre(Dppre::from_bits(ppre3_bits));
        w.set_hpre(hpre_bits)
    });
    // Ensure core prescaler value is valid before future lower
    // core voltage
    while RCC.d1cfgr().read().d1cpre().to_bits() != d1cpre_bits {}

    flash_setup(rcc_aclk, pwr_vos);

    // APB1 / APB2 Prescaler
    RCC.d2cfgr().modify(|w| {
        w.set_d2ppre1(Dppre::from_bits(ppre1_bits));
        w.set_d2ppre2(Dppre::from_bits(ppre2_bits));
    });

    // APB4 Prescaler
    RCC.d3cfgr().modify(|w| w.set_d3ppre(Dppre::from_bits(ppre4_bits)));

    // Peripheral Clock (per_ck)
    RCC.d1ccipr().modify(|w| w.set_ckpersel(ckpersel));

    // ADC clock MUX
    RCC.d3ccipr().modify(|w| w.set_adcsel(config.adc_clock_source.adcsel()));

    let adc_ker_ck = match config.adc_clock_source {
        AdcClockSource::Pll2PCk => pll2_p_ck.map(Hertz),
        AdcClockSource::Pll3RCk => pll3_r_ck.map(Hertz),
        AdcClockSource::PerCk => Some(per_ck),
    };

    // Set timer clocks prescaler setting
    RCC.cfgr().modify(|w| w.set_timpre(timpre));

    // Select system clock source
    let sw = match (sys_use_pll1_p, config.hse.is_some()) {
        (true, _) => Sw::PLL1,
        (false, true) => Sw::HSE,
        _ => Sw::HSI,
    };
    RCC.cfgr().modify(|w| w.set_sw(sw));
    while RCC.cfgr().read().sws().to_bits() != sw.to_bits() {}

    // IO compensation cell - Requires CSI clock and SYSCFG
    assert!(RCC.cr().read().csirdy());
    RCC.apb4enr().modify(|w| w.set_syscfgen(true));

    // Enable the compensation cell, using back-bias voltage code
    // provide by the cell.
    critical_section::with(|_| {
        SYSCFG.cccsr().modify(|w| {
            w.set_en(true);
            w.set_cs(false);
            w.set_hslv(false);
        })
    });
    while !SYSCFG.cccsr().read().ready() {}

    let core_clocks = CoreClocks {
        hclk: Hertz(rcc_hclk),
        pclk1: Hertz(rcc_pclk1),
        pclk2: Hertz(rcc_pclk2),
        pclk3: Hertz(rcc_pclk3),
        pclk4: Hertz(rcc_pclk4),
        ppre1,
        ppre2,
        ppre3,
        ppre4,
        csi_ck: Some(CSI_FREQ),
        hsi_ck: Some(HSI_FREQ),
        hsi48_ck: Some(HSI48_FREQ),
        lsi_ck: Some(LSI_FREQ),
        per_ck: Some(per_ck),
        hse_ck,
        pll1_p_ck: pll1_p_ck.map(Hertz),
        pll1_q_ck: pll1_q_ck.map(Hertz),
        pll1_r_ck: pll1_r_ck.map(Hertz),
        pll2_p_ck: pll2_p_ck.map(Hertz),
        pll2_q_ck: pll2_q_ck.map(Hertz),
        pll2_r_ck: pll2_r_ck.map(Hertz),
        pll3_p_ck: pll3_p_ck.map(Hertz),
        pll3_q_ck: pll3_q_ck.map(Hertz),
        pll3_r_ck: pll3_r_ck.map(Hertz),
        timx_ker_ck: rcc_timerx_ker_ck.map(Hertz),
        timy_ker_ck: rcc_timery_ker_ck.map(Hertz),
        adc_ker_ck,
        sys_ck,
        c_ck: Hertz(sys_d1cpre_ck),
    };

    set_freqs(Clocks {
        sys: core_clocks.c_ck,
        ahb1: core_clocks.hclk,
        ahb2: core_clocks.hclk,
        ahb3: core_clocks.hclk,
        ahb4: core_clocks.hclk,
        apb1: core_clocks.pclk1,
        apb2: core_clocks.pclk2,
        apb4: core_clocks.pclk4,
        apb1_tim: core_clocks.timx_ker_ck.unwrap_or(core_clocks.pclk1),
        apb2_tim: core_clocks.timy_ker_ck.unwrap_or(core_clocks.pclk2),
        adc: core_clocks.adc_ker_ck,
    });
}

mod pll {
    use super::{Hertz, RCC};

    const VCO_MIN: u32 = 150_000_000;
    const VCO_MAX: u32 = 420_000_000;

    #[derive(Default)]
    pub struct PllConfig {
        pub p_ck: Option<Hertz>,
        pub q_ck: Option<Hertz>,
        pub r_ck: Option<Hertz>,
    }

    pub(super) struct PllConfigResults {
        pub ref_x_ck: u32,
        pub pll_x_m: u32,
        pub pll_x_p: u32,
        pub vco_ck_target: u32,
    }

    fn vco_output_divider_setup(output: u32, plln: usize) -> (u32, u32) {
        let pll_x_p = if plln == 0 {
            if output > VCO_MAX / 2 {
                1
            } else {
                ((VCO_MAX / output) | 1) - 1 // Must be even or unity
            }
        } else {
            // Specific to PLL2/3, will subtract 1 later
            if output > VCO_MAX / 2 {
                1
            } else {
                VCO_MAX / output
            }
        };

        let vco_ck = output * pll_x_p;

        assert!(pll_x_p < 128);
        assert!(vco_ck >= VCO_MIN);
        assert!(vco_ck <= VCO_MAX);

        (vco_ck, pll_x_p)
    }

    /// # Safety
    ///
    /// Must have exclusive access to the RCC register block
    fn vco_setup(pll_src: u32, requested_output: u32, plln: usize) -> PllConfigResults {
        use crate::pac::rcc::vals::{Pllrge, Pllvcosel};

        let (vco_ck_target, pll_x_p) = vco_output_divider_setup(requested_output, plln);

        // Input divisor, resulting in a reference clock in the range
        // 1 to 2 MHz. Choose the highest reference clock (lowest m)
        let pll_x_m = (pll_src + 1_999_999) / 2_000_000;
        assert!(pll_x_m < 64);

        // Calculate resulting reference clock
        let ref_x_ck = pll_src / pll_x_m;
        assert!((1_000_000..=2_000_000).contains(&ref_x_ck));

        RCC.pllcfgr().modify(|w| {
            w.set_pllvcosel(plln, Pllvcosel::MEDIUMVCO);
            w.set_pllrge(plln, Pllrge::RANGE1);
        });
        PllConfigResults {
            ref_x_ck,
            pll_x_m,
            pll_x_p,
            vco_ck_target,
        }
    }

    /// # Safety
    ///
    /// Must have exclusive access to the RCC register block
    pub(super) fn pll_setup(pll_src: u32, config: &PllConfig, plln: usize) -> (Option<u32>, Option<u32>, Option<u32>) {
        use crate::pac::rcc::vals::Divp;

        match config.p_ck {
            Some(requested_output) => {
                let config_results = vco_setup(pll_src, requested_output.0, plln);
                let PllConfigResults {
                    ref_x_ck,
                    pll_x_m,
                    pll_x_p,
                    vco_ck_target,
                } = config_results;

                RCC.pllckselr().modify(|w| w.set_divm(plln, pll_x_m as u8));

                // Feedback divider. Integer only
                let pll_x_n = vco_ck_target / ref_x_ck;
                assert!(pll_x_n >= 4);
                assert!(pll_x_n <= 512);
                RCC.plldivr(plln).modify(|w| w.set_divn1((pll_x_n - 1) as u16));

                // No FRACN
                RCC.pllcfgr().modify(|w| w.set_pllfracen(plln, false));
                let vco_ck = ref_x_ck * pll_x_n;

                RCC.plldivr(plln)
                    .modify(|w| w.set_divp1(Divp::from_bits((pll_x_p - 1) as u8)));
                RCC.pllcfgr().modify(|w| w.set_divpen(plln, true));

                // Calulate additional output dividers
                let q_ck = match config.q_ck {
                    Some(Hertz(ck)) if ck > 0 => {
                        let div = (vco_ck + ck - 1) / ck;
                        RCC.plldivr(plln).modify(|w| w.set_divq1((div - 1) as u8));
                        RCC.pllcfgr().modify(|w| w.set_divqen(plln, true));
                        Some(vco_ck / div)
                    }
                    _ => None,
                };
                let r_ck = match config.r_ck {
                    Some(Hertz(ck)) if ck > 0 => {
                        let div = (vco_ck + ck - 1) / ck;
                        RCC.plldivr(plln).modify(|w| w.set_divr1((div - 1) as u8));
                        RCC.pllcfgr().modify(|w| w.set_divren(plln, true));
                        Some(vco_ck / div)
                    }
                    _ => None,
                };

                (Some(vco_ck / pll_x_p), q_ck, r_ck)
            }
            None => {
                assert!(
                    config.q_ck.is_none(),
                    "Must set PLL P clock for Q clock to take effect!"
                );
                assert!(
                    config.r_ck.is_none(),
                    "Must set PLL P clock for R clock to take effect!"
                );
                (None, None, None)
            }
        }
    }
}
