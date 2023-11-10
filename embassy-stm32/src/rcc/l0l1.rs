pub use crate::pac::pwr::vals::Vos as VoltageScale;
pub use crate::pac::rcc::vals::{
    Hpre as AHBPrescaler, Msirange as MSIRange, Plldiv as PLLDiv, Plldiv as PllDiv, Pllmul as PLLMul, Pllmul as PllMul,
    Pllsrc as PLLSource, Ppre as APBPrescaler, Sw as ClockSrc,
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
    /// PLL source
    pub source: PLLSource,

    /// PLL multiplication factor.
    pub mul: PllMul,

    /// PLL main output division factor.
    pub div: PllDiv,
}

/// Clocks configutation
pub struct Config {
    // base clock sources
    pub msi: Option<MSIRange>,
    pub hsi: bool,
    pub hse: Option<Hse>,
    #[cfg(crs)]
    pub hsi48: Option<super::Hsi48Config>,

    pub pll: Option<Pll>,

    pub mux: ClockSrc,
    pub ahb_pre: AHBPrescaler,
    pub apb1_pre: APBPrescaler,
    pub apb2_pre: APBPrescaler,

    pub ls: super::LsConfig,
    pub voltage_scale: VoltageScale,
}

impl Default for Config {
    #[inline]
    fn default() -> Config {
        Config {
            msi: Some(MSIRange::RANGE5),
            hse: None,
            hsi: false,
            #[cfg(crs)]
            hsi48: Some(Default::default()),

            pll: None,

            mux: ClockSrc::MSI,
            ahb_pre: AHBPrescaler::DIV1,
            apb1_pre: APBPrescaler::DIV1,
            apb2_pre: APBPrescaler::DIV1,
            voltage_scale: VoltageScale::RANGE1,
            ls: Default::default(),
        }
    }
}

pub(crate) unsafe fn init(config: Config) {
    // Set voltage scale
    while PWR.csr().read().vosf() {}
    PWR.cr().write(|w| w.set_vos(config.voltage_scale));
    while PWR.csr().read().vosf() {}

    let rtc = config.ls.init();

    let msi = config.msi.map(|range| {
        RCC.icscr().modify(|w| w.set_msirange(range));

        RCC.cr().modify(|w| w.set_msion(true));
        while !RCC.cr().read().msirdy() {}

        Hertz(32_768 * (1 << (range as u8 + 1)))
    });

    let hsi = config.hsi.then(|| {
        RCC.cr().modify(|w| w.set_hsion(true));
        while !RCC.cr().read().hsirdy() {}

        HSI_FREQ
    });

    let hse = config.hse.map(|hse| {
        RCC.cr().modify(|w| {
            w.set_hsebyp(hse.mode == HseMode::Bypass);
            w.set_hseon(true);
        });
        while !RCC.cr().read().hserdy() {}

        hse.freq
    });

    let pll = config.pll.map(|pll| {
        let freq = match pll.source {
            PLLSource::HSE => hse.unwrap(),
            PLLSource::HSI => hsi.unwrap(),
        };

        // Disable PLL
        RCC.cr().modify(|w| w.set_pllon(false));
        while RCC.cr().read().pllrdy() {}

        let freq = freq * pll.mul / pll.div;

        assert!(freq <= Hertz(32_000_000));

        RCC.cfgr().write(move |w| {
            w.set_pllmul(pll.mul);
            w.set_plldiv(pll.div);
            w.set_pllsrc(pll.source);
        });

        // Enable PLL
        RCC.cr().modify(|w| w.set_pllon(true));
        while !RCC.cr().read().pllrdy() {}

        freq
    });

    let sys_clk = match config.mux {
        ClockSrc::HSE => hse.unwrap(),
        ClockSrc::HSI => hsi.unwrap(),
        ClockSrc::MSI => msi.unwrap(),
        ClockSrc::PLL1_P => pll.unwrap(),
    };

    let wait_states = match (config.voltage_scale, sys_clk.0) {
        (VoltageScale::RANGE1, ..=16_000_000) => 0,
        (VoltageScale::RANGE2, ..=8_000_000) => 0,
        (VoltageScale::RANGE3, ..=4_200_000) => 0,
        _ => 1,
    };

    #[cfg(stm32l1)]
    FLASH.acr().write(|w| w.set_acc64(true));
    FLASH.acr().modify(|w| w.set_prften(true));
    FLASH.acr().modify(|w| w.set_latency(wait_states != 0));

    RCC.cfgr().modify(|w| {
        w.set_sw(config.mux);
        w.set_hpre(config.ahb_pre);
        w.set_ppre1(config.apb1_pre);
        w.set_ppre2(config.apb2_pre);
    });

    let hclk1 = sys_clk / config.ahb_pre;
    let (pclk1, pclk1_tim) = super::util::calc_pclk(hclk1, config.apb1_pre);
    let (pclk2, pclk2_tim) = super::util::calc_pclk(hclk1, config.apb2_pre);

    #[cfg(crs)]
    let _hsi48 = config.hsi48.map(|config| {
        // Select HSI48 as USB clock
        RCC.ccipr().modify(|w| w.set_hsi48msel(true));
        super::init_hsi48(config)
    });

    set_freqs(Clocks {
        sys: sys_clk,
        hclk1,
        pclk1,
        pclk2,
        pclk1_tim,
        pclk2_tim,
        rtc,
    });
}
