use crate::pac::flash::vals::Latency;
pub use crate::pac::pwr::vals::Vos as VoltageRange;
pub use crate::pac::rcc::vals::{
    Hpre as AHBPrescaler, Hsidiv as HsiSysDiv, Pllm as PllPreDiv, Plln as PllMul, Pllp as PllPDiv, Pllq as PllQDiv,
    Pllr as PllRDiv, Pllsrc as PllSource, Ppre as APBPrescaler, Sw as Sysclk,
};
use crate::pac::{FLASH, PWR, RCC};
use crate::rcc::LSI_FREQ;
use crate::time::Hertz;

/// HSI speed
pub const HSI_FREQ: Hertz = Hertz(16_000_000);

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

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Hsi {
    /// Division factor for HSISYS clock. Default is 1.
    pub sys_div: HsiSysDiv,
}

/// PLL Configuration
///
/// Use this struct to configure the PLL source, input frequency, multiplication factor, and output
/// dividers. Be sure to keep check the datasheet for your specific part for the appropriate
/// frequency ranges for each of these settings.
#[derive(Clone, Copy)]
pub struct Pll {
    /// PLL Source clock selection.
    pub source: PllSource,

    /// PLL pre-divider
    pub prediv: PllPreDiv,

    /// PLL multiplication factor for VCO
    pub mul: PllMul,

    /// PLL division factor for P clock (ADC Clock)
    pub divp: Option<PllPDiv>,

    /// PLL division factor for Q clock (USB, I2S23, SAI1, FDCAN, QSPI)
    pub divq: Option<PllQDiv>,

    /// PLL division factor for R clock (SYSCLK)
    pub divr: Option<PllRDiv>,
}

/// Flash prefetch (`FLASH_ACR.PRFTEN`).
///
/// On the STM32G0B0/G0B1/G0C1 in dual-bank mode a code prefetch can fail on a branch or function call across
/// the flash banks ("Prefetch failure when branching across flash memory banks": ES0547 2.2.8, ES0548 2.2.10,
/// ES0549 2.2.10; not fixed in silicon revisions A, Z and Y; the only workaround is prefetch off).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum FlashPrefetch {
    /// Prefetch off.
    Off,
    /// Prefetch on. On a dual-bank device the caller vouches that no code branches across the flash banks.
    ///
    /// When code must sit in both banks with prefetch on, keep the cross-bank calls to a few known entry points
    /// and route each through a function in RAM that clears `PRFTEN`, makes the call and sets `PRFTEN` again.
    /// The errata list no workaround; a trampoline that keeps prefetch on during the crossing is not covered.
    On,
    /// Prefetch on, unless the device runs in dual-bank mode and the program's code (`__stext..__etext` of
    /// cortex-m-rt) reaches into bank 2; without the `rt` feature, off on a device in dual-bank mode.
    ///
    /// For applications that want the prefetch on a dual-bank device. Only the program's own code is checked, so
    /// not for a bootloader that jumps into an application in the other bank: that jump branches across the banks.
    IfCodeInOneBank,
    /// Prefetch on, unless the device runs in dual-bank mode (the default).
    ///
    /// Safe for any program, bootloaders included. An application on a dual-bank device gets the prefetch back
    /// with [`FlashPrefetch::IfCodeInOneBank`]; it then sets it from its own code when it initializes the clocks.
    IfSingleBank,
}

/// Clocks configuration
#[non_exhaustive]
#[derive(Clone, Copy)]
pub struct Config {
    /// HSI Configuration
    pub hsi: Option<Hsi>,

    /// HSE Configuration
    pub hse: Option<Hse>,

    /// System Clock Configuration
    pub sys: Sysclk,

    /// HSI48 Configuration
    #[cfg(crs)]
    pub hsi48: Option<super::Hsi48Config>,

    /// PLL Configuration
    pub pll: Option<Pll>,

    /// If PLL is requested as the main clock source in the `sys` field then the PLL configuration
    /// MUST turn on the PLLR output.
    pub ahb_pre: AHBPrescaler,
    pub apb1_pre: APBPrescaler,

    /// Low-Speed Clock Configuration
    pub ls: super::LsConfig,

    pub low_power_run: bool,

    pub voltage_range: VoltageRange,

    /// Per-peripheral kernel clock selection muxes
    pub mux: super::mux::ClockMux,

    /// Flash prefetch, see [`FlashPrefetch`].
    pub flash_prefetch: FlashPrefetch,
}

impl Config {
    pub const fn new() -> Self {
        Config {
            hsi: Some(Hsi {
                sys_div: HsiSysDiv::Div1,
            }),
            hse: None,
            sys: Sysclk::Hsi,
            #[cfg(crs)]
            hsi48: Some(crate::rcc::Hsi48Config::new()),
            pll: None,
            ahb_pre: AHBPrescaler::Div1,
            apb1_pre: APBPrescaler::Div1,
            low_power_run: false,
            ls: crate::rcc::LsConfig::new(),
            voltage_range: VoltageRange::Range1,
            mux: super::mux::ClockMux::default(),
            flash_prefetch: FlashPrefetch::IfSingleBank,
        }
    }
}

impl Default for Config {
    fn default() -> Config {
        Self::new()
    }
}

#[derive(Default)]
pub struct PllFreq {
    pub pll_p: Option<Hertz>,
    pub pll_q: Option<Hertz>,
    pub pll_r: Option<Hertz>,
}

pub(crate) unsafe fn init(config: Config) {
    // Turn on the HSI
    RCC.cr().modify(|w| {
        w.set_hsion(true);
        if let Some(hsi) = config.hsi {
            w.set_hsidiv(hsi.sys_div);
        }
    });
    while !RCC.cr().read().hsirdy() {}

    // Use the HSI clock as system clock during the actual clock setup
    RCC.cfgr().modify(|w| w.set_sw(Sysclk::Hsi));
    while RCC.cfgr().read().sws() != Sysclk::Hsi {}

    // Configure HSI
    let (hsi, hsisys) = match config.hsi {
        None => (None, None),
        Some(hsi) => (Some(HSI_FREQ), Some(HSI_FREQ / hsi.sys_div)),
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

    let pll = config
        .pll
        .map(|pll_config| {
            let src_freq = match pll_config.source {
                PllSource::Hsi => unwrap!(hsi),
                PllSource::Hse => unwrap!(hse),
                _ => unreachable!(),
            };

            // Disable PLL before configuration
            RCC.cr().modify(|w| w.set_pllon(false));
            while RCC.cr().read().pllrdy() {}

            let in_freq = src_freq / pll_config.prediv;
            rcc_assert!(max::PLL_IN.contains(&in_freq));
            let internal_freq = in_freq * pll_config.mul;
            rcc_assert!(max::PLL_VCO.contains(&internal_freq));

            RCC.pllcfgr().write(|w| {
                w.set_plln(pll_config.mul);
                w.set_pllm(pll_config.prediv);
                w.set_pllsrc(pll_config.source.into());
            });

            let pll_p_freq = pll_config.divp.map(|div_p| {
                RCC.pllcfgr().modify(|w| {
                    w.set_pllp(div_p);
                    w.set_pllpen(true);
                });
                let freq = internal_freq / div_p;
                rcc_assert!(max::PLL_P.contains(&freq));
                freq
            });

            let pll_q_freq = pll_config.divq.map(|div_q| {
                RCC.pllcfgr().modify(|w| {
                    w.set_pllq(div_q);
                    w.set_pllqen(true);
                });
                let freq = internal_freq / div_q;
                rcc_assert!(max::PLL_Q.contains(&freq));
                freq
            });

            let pll_r_freq = pll_config.divr.map(|div_r| {
                RCC.pllcfgr().modify(|w| {
                    w.set_pllr(div_r);
                    w.set_pllren(true);
                });
                let freq = internal_freq / div_r;
                rcc_assert!(max::PLL_R.contains(&freq));
                freq
            });

            // Enable the PLL
            RCC.cr().modify(|w| w.set_pllon(true));
            while !RCC.cr().read().pllrdy() {}

            PllFreq {
                pll_p: pll_p_freq,
                pll_q: pll_q_freq,
                pll_r: pll_r_freq,
            }
        })
        .unwrap_or_default();

    let rtc = config.ls.init();

    let sys = match config.sys {
        Sysclk::Hsi => unwrap!(hsisys),
        Sysclk::Hse => unwrap!(hse),
        Sysclk::Pll1R => unwrap!(pll.pll_r),
        Sysclk::Lsi => {
            assert!(config.ls.lsi);
            LSI_FREQ
        }
        Sysclk::Lse => unwrap!(config.ls.lse).frequency,
        _ => unreachable!(),
    };

    rcc_assert!(max::SYSCLK.contains(&sys));

    // Calculate the AHB frequency (HCLK), among other things so we can calculate the correct flash read latency.
    let hclk = sys / config.ahb_pre;
    rcc_assert!(max::HCLK.contains(&hclk));

    let (pclk1, pclk1_tim) = super::util::calc_pclk(hclk, config.apb1_pre);
    rcc_assert!(max::PCLK.contains(&pclk1));

    let latency = match (config.voltage_range, hclk.0) {
        (VoltageRange::Range1, ..=24_000_000) => Latency::Ws0,
        (VoltageRange::Range1, ..=48_000_000) => Latency::Ws1,
        (VoltageRange::Range1, _) => Latency::Ws2,
        (VoltageRange::Range2, ..=8_000_000) => Latency::Ws0,
        (VoltageRange::Range2, ..=16_000_000) => Latency::Ws1,
        (VoltageRange::Range2, _) => Latency::Ws2,
        _ => unreachable!(),
    };

    // Configure flash read access latency based on voltage scale and frequency (RM0444 3.3.4)
    FLASH.acr().modify(|w| {
        w.set_latency(latency);
    });

    // Spin until the effective flash latency is set.
    while FLASH.acr().read().latency() != latency {}

    // Enable prefetch after switching latency (RM0444 3.3.3): with wait states and prefetch off, every
    // sequential fetch stalls. ICEN is already on after reset.
    let prefetch = match config.flash_prefetch {
        FlashPrefetch::Off => false,
        FlashPrefetch::On => true,
        FlashPrefetch::IfCodeInOneBank => code_in_one_bank(),
        FlashPrefetch::IfSingleBank => {
            let dual_bank = dual_bank();
            if dual_bank {
                info!("flash prefetch off: dual-bank flash (ES0547/ES0548/ES0549), see FlashPrefetch::IfCodeInOneBank");
            }
            !dual_bank
        }
    };
    FLASH.acr().modify(|w| w.set_prften(prefetch));

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

    // Disable the HSI48, if not used
    #[cfg(crs)]
    if config.hsi48.is_none() {
        super::disable_hsi48();
    }

    if config.low_power_run {
        assert!(sys <= Hertz(2_000_000));
        PWR.cr1().modify(|w| w.set_lpr(true));
    }

    config.mux.init();

    set_clocks!(
        sys: Some(sys),
        hclk1: Some(hclk),
        pclk1: Some(pclk1),
        pclk1_tim: Some(pclk1_tim),
        pll1_p: pll.pll_p,
        pll1_q: pll.pll_q,
        pll1_r: pll.pll_r,
        hsi: hsi,
        hse: hse,
        #[cfg(crs)]
        hsi48: hsi48,
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
    pub(crate) const SYSCLK: RangeInclusive<Hertz> = Hertz(0)..=Hertz(64_000_000);
    pub(crate) const PCLK: RangeInclusive<Hertz> = Hertz(8)..=Hertz(64_000_000);
    pub(crate) const HCLK: RangeInclusive<Hertz> = Hertz(0)..=Hertz(64_000_000);
    pub(crate) const PLL_IN: RangeInclusive<Hertz> = Hertz(2_660_000)..=Hertz(16_000_000);
    pub(crate) const PLL_VCO: RangeInclusive<Hertz> = Hertz(96_000_000)..=Hertz(344_000_000);
    pub(crate) const PLL_P: RangeInclusive<Hertz> = Hertz(3_090_000)..=Hertz(122_000_000);
    pub(crate) const PLL_Q: RangeInclusive<Hertz> = Hertz(12_000_000)..=Hertz(128_000_000);
    pub(crate) const PLL_R: RangeInclusive<Hertz> = Hertz(12_000_000)..=Hertz(64_000_000);
}

/// Whether the flash runs in dual-bank mode on a device with the cross-bank prefetch erratum (ES0547 2.2.8,
/// ES0548/ES0549 2.2.10).
fn dual_bank() -> bool {
    // ES0547 2.2.8: every G0B0 has 512 KiB and always operates in dual-bank configuration ("DUAL_BANK option bit
    // is not used"), whatever the `single-bank` / `dual-bank` feature selects.
    #[cfg(stm32g0b0)]
    return true;
    // A 256 KiB G0B1/G0C1 follows its DUAL_BANK option bit, whatever the feature selects.
    #[cfg(all(any(stm32g0b1, stm32g0c1), bank_setup_configurable))]
    return crate::flash::dual_bank_mode();
    // The 512 KiB G0B1/G0C1 are always dual-bank, the 128 KiB ones single-bank only.
    #[cfg(all(any(stm32g0b1, stm32g0c1), not(bank_setup_configurable)))]
    return cfg!(flash_dual_bank);
    #[cfg(not(any(stm32g0b0, stm32g0b1, stm32g0c1)))]
    false
}

/// Whether no code can branch across the flash banks: the flash is in single-bank mode, or all of `.text` sits
/// in one bank.
fn code_in_one_bank() -> bool {
    if !dual_bank() {
        return true;
    }
    #[cfg(stm32g0b0)]
    return text_in_one_bank(crate::_generated::FLASH_BASE + crate::_generated::FLASH_SIZE / 2);
    #[cfg(all(any(stm32g0b1, stm32g0c1), flash_dual_bank))]
    return text_in_one_bank(crate::_generated::FLASH_BANK2_BASE as usize);
    // A 256 KiB G0B1/G0C1 built for single-bank, but the option bytes say dual-bank: prefetch off.
    #[cfg(all(any(stm32g0b1, stm32g0c1), not(flash_dual_bank), bank_setup_configurable))]
    {
        warn!("flash prefetch off: built for single-bank flash, but the option bytes select dual-bank");
        return false;
    }
    #[cfg(not(any(
        stm32g0b0,
        all(any(stm32g0b1, stm32g0c1), any(flash_dual_bank, bank_setup_configurable))
    )))]
    true
}

/// Whether the program's code (`__stext..__etext` of cortex-m-rt) sits on one side of `bank2`, the bank 2 base;
/// without the `rt` feature the code's place is unknown, so `false`.
#[cfg(any(stm32g0b0, all(any(stm32g0b1, stm32g0c1), flash_dual_bank)))]
fn text_in_one_bank(bank2: usize) -> bool {
    #[cfg(feature = "rt")]
    let one_bank = {
        unsafe extern "C" {
            static __stext: u8;
            static __etext: u8;
        }
        let (start, end) = (&raw const __stext as usize, &raw const __etext as usize);
        end <= bank2 || start >= bank2
    };
    #[cfg(not(feature = "rt"))]
    let one_bank = false;
    if !one_bank {
        warn!(
            "flash prefetch off: code spans both flash banks (bank 2 at {:#x}, ES0547/ES0548/ES0549)",
            bank2
        );
    }
    one_bank
}
