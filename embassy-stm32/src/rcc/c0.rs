use crate::pac::flash::vals::Latency;
pub use crate::pac::rcc::vals::{
    Hpre as AHBPrescaler, Hsidiv as HsiSysDiv, Hsikerdiv as HsiKerDiv, Ppre as APBPrescaler, Sw as Sysclk,
};
use crate::pac::{FLASH, RCC};
use crate::time::Hertz;

/// HSI speed
pub const HSI_FREQ: Hertz = Hertz(48_000_000);

/// HSE Mode
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum HseMode {
    /// crystal/ceramic oscillator (HSEBYP=0)
    Oscillator,
    /// external analog clock (low swing) (HSEBYP=1)
    Bypass,
}

/// HSE Configuration
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Hse {
    /// HSE frequency.
    pub freq: Hertz,
    /// HSE mode.
    pub mode: HseMode,
}

/// HSI Configuration
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Hsi {
    /// Division factor for HSISYS clock. Default is 4.
    pub sys_div: HsiSysDiv,
    /// Division factor for HSIKER clock. Default is 3.
    pub ker_div: HsiKerDiv,
}

/// Clocks configutation
#[non_exhaustive]
pub struct Config {
    /// HSI Configuration
    pub hsi: Option<Hsi>,

    /// HSE Configuration
    pub hse: Option<Hse>,

    /// System Clock Configuration
    pub sys: Sysclk,

    pub ahb_pre: AHBPrescaler,
    pub apb1_pre: APBPrescaler,

    /// Low-Speed Clock Configuration
    pub ls: super::LsConfig,

    /// Per-peripheral kernel clock selection muxes
    pub mux: super::mux::ClockMux,
}

impl Default for Config {
    #[inline]
    fn default() -> Config {
        Config {
            hsi: Some(Hsi {
                sys_div: HsiSysDiv::DIV4,
                ker_div: HsiKerDiv::DIV3,
            }),
            hse: None,
            sys: Sysclk::HSISYS,
            ahb_pre: AHBPrescaler::DIV1,
            apb1_pre: APBPrescaler::DIV1,
            ls: Default::default(),
            mux: Default::default(),
        }
    }
}

pub(crate) unsafe fn init(config: Config) {
    // Turn on the HSI
    match config.hsi {
        None => RCC.cr().modify(|w| w.set_hsion(true)),
        Some(hsi) => RCC.cr().modify(|w| {
            w.set_hsidiv(hsi.sys_div);
            w.set_hsikerdiv(hsi.ker_div);
            w.set_hsion(true);
        }),
    }
    while !RCC.cr().read().hsirdy() {}

    // Use the HSI clock as system clock during the actual clock setup
    RCC.cfgr().modify(|w| w.set_sw(Sysclk::HSISYS));
    while RCC.cfgr().read().sws() != Sysclk::HSISYS {}

    // Configure HSI
    let (hsi, hsisys, hsiker) = match config.hsi {
        None => (None, None, None),
        Some(hsi) => (
            Some(HSI_FREQ),
            Some(HSI_FREQ / hsi.sys_div),
            Some(HSI_FREQ / hsi.ker_div),
        ),
    };

    // Configure HSE
    let hse = match config.hse {
        None => {
            RCC.cr().modify(|w| w.set_hseon(false));
            None
        }
        Some(hse) => {
            match hse.mode {
                HseMode::Bypass => assert!(max::HSE_BYP.contains(&hse.freq)),
                HseMode::Oscillator => assert!(max::HSE_OSC.contains(&hse.freq)),
            }

            RCC.cr().modify(|w| w.set_hsebyp(hse.mode != HseMode::Oscillator));
            RCC.cr().modify(|w| w.set_hseon(true));
            while !RCC.cr().read().hserdy() {}
            Some(hse.freq)
        }
    };

    let sys = match config.sys {
        Sysclk::HSISYS => unwrap!(hsisys),
        Sysclk::HSE => unwrap!(hse),
        _ => unreachable!(),
    };

    assert!(max::SYSCLK.contains(&sys));

    // Calculate the AHB frequency (HCLK), among other things so we can calculate the correct flash read latency.
    let hclk = sys / config.ahb_pre;
    assert!(max::HCLK.contains(&hclk));

    let (pclk1, pclk1_tim) = super::util::calc_pclk(hclk, config.apb1_pre);
    assert!(max::PCLK.contains(&pclk1));

    let latency = match hclk.0 {
        ..=24_000_000 => Latency::WS0,
        _ => Latency::WS1,
    };

    // Configure flash read access latency based on voltage scale and frequency
    FLASH.acr().modify(|w| {
        w.set_latency(latency);
    });

    // Spin until the effective flash latency is set.
    while FLASH.acr().read().latency() != latency {}

    // Now that boost mode and flash read access latency are configured, set up SYSCLK
    RCC.cfgr().modify(|w| {
        w.set_sw(config.sys);
        w.set_hpre(config.ahb_pre);
        w.set_ppre(config.apb1_pre);
    });
    while RCC.cfgr().read().sws() != config.sys {}

    // Disable HSI if not used
    if config.hsi.is_none() {
        RCC.cr().modify(|w| w.set_hsion(false));
    }

    let rtc = config.ls.init();

    config.mux.init();

    set_clocks!(
        sys: Some(sys),
        hclk1: Some(hclk),
        pclk1: Some(pclk1),
        pclk1_tim: Some(pclk1_tim),
        hsi: hsi,
        hsiker: hsiker,
        hse: hse,
        rtc: rtc,

        // TODO
        lsi: None,
        lse: None,
    );
}

mod max {
    use core::ops::RangeInclusive;

    use crate::time::Hertz;

    pub(crate) const HSE_OSC: RangeInclusive<Hertz> = Hertz(4_000_000)..=Hertz(48_000_000);
    pub(crate) const HSE_BYP: RangeInclusive<Hertz> = Hertz(0)..=Hertz(48_000_000);
    pub(crate) const SYSCLK: RangeInclusive<Hertz> = Hertz(0)..=Hertz(48_000_000);
    pub(crate) const PCLK: RangeInclusive<Hertz> = Hertz(8)..=Hertz(48_000_000);
    pub(crate) const HCLK: RangeInclusive<Hertz> = Hertz(0)..=Hertz(48_000_000);
}
