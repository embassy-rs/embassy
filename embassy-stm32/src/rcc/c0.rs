use crate::pac::flash::vals::Latency;
#[cfg(rcc_c0v2)]
pub use crate::pac::rcc::vals::Sysdiv as SysDiv;
pub use crate::pac::rcc::vals::{
    Hpre as AHBPrescaler, Hsidiv as HsiDiv, Hsikerdiv as HsiKerDiv, Ppre as APBPrescaler, Sw as Sysclk,
};
use crate::pac::{FLASH, RCC};
use crate::rcc::LSI_FREQ;
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
    /// Division factor (HSIDIV) applied to HSI to produce HSISYS. Default is 4.
    pub div: HsiDiv,
    /// Division factor for HSIKER clock. Default is 3.
    pub ker_div: HsiKerDiv,
}

/// Clocks configutation
#[non_exhaustive]
#[derive(Clone, Copy)]
pub struct Config {
    /// HSI Configuration
    pub hsi: Option<Hsi>,

    /// HSE Configuration
    pub hse: Option<Hse>,

    /// System Clock Configuration
    pub sys: Sysclk,

    /// Division factor (SYSDIV) applied to the output of the system clock mux to produce SYSCLK.
    #[cfg(rcc_c0v2)]
    pub sys_div: SysDiv,

    /// HSI48 Configuration
    #[cfg(crs)]
    pub hsi48: Option<super::Hsi48Config>,

    pub ahb_pre: AHBPrescaler,
    pub apb1_pre: APBPrescaler,

    /// Low-Speed Clock Configuration
    pub ls: super::LsConfig,

    /// Per-peripheral kernel clock selection muxes
    pub mux: super::mux::ClockMux,
}

impl Config {
    pub const fn new() -> Self {
        Config {
            hsi: Some(Hsi {
                div: HsiDiv::Div4,
                ker_div: HsiKerDiv::Div3,
            }),
            hse: None,
            sys: Sysclk::Hsisys,
            #[cfg(rcc_c0v2)]
            sys_div: SysDiv::Div1,
            #[cfg(crs)]
            hsi48: Some(crate::rcc::Hsi48Config::new()),
            ahb_pre: AHBPrescaler::Div1,
            apb1_pre: APBPrescaler::Div1,
            ls: crate::rcc::LsConfig::new(),
            mux: super::mux::ClockMux::default(),
        }
    }
}

impl Default for Config {
    fn default() -> Config {
        Self::new()
    }
}

pub(crate) unsafe fn init(config: Config) {
    // Configure the maximum flash read access latency up front, before anything that can raise the
    // core frequency. Nothing below knows what the incoming configuration is: after a bootloader
    // hands over without an intervening reset, HSIDIV may already be at /1, so even the switch to
    // HSISYS a few lines down can take the core to 48 MHz. The latency is relaxed to the value the
    // final HCLK actually requires once the clock tree has settled.
    FLASH.acr().modify(|w| w.set_latency(Latency::Ws1));
    while FLASH.acr().read().latency() != Latency::Ws1 {}

    // Turn on the HSI and use it, at whatever divider it is currently set to, as system clock
    // during the actual clock setup.
    RCC.cr().modify(|w| w.set_hsion(true));
    while !RCC.cr().read().hsirdy() {}

    RCC.cfgr().modify(|w| w.set_sw(Sysclk::Hsisys));
    while RCC.cfgr().read().sws() != Sysclk::Hsisys {}

    // Configure HSI
    let (hsi, hsisys, hsiker) = match config.hsi {
        None => (None, None, None),
        Some(hsi) => (Some(HSI_FREQ), Some(HSI_FREQ / hsi.div), Some(HSI_FREQ / hsi.ker_div)),
    };

    // Configure HSE
    let hse = match config.hse {
        None => {
            RCC.cr().modify(|w| w.set_hseon(false));
            None
        }
        Some(hse) => {
            match hse.mode {
                HseMode::Bypass => rcc_assert!(max::HSE_BYP.contains(&hse.freq)),
                HseMode::Oscillator => rcc_assert!(max::HSE_OSC.contains(&hse.freq)),
            }

            RCC.cr().modify(|w| w.set_hsebyp(hse.mode != HseMode::Oscillator));
            RCC.cr().modify(|w| w.set_hseon(true));
            while !RCC.cr().read().hserdy() {}
            Some(hse.freq)
        }
    };

    // Configure HSI48 if required
    #[cfg(crs)]
    let hsi48 = config.hsi48.map(super::init_hsi48);

    let rtc = config.ls.init();

    // Output of the system clock mux.
    let sys = match config.sys {
        Sysclk::Hsisys => unwrap!(hsisys),
        Sysclk::Hse => unwrap!(hse),
        Sysclk::Lsi => {
            assert!(config.ls.lsi);
            LSI_FREQ
        }
        Sysclk::Lse => unwrap!(config.ls.lse).frequency,
        _ => unreachable!(),
    };

    // SYSDIV divides the mux output to produce SYSCLK.
    #[cfg(rcc_c0v2)]
    let sys = sys / config.sys_div;

    rcc_assert!(max::SYSCLK.contains(&sys));

    // Calculate the AHB frequency (HCLK), among other things so we can calculate the correct flash read latency.
    let hclk = sys / config.ahb_pre;
    rcc_assert!(max::HCLK.contains(&hclk));

    let (pclk1, pclk1_tim) = super::util::calc_pclk(hclk, config.apb1_pre);
    rcc_assert!(max::PCLK.contains(&pclk1));

    let latency = match hclk.0 {
        ..=24_000_000 => Latency::Ws0,
        _ => Latency::Ws1,
    };

    // Configure the HSI dividers.
    if let Some(hsi) = config.hsi {
        RCC.cr().modify(|w| {
            w.set_hsidiv(hsi.div);
            w.set_hsikerdiv(hsi.ker_div);
        });
    }

    // Now that the flash read access latency is configured, set up SYSCLK. SYSDIV has no ready
    // flag; RM0490 documents reading a divider back to check its content, which also keeps the
    // write from still being in flight when the latency is relaxed below.
    #[cfg(rcc_c0v2)]
    {
        RCC.cr().modify(|w| w.set_sysdiv(config.sys_div));
        while RCC.cr().read().sysdiv() != config.sys_div {}
    }

    RCC.cfgr().modify(|w| {
        w.set_sw(config.sys);
        w.set_hpre(config.ahb_pre);
        w.set_ppre(config.apb1_pre);
    });
    while RCC.cfgr().read().sws() != config.sys {}

    // The clock tree has settled, so the flash read access latency can be relaxed to the value the
    // final HCLK actually requires. Spin until the effective flash latency is set.
    if latency != Latency::Ws1 {
        FLASH.acr().modify(|w| w.set_latency(latency));
        while FLASH.acr().read().latency() != latency {}
    }

    // Disable HSI if not used
    if config.hsi.is_none() {
        RCC.cr().modify(|w| w.set_hsion(false));
    }

    // Disable the HSI48, if not used
    #[cfg(crs)]
    if config.hsi48.is_none() {
        super::disable_hsi48();
    }

    config.mux.init();

    set_clocks!(
        sys: Some(sys),
        hclk1: Some(hclk),
        pclk1: Some(pclk1),
        pclk1_tim: Some(pclk1_tim),
        hsi: hsi,
        hsiker: hsiker,
        hse: hse,
        #[cfg(crs)]
        hsi48: hsi48,
        rtc: rtc,

        // TODO
        lsi: None,
        lse: None,
    );

    RCC.ccipr()
        .modify(|w| w.set_adc1sel(stm32_metapac::rcc::vals::Adcsel::Sys));
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
