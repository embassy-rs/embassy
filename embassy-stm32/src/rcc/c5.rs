use crate::pac::RCC;
pub use crate::pac::rcc::vals::{Hpre as AHBPrescaler, Ppre as APBPrescaler, Sw as Sysclk};
use crate::time::Hertz;

/// HSI speed
pub const HSI_FREQ: Hertz = Hertz(144_000_000);

pub enum HsiMode {
    /// Full 144MHz speed
    FullSpeed,

    /// HSI divided by 3 144/3=48MHz
    Div3,
}

/// Configuration of the core clocks
#[non_exhaustive]
#[derive(Clone, Copy)]
pub struct Config {
    ///
    pub hsi: bool,

    pub hsi_div3: bool,

    pub sys: Sysclk,

    pub ahb_pre: AHBPrescaler,

    pub apb1_pre: APBPrescaler,
    pub apb2_pre: APBPrescaler,
    pub apb3_pre: APBPrescaler,
}

impl Config {
    pub const fn new() -> Self {
        Self {
            hsi: false,
            hsi_div3: true,
            sys: Sysclk::Hsidiv3,
            ahb_pre: AHBPrescaler::Div1,
            apb1_pre: APBPrescaler::Div1,
            apb2_pre: APBPrescaler::Div1,
            apb3_pre: APBPrescaler::Div1,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

pub(crate) unsafe fn init(config: Config) {
    RCC.cr().modify(|w| {
        w.set_hsion(config.hsi);
        w.set_hsidiv3on(config.hsi_div3);
    });

    if config.hsi {
        while !RCC.cr().read().hsirdy() {}
    }
    if config.hsi_div3 {
        while !RCC.cr().read().hsidiv3rdy() {}
    }

    // Configure HSI
    let hsi = config.hsi.then_some(HSI_FREQ);
    let hsi_div3 = config.hsi_div3.then_some(Hertz(HSI_FREQ.0 / 3));

    RCC.cfgr().modify(|w| w.set_sw(Sysclk::Hsi));
    while RCC.cfgr().read().sws() != Sysclk::Hsi {}

    // TODO: Configure HSE

    // Configure sysclk
    let sys = match config.sys {
        Sysclk::Hsidiv3 => unwrap!(hsi_div3),
        Sysclk::Hsi => unwrap!(hsi),
        Sysclk::Hse => unimplemented!(),
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

// TODO: Do this once the FLASH peripheral gains its registers in the pac
//   The reset values should be safe for all speeds
fn flash_setup(_hclk: Hertz) {
    // let (latency, wrhighfreq) = match hclk.0 {
    //     ..=34_000_000 => (0, 0b00),
    //     34_000_000..=68_000_000 => (1, 0b00),
    //     68_000_000..=102_000_000 => (2, 0b01),
    //     102_000_000..=136_000_000 => (3, 0b01),
    //     136_000_000..=144_000_000 => (4, 0b10),
    // };

    // debug!("flash: latency={} wrhighfreq={}", latency, wrhighfreq);

    //FLASH.acr().write(|w| {
    //    w.set_wrhighfreq(wrhighfreq);
    //    w.set_latency(latency);
    //});
    //while FLASH.acr().read().latency() != latency {}
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
