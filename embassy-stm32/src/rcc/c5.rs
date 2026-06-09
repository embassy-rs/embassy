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

pub(crate) unsafe fn init(config: Config) {
    RCC.cr().modify(|w| {
        w.set_hsison(config.hsi);
        w.set_hsidiv3on(config.hsi_div3);
    });

    if config.hsi {
        while !RCC.cr().read().hsisrdy() {}
    }
    if config.hsi_div3 {
        while !RCC.cr().read().hsidiv3rdy() {}
    }

    // Configure HSI
    let hsi = config.hsi.then_some(HSI_FREQ);
    let hsi_div3 = config.hsi_div3.then_some(Hertz(HSI_FREQ.0 / 3));

    RCC.cfgr().modify(|w| w.set_sw(Sysclk::Hsis));
    while RCC.cfgr().read().sws() != Sysclk::Hsis {}

    // TODO: Configure HSE

    // Configure sysclk
    let sys = match config.sys {
        Sysclk::Hsidiv3 => unwrap!(hsi_div3),
        Sysclk::Hsis => unwrap!(hsi),
        Sysclk::Hse => unimplemented!(),
        Sysclk::Psis => unimplemented!(),
    };

    let hclk = sys / config.ahb_pre;
    let hclk_max = Hertz(144_000_000);
    assert!(hclk <= hclk_max);

    let pclk_max = Hertz(144_000_000);
    let apb1 = hclk / config.apb1_pre;
    assert!(apb1 <= pclk_max);

    let apb2 = hclk / config.apb2_pre;
    assert!(apb2 <= pclk_max);

    let apb3 = hclk / config.apb3_pre;
    assert!(apb3 <= pclk_max);

    flash_setup(hclk);

    //let rtc = config.ls.init();

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
