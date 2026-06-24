use stm32_metapac::FLASH;
use stm32_metapac::rcc::vals::Hseext;

use crate::pac::RCC;
pub use crate::pac::rcc::vals::{Hpre as AHBPrescaler, Ppre as APBPrescaler, Sw as Sysclk};
use crate::time::Hertz;

/// HSI speed
pub const HSI_FREQ: Hertz = Hertz(144_000_000);

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

/// Configuration of the core clocks
#[non_exhaustive]
#[derive(Clone, Copy)]
pub struct Config {
    /// Enable HSI full speed tap (144MHz)
    pub hsi: bool,

    /// Enable HSI Div 3 tap (48MHz)
    pub hsi_div3: bool,

    pub hse: Option<Hse>,

    /// System Clock Configuration
    pub sys: Sysclk,

    pub ahb_pre: AHBPrescaler,
    pub apb1_pre: APBPrescaler,
    pub apb2_pre: APBPrescaler,
    pub apb3_pre: APBPrescaler,

    /// Per-peripheral kernel clock selection muxes
    pub mux: super::mux::ClockMux,
}

impl Config {
    pub const fn new() -> Self {
        Self {
            hsi: false,
            hsi_div3: true,
            hse: None,
            sys: Sysclk::Hsidiv3,
            ahb_pre: AHBPrescaler::Div1,
            apb1_pre: APBPrescaler::Div1,
            apb2_pre: APBPrescaler::Div1,
            apb3_pre: APBPrescaler::Div1,
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
    // Turn on clock sources. Keep hsidiv3(the default)
    // on for now since that will be used during init
    RCC.cr().modify(|w| {
        w.set_hsion(config.hsi);
        w.set_hsidiv3on(true);
    });

    if config.hsi {
        while !RCC.cr().read().hsirdy() {}
    }
    if config.hsi_div3 {
        while !RCC.cr().read().hsidiv3rdy() {}
    }

    // Use the HSI/3 clock as system clock during the actual clock setup
    RCC.cfgr().modify(|w| w.set_sw(Sysclk::Hsidiv3));
    while RCC.cfgr().read().sws() != Sysclk::Hsidiv3 {}

    // Configure HSI
    let hsi = config.hsi.then_some(HSI_FREQ);
    let hsi_div3 = config.hsi_div3.then_some(Hertz(HSI_FREQ.0 / 3));

    // Turn off unused clock sources
    RCC.cr().modify(|w| {
        w.set_hsion(config.hsi);
        w.set_hsidiv3on(config.hsi_div3);
    });

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

    // Configure sysclk
    let sys = match config.sys {
        Sysclk::Hsidiv3 => unwrap!(hsi_div3),
        Sysclk::Hsi => unwrap!(hsi),
        Sysclk::Hse => unwrap!(hse),
        Sysclk::Psi => unimplemented!(),
    };
    assert!(max::SYSCLK.contains(&sys));

    let hclk = sys / config.ahb_pre;
    assert!(max::HCLK.contains(&hclk));

    let apb1 = hclk / config.apb1_pre;
    assert!(max::PCLK.contains(&apb1));

    let apb2 = hclk / config.apb2_pre;
    assert!(max::PCLK.contains(&apb2));

    let apb3 = hclk / config.apb3_pre;
    assert!(max::PCLK.contains(&apb3));

    flash_setup(hclk);

    //let rtc = config.ls.init();

    RCC.cfgr2().modify(|w| w.set_hpre(config.ahb_pre));
    while RCC.cfgr2().read().hpre() != config.ahb_pre {}

    RCC.cfgr2().modify(|w| {
        w.set_ppre1(config.apb1_pre);
        w.set_ppre2(config.apb2_pre);
        w.set_ppre3(config.apb3_pre);
    });

    RCC.cfgr().modify(|w| w.set_sw(config.sys));
    while RCC.cfgr().read().sws() != config.sys {}

    config.mux.init();

    set_clocks!(
        sys: Some(sys),
        hclk1: Some(hclk),
        pclk1: Some(apb1),
        pclk1_tim: Some(hclk),
        pclk2: Some(apb2),
        pclk2_tim: Some(hclk),

        pclk3: Some(apb3),

        hsi: hsi,
        hsik: None,
        hse: hse,
        psi: None,
        psik: None,

        // TODO
        lsi: None,
        lse: None,

        rtc: None,
    );
}

fn flash_setup(hclk: Hertz) {
    let (latency, wrhighfreq) = match hclk.0 {
        ..=34_000_000 => (0, 0b00),
        ..=68_000_000 => (1, 0b00),
        ..=102_000_000 => (2, 0b01),
        ..=136_000_000 => (3, 0b01),
        ..=144_000_000 => (4, 0b10),
        _ => unreachable!(),
    };
    let latency = latency.into();

    debug!("flash: latency={} wrhighfreq={}", latency, wrhighfreq);

    FLASH.acr().write(|w| {
        w.set_wrhighfreq(wrhighfreq);
        w.set_latency(latency);
    });
    while FLASH.acr().read().latency() != latency {}
}

mod max {
    use core::ops::RangeInclusive;

    use crate::time::Hertz;

    // pub(crate) const HSE_OSC: RangeInclusive<Hertz> = Hertz(4_000_000)..=Hertz(50_000_000);
    // pub(crate) const HSE_BYP_ANALOG: RangeInclusive<Hertz> = Hertz(4_000_000)..=Hertz(50_000_000);
    // pub(crate) const HSE_BYP_DIGITAL: RangeInclusive<Hertz> = Hertz(0)..=Hertz(50_000_000);
    pub(crate) const SYSCLK: RangeInclusive<Hertz> = Hertz(0)..=Hertz(144_000_000);
    pub(crate) const PCLK: RangeInclusive<Hertz> = Hertz(0)..=Hertz(144_000_000);
    pub(crate) const HCLK: RangeInclusive<Hertz> = Hertz(0)..=Hertz(144_000_000);
}
