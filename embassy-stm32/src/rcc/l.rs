#[cfg(any(stm32l0, stm32l1, stm32wl))]
pub use crate::pac::pwr::vals::Vos as VoltageScale;
use crate::pac::rcc::regs::Cfgr;
#[cfg(any(stm32wb, stm32wl))]
pub use crate::pac::rcc::vals::Hsepre as HsePrescaler;
pub use crate::pac::rcc::vals::{Hpre as AHBPrescaler, Msirange as MSIRange, Ppre as APBPrescaler, Sw as Sysclk};
use crate::pac::{FLASH, RCC};
use crate::rcc::LSI_FREQ;
#[cfg(dsihost)]
use crate::rcc::dsi;
#[cfg(dsihost)]
pub use crate::rcc::dsi::{DsiHostPllConfig, DsiPllInput, DsiPllNdiv, DsiPllOutput};
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
    /// HSE prescaler
    #[cfg(any(stm32wb, stm32wl))]
    pub prescaler: HsePrescaler,
}

/// Clocks configuration
#[derive(Clone, Copy)]
pub struct Config {
    // base clock sources
    pub msi: Option<MSIRange>,
    pub hsi: bool,
    pub hse: Option<Hse>,
    #[cfg(crs)]
    pub hsi48: Option<super::Hsi48Config>,

    // pll
    pub pll: Option<Pll>,
    #[cfg(any(stm32l4, stm32l5, stm32wb))]
    pub pllsai1: Option<Pll>,
    #[cfg(any(stm32l47x, stm32l48x, stm32l49x, stm32l4ax, rcc_l4plus, stm32l5))]
    pub pllsai2: Option<Pll>,

    // sysclk, buses.
    pub sys: Sysclk,
    pub ahb_pre: AHBPrescaler,
    pub apb1_pre: APBPrescaler,
    #[cfg(not(stm32u0))]
    pub apb2_pre: APBPrescaler,
    #[cfg(any(stm32wl5x, stm32wb))]
    pub core2_ahb_pre: AHBPrescaler,
    #[cfg(any(stm32wl, stm32wb))]
    pub shared_ahb_pre: AHBPrescaler,

    // low speed LSI/LSE/RTC
    pub ls: super::LsConfig,

    #[cfg(dsihost)]
    pub dsi: Option<DsiHostPllConfig>,

    #[cfg(any(stm32l0, stm32l1, stm32wl))]
    pub voltage_scale: VoltageScale,

    /// Per-peripheral kernel clock selection muxes
    pub mux: super::mux::ClockMux,
}

impl Config {
    pub const fn new() -> Self {
        Config {
            hse: None,
            hsi: false,
            msi: Some(MSIRange::Range4m),
            sys: Sysclk::Msi,
            ahb_pre: AHBPrescaler::Div1,
            apb1_pre: APBPrescaler::Div1,
            #[cfg(not(stm32u0))]
            apb2_pre: APBPrescaler::Div1,
            #[cfg(any(stm32wl5x, stm32wb))]
            core2_ahb_pre: AHBPrescaler::Div1,
            #[cfg(any(stm32wl, stm32wb))]
            shared_ahb_pre: AHBPrescaler::Div1,
            #[cfg(dsihost)]
            dsi: None,
            pll: None,
            #[cfg(any(stm32l4, stm32l5, stm32wb))]
            pllsai1: None,
            #[cfg(any(stm32l47x, stm32l48x, stm32l49x, stm32l4ax, rcc_l4plus, stm32l5))]
            pllsai2: None,
            #[cfg(crs)]
            hsi48: Some(crate::rcc::Hsi48Config::new()),
            ls: crate::rcc::LsConfig::new(),
            #[cfg(any(stm32l0, stm32l1, stm32wl))]
            voltage_scale: VoltageScale::Range1,
            mux: super::mux::ClockMux::default(),
        }
    }

    #[cfg(stm32wb)]
    pub const fn new_wpan() -> Self {
        Self {
            hse: Some(Hse {
                freq: Hertz(32_000_000),
                mode: HseMode::Oscillator,
                prescaler: HsePrescaler::Div1,
            }),
            sys: Sysclk::Pll1R,
            #[cfg(crs)]
            hsi48: Some(super::Hsi48Config { sync_from_usb: false }),
            msi: None,
            hsi: false,

            ls: super::LsConfig::default_lse(),

            pll: Some(Pll {
                source: PllSource::Hse,
                prediv: PllPreDiv::Div2,
                mul: PllMul::Mul12,
                divp: Some(PllPDiv::Div3), // 32 / 2 * 12 / 3 = 64Mhz
                divq: Some(PllQDiv::Div4), // 32 / 2 * 12 / 4 = 48Mhz
                divr: Some(PllRDiv::Div3), // 32 / 2 * 12 / 3 = 64Mhz
            }),
            pllsai1: None,

            ahb_pre: AHBPrescaler::Div1,
            core2_ahb_pre: AHBPrescaler::Div2,
            shared_ahb_pre: AHBPrescaler::Div1,
            apb1_pre: APBPrescaler::Div1,
            apb2_pre: APBPrescaler::Div1,

            mux: {
                use crate::pac::rcc::vals::Rfwkpsel;

                let mut mux = super::mux::ClockMux::default();

                mux.rfwkpsel = Rfwkpsel::Lse;
                mux
            },
        }
    }
}

impl Default for Config {
    fn default() -> Config {
        Self::new()
    }
}

#[cfg(stm32wb)]
#[deprecated = "Use `Config::new_wpan`"]
pub const WPAN_DEFAULT: Config = Config::new_wpan();

fn msi_enable(range: MSIRange) {
    #[cfg(any(stm32l4, stm32l5, stm32wb, stm32wl, stm32u0))]
    RCC.cr().modify(|w| {
        #[cfg(not(stm32wb))]
        w.set_msirgsel(crate::pac::rcc::vals::Msirgsel::Cr);
        w.set_msirange(range);
        w.set_msipllen(false);
    });
    #[cfg(any(stm32l0, stm32l1))]
    RCC.icscr().modify(|w| w.set_msirange(range));

    RCC.cr().modify(|w| w.set_msion(true));
    while !RCC.cr().read().msirdy() {}
}

pub(crate) unsafe fn init(config: Config) {
    // Switch to MSI to prevent problems with PLL configuration.
    if !RCC.cr().read().msion() {
        // Turn on MSI and configure it to 4MHz.
        msi_enable(MSIRange::Range4m)
    }
    if RCC.cfgr().read().sws() != Sysclk::Msi {
        // Set MSI as a clock source, reset prescalers.
        RCC.cfgr().write_value(Cfgr::default());
        // Wait for clock switch status bits to change.
        while RCC.cfgr().read().sws() != Sysclk::Msi {}
    }

    // Set the maximum flash wait states now: `config.msi` below may raise the
    // MSI frequency (up to 48 MHz) while MSI is the active sysclk, before the
    // computed latency is written. Raising latency is always safe; the exact
    // value for the target clocks is set later. stm32l0/l1 are not affected:
    // their MSI tops out at 4.194 MHz, within zero wait states at every
    // voltage range.
    #[cfg(any(stm32l4, stm32l5, stm32wb, stm32wl, stm32u0))]
    {
        #[cfg(any(all(stm32l4, not(rcc_l4plus)), stm32wb))]
        const MAX_LATENCY: u8 = 4;
        #[cfg(any(rcc_l4plus, stm32l5))]
        const MAX_LATENCY: u8 = 5;
        #[cfg(any(stm32wl, stm32u0))]
        const MAX_LATENCY: u8 = 2;

        FLASH.acr().modify(|w| w.set_latency(MAX_LATENCY));
        while FLASH.acr().read().latency() != MAX_LATENCY {}
    }

    // Set voltage scale
    #[cfg(any(stm32l0, stm32l1))]
    {
        while crate::pac::PWR.csr().read().vosf() {}
        crate::pac::PWR.cr().write(|w| w.set_vos(config.voltage_scale));
        while crate::pac::PWR.csr().read().vosf() {}
    }

    #[cfg(stm32l5)]
    {
        crate::pac::PWR.cr1().modify(|w| {
            w.set_vos(crate::pac::pwr::vals::Vos::Range0);
        });
        // RM0438 8.2.5: VOS resets to Range 2, so no clock may be raised until VOSF reports the new range.
        while crate::pac::PWR.sr2().read().vosf() {}
    }

    #[cfg(stm32wl)]
    {
        crate::pac::PWR.cr1().modify(|w| w.set_vos(config.voltage_scale));
        while crate::pac::PWR.sr2().read().vosf() {}
    }

    let rtc = config.ls.init();

    let lse = config.ls.lse.map(|l| l.frequency);
    let lsi = config.ls.lsi.then_some(LSI_FREQ);

    let msi = config.msi.map(|range| {
        msi_enable(range);
        msirange_to_hertz(range)
    });

    // If LSE is enabled and the right freq, enable calibration of MSI
    #[cfg(any(stm32l4, stm32l5, stm32wb, stm32wl))]
    if config.ls.lse.map(|x| x.frequency) == Some(Hertz(32_768)) {
        RCC.cr().modify(|w| w.set_msipllen(true));
    }

    let hsi = config.hsi.then(|| {
        RCC.cr().modify(|w| w.set_hsion(true));
        while !RCC.cr().read().hsirdy() {}

        HSI_FREQ
    });

    let hse = config.hse.map(|hse| {
        RCC.cr().modify(|w| {
            #[cfg(stm32wl)]
            w.set_hsebyppwr(hse.mode == HseMode::Bypass);
            #[cfg(not(stm32wl))]
            w.set_hsebyp(hse.mode == HseMode::Bypass);
            w.set_hseon(true);
        });
        while !RCC.cr().read().hserdy() {}

        hse.freq
    });

    #[cfg(crs)]
    let hsi48 = config.hsi48.map(|config| super::init_hsi48(config));
    #[cfg(not(crs))]
    let hsi48: Option<Hertz> = None;

    let _plls = [
        &config.pll,
        #[cfg(any(stm32l4, stm32l5, stm32wb))]
        &config.pllsai1,
        #[cfg(any(stm32l47x, stm32l48x, stm32l49x, stm32l4ax, rcc_l4plus, stm32l5))]
        &config.pllsai2,
    ];

    // L4 has shared PLLSRC, PLLM, check it's equal in all PLLs.
    #[cfg(all(stm32l4, not(rcc_l4plus)))]
    match super::util::get_equal(_plls.into_iter().flatten().map(|p| (p.source, p.prediv))) {
        Err(()) => panic!("Source must be equal across all enabled PLLs."),
        Ok(None) => {}
        Ok(Some((source, prediv))) => RCC.pllcfgr().write(|w| {
            w.set_pllm(prediv);
            w.set_pllsrc(source);
        }),
    };

    // L4+, WL has shared PLLSRC, check it's equal in all PLLs.
    #[cfg(any(rcc_l4plus, stm32wl))]
    match super::util::get_equal(_plls.into_iter().flatten().map(|p| p.source)) {
        Err(()) => panic!("Source must be equal across all enabled PLLs."),
        Ok(None) => {}
        Ok(Some(source)) => RCC.pllcfgr().write(|w| {
            w.set_pllsrc(source);
        }),
    };

    let pll_input = PllInput {
        hse,
        hsi,
        #[cfg(any(stm32l4, stm32l5, stm32wb, stm32wl, stm32u0))]
        msi,
    };
    let pll = config.pll.map_or_else(
        || {
            pll_enable(PllInstance::Pll, false);
            PllOutput::default()
        },
        |c| init_pll(PllInstance::Pll, Some(c), &pll_input),
    );
    #[cfg(any(stm32l4, stm32l5, stm32wb))]
    let pllsai1 = config.pllsai1.map_or_else(
        || {
            pll_enable(PllInstance::Pllsai1, false);
            PllOutput::default()
        },
        |c| init_pll(PllInstance::Pllsai1, Some(c), &pll_input),
    );
    #[cfg(any(stm32l47x, stm32l48x, stm32l49x, stm32l4ax, rcc_l4plus, stm32l5))]
    let pllsai2 = config.pllsai2.map_or_else(
        || {
            pll_enable(PllInstance::Pllsai2, false);
            PllOutput::default()
        },
        |c| init_pll(PllInstance::Pllsai2, Some(c), &pll_input),
    );

    let sys_clk = match config.sys {
        Sysclk::Hse => hse.unwrap(),
        Sysclk::Hsi => hsi.unwrap(),
        Sysclk::Msi => msi.unwrap(),
        Sysclk::Pll1R => pll.r.unwrap(),
        #[cfg(stm32u0)]
        Sysclk::Lsi | Sysclk::Lse => todo!(),
        #[cfg(stm32u0)]
        Sysclk::_RESERVED_6 | Sysclk::_RESERVED_7 => unreachable!(),
    };

    #[cfg(rcc_l4plus)]
    assert!(sys_clk.0 <= 120_000_000);
    #[cfg(all(stm32l4, not(rcc_l4plus)))]
    assert!(sys_clk.0 <= 80_000_000);
    #[cfg(stm32l5)]
    assert!(sys_clk.0 <= 110_000_000);

    #[cfg(stm32wl)]
    match config.voltage_scale {
        VoltageScale::Range1 => assert!(sys_clk.0 <= 48_000_000),
        VoltageScale::Range2 => assert!(sys_clk.0 <= 16_000_000),
        _ => panic!("invalid voltage scale"),
    }

    let hclk1 = sys_clk / config.ahb_pre;
    let (pclk1, pclk1_tim) = super::util::calc_pclk(hclk1, config.apb1_pre);
    #[cfg(not(stm32u0))]
    let (pclk2, pclk2_tim) = super::util::calc_pclk(hclk1, config.apb2_pre);
    #[cfg(any(stm32l4, stm32l5, stm32wlex))]
    let hclk2 = hclk1;
    #[cfg(any(stm32wl5x, stm32wb))]
    let hclk2 = sys_clk / config.core2_ahb_pre;
    #[cfg(any(stm32l4, stm32l5, stm32wlex))]
    let hclk3 = hclk1;
    #[cfg(any(stm32wl5x, stm32wb))]
    let hclk3 = sys_clk / config.shared_ahb_pre;

    // Set flash wait states
    #[cfg(any(stm32l0, stm32l1))]
    let latency = flash_latency(config.voltage_scale, sys_clk.0);
    #[cfg(any(stm32l4, stm32l5, stm32u0))]
    let latency = flash_latency(hclk1.0);
    #[cfg(stm32wl)]
    let latency = flash_latency(config.voltage_scale, hclk3.0);
    #[cfg(stm32wb)]
    let latency = flash_latency(hclk3.0);

    // RM0432 5.1.8: PWR_CR5 resets to Range 1 normal, which stops at 80 MHz; going above needs boost.
    #[cfg(rcc_l4plus)]
    if sys_clk.0 > 80_000_000 {
        crate::pac::PWR.cr5().modify(|w| w.set_r1mode(false));
    }

    #[cfg(stm32l1)]
    FLASH.acr().write(|w| w.set_acc64(true));

    FLASH.acr().modify(|w| w.set_latency(latency));
    while FLASH.acr().read().latency() != latency {}

    // enable prefetch (prft) after switching latency
    #[cfg(not(stm32l5))]
    FLASH.acr().modify(|w| w.set_prften(true));

    // RM0432 5.1.8 / RM0438 8.2.5: above 80 MHz the AHB prescaler must divide by two across the switch.
    #[cfg(any(rcc_l4plus, stm32l5))]
    let (hpre_stepped, ahb_pre) = if sys_clk.0 > 80_000_000 && config.ahb_pre == AHBPrescaler::Div1 {
        (true, AHBPrescaler::Div2)
    } else {
        (false, config.ahb_pre)
    };
    #[cfg(not(any(rcc_l4plus, stm32l5)))]
    let (hpre_stepped, ahb_pre) = (false, config.ahb_pre);

    RCC.cfgr().modify(|w| {
        w.set_sw(config.sys);
        w.set_hpre(ahb_pre);
        #[cfg(stm32u0)]
        w.set_ppre(config.apb1_pre);
        #[cfg(not(stm32u0))]
        w.set_ppre1(config.apb1_pre);
        #[cfg(not(stm32u0))]
        w.set_ppre2(config.apb2_pre);
    });
    while RCC.cfgr().read().sws() != config.sys {}

    if hpre_stepped {
        RCC.cfgr().modify(|w| w.set_hpre(config.ahb_pre));
    }

    #[cfg(any(stm32wl, stm32wb))]
    {
        RCC.extcfgr().modify(|w| {
            w.set_shdhpre(config.shared_ahb_pre);
            #[cfg(any(stm32wl5x, stm32wb))]
            w.set_c2hpre(config.core2_ahb_pre);
        });
        while !RCC.extcfgr().read().shdhpref() {}
        #[cfg(any(stm32wl5x, stm32wb))]
        while !RCC.extcfgr().read().c2hpref() {}
    }

    // Disable HSI if not used
    if !config.hsi {
        RCC.cr().modify(|w| w.set_hsion(false));
    }

    // Disable the HSI48, if not used
    #[cfg(crs)]
    if config.hsi48.is_none() {
        super::disable_hsi48();
    }

    config.mux.init();

    set_clocks!(
        sys: Some(sys_clk),
        hclk1: Some(hclk1),
        #[cfg(any(stm32l4, stm32l5, stm32wb, stm32wl))]
        hclk2: Some(hclk2),
        #[cfg(any(stm32l4, stm32l5, stm32wb, stm32wl))]
        hclk3: Some(hclk3),
        pclk1: Some(pclk1),
        #[cfg(not(stm32u0))]
        pclk2: Some(pclk2),
        pclk1_tim: Some(pclk1_tim),
        #[cfg(not(stm32u0))]
        pclk2_tim: Some(pclk2_tim),
        #[cfg(stm32wl)]
        pclk3: Some(hclk3),
        hsi: hsi,
        hse: hse,
        msi: msi,
        hsi48: hsi48,

        #[cfg(any(stm32l0, stm32l1))]
        pll1_vco: pll.vco,

        #[cfg(not(any(stm32l0, stm32l1)))]
        pll1_p: pll.p,
        #[cfg(not(any(stm32l0, stm32l1)))]
        pll1_q: pll.q,
        pll1_r: pll.r,

        #[cfg(any(stm32l4, stm32l5, stm32wb))]
        pllsai1_p: pllsai1.p,
        #[cfg(any(stm32l4, stm32l5, stm32wb))]
        pllsai1_q: pllsai1.q,
        #[cfg(any(stm32l4, stm32l5, stm32wb))]
        pllsai1_r: pllsai1.r,

        #[cfg(not(any(stm32l47x, stm32l48x, stm32l49x, stm32l4ax, rcc_l4plus, stm32l5)))]
        pllsai2_p: None,
        #[cfg(not(any(stm32l47x, stm32l48x, stm32l49x, stm32l4ax, rcc_l4plus, stm32l5)))]
        pllsai2_q: None,
        #[cfg(not(any(stm32l47x, stm32l48x, stm32l49x, stm32l4ax, rcc_l4plus, stm32l5)))]
        pllsai2_r: None,

        #[cfg(any(stm32l47x, stm32l48x, stm32l49x, stm32l4ax, rcc_l4plus, stm32l5))]
        pllsai2_p: pllsai2.p,
        #[cfg(any(stm32l47x, stm32l48x, stm32l49x, stm32l4ax, rcc_l4plus, stm32l5))]
        pllsai2_q: pllsai2.q,
        #[cfg(any(stm32l47x, stm32l48x, stm32l49x, stm32l4ax, rcc_l4plus, stm32l5))]
        pllsai2_r: pllsai2.r,

        #[cfg(dsihost)]
        dsi_phy: config.dsi.map(|config| dsi::configure_pll(hse, config)),

        rtc: rtc,
        lse: lse,
        lsi: lsi,

        // TODO
        sai1_extclk: None,
        sai2_extclk: None,
    );
}

#[cfg(any(stm32l0, stm32l1))]
type FlashLatency = bool;
#[cfg(not(any(stm32l0, stm32l1)))]
type FlashLatency = u8;

/// Flash wait states for a flash clock of `hz`: SYSCLK on L0/L1, HCLK3 on WB/WL, HCLK1 otherwise.
#[cfg(any(stm32l0, stm32l1))]
const fn flash_latency(vos: VoltageScale, hz: u32) -> FlashLatency {
    match (vos, hz) {
        (VoltageScale::Range1, ..=16_000_000) => false,
        (VoltageScale::Range2, ..=8_000_000) => false,
        (VoltageScale::Range3, ..=4_200_000) => false,
        _ => true,
    }
}

/// Flash wait states for a flash clock of `hz`: SYSCLK on L0/L1, HCLK3 on WB/WL, HCLK1 otherwise.
#[cfg(stm32wl)]
const fn flash_latency(vos: VoltageScale, hz: u32) -> FlashLatency {
    match (vos, hz) {
        // RM0461 Table 5: Number of wait states according to flash clock (HCLK3) frequency
        (VoltageScale::Range1, ..=18_000_000) => 0,
        (VoltageScale::Range1, ..=36_000_000) => 1,
        (VoltageScale::Range1, _) => 2,
        (VoltageScale::Range2, ..=6_000_000) => 0,
        (VoltageScale::Range2, ..=12_000_000) => 1,
        (VoltageScale::Range2, _) => 2,
        _ => 2,
    }
}

/// Flash wait states for a flash clock of `hz`: SYSCLK on L0/L1, HCLK3 on WB/WL, HCLK1 otherwise.
#[cfg(any(stm32l4, stm32l5, stm32wb, stm32u0))]
const fn flash_latency(hz: u32) -> FlashLatency {
    #[cfg(all(stm32l4, not(rcc_l4plus)))]
    return match hz {
        0..=16_000_000 => 0,
        0..=32_000_000 => 1,
        0..=48_000_000 => 2,
        0..=64_000_000 => 3,
        _ => 4,
    };
    // RM0432 Table 12, VCORE Range 1 (boost and normal share these steps).
    // L5: VCORE Range 0 (performance), others TODO
    #[cfg(any(rcc_l4plus, stm32l5))]
    return match hz {
        0..=20_000_000 => 0,
        0..=40_000_000 => 1,
        0..=60_000_000 => 2,
        0..=80_000_000 => 3,
        0..=100_000_000 => 4,
        _ => 5,
    };
    // VOS RANGE1, others TODO.
    #[cfg(stm32wb)]
    return match hz {
        ..=18_000_000 => 0,
        ..=36_000_000 => 1,
        ..=54_000_000 => 2,
        ..=64_000_000 => 3,
        _ => 4,
    };
    // VOS RANGE1, others TODO.
    #[cfg(stm32u0)]
    return match hz {
        ..=24_000_000 => 0,
        ..=48_000_000 => 1,
        _ => 2,
    };
}

/// The clocks to bring back after Stop.
///
/// Stop turns off HSE, the PLLs, HSI48 and whichever of MSI and HSI16 isn't the wake-up clock,
/// and the chip wakes up on MSI (at its current range) or HSI16, as chosen by STOPWUCK. L1 has no
/// STOPWUCK and always wakes up on MSI. The configuration registers, the voltage range and the
/// flash latency are kept (RM0351 6.3, RM0438 9.4, RM0503 5.3, RM0434 6.3, RM0453 7.3, RM0377
/// 6.3.9, RM0038 5.1.8). WB and WL5x also reset C2HPRE when waking up on HSI16.
#[cfg(feature = "low-power")]
#[derive(Clone, Copy)]
pub(crate) struct StopRestore {
    cr: crate::pac::rcc::regs::Cr,
    #[cfg(crs)]
    hsi48: bool,
    sw: Sysclk,
    latency: FlashLatency,
    /// The flash latency the wake-up clock needs, if higher than `latency`.
    wake_latency: Option<FlashLatency>,
    /// L4+ and L5 above 80 MHz: the AHB prescaler must divide by two across the switch.
    #[cfg(any(rcc_l4plus, stm32l5))]
    hpre_step: bool,
    #[cfg(any(stm32wl5x, stm32wb))]
    c2hpre: AHBPrescaler,
}

#[cfg(feature = "low-power")]
impl StopRestore {
    pub(crate) fn capture() -> Self {
        let cfgr = RCC.cfgr().read();

        #[cfg(any(stm32wb, stm32wl, stm32u0))]
        let wake_on_hsi = cfgr.stopwuck();
        #[cfg(not(any(stm32wb, stm32wl, stm32u0, stm32l1)))]
        let wake_on_hsi = cfgr.stopwuck() == crate::pac::rcc::vals::Stopwuck::Hsi;

        #[cfg(stm32l0)]
        let wake_hz = match (wake_on_hsi, RCC.cr().read().hsidiven()) {
            (true, false) => HSI_FREQ.0,
            (true, true) => HSI_FREQ.0 / 4,
            (false, _) => msirange_to_hertz(RCC.icscr().read().msirange()).0,
        };
        #[cfg(stm32l1)]
        let wake_hz = msirange_to_hertz(RCC.icscr().read().msirange()).0;
        #[cfg(not(any(stm32l0, stm32l1)))]
        let wake_hz = if wake_on_hsi {
            HSI_FREQ.0
        } else {
            #[cfg(not(stm32wb))]
            let hz = match RCC.cr().read().msirgsel() {
                crate::pac::rcc::vals::Msirgsel::Cr => msirange_to_hertz(RCC.cr().read().msirange()).0,
                // MSISRANGE only goes up to 8 MHz.
                crate::pac::rcc::vals::Msirgsel::Csr => 8_000_000,
            };
            #[cfg(stm32wb)]
            let hz = msirange_to_hertz(RCC.cr().read().msirange()).0;
            hz
        };

        #[cfg(any(stm32l0, stm32l1))]
        let wake_latency = flash_latency(crate::pac::PWR.cr().read().vos(), wake_hz);
        #[cfg(stm32wl)]
        let wake_latency = flash_latency(
            crate::pac::PWR.cr1().read().vos(),
            (Hertz(wake_hz) / RCC.extcfgr().read().shdhpre()).0,
        );
        #[cfg(stm32wb)]
        let wake_latency = flash_latency((Hertz(wake_hz) / RCC.extcfgr().read().shdhpre()).0);
        #[cfg(any(stm32l4, stm32l5, stm32u0))]
        let wake_latency = flash_latency((Hertz(wake_hz) / cfgr.hpre()).0);

        let latency = FLASH.acr().read().latency();

        Self {
            cr: RCC.cr().read(),
            #[cfg(crs)]
            hsi48: super::hsi48_enabled(),
            sw: cfgr.sws(),
            latency,
            wake_latency: (wake_latency > latency).then_some(wake_latency),
            #[cfg(any(rcc_l4plus, stm32l5))]
            hpre_step: unsafe { super::get_freqs() }.sys.to_hertz().unwrap().0 > 80_000_000
                && cfgr.hpre() == AHBPrescaler::Div1,
            #[cfg(any(stm32wl5x, stm32wb))]
            c2hpre: RCC.extcfgr().read().c2hpre(),
        }
    }

    pub(crate) fn enter(&self) {
        if let Some(latency) = self.wake_latency {
            FLASH.acr().modify(|w| w.set_latency(latency));
            while FLASH.acr().read().latency() != latency {}
        }
    }

    pub(crate) fn exit(&self) {
        let s = self.cr;
        RCC.cr().modify(|w| {
            w.set_hseon(w.hseon() | s.hseon());
            w.set_hsion(w.hsion() | s.hsion());
            w.set_msion(w.msion() | s.msion());
        });
        #[cfg(crs)]
        if self.hsi48 {
            super::hsi48_enable();
        }
        loop {
            let cr = RCC.cr().read();
            if (cr.hserdy() | !s.hseon()) && (cr.hsirdy() | !s.hsion()) && (cr.msirdy() | !s.msion()) {
                break;
            }
        }

        RCC.cr().modify(|w| {
            w.set_pllon(w.pllon() | s.pllon());
            #[cfg(any(stm32l4, stm32l5, stm32wb))]
            w.set_pllsai1on(w.pllsai1on() | s.pllsai1on());
            #[cfg(any(stm32l47x, stm32l48x, stm32l49x, stm32l4ax, rcc_l4plus, stm32l5))]
            w.set_pllsai2on(w.pllsai2on() | s.pllsai2on());
        });
        loop {
            let cr = RCC.cr().read();
            #[allow(unused_mut)]
            let mut ready = cr.pllrdy() | !s.pllon();
            #[cfg(any(stm32l4, stm32l5, stm32wb))]
            {
                ready &= cr.pllsai1rdy() | !s.pllsai1on();
            }
            #[cfg(any(stm32l47x, stm32l48x, stm32l49x, stm32l4ax, rcc_l4plus, stm32l5))]
            {
                ready &= cr.pllsai2rdy() | !s.pllsai2on();
            }
            if ready {
                break;
            }
        }
        #[cfg(crs)]
        if self.hsi48 {
            super::hsi48_wait_ready();
        }

        if RCC.cfgr().read().sws() != self.sw {
            #[cfg(any(rcc_l4plus, stm32l5))]
            if self.hpre_step {
                RCC.cfgr().modify(|w| w.set_hpre(AHBPrescaler::Div2));
            }
            RCC.cfgr().modify(|w| w.set_sw(self.sw));
            while RCC.cfgr().read().sws() != self.sw {}
            #[cfg(any(rcc_l4plus, stm32l5))]
            if self.hpre_step {
                // RM0432 5.1.8: wait at least 1 us before going to full speed.
                cortex_m::asm::delay(unsafe { super::get_freqs() }.sys.to_hertz().unwrap().0 / 1_000_000);
                RCC.cfgr().modify(|w| w.set_hpre(AHBPrescaler::Div1));
            }
        }

        #[cfg(any(stm32wl5x, stm32wb))]
        if RCC.extcfgr().read().c2hpre() != self.c2hpre {
            RCC.extcfgr().modify(|w| w.set_c2hpre(self.c2hpre));
            while !RCC.extcfgr().read().c2hpref() {}
        }

        if self.wake_latency.is_some() {
            FLASH.acr().modify(|w| w.set_latency(self.latency));
            while FLASH.acr().read().latency() != self.latency {}
        }

        if !s.hsion() || !s.msion() {
            RCC.cr().modify(|w| {
                w.set_hsion(w.hsion() & s.hsion());
                w.set_msion(w.msion() & s.msion());
            });
        }
    }
}

/// Switch the system clock to HSI16 before Stop, as the WB coordination with CPU2 in
/// `low_power` does. `StopRestore::exit` switches back after Stop.
#[cfg(all(feature = "low-power", stm32wb))]
pub(crate) fn stop_switch_to_hsi() {
    RCC.cr().modify(|w| w.set_hsion(true));
    while !RCC.cr().read().hsirdy() {}
    RCC.cfgr().modify(|w| w.set_sw(Sysclk::Hsi));
    while RCC.cfgr().read().sws() != Sysclk::Hsi {}
    RCC.smpscr().modify(|w| w.set_smpssel(crate::pac::rcc::vals::Smps::Hsi));
}

#[cfg(any(stm32l0, stm32l1))]
fn msirange_to_hertz(range: MSIRange) -> Hertz {
    Hertz(32_768 * (1 << (range as u8 + 1)))
}

#[cfg(any(stm32l4, stm32l5, stm32wb, stm32wl, stm32u0))]
fn msirange_to_hertz(range: MSIRange) -> Hertz {
    match range {
        MSIRange::Range100k => Hertz(100_000),
        MSIRange::Range200k => Hertz(200_000),
        MSIRange::Range400k => Hertz(400_000),
        MSIRange::Range800k => Hertz(800_000),
        MSIRange::Range1m => Hertz(1_000_000),
        MSIRange::Range2m => Hertz(2_000_000),
        MSIRange::Range4m => Hertz(4_000_000),
        MSIRange::Range8m => Hertz(8_000_000),
        MSIRange::Range16m => Hertz(16_000_000),
        MSIRange::Range24m => Hertz(24_000_000),
        MSIRange::Range32m => Hertz(32_000_000),
        MSIRange::Range48m => Hertz(48_000_000),
        _ => unreachable!(),
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum PllInstance {
    Pll,
    #[cfg(any(stm32l4, stm32l5, stm32wb))]
    Pllsai1,
    #[cfg(any(stm32l47x, stm32l48x, stm32l49x, stm32l4ax, rcc_l4plus, stm32l5))]
    Pllsai2,
}

fn pll_enable(instance: PllInstance, enabled: bool) {
    match instance {
        PllInstance::Pll => {
            RCC.cr().modify(|w| w.set_pllon(enabled));
            while RCC.cr().read().pllrdy() != enabled {}
        }
        #[cfg(any(stm32l4, stm32l5, stm32wb))]
        PllInstance::Pllsai1 => {
            RCC.cr().modify(|w| w.set_pllsai1on(enabled));
            while RCC.cr().read().pllsai1rdy() != enabled {}
        }
        #[cfg(any(stm32l47x, stm32l48x, stm32l49x, stm32l4ax, rcc_l4plus, stm32l5))]
        PllInstance::Pllsai2 => {
            RCC.cr().modify(|w| w.set_pllsai2on(enabled));
            while RCC.cr().read().pllsai2rdy() != enabled {}
        }
    }
}

pub use pll::*;

#[cfg(any(stm32l0, stm32l1))]
mod pll {
    use super::{PllInstance, pll_enable};
    use crate::pac::RCC;
    pub use crate::pac::rcc::vals::{Plldiv as PllDiv, Pllmul as PllMul, Pllsrc as PllSource};
    use crate::time::Hertz;

    #[derive(Clone, Copy)]
    pub struct Pll {
        /// PLL source
        pub source: PllSource,

        /// PLL multiplication factor.
        pub mul: PllMul,

        /// PLL main output division factor.
        pub div: PllDiv,
    }

    pub(super) struct PllInput {
        pub hsi: Option<Hertz>,
        pub hse: Option<Hertz>,
    }

    #[allow(unused)]
    #[derive(Default)]
    pub(super) struct PllOutput {
        pub r: Option<Hertz>,
        pub vco: Option<Hertz>,
    }

    pub(super) fn init_pll(instance: PllInstance, config: Option<Pll>, input: &PllInput) -> PllOutput {
        // Disable PLL
        pll_enable(instance, false);

        let Some(pll) = config else { return PllOutput::default() };

        let pll_src = match pll.source {
            PllSource::Hse => unwrap!(input.hse),
            PllSource::Hsi => unwrap!(input.hsi),
        };

        let vco_freq = pll_src * pll.mul;

        let r = vco_freq / pll.div;

        assert!(r <= Hertz(32_000_000));

        RCC.cfgr().write(move |w| {
            w.set_pllmul(pll.mul);
            w.set_plldiv(pll.div);
            w.set_pllsrc(pll.source);
        });

        // Enable PLL
        pll_enable(instance, true);

        PllOutput {
            r: Some(r),
            vco: Some(vco_freq),
        }
    }
}

#[cfg(any(stm32l4, stm32l5, stm32wb, stm32wl, stm32u0))]
mod pll {
    use super::{PllInstance, pll_enable};
    use crate::pac::RCC;
    pub use crate::pac::rcc::vals::{
        Pllm as PllPreDiv, Plln as PllMul, Pllp as PllPDiv, Pllq as PllQDiv, Pllr as PllRDiv, Pllsrc as PllSource,
    };
    use crate::time::Hertz;

    #[derive(Clone, Copy)]
    pub struct Pll {
        /// PLL source
        pub source: PllSource,

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

    pub(super) struct PllInput {
        pub hsi: Option<Hertz>,
        pub hse: Option<Hertz>,
        pub msi: Option<Hertz>,
    }

    #[allow(unused)]
    #[derive(Default)]
    pub(super) struct PllOutput {
        pub p: Option<Hertz>,
        pub q: Option<Hertz>,
        pub r: Option<Hertz>,
    }

    pub(super) fn init_pll(instance: PllInstance, config: Option<Pll>, input: &PllInput) -> PllOutput {
        // Disable PLL
        pll_enable(instance, false);

        let Some(pll) = config else { return PllOutput::default() };

        let pll_src = match pll.source {
            PllSource::Disable => panic!("must not select PLL source as DISABLE"),
            PllSource::Hse => unwrap!(input.hse),
            PllSource::Hsi => unwrap!(input.hsi),
            PllSource::Msi => unwrap!(input.msi),
        };

        let vco_freq = pll_src / pll.prediv * pll.mul;

        let p = pll.divp.map(|div| vco_freq / div);
        let q = pll.divq.map(|div| vco_freq / div);
        let r = pll.divr.map(|div| vco_freq / div);

        #[cfg(stm32l5)]
        if instance == PllInstance::Pllsai2 {
            assert!(q.is_none(), "PLLSAI2_Q is not available on L5");
            assert!(r.is_none(), "PLLSAI2_R is not available on L5");
        }

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
            #[cfg(any(stm32l4, stm32l5, stm32wb))]
            PllInstance::Pllsai1 => RCC.pllsai1cfgr().write(|w| {
                #[cfg(any(rcc_l4plus, stm32l5))]
                w.set_pllm(pll.prediv);
                #[cfg(stm32l5)]
                w.set_pllsrc(pll.source);
                write_fields!(w);
            }),
            #[cfg(any(stm32l47x, stm32l48x, stm32l49x, stm32l4ax, rcc_l4plus, stm32l5))]
            PllInstance::Pllsai2 => RCC.pllsai2cfgr().write(|w| {
                #[cfg(any(rcc_l4plus, stm32l5))]
                w.set_pllm(pll.prediv);
                #[cfg(stm32l5)]
                w.set_pllsrc(pll.source);
                write_fields!(w);
            }),
        }

        // Enable PLL
        pll_enable(instance, true);

        PllOutput { p, q, r }
    }
}
