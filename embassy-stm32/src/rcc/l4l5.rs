use crate::pac::rcc::regs::Cfgr;
#[cfg(any(stm32l4, stm32l5, stm32wb))]
pub use crate::pac::rcc::vals::Clk48sel as Clk48Src;
#[cfg(any(stm32wb, stm32wl))]
pub use crate::pac::rcc::vals::Hsepre as HsePrescaler;
pub use crate::pac::rcc::vals::{
    Adcsel as AdcClockSource, Hpre as AHBPrescaler, Msirange as MSIRange, Pllm as PllPreDiv, Plln as PllMul,
    Pllp as PllPDiv, Pllq as PllQDiv, Pllr as PllRDiv, Pllsrc as PllSource, Ppre as APBPrescaler, Sw as ClockSrc,
};
use crate::pac::{FLASH, RCC};
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
    /// HSE prescaler
    #[cfg(any(stm32wb, stm32wl))]
    pub prescaler: HsePrescaler,
}

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

/// Clocks configuration
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
    pub mux: ClockSrc,
    pub ahb_pre: AHBPrescaler,
    pub apb1_pre: APBPrescaler,
    pub apb2_pre: APBPrescaler,
    #[cfg(any(stm32wl5x, stm32wb))]
    pub core2_ahb_pre: AHBPrescaler,
    #[cfg(any(stm32wl, stm32wb))]
    pub shared_ahb_pre: AHBPrescaler,

    // muxes
    #[cfg(any(stm32l4, stm32l5, stm32wb))]
    pub clk48_src: Clk48Src,

    // low speed LSI/LSE/RTC
    pub ls: super::LsConfig,

    pub adc_clock_source: AdcClockSource,
}

impl Default for Config {
    #[inline]
    fn default() -> Config {
        Config {
            hse: None,
            hsi: false,
            msi: Some(MSIRange::RANGE4M),
            mux: ClockSrc::MSI,
            ahb_pre: AHBPrescaler::DIV1,
            apb1_pre: APBPrescaler::DIV1,
            apb2_pre: APBPrescaler::DIV1,
            #[cfg(any(stm32wl5x, stm32wb))]
            core2_ahb_pre: AHBPrescaler::DIV1,
            #[cfg(any(stm32wl, stm32wb))]
            shared_ahb_pre: AHBPrescaler::DIV1,
            pll: None,
            #[cfg(any(stm32l4, stm32l5, stm32wb))]
            pllsai1: None,
            #[cfg(any(stm32l47x, stm32l48x, stm32l49x, stm32l4ax, rcc_l4plus, stm32l5))]
            pllsai2: None,
            #[cfg(crs)]
            hsi48: Some(Default::default()),
            #[cfg(any(stm32l4, stm32l5, stm32wb))]
            clk48_src: Clk48Src::HSI48,
            ls: Default::default(),
            adc_clock_source: AdcClockSource::SYS,
        }
    }
}

#[cfg(stm32wb)]
pub const WPAN_DEFAULT: Config = Config {
    hse: Some(Hse {
        freq: Hertz(32_000_000),
        mode: HseMode::Oscillator,
        prescaler: HsePrescaler::DIV1,
    }),
    mux: ClockSrc::PLL1_R,
    #[cfg(crs)]
    hsi48: Some(super::Hsi48Config { sync_from_usb: false }),
    msi: None,
    hsi: false,
    clk48_src: Clk48Src::PLL1_Q,

    ls: super::LsConfig::default_lse(),

    pll: Some(Pll {
        source: PllSource::HSE,
        prediv: PllPreDiv::DIV2,
        mul: PllMul::MUL12,
        divp: Some(PllPDiv::DIV3), // 32 / 2 * 12 / 3 = 64Mhz
        divq: Some(PllQDiv::DIV4), // 32 / 2 * 12 / 4 = 48Mhz
        divr: Some(PllRDiv::DIV3), // 32 / 2 * 12 / 3 = 64Mhz
    }),
    pllsai1: None,

    ahb_pre: AHBPrescaler::DIV1,
    core2_ahb_pre: AHBPrescaler::DIV2,
    shared_ahb_pre: AHBPrescaler::DIV1,
    apb1_pre: APBPrescaler::DIV1,
    apb2_pre: APBPrescaler::DIV1,
    adc_clock_source: AdcClockSource::SYS,
};

pub(crate) unsafe fn init(config: Config) {
    // Switch to MSI to prevent problems with PLL configuration.
    if !RCC.cr().read().msion() {
        // Turn on MSI and configure it to 4MHz.
        RCC.cr().modify(|w| {
            #[cfg(not(stm32wb))]
            w.set_msirgsel(crate::pac::rcc::vals::Msirgsel::CR);
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

    #[cfg(stm32l5)]
    crate::pac::PWR.cr1().modify(|w| {
        w.set_vos(crate::pac::pwr::vals::Vos::RANGE0);
    });

    let rtc = config.ls.init();

    let msi = config.msi.map(|range| {
        // Enable MSI
        RCC.cr().modify(|w| {
            #[cfg(not(stm32wb))]
            w.set_msirgsel(crate::pac::rcc::vals::Msirgsel::CR);
            w.set_msirange(range);
            w.set_msion(true);

            // If LSE is enabled, enable calibration of MSI
            w.set_msipllen(config.ls.lse.is_some());
        });
        while !RCC.cr().read().msirdy() {}

        msirange_to_hertz(range)
    });

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
    let _hsi48 = config.hsi48.map(super::init_hsi48);
    #[cfg(not(crs))]
    let _hsi48: Option<Hertz> = None;

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

    let pll_input = PllInput { hse, hsi, msi };
    let pll = init_pll(PllInstance::Pll, config.pll, &pll_input);
    #[cfg(any(stm32l4, stm32l5, stm32wb))]
    let pllsai1 = init_pll(PllInstance::Pllsai1, config.pllsai1, &pll_input);
    #[cfg(any(stm32l47x, stm32l48x, stm32l49x, stm32l4ax, rcc_l4plus, stm32l5))]
    let _pllsai2 = init_pll(PllInstance::Pllsai2, config.pllsai2, &pll_input);

    let sys_clk = match config.mux {
        ClockSrc::HSE => hse.unwrap(),
        ClockSrc::HSI => hsi.unwrap(),
        ClockSrc::MSI => msi.unwrap(),
        ClockSrc::PLL1_R => pll.r.unwrap(),
    };

    #[cfg(stm32l4)]
    RCC.ccipr().modify(|w| w.set_clk48sel(config.clk48_src));
    #[cfg(stm32l5)]
    RCC.ccipr1().modify(|w| w.set_clk48sel(config.clk48_src));
    #[cfg(any(stm32l4, stm32l5, stm32wb))]
    let _clk48 = match config.clk48_src {
        Clk48Src::HSI48 => _hsi48,
        Clk48Src::MSI => msi,
        Clk48Src::PLLSAI1_Q => pllsai1.q,
        Clk48Src::PLL1_Q => pll.q,
    };

    #[cfg(rcc_l4plus)]
    assert!(sys_clk.0 <= 120_000_000);
    #[cfg(all(stm32l4, not(rcc_l4plus)))]
    assert!(sys_clk.0 <= 80_000_000);

    let hclk1 = sys_clk / config.ahb_pre;
    let (pclk1, pclk1_tim) = super::util::calc_pclk(hclk1, config.apb1_pre);
    let (pclk2, pclk2_tim) = super::util::calc_pclk(hclk1, config.apb2_pre);
    #[cfg(not(any(stm32wl5x, stm32wb)))]
    let hclk2 = hclk1;
    #[cfg(any(stm32wl5x, stm32wb))]
    let hclk2 = sys_clk / config.core2_ahb_pre;
    #[cfg(not(any(stm32wl, stm32wb)))]
    let hclk3 = hclk1;
    #[cfg(any(stm32wl, stm32wb))]
    let hclk3 = sys_clk / config.shared_ahb_pre;

    // Set flash wait states
    #[cfg(stm32l4)]
    let latency = match hclk1.0 {
        0..=16_000_000 => 0,
        0..=32_000_000 => 1,
        0..=48_000_000 => 2,
        0..=64_000_000 => 3,
        _ => 4,
    };
    #[cfg(stm32l5)]
    let latency = match hclk1.0 {
        // VCORE Range 0 (performance), others TODO
        0..=20_000_000 => 0,
        0..=40_000_000 => 1,
        0..=60_000_000 => 2,
        0..=80_000_000 => 3,
        0..=100_000_000 => 4,
        _ => 5,
    };
    #[cfg(stm32wl)]
    let latency = match hclk3.0 {
        // VOS RANGE1, others TODO.
        ..=18_000_000 => 0,
        ..=36_000_000 => 1,
        _ => 2,
    };
    #[cfg(stm32wb)]
    let latency = match hclk3.0 {
        // VOS RANGE1, others TODO.
        ..=18_000_000 => 0,
        ..=36_000_000 => 1,
        ..=54_000_000 => 2,
        ..=64_000_000 => 3,
        _ => 4,
    };

    FLASH.acr().modify(|w| w.set_latency(latency));
    while FLASH.acr().read().latency() != latency {}

    RCC.cfgr().modify(|w| {
        w.set_sw(config.mux);
        w.set_hpre(config.ahb_pre);
        w.set_ppre1(config.apb1_pre);
        w.set_ppre2(config.apb2_pre);
    });
    while RCC.cfgr().read().sws() != config.mux {}

    #[cfg(stm32l5)]
    RCC.ccipr1().modify(|w| w.set_adcsel(config.adc_clock_source));
    #[cfg(not(stm32l5))]
    RCC.ccipr().modify(|w| w.set_adcsel(config.adc_clock_source));

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

    set_freqs(Clocks {
        sys: sys_clk,
        hclk1,
        hclk2,
        hclk3,
        pclk1,
        pclk2,
        pclk1_tim,
        pclk2_tim,
        #[cfg(stm32wl)]
        pclk3: hclk3,
        #[cfg(rcc_l4)]
        hsi: None,
        #[cfg(rcc_l4)]
        lse: None,
        #[cfg(rcc_l4)]
        pllsai1_p: None,
        #[cfg(rcc_l4)]
        pllsai2_p: None,
        #[cfg(rcc_l4)]
        pll1_p: None,
        #[cfg(rcc_l4)]
        pll1_q: None,
        #[cfg(rcc_l4)]
        sai1_extclk: None,
        #[cfg(rcc_l4)]
        sai2_extclk: None,
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

struct PllInput {
    hsi: Option<Hertz>,
    hse: Option<Hertz>,
    msi: Option<Hertz>,
}

#[allow(unused)]
#[derive(Default)]
struct PllOutput {
    p: Option<Hertz>,
    q: Option<Hertz>,
    r: Option<Hertz>,
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

fn init_pll(instance: PllInstance, config: Option<Pll>, input: &PllInput) -> PllOutput {
    // Disable PLL
    pll_enable(instance, false);

    let Some(pll) = config else { return PllOutput::default() };

    let pll_src = match pll.source {
        PllSource::DISABLE => panic!("must not select PLL source as DISABLE"),
        PllSource::HSE => input.hse,
        PllSource::HSI => input.hsi,
        PllSource::MSI => input.msi,
    };

    let pll_src = pll_src.unwrap();

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
