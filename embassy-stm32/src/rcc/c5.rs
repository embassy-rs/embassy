use stm32_metapac::FLASH;
use stm32_metapac::rcc::vals::{Adcdacpre, Hseext, Hsikdiv, Psiref, Psirefsrc};

use crate::pac::RCC;
pub use crate::pac::rcc::vals::{
    Hpre as AHBPrescaler, Ppre as APBPrescaler, Psifreq as PsiFrequency, Psikdiv as PsiKernelDivider, Sw as Sysclk,
};
use crate::rcc::{LSI_FREQ, LsConfig};
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

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct HsiKernel {
    pub div: Hsikdiv,
}

/// Reference clock for the PSI oscillator.
///
/// LSE references are not supported yet.
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum PsiSource {
    /// HSI divided by 18 (8 MHz). Requires `Config::hsi` to be enabled.
    Hsi,
    /// HSE. Requires `Config::hse` at 8, 16, 24, 25, 32, 48, or 50 MHz.
    /// 25 and 50 MHz references only support a 100 MHz PSI output.
    Hse,
}

/// Configuration of the PSI oscillator and its independent outputs.
#[derive(Clone, Copy)]
pub struct Psi {
    /// Reference clock source, enabled separately in `Config`.
    pub source: PsiSource,
    /// Oscillator frequency. 160 MHz is only usable through a divided output.
    pub frequency: PsiFrequency,
    /// Enable the undivided PSIS output for SYSCLK or peripherals (at most 144 MHz).
    pub sys: bool,
    /// Enable the PSI / 3 output. USB requires a 144 MHz PSI oscillator.
    pub div3: bool,
    /// Enable PSIK with this divider, or disable it with `None`.
    pub kernel_div: Option<PsiKernelDivider>,
}

/// Configuration of the core clocks
#[non_exhaustive]
#[derive(Clone, Copy)]
pub struct Config {
    /// Enable HSI full speed tap (144MHz)
    pub hsi: bool,

    /// Enable HSI Div 3 tap (48MHz)
    pub hsi_div3: bool,

    pub hsik: Option<HsiKernel>,

    pub hse: Option<Hse>,

    /// Low-speed clocks and RTC clock source.
    /// The RTC domain is not reset during initialization, as this would erase SRAM2.
    /// Changing a previously selected RTC clock source requires a power cycle.
    pub ls: LsConfig,

    /// PSI oscillator configuration. `None` disables all PSI outputs.
    pub psi: Option<Psi>,

    /// System Clock Configuration
    pub sys: Sysclk,

    pub ahb_pre: AHBPrescaler,
    pub apb1_pre: APBPrescaler,
    pub apb2_pre: APBPrescaler,

    pub apb3_pre: APBPrescaler,

    pub adcdac_pre: Adcdacpre,

    /// Per-peripheral kernel clock selection muxes
    pub mux: super::mux::ClockMux,
}

impl Config {
    pub const fn new() -> Self {
        Self {
            hsi: false,
            hsi_div3: true,
            hsik: None,
            hse: None,
            ls: LsConfig::new(),
            psi: None,
            sys: Sysclk::Hsidiv3,
            ahb_pre: AHBPrescaler::Div1,
            apb1_pre: APBPrescaler::Div1,
            apb2_pre: APBPrescaler::Div1,
            apb3_pre: APBPrescaler::Div1,
            adcdac_pre: Adcdacpre::Div4,
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
    // Ensure flash remains safe for every possible incoming clock configuration.
    const MAX_FLASH_LATENCY: u8 = 4;
    const MAX_FLASH_WRHIGHFREQ: u8 = 0b10;

    FLASH.acr().modify(|w| {
        w.set_wrhighfreq(MAX_FLASH_WRHIGHFREQ);
        w.set_latency(MAX_FLASH_LATENCY);
    });

    while FLASH.acr().read().latency() != MAX_FLASH_LATENCY || FLASH.acr().read().wrhighfreq() != MAX_FLASH_WRHIGHFREQ {
    }

    // Enable the safe fallback clock without disabling the current clock.
    RCC.cr().modify(|w| w.set_hsidiv3on(true));
    while !RCC.cr().read().hsidiv3rdy() {}

    // Switch to the safe fallback before modifying other clock sources.
    RCC.cfgr().modify(|w| w.set_sw(Sysclk::Hsidiv3));
    while RCC.cfgr().read().sws() != Sysclk::Hsidiv3 {}

    // Stop every PSI output before changing its reference source or frequency.
    // This also releases HSE if a bootloader used it as the PSI reference.
    RCC.cr().modify(|w| {
        w.set_psion(false);
        w.set_psidiv3on(false);
        w.set_psikon(false);
        w.set_psikeron(false);
    });
    while {
        let cr = RCC.cr().read();
        cr.psirdy() || cr.psidiv3rdy() || cr.psikrdy()
    } {}

    // Configure HSI
    let hsi = config.hsi.then_some(HSI_FREQ);
    let hsi_div3 = config.hsi_div3.then_some(Hertz(HSI_FREQ.0 / 3));

    if config.hsi {
        RCC.cr().modify(|w| w.set_hsion(true));
        while !RCC.cr().read().hsirdy() {}
    }
    if config.hsi_div3 {
        while !RCC.cr().read().hsidiv3rdy() {}
    }

    // Configure HSIK
    let hsik = match config.hsik {
        Some(hsik) => {
            RCC.cr2().modify(|w| w.set_hsikdiv(hsik.div));
            RCC.cr().modify(|w| w.set_hsikon(true));
            while !RCC.cr().read().hsikrdy() {}
            Some(HSI_FREQ / hsik.div)
        }
        None => None,
    };

    // Configure HSE
    let hse = match config.hse {
        None => {
            RCC.cr().modify(|w| w.set_hseon(false));
            None
        }
        Some(hse) => {
            let valid_range = match hse.mode {
                HseMode::Oscillator => &max::HSE_OSC,
                HseMode::Bypass => &max::HSE_BYP_ANALOG,
                HseMode::BypassDigital => &max::HSE_BYP_DIGITAL,
            };

            assert!(
                valid_range.contains(&hse.freq),
                "HSE frequency is outside the supported range"
            );

            RCC.cr().modify(|w| w.set_hseon(false));
            while RCC.cr().read().hserdy() {}

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

    let (psi, psidiv3, psik) = match config.psi {
        Some(psi) => {
            let (source, reference) = match psi.source {
                PsiSource::Hsi => {
                    assert!(hsi.is_some(), "PSI HSI reference requires config.rcc.hsi");
                    (Psirefsrc::Hsi, Psiref::F8mHz)
                }
                PsiSource::Hse => (
                    Psirefsrc::Hse,
                    psi_hse_reference(unwrap!(hse, "PSI HSE reference requires config.rcc.hse"), psi.frequency),
                ),
            };
            let frequency = psi_frequency(psi.frequency);
            let sys = psi.sys.then_some(frequency);
            if let Some(sys) = sys {
                assert!(max::SYSCLK.contains(&sys), "PSIS must not exceed 144 MHz");
            }
            let div3 = psi.div3.then_some(frequency / 3u32);
            let kernel = psi.kernel_div.map(|div| frequency / div);
            if let Some(kernel) = kernel {
                assert!(max::HCLK.contains(&kernel), "PSIK must not exceed 144 MHz");
            }

            RCC.cr2().modify(|w| {
                w.set_psirefsrc(source);
                w.set_psiref(reference);
                w.set_psifreq(psi.frequency);
                if let Some(div) = psi.kernel_div {
                    w.set_psikdiv(div);
                }
            });
            RCC.cr().modify(|w| {
                w.set_psion(psi.sys);
                w.set_psidiv3on(psi.div3);
                w.set_psikon(psi.kernel_div.is_some());
            });
            if psi.sys {
                while !RCC.cr().read().psirdy() {}
            }
            if psi.div3 {
                while !RCC.cr().read().psidiv3rdy() {}
            }
            if psi.kernel_div.is_some() {
                while !RCC.cr().read().psikrdy() {}
            }
            (sys, div3, kernel)
        }
        None => (None, None, None),
    };

    // Configure sysclk
    let sys = match config.sys {
        Sysclk::Hsidiv3 => unwrap!(hsi_div3),
        Sysclk::Hsi => unwrap!(hsi),
        Sysclk::Hse => unwrap!(hse),
        Sysclk::Psi => unwrap!(psi, "PSI system clock requires config.rcc.psi.sys"),
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

    let adc_dac = match config.mux.adcdacsel {
        stm32_metapac::rcc::vals::Adcdacsel::Hclk1 => hclk,
        stm32_metapac::rcc::vals::Adcdacsel::Psi => unwrap!(psi, "ADC/DAC requires PSIS"),
        stm32_metapac::rcc::vals::Adcdacsel::Psik => unwrap!(psik, "ADC/DAC requires PSIK"),
        stm32_metapac::rcc::vals::Adcdacsel::Hsik => unwrap!(hsik, "ADC/DAC requires HSIK"),
    };
    let adc_dac = adc_dac / config.adcdac_pre;
    assert!(max::ADC_DAC.contains(&adc_dac));

    let rtc = config.ls.init();

    RCC.cfgr2().modify(|w| w.set_hpre(config.ahb_pre));
    while RCC.cfgr2().read().hpre() != config.ahb_pre {}

    RCC.cfgr2().modify(|w| {
        w.set_ppre1(config.apb1_pre);
        w.set_ppre2(config.apb2_pre);
        w.set_ppre3(config.apb3_pre);
    });

    RCC.cfgr().modify(|w| w.set_sw(config.sys));
    while RCC.cfgr().read().sws() != config.sys {}

    // The final HCLK is now active, so latency can be relaxed
    flash_setup(hclk);

    RCC.cr().modify(|w| {
        w.set_hsion(config.hsi);
        w.set_hsidiv3on(config.hsi_div3);
    });

    if !config.hsi {
        while RCC.cr().read().hsirdy() {}
    }

    if !config.hsi_div3 {
        while RCC.cr().read().hsidiv3rdy() {}
    }

    RCC.ccipr2().modify(|w| {
        w.set_adcdacpre(config.adcdac_pre);
    });
    config.mux.init();

    set_clocks!(
        sys: Some(sys),
        hclk1: Some(hclk),
        hclk2: Some(hclk),
        hclk4: Some(hclk),
        pclk1: Some(apb1),
        pclk1_tim: Some(hclk),
        pclk2: Some(apb2),
        pclk2_tim: Some(hclk),

        pclk3: Some(apb3),

        hsi: hsi,
        hsidiv3: hsi_div3,
        hsik: hsik,
        hse: hse,
        psi: psi,
        psidiv3: psidiv3,
        psik: psik,

        lsi: config.ls.lsi.then_some(LSI_FREQ),
        lse: config.ls.lse.map(|lse| lse.frequency),
        rtc: rtc,

        audioclk: None,
    );
}

fn psi_frequency(frequency: PsiFrequency) -> Hertz {
    match frequency {
        PsiFrequency::F100mHz => Hertz(100_000_000),
        PsiFrequency::F144mHz => Hertz(144_000_000),
        PsiFrequency::F160mHz => Hertz(160_000_000),
        _ => panic!("invalid PSI frequency"),
    }
}

fn psi_hse_reference(hse: Hertz, frequency: PsiFrequency) -> Psiref {
    // RM0522, table 66: 25/50 MHz references only support 100 MHz PSI.
    match hse.0 {
        8_000_000 => Psiref::F8mHz,
        16_000_000 => Psiref::F16mHz,
        24_000_000 => Psiref::F24mHz,
        25_000_000 if frequency == PsiFrequency::F100mHz => Psiref::F25mHz,
        32_000_000 => Psiref::F32mHz,
        48_000_000 => Psiref::F48mHz,
        50_000_000 if frequency == PsiFrequency::F100mHz => Psiref::F50mHz,
        _ => panic!("unsupported HSE frequency for the selected PSI frequency"),
    }
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

    FLASH.acr().modify(|w| {
        w.set_wrhighfreq(wrhighfreq);
        w.set_latency(latency);
    });
    while FLASH.acr().read().latency() != latency {}
}

mod max {
    use core::ops::RangeInclusive;

    use crate::time::Hertz;

    pub(crate) const HSE_OSC: RangeInclusive<Hertz> = Hertz(4_000_000)..=Hertz(50_000_000);
    pub(crate) const HSE_BYP_ANALOG: RangeInclusive<Hertz> = Hertz(4_000_000)..=Hertz(50_000_000);
    pub(crate) const HSE_BYP_DIGITAL: RangeInclusive<Hertz> = Hertz(1)..=Hertz(50_000_000);
    pub(crate) const SYSCLK: RangeInclusive<Hertz> = Hertz(0)..=Hertz(144_000_000);
    pub(crate) const PCLK: RangeInclusive<Hertz> = Hertz(0)..=Hertz(144_000_000);
    pub(crate) const HCLK: RangeInclusive<Hertz> = Hertz(0)..=Hertz(144_000_000);
    pub(crate) const ADC_DAC: RangeInclusive<Hertz> = Hertz(8_000_000)..=Hertz(36_000_000);
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;

    #[test]
    fn psi_output_frequencies() {
        assert_eq!(psi_frequency(PsiFrequency::F100mHz), Hertz(100_000_000));
        assert_eq!(psi_frequency(PsiFrequency::F144mHz), Hertz(144_000_000));
        assert_eq!(psi_frequency(PsiFrequency::F160mHz), Hertz(160_000_000));
    }

    #[test]
    fn psi_hse_reference_combinations() {
        for frequency in [PsiFrequency::F100mHz, PsiFrequency::F144mHz, PsiFrequency::F160mHz] {
            for (hz, reference) in [
                (8_000_000, Psiref::F8mHz),
                (16_000_000, Psiref::F16mHz),
                (24_000_000, Psiref::F24mHz),
                (32_000_000, Psiref::F32mHz),
                (48_000_000, Psiref::F48mHz),
            ] {
                assert!(psi_hse_reference(Hertz(hz), frequency) == reference);
            }
        }
        assert!(psi_hse_reference(Hertz(25_000_000), PsiFrequency::F100mHz) == Psiref::F25mHz);
        assert!(psi_hse_reference(Hertz(50_000_000), PsiFrequency::F100mHz) == Psiref::F50mHz);
    }

    #[test]
    fn psi_rejects_invalid_hse_references() {
        for frequency in [PsiFrequency::F144mHz, PsiFrequency::F160mHz] {
            for hz in [25_000_000, 50_000_000] {
                assert!(std::panic::catch_unwind(|| psi_hse_reference(Hertz(hz), frequency)).is_err());
            }
        }
        for hz in [0, 4_000_000, 12_000_000, 47_999_999] {
            assert!(std::panic::catch_unwind(|| psi_hse_reference(Hertz(hz), PsiFrequency::F100mHz)).is_err());
        }
    }
}
