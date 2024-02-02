pub use crate::pac::pwr::vals::Vos as VoltageScale;
use crate::pac::rcc::regs::Cfgr1;
pub use crate::pac::rcc::vals::{
    Adcsel as AdcClockSource, Hpre as AHBPrescaler, Hsepre as HsePrescaler, Ppre as APBPrescaler, Sw as ClockSrc,
};
use crate::pac::{FLASH, RCC};
use crate::time::Hertz;

/// HSI speed
pub const HSI_FREQ: Hertz = Hertz(16_000_000);
// HSE speed
pub const HSE_FREQ: Hertz = Hertz(32_000_000);

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Hse {
    pub prescaler: HsePrescaler,
}

/// Clocks configuration
pub struct Config {
    // base clock sources
    pub hsi: bool,
    pub hse: Option<Hse>,

    // sysclk, buses.
    pub mux: ClockSrc,
    pub ahb_pre: AHBPrescaler,
    pub apb1_pre: APBPrescaler,
    pub apb2_pre: APBPrescaler,
    pub apb7_pre: APBPrescaler,

    // low speed LSI/LSE/RTC
    pub ls: super::LsConfig,

    pub adc_clock_source: AdcClockSource,

    pub voltage_scale: VoltageScale,
}

impl Default for Config {
    #[inline]
    fn default() -> Config {
        Config {
            hse: None,
            hsi: true,
            mux: ClockSrc::HSI,
            ahb_pre: AHBPrescaler::DIV1,
            apb1_pre: APBPrescaler::DIV1,
            apb2_pre: APBPrescaler::DIV1,
            apb7_pre: APBPrescaler::DIV1,
            ls: Default::default(),
            adc_clock_source: AdcClockSource::HCLK1,
            voltage_scale: VoltageScale::RANGE2,
        }
    }
}

fn hsi_enable() {
    RCC.cr().modify(|w| w.set_hsion(true));
    while !RCC.cr().read().hsirdy() {}
}

pub(crate) unsafe fn init(config: Config) {
    // Switch to HSI to prevent problems with PLL configuration.
    if !RCC.cr().read().hsion() {
        hsi_enable()
    }
    if RCC.cfgr1().read().sws() != ClockSrc::HSI {
        // Set HSI as a clock source, reset prescalers.
        RCC.cfgr1().write_value(Cfgr1::default());
        // Wait for clock switch status bits to change.
        while RCC.cfgr1().read().sws() != ClockSrc::HSI {}
    }

    // Set voltage scale
    crate::pac::PWR.vosr().write(|w| w.set_vos(config.voltage_scale));
    while !crate::pac::PWR.vosr().read().vosrdy() {}

    let rtc = config.ls.init();

    let hsi = config.hsi.then(|| {
        hsi_enable();

        HSI_FREQ
    });

    let hse = config.hse.map(|hse| {
        RCC.cr().write(|w| {
            w.set_hseon(true);
            w.set_hsepre(hse.prescaler);
        });
        while !RCC.cr().read().hserdy() {}

        HSE_FREQ
    });

    let sys_clk = match config.mux {
        ClockSrc::HSE => hse.unwrap(),
        ClockSrc::HSI => hsi.unwrap(),
        ClockSrc::_RESERVED_1 => unreachable!(),
        ClockSrc::PLL1_R => todo!(),
    };

    assert!(sys_clk.0 <= 100_000_000);

    let hclk1 = sys_clk / config.ahb_pre;
    let hclk2 = hclk1;
    let hclk4 = hclk1;
    // TODO: hclk5
    let (pclk1, pclk1_tim) = super::util::calc_pclk(hclk1, config.apb1_pre);
    let (pclk2, pclk2_tim) = super::util::calc_pclk(hclk1, config.apb2_pre);
    let (pclk7, _) = super::util::calc_pclk(hclk1, config.apb7_pre);

    // Set flash wait states
    let flash_latency = match config.voltage_scale {
        VoltageScale::RANGE1 => match sys_clk.0 {
            ..=32_000_000 => 0,
            ..=64_000_000 => 1,
            ..=96_000_000 => 2,
            ..=100_000_000 => 3,
            _ => 4,
        },
        VoltageScale::RANGE2 => match sys_clk.0 {
            ..=8_000_000 => 0,
            ..=16_000_000 => 1,
            _ => 2,
        },
    };

    FLASH.acr().modify(|w| w.set_latency(flash_latency));
    while FLASH.acr().read().latency() != flash_latency {}

    // Set sram wait states
    let _sram_latency = match config.voltage_scale {
        VoltageScale::RANGE1 => 0,
        VoltageScale::RANGE2 => match sys_clk.0 {
            ..=12_000_000 => 0,
            ..=16_000_000 => 1,
            _ => 2,
        },
    };
    // TODO: Set the SRAM wait states

    RCC.cfgr1().modify(|w| {
        w.set_sw(config.mux);
    });
    while RCC.cfgr1().read().sws() != config.mux {}

    RCC.cfgr2().modify(|w| {
        w.set_hpre(config.ahb_pre);
        w.set_ppre1(config.apb1_pre);
        w.set_ppre2(config.apb2_pre);
    });

    RCC.ccipr3().modify(|w| w.set_adcsel(config.adc_clock_source));

    set_clocks!(
        sys: Some(sys_clk),
        hclk1: Some(hclk1),
        hclk2: Some(hclk2),
        hclk4: Some(hclk4),
        pclk1: Some(pclk1),
        pclk2: Some(pclk2),
        pclk7: Some(pclk7),
        pclk1_tim: Some(pclk1_tim),
        pclk2_tim: Some(pclk2_tim),
        rtc: rtc,
        hse: hse,
        hsi: hsi,

        // TODO
        lse: None,
        lsi: None,
        pll1_q: None,
    );
}
