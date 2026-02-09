use stm32_metapac::pwr::vals::{
    Vddio2rdy, Vddio2sv, Vddio2vrsel, Vddio3rdy, Vddio3sv, Vddio3vrsel, Vddio4sv, Vddio5sv,
};
use stm32_metapac::rcc::vals::{Cpusw, Cpusws, Hseext, Hsitrim, Msifreqsel, Pllmodssdis, Syssw, Syssws, Timpre};
pub use stm32_metapac::rcc::vals::{
    Hpre as AhbPrescaler, Hsidiv as HsiPrescaler, Hsitrim as HsiCalibration, Icint, Icsel, Plldivm, Pllpdiv, Pllsel,
    Ppre as ApbPrescaler, Xspisel as XspiClkSrc,
};
use stm32_metapac::syscfg::vals::{Vddio2cccrEn, Vddio3cccrEn, Vddio4cccrEn};

use crate::pac::{PWR, RCC, RISAF3, SYSCFG};
use crate::time::Hertz;

pub const HSI_FREQ: Hertz = Hertz(64_000_000);
pub const LSE_FREQ: Hertz = Hertz(32_768);

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum HseMode {
    /// crystal/ceramic oscillator
    Oscillator,
    /// oscillator bypassed with external clock (analog)
    Bypass,
    /// oscillator bypassed with external digital clock
    BypassDigital,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Hse {
    /// HSE frequency.
    pub freq: Hertz,
    /// HSE oscillator mode.
    pub mode: HseMode,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Hsi {
    pub pre: HsiPrescaler,
    pub trim: Hsitrim,
}

#[derive(Clone, Copy, PartialEq)]
pub enum SupplyConfig {
    Smps,
    External,
}

#[derive(Clone, Copy, PartialEq)]
pub enum CpuClk {
    Hsi,
    Msi,
    Hse,
    Ic1,
}

impl CpuClk {
    const fn to_bits(self) -> u8 {
        match self {
            Self::Hsi => 0x0,
            Self::Msi => 0x1,
            Self::Hse => 0x2,
            Self::Ic1 => 0x3,
        }
    }
}

/// Configuration for an internal clock (IC) divider.
///
/// Used for configuring XSPI kernel clocks (IC3 for XSPI1, IC4 for XSPI2).
#[derive(Clone, Copy, PartialEq)]
pub struct IcConfig {
    /// Clock source selection (PLL1, PLL2, etc.)
    pub source: Icsel,
    /// Divider value (1-256)
    pub divider: Icint,
}

#[derive(Clone, Copy, PartialEq)]
pub enum SysClk {
    Hsi,
    Msi,
    Hse,
    Ic2,
}

impl SysClk {
    const fn to_bits(self) -> u8 {
        match self {
            Self::Hsi => 0x0,
            Self::Msi => 0x1,
            Self::Hse => 0x2,
            Self::Ic2 => 0x3,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct Msi {
    pub freq: Msifreqsel,
    pub trim: u8,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Pll {
    Oscillator {
        source: Pllsel,
        divm: Plldivm,
        fractional: u32,
        divn: u16,
        divp1: Pllpdiv,
        divp2: Pllpdiv,
    },
    Bypass {
        source: Pllsel,
    },
}

/// Configuration of the core clocks
#[non_exhaustive]
#[derive(Clone, Copy)]
pub struct Config {
    pub hsi: Option<Hsi>,
    pub hse: Option<Hse>,
    pub msi: Option<Msi>,
    pub lsi: bool,
    pub lse: bool,

    pub cpu: CpuClk,
    pub sys: SysClk,

    pub pll1: Option<Pll>,
    pub pll2: Option<Pll>,
    pub pll3: Option<Pll>,
    pub pll4: Option<Pll>,

    pub ic1: Option<IcConfig>,
    pub ic2: Option<IcConfig>,
    pub ic3: Option<IcConfig>,
    pub ic4: Option<IcConfig>,
    pub ic5: Option<IcConfig>,
    pub ic6: Option<IcConfig>,
    pub ic7: Option<IcConfig>,
    pub ic8: Option<IcConfig>,
    pub ic9: Option<IcConfig>,
    pub ic10: Option<IcConfig>,
    pub ic11: Option<IcConfig>,
    pub ic12: Option<IcConfig>,
    pub ic13: Option<IcConfig>,
    pub ic14: Option<IcConfig>,
    pub ic15: Option<IcConfig>,
    pub ic16: Option<IcConfig>,
    pub ic17: Option<IcConfig>,
    pub ic18: Option<IcConfig>,
    pub ic19: Option<IcConfig>,
    pub ic20: Option<IcConfig>,

    pub ahb: AhbPrescaler,
    pub apb1: ApbPrescaler,
    pub apb2: ApbPrescaler,
    pub apb4: ApbPrescaler,
    pub apb5: ApbPrescaler,

    pub supply_config: SupplyConfig,

    /// VddIO2 voltage range (Ports O/P, XSPI1)
    /// true = 1.8V, false = 3.3V (default)
    pub vddio2_1v8: bool,
    /// VddIO3 voltage range (Port N, XSPI2)
    /// true = 1.8V, false = 3.3V (default)
    pub vddio3_1v8: bool,

    /// Per-peripheral kernel clock selection muxes
    pub mux: super::mux::ClockMux,
}

impl Config {
    pub const fn new() -> Self {
        Self {
            hsi: Some(Hsi {
                pre: HsiPrescaler::DIV1,
                trim: HsiCalibration::from_bits(32),
            }),
            hse: None,
            msi: None,
            lsi: true,
            lse: false,

            cpu: CpuClk::Hsi,
            sys: SysClk::Hsi,

            pll1: Some(Pll::Bypass { source: Pllsel::HSI }),
            pll2: Some(Pll::Bypass { source: Pllsel::HSI }),
            pll3: Some(Pll::Bypass { source: Pllsel::HSI }),
            pll4: Some(Pll::Bypass { source: Pllsel::HSI }),

            ic1: None,
            ic2: None,
            ic3: None,
            ic4: None,
            ic5: None,
            ic6: None,
            ic7: None,
            ic8: None,
            ic9: None,
            ic10: None,
            ic11: None,
            ic12: None,
            ic13: None,
            ic14: None,
            ic15: None,
            ic16: None,
            ic17: None,
            ic18: None,
            ic19: None,
            ic20: None,

            ahb: AhbPrescaler::DIV2,
            apb1: ApbPrescaler::DIV1,
            apb2: ApbPrescaler::DIV1,
            apb4: ApbPrescaler::DIV1,
            apb5: ApbPrescaler::DIV1,

            supply_config: SupplyConfig::Smps,

            vddio2_1v8: false, // Default to 3.3V
            vddio3_1v8: false, // Default to 3.3V

            mux: super::mux::ClockMux::default(),
        }
    }
}

#[allow(dead_code)]
struct ClocksOutput {
    cpuclk: Hertz,
    sysclk: Hertz,
    pclk_tim: Hertz,
    ahb: Hertz,
    apb1: Hertz,
    apb2: Hertz,
    apb4: Hertz,
    apb5: Hertz,
}

struct ClocksInput {
    hsi: Option<Hertz>,
    msi: Option<Hertz>,
    hse: Option<Hertz>,
    ic1: Option<Hertz>,
    ic2: Option<Hertz>,
    ic3: Option<Hertz>,
    ic4: Option<Hertz>,
    ic5: Option<Hertz>,
    ic6: Option<Hertz>,
    ic7: Option<Hertz>,
    ic8: Option<Hertz>,
    ic9: Option<Hertz>,
    ic10: Option<Hertz>,
    ic11: Option<Hertz>,
    ic12: Option<Hertz>,
    ic13: Option<Hertz>,
    ic14: Option<Hertz>,
    ic15: Option<Hertz>,
    ic16: Option<Hertz>,
    ic17: Option<Hertz>,
    ic18: Option<Hertz>,
    ic19: Option<Hertz>,
    ic20: Option<Hertz>,
}

fn init_clocks(config: Config, input: &ClocksInput) -> ClocksOutput {
    // IC configuration
    for (index, ic) in [
        config.ic1,
        config.ic2,
        config.ic3,
        config.ic4,
        config.ic5,
        config.ic6,
        config.ic7,
        config.ic8,
        config.ic9,
        config.ic10,
        config.ic11,
        config.ic12,
        config.ic13,
        config.ic14,
        config.ic15,
        config.ic16,
        config.ic17,
        config.ic18,
        config.ic19,
        config.ic20,
    ]
    .iter()
    .enumerate()
    {
        // Skip disabled ICs
        let Some(ic) = ic else { continue };

        let ic_source = ic.source.to_bits();
        if !pll_source_ready(ic_source) {
            panic!(
                "IC{} source was set to PLL{}, but it is not currently enabled",
                index + 1,
                ic_source
            )
        }

        RCC.iccfgr(index).write(|w| {
            w.set_icsel(ic.source);
            w.set_icint(ic.divider);
        });
        RCC.divensr().modify(|w| w.0 = 1 << index);
    }

    // handle increasing dividers
    debug!("configuring increasing pclk dividers");
    RCC.cfgr2().modify(|w| {
        if config.apb1 > w.ppre1() {
            debug!("  - APB1");
            w.set_ppre1(config.apb1);
        }
        if config.apb2 > w.ppre2() {
            debug!("  - APB2");
            w.set_ppre2(config.apb2);
        }
        if config.apb4 > w.ppre4() {
            debug!("  - APB4");
            w.set_ppre4(config.apb4);
        }
        if config.apb5 > w.ppre5() {
            debug!("  - APB5");
            w.set_ppre5(config.apb5);
        }
        if config.ahb > w.hpre() {
            debug!("  - AHB");
            w.set_hpre(config.ahb);
        }
    });
    // cpuclk
    debug!("configuring cpuclk");
    match config.cpu {
        CpuClk::Hsi if !RCC.sr().read().hsirdy() => panic!("HSI is not ready to be selected as CPU clock source"),
        CpuClk::Msi if !RCC.sr().read().msirdy() => panic!("MSI is not ready to be selected as CPU clock source"),
        CpuClk::Hse if !RCC.sr().read().hserdy() => panic!("HSE is not ready to be selected as CPU clock source"),
        CpuClk::Ic1 if !ic_enabled(1) => panic!("IC1 is not ready to be selected as CPU clock source"),
        _ => {}
    }
    // set source
    let cpusw = Cpusw::from_bits(config.cpu.to_bits());
    RCC.cfgr().modify(|w| w.set_cpusw(cpusw));
    // wait for changes to take effect
    while RCC.cfgr().read().cpusws() != Cpusws::from_bits(config.cpu.to_bits()) {}

    // sysclk
    debug!("configuring sysclk");
    match config.sys {
        SysClk::Hsi if !RCC.sr().read().hsirdy() => panic!("HSI is not ready to be selected as system clock source"),
        SysClk::Msi if !RCC.sr().read().msirdy() => panic!("MSI is not ready to be selected as system clock source"),
        SysClk::Hse if !RCC.sr().read().hserdy() => panic!("HSE is not ready to be selected as system clock source"),
        SysClk::Ic2 if !ic_enabled(2) || !ic_enabled(6) || !ic_enabled(11) => panic!(
            "IC2 is not ready to be selected as system clock source (make sure that IC6 and IC11 were configured as well)"
        ),
        _ => {}
    }
    // switch the system bus clock
    let syssw = Syssw::from_bits(config.sys.to_bits());
    RCC.cfgr().modify(|w| w.set_syssw(syssw));
    // wait for changes to be applied
    while RCC.cfgr().read().syssws() != Syssws::from_bits(config.sys.to_bits()) {}

    // decreasing dividers
    debug!("configuring decreasing pclk dividers");
    RCC.cfgr2().modify(|w| {
        if config.ahb < w.hpre() {
            debug!("  - AHB");
            w.set_hpre(config.ahb);
        }
        if config.apb1 < w.ppre1() {
            debug!("  - APB1");
            w.set_ppre1(config.apb1);
        }
        if config.apb2 < w.ppre2() {
            debug!("  - APB2");
            w.set_ppre2(config.apb2);
        }
        if config.apb4 < w.ppre4() {
            debug!("  - APB4");
            w.set_ppre4(config.apb4);
        }
        if config.apb5 < w.ppre5() {
            debug!("  - APB5");
            w.set_ppre5(config.apb5);
        }
    });

    let cpuclk = match config.cpu {
        CpuClk::Hsi => unwrap!(input.hsi),
        CpuClk::Msi => unwrap!(input.msi),
        CpuClk::Hse => unwrap!(input.hse),
        CpuClk::Ic1 => unwrap!(input.ic1),
    };

    let sysclk = match config.sys {
        SysClk::Hsi => unwrap!(input.hsi),
        SysClk::Msi => unwrap!(input.msi),
        SysClk::Hse => unwrap!(input.hse),
        SysClk::Ic2 => unwrap!(input.ic2),
    };

    let timpre: u32 = match RCC.cfgr2().read().timpre() {
        Timpre::DIV1 => 1,
        Timpre::DIV2 => 2,
        Timpre::DIV4 => 4,
        Timpre::_RESERVED_3 => 8,
    };

    let hpre = periph_prescaler_to_value(config.ahb.to_bits());
    let ppre1 = periph_prescaler_to_value(config.apb1.to_bits());
    let ppre2 = periph_prescaler_to_value(config.apb2.to_bits());
    let ppre4 = periph_prescaler_to_value(config.apb4.to_bits());
    let ppre5 = periph_prescaler_to_value(config.apb5.to_bits());

    // enable all peripherals in sleep mode
    enable_low_power_peripherals();

    // enable interrupts
    unsafe {
        core::arch::asm!("cpsie i");
    }

    ClocksOutput {
        cpuclk,
        sysclk,
        pclk_tim: sysclk / timpre,
        ahb: Hertz(sysclk.0 / hpre as u32),
        apb1: sysclk / hpre / ppre1,
        apb2: sysclk / hpre / ppre2,
        apb4: sysclk / hpre / ppre4,
        apb5: sysclk / hpre / ppre5,
    }
}

fn enable_low_power_peripherals() {
    // AHB1-5
    RCC.ahb1lpenr().modify(|w| {
        w.set_adc12lpen(true);
        w.set_gpdma1lpen(true);
    });
    RCC.ahb2lpenr().modify(|w| {
        w.set_adf1lpen(true);
        w.set_mdf1lpen(true);
        w.set_ramcfglpen(true);
    });
    RCC.ahb3lpenr().modify(|w| {
        w.set_risaflpen(true);
        w.set_iaclpen(true);
        w.set_rifsclpen(true);
        w.set_pkalpen(true);
        w.set_saeslpen(true);
        w.set_cryplpen(true);
        w.set_hashlpen(true);
        w.set_rnglpen(true);
    });
    RCC.ahb4lpenr().modify(|w| {
        w.set_crclpen(true);
        w.set_pwrlpen(true);
        w.set_gpioqlpen(true);
        w.set_gpioplpen(true);
        w.set_gpioolpen(true);
        w.set_gpionlpen(true);
        w.set_gpiohlpen(true);
        w.set_gpioglpen(true);
        w.set_gpioflpen(true);
        w.set_gpioelpen(true);
        w.set_gpiodlpen(true);
        w.set_gpioclpen(true);
        w.set_gpioblpen(true);
        w.set_gpioalpen(true);
    });
    RCC.ahb5lpenr().modify(|w| {
        w.set_npulpen(true);
        w.set_npucachelpen(true);
        w.set_otg2lpen(true);
        w.set_otgphy2lpen(true);
        w.set_otgphy1lpen(true);
        w.set_otg1lpen(true);
        w.set_eth1lpen(true);
        w.set_eth1rxlpen(true);
        w.set_eth1txlpen(true);
        w.set_eth1maclpen(true);
        w.set_gpulpen(true);
        w.set_gfxmmulpen(true);
        w.set_mce4lpen(true);
        w.set_xspi3lpen(true);
        w.set_mce3lpen(true);
        w.set_mce2lpen(true);
        w.set_mce1lpen(true);
        w.set_xspimlpen(true);
        w.set_xspi2lpen(true);
        w.set_sdmmc1lpen(true);
        w.set_sdmmc2lpen(true);
        w.set_pssilpen(true);
        w.set_xspi1lpen(true);
        w.set_fmclpen(true);
        w.set_jpeglpen(true);
        w.set_dma2dlpen(true);
        w.set_hpdma1lpen(true);
    });

    // APB1-5
    RCC.apb1llpenr().modify(|w| {
        w.set_uart8lpen(true);
        w.set_uart7lpen(true);
        w.set_i3c2lpen(true);
        w.set_i3c1lpen(true);
        w.set_i2c3lpen(true);
        w.set_i2c2lpen(true);
        w.set_i2c1lpen(true);
        w.set_uart5lpen(true);
        w.set_uart4lpen(true);
        w.set_usart3lpen(true);
        w.set_usart2lpen(true);
        w.set_spdifrx1lpen(true);
        w.set_spi3lpen(true);
        w.set_spi2lpen(true);
        w.set_tim11lpen(true);
        w.set_tim10lpen(true);
        w.set_wwdglpen(true);
        w.set_lptim1lpen(true);
        w.set_tim14lpen(true);
        w.set_tim13lpen(true);
        w.set_tim12lpen(true);
        w.set_tim7lpen(true);
        w.set_tim6lpen(true);
        w.set_tim5lpen(true);
        w.set_tim4lpen(true);
        w.set_tim3lpen(true);
        w.set_tim2lpen(true);
    });
    RCC.apb1hlpenr().modify(|w| {
        w.set_ucpd1lpen(true);
        w.set_fdcanlpen(true);
        w.set_mdioslpen(true);
    });
    RCC.apb2lpenr().modify(|w| {
        w.set_sai2lpen(true);
        w.set_sai1lpen(true);
        w.set_spi5lpen(true);
        w.set_tim9lpen(true);
        w.set_tim17lpen(true);
        w.set_tim16lpen(true);
        w.set_tim15lpen(true);
        w.set_tim18lpen(true);
        w.set_spi4lpen(true);
        w.set_spi1lpen(true);
        w.set_usart10lpen(true);
        w.set_uart9lpen(true);
        w.set_usart6lpen(true);
        w.set_usart1lpen(true);
        w.set_tim8lpen(true);
        w.set_tim1lpen(true);
    });
    RCC.apb3lpenr().modify(|w| {
        w.set_dftlpen(true);
    });
    RCC.apb4llpenr().modify(|w| {
        w.set_rtcapblpen(true);
        w.set_rtclpen(true);
        w.set_vrefbuflpen(true);
        w.set_lptim5lpen(true);
        w.set_lptim4lpen(true);
        w.set_lptim3lpen(true);
        w.set_lptim2lpen(true);
        w.set_i2c4lpen(true);
        w.set_spi6lpen(true);
        w.set_lpuart1lpen(true);
        w.set_hdplpen(true);
    });
    RCC.apb4hlpenr().modify(|w| {
        w.set_dtslpen(true);
        w.set_bseclpen(true);
        w.set_syscfglpen(true);
    });
    RCC.apb5lpenr().modify(|w| {
        w.set_csilpen(true);
        w.set_venclpen(true);
        w.set_gfxtimlpen(true);
        w.set_dcmilpen(true);
        w.set_ltdclpen(true);
    });

    RCC.buslpenr().modify(|w| {
        w.set_aclknclpen(true);
        w.set_aclknlpen(true);
    });

    RCC.memlpenr().modify(|w| {
        w.set_bootromlpen(true);
        w.set_vencramlpen(true);
        w.set_npucacheramlpen(true);
        w.set_flexramlpen(true);
        w.set_axisram2lpen(true);
        w.set_axisram1lpen(true);
        w.set_bkpsramlpen(true);
        w.set_ahbsram2lpen(true);
        w.set_ahbsram1lpen(true);
        w.set_axisram6lpen(true);
        w.set_axisram5lpen(true);
        w.set_axisram4lpen(true);
        w.set_axisram3lpen(true);
    });

    RCC.misclpenr().modify(|w| {
        w.set_perlpen(true);
        w.set_xspiphycomplpen(true);
        w.set_dbglpen(true);
    });
}

const fn periph_prescaler_to_value(bits: u8) -> u8 {
    match bits {
        0 => 1,
        1 => 2,
        2 => 4,
        3 => 8,
        4 => 16,
        5 => 32,
        6 => 64,
        7.. => 128,
    }
}

fn pll_source_ready(source: u8) -> bool {
    match source {
        0x0 if RCC.sr().read().pllrdy(0) || RCC.pllcfgr1(0).read().pllbyp() => true,
        0x1 if RCC.sr().read().pllrdy(1) || RCC.pllcfgr1(1).read().pllbyp() => true,
        0x2 if RCC.sr().read().pllrdy(2) || RCC.pllcfgr1(2).read().pllbyp() => true,
        0x3 if RCC.sr().read().pllrdy(3) || RCC.pllcfgr1(3).read().pllbyp() => true,
        _ => false,
    }
}

fn ic_enabled(ic: u8) -> bool {
    ic > 0 && ic <= 20 && ((RCC.divenr().read().0 >> (ic - 1)) & 0x1) != 0
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

fn power_supply_config(supply_config: SupplyConfig) {
    // power supply config
    PWR.cr1().modify(|w| {
        w.set_sden(match supply_config {
            SupplyConfig::External => false,
            SupplyConfig::Smps => true,
        });
    });

    // Validate supply configuration
    while !PWR.voscr().read().actvosrdy() {}
}

struct PllInput {
    hsi: Option<Hertz>,
    msi: Option<Hertz>,
    hse: Option<Hertz>,
    i2s_ckin: Option<Hertz>,
}

#[derive(Clone, Copy, Default)]
#[allow(dead_code)]
struct PllOutput {
    divm: Option<Hertz>,
    divn: Option<Hertz>,
    divp1: Option<Hertz>,
    divp2: Option<Hertz>,
    output: Option<Hertz>,
}

fn disable_pll(pll_index: usize) {
    let cfgr1 = RCC.pllcfgr1(pll_index);
    let cfgr3 = RCC.pllcfgr3(pll_index);

    cfgr3.modify(|w| w.set_pllpdiven(false));
    RCC.ccr().write(|w| w.set_pllonc(pll_index, true));
    // wait till disabled
    while RCC.sr().read().pllrdy(pll_index) {}

    // clear bypass mode
    cfgr1.modify(|w| w.set_pllbyp(false));
}

fn init_pll(pll_config: Option<Pll>, pll_index: usize, input: &PllInput) -> PllOutput {
    let cfgr1 = RCC.pllcfgr1(pll_index);
    let cfgr2 = RCC.pllcfgr2(pll_index);
    let cfgr3 = RCC.pllcfgr3(pll_index);

    match pll_config {
        Some(Pll::Oscillator {
            source,
            divm,
            fractional,
            divn,
            divp1,
            divp2,
        }) => {
            // ensure pll is disabled
            debug!("PLL{}: disabling", pll_index + 1);
            RCC.ccr().write(|w| w.set_pllonc(pll_index, true));
            while RCC.sr().read().pllrdy(pll_index) {}
            debug!("PLL{}: disabled", pll_index + 1);

            // ensure PLLxMODSSDIS=1 to work in fractional mode
            cfgr3.modify(|w| w.set_pllmodssdis(Pllmodssdis::FRACTIONAL_DIVIDE));
            // clear bypass mode
            cfgr1.modify(|w| w.set_pllbyp(false));
            // configure the pll clock source, mul and div factors
            cfgr1.modify(|w| {
                w.set_pllsel(source);
                w.set_plldivm(divm);
                w.set_plldivn(divn);
            });

            let in_clk = match source {
                Pllsel::HSI => unwrap!(input.hsi),
                Pllsel::MSI => unwrap!(input.msi),
                Pllsel::HSE => unwrap!(input.hse),
                Pllsel::I2S_CKIN => unwrap!(input.i2s_ckin),
                _ => panic!("reserved PLL source not allowed"),
            };

            let m = divm.to_bits() as u32;
            let n = divn as u32;

            cfgr3.modify(|w| {
                w.set_pllpdiv1(divp1);
                w.set_pllpdiv2(divp2);
            });

            let p1 = divp1.to_bits() as u32;
            let p2 = divp2.to_bits() as u32;

            // configure pll divnfrac
            cfgr2.modify(|w| w.set_plldivnfrac(fractional));
            // clear pllxmoddsen
            cfgr3.modify(|w| w.set_pllmoddsen(false));
            // fractional mode
            if fractional != 0 {
                cfgr3.modify(|w| {
                    w.set_pllmoddsen(true);
                    w.set_plldacen(true);
                })
            }
            // enable pll post divider output
            cfgr3.modify(|w| {
                w.set_pllmodssrst(true);
                w.set_pllpdiven(true);
            });
            // enable the pll
            debug!("PLL{}: enabling", pll_index + 1);
            RCC.csr().write(|w| w.set_pllons(pll_index, true));
            // wait until ready
            debug!("PLL{}: waiting for ready", pll_index + 1);
            while !RCC.sr().read().pllrdy(pll_index) {}
            debug!("PLL{}: ready", pll_index + 1);

            PllOutput {
                divm: Some(Hertz(m)),
                divn: Some(Hertz(n)),
                divp1: Some(Hertz(p1)),
                divp2: Some(Hertz(p2)),
                output: Some(Hertz(in_clk.0 / m * n / p1)),
            }
        }
        Some(Pll::Bypass { source }) => {
            // check if source is ready
            if !pll_source_ready(source.to_bits()) {
                panic!("PLL source is not ready")
            }

            // ensure pll is disabled
            RCC.ccr().write(|w| w.set_pllonc(pll_index, true));
            while RCC.sr().read().pllrdy(pll_index) {}

            cfgr1.modify(|w| {
                w.set_pllbyp(true);
                w.set_pllsel(source);
            });

            let in_clk = match source {
                Pllsel::HSI => unwrap!(input.hsi),
                Pllsel::MSI => unwrap!(input.msi),
                Pllsel::HSE => unwrap!(input.hse),
                Pllsel::I2S_CKIN => unwrap!(input.i2s_ckin),
                _ => panic!("reserved PLL source not allowed"),
            };

            PllOutput {
                output: Some(in_clk),
                ..Default::default()
            }
        }
        None => {
            disable_pll(pll_index);

            PllOutput::default()
        }
    }
}

#[allow(dead_code)]
struct OscOutput {
    hsi: Option<Hertz>,
    hse: Option<Hertz>,
    msi: Option<Hertz>,
    lsi: Option<Hertz>,
    lse: Option<Hertz>,
    pll1: Option<Hertz>,
    pll2: Option<Hertz>,
    pll3: Option<Hertz>,
    pll4: Option<Hertz>,
    ic1sel: Icsel,
    ic2sel: Icsel,
    ic6sel: Icsel,
    ic11sel: Icsel,
}

fn init_osc(config: Config) -> OscOutput {
    let (cpu_src, sys_src) = {
        let reg = RCC.cfgr().read();
        (reg.cpusws(), reg.syssws())
    };
    let pll1_src = RCC.pllcfgr1(0).read().pllsel();
    let pll2_src = RCC.pllcfgr1(1).read().pllsel();
    let pll3_src = RCC.pllcfgr1(2).read().pllsel();
    let pll4_src = RCC.pllcfgr1(3).read().pllsel();
    let rcc_sr = RCC.sr().read();

    debug!("configuring HSE");

    // hse configuration
    let hse = if let Some(hse) = config.hse {
        match hse.mode {
            HseMode::Oscillator => {
                debug!("HSE in oscillator mode");
            }
            HseMode::Bypass => {
                debug!("HSE in bypass mode");
                RCC.hsecfgr().modify(|w| {
                    w.set_hsebyp(true);
                    w.set_hseext(Hseext::ANALOG);
                });
            }
            HseMode::BypassDigital => {
                debug!("HSE in bypass digital mode");
                RCC.hsecfgr().modify(|w| {
                    w.set_hsebyp(true);
                    w.set_hseext(Hseext::DIGITAL);
                });
            }
        }
        RCC.csr().write(|w| w.set_hseons(true));

        // wait until the hse is ready
        while !RCC.sr().read().hserdy() {}

        Some(hse.freq)
    } else if cpu_src == Cpusws::HSE
        || sys_src == Syssws::HSE
        || (pll1_src == Pllsel::HSE && rcc_sr.pllrdy(0))
        || (pll2_src == Pllsel::HSE && rcc_sr.pllrdy(1))
        || (pll3_src == Pllsel::HSE && rcc_sr.pllrdy(2))
        || (pll4_src == Pllsel::HSE && rcc_sr.pllrdy(3))
    {
        panic!(
            "When the HSE is used as cpu/system bus clock or clock source for any PLL, it is not allowed to be disabled"
        );
    } else {
        debug!("HSE off");

        RCC.ccr().write(|w| w.set_hseonc(true));
        RCC.hsecfgr().modify(|w| {
            w.set_hseext(Hseext::ANALOG);
            w.set_hsebyp(false);
        });

        // wait until the hse is disabled
        while RCC.sr().read().hserdy() {}

        None
    };

    // hsi configuration
    debug!("configuring HSI");
    let hsi = if let Some(hsi) = config.hsi {
        RCC.csr().write(|w| w.set_hsions(true));
        while !RCC.sr().read().hsirdy() {}

        // set divider and calibration
        RCC.hsicfgr().modify(|w| {
            w.set_hsidiv(hsi.pre);
            w.set_hsitrim(hsi.trim);
        });

        Some(HSI_FREQ / hsi.pre)
    } else if cpu_src == Cpusws::HSI
        || sys_src == Syssws::HSI
        || (pll1_src == Pllsel::HSI && rcc_sr.pllrdy(0))
        || (pll2_src == Pllsel::HSI && rcc_sr.pllrdy(1))
        || (pll3_src == Pllsel::HSI && rcc_sr.pllrdy(2))
        || (pll4_src == Pllsel::HSI && rcc_sr.pllrdy(3))
    {
        panic!(
            "When the HSI is used as cpu/system bus clock or clock source for any PLL, it is not allowed to be disabled"
        );
    } else {
        debug!("HSI off");

        RCC.ccr().write(|w| w.set_hsionc(true));
        while RCC.sr().read().hsirdy() {}

        None
    };

    // msi configuration
    debug!("configuring MSI");
    let msi = if let Some(msi) = config.msi {
        RCC.msicfgr().modify(|w| w.set_msifreqsel(msi.freq));
        RCC.csr().write(|w| w.set_msions(true));
        while !RCC.sr().read().msirdy() {}
        RCC.msicfgr().modify(|w| w.set_msitrim(msi.trim));

        Some(match msi.freq {
            Msifreqsel::_4MHZ => Hertz::mhz(4),
            Msifreqsel::_16MHZ => Hertz::mhz(16),
        })
    } else if cpu_src == Cpusws::MSI
        || sys_src == Syssws::MSI
        || (pll1_src == Pllsel::MSI && rcc_sr.pllrdy(0))
        || (pll2_src == Pllsel::MSI && rcc_sr.pllrdy(1))
        || (pll3_src == Pllsel::MSI && rcc_sr.pllrdy(2))
        || (pll4_src == Pllsel::MSI && rcc_sr.pllrdy(3))
    {
        panic!(
            "When the MSI is used as cpu/system bus clock or clock source for any PLL, it is not allowed to be disabled"
        );
    } else {
        RCC.ccr().write(|w| w.set_msionc(true));
        while RCC.sr().read().msirdy() {}

        None
    };

    // lsi configuration
    debug!("configuring LSI");
    let lsi = if config.lsi {
        RCC.csr().write(|w| w.set_lsions(true));
        while !RCC.sr().read().lsirdy() {}
        Some(super::LSI_FREQ)
    } else {
        RCC.ccr().write(|w| w.set_lsionc(true));
        while RCC.sr().read().lsirdy() {}
        None
    };

    // lse configuration
    debug!("configuring LSE");
    let lse = if config.lse {
        RCC.csr().write(|w| w.set_lseons(true));
        while !RCC.sr().read().lserdy() {}
        Some(LSE_FREQ)
    } else {
        RCC.ccr().write(|w| w.set_lseonc(true));
        while RCC.sr().read().lserdy() {}
        None
    };

    let pll_input = PllInput {
        hse,
        msi,
        hsi,
        i2s_ckin: None,
    };

    // pll1,2,3,4 config
    let pll_configs = [config.pll1, config.pll2, config.pll3, config.pll4];
    let mut pll_outputs: [PllOutput; 4] = [PllOutput::default(); 4];

    let ic1_src = RCC.iccfgr(0).read().icsel();
    let ic2_src = RCC.iccfgr(1).read().icsel();
    let ic6_src = RCC.iccfgr(5).read().icsel();
    let ic11_src = RCC.iccfgr(10).read().icsel();

    // If config wants a non-IC1 CPU source (HSI/HSE/MSI), switch now before
    // touching PLLs. This prevents panicking when trying to reconfigure a PLL
    // that's currently in use by IC1.
    let cpu_src = if cpu_src == Cpusws::IC1 && !matches!(config.cpu, CpuClk::Ic1 { .. }) {
        // Switch CPU clock to the target source first
        debug!("switching CPU away from IC1 before PLL reconfiguration");
        let cpusw = Cpusw::from_bits(config.cpu.to_bits());
        RCC.cfgr().modify(|w| w.set_cpusw(cpusw));
        while RCC.cfgr().read().cpusws() != Cpusws::from_bits(config.cpu.to_bits()) {}
        // Return the new CPU source
        RCC.cfgr().read().cpusws()
    } else {
        cpu_src
    };

    // If config wants a non-IC2 sys source (HSI/HSE/MSI), switch now before
    // touching PLLs. This prevents panicking when trying to reconfigure a PLL
    // that's currently in use by IC2, IC6, or IC11.
    let sys_src = if sys_src == Syssws::IC2 && !matches!(config.sys, SysClk::Ic2 { .. }) {
        // Switch system clock to the target source first
        debug!("switching sys clock away from IC2 before PLL reconfiguration");
        let syssw = Syssw::from_bits(config.sys.to_bits());
        RCC.cfgr().modify(|w| w.set_syssw(syssw));
        while RCC.cfgr().read().syssws() != Syssws::from_bits(config.sys.to_bits()) {}
        // Return the new sys source
        RCC.cfgr().read().syssws()
    } else {
        sys_src
    };

    for (n, (&pll, out)) in pll_configs.iter().zip(pll_outputs.iter_mut()).enumerate() {
        debug!("configuring PLL{}", n + 1);
        let pll_ready = RCC.sr().read().pllrdy(n);

        if is_new_pll_config(pll, n) {
            let this_pll = Icsel::from_bits(n as u8);

            if cpu_src == Cpusws::IC1 && ic1_src == this_pll {
                panic!("PLL should not be disabled / reconfigured if used for IC1 (cpuclksrc)")
            }

            if sys_src == Syssws::IC2 && (ic2_src == this_pll || ic6_src == this_pll || ic11_src == this_pll) {
                panic!("PLL should not be disabled / reconfigured if used for IC2, IC6 or IC11 (sysclksrc)")
            }

            *out = pll.map_or_else(
                || {
                    disable_pll(n);
                    PllOutput::default()
                },
                |c| init_pll(Some(c), n, &pll_input),
            );
        } else if pll.is_some() && !pll_ready {
            RCC.csr().write(|w| w.set_pllons(n, true));
            while !RCC.sr().read().pllrdy(n) {}
        }
    }

    OscOutput {
        hsi,
        hse,
        msi,
        lsi,
        lse,
        pll1: pll_outputs[0].output,
        pll2: pll_outputs[1].output,
        pll3: pll_outputs[2].output,
        pll4: pll_outputs[3].output,
        ic1sel: ic1_src,
        ic2sel: ic2_src,
        ic6sel: ic6_src,
        ic11sel: ic11_src,
    }
}

fn is_new_pll_config(pll: Option<Pll>, pll_index: usize) -> bool {
    let cfgr1 = RCC.pllcfgr1(pll_index).read();
    let cfgr2 = RCC.pllcfgr2(pll_index).read();
    let cfgr3 = RCC.pllcfgr3(pll_index).read();

    let ready = RCC.sr().read().pllrdy(pll_index);
    let bypass = cfgr1.pllbyp();

    match (pll, ready, bypass) {
        (None, true, _) => return true,
        (Some(_), false, _) => return true,
        (Some(conf), true, bypass) => match (conf, bypass) {
            (Pll::Bypass { .. }, false) => return true,
            (Pll::Oscillator { .. }, true) => return true,
            _ => {}
        },
        _ => {}
    }

    match pll {
        Some(Pll::Bypass { source }) => cfgr1.pllsel() != source,
        Some(Pll::Oscillator {
            source,
            divm: m,
            fractional,
            divn: n,
            divp1: p1,
            divp2: p2,
        }) => {
            cfgr1.pllsel() != source
                || cfgr1.plldivm() != m
                || cfgr1.plldivn() != n
                || cfgr2.plldivnfrac() != fractional
                || cfgr3.pllpdiv1() != p1
                || cfgr3.pllpdiv2() != p2
        }
        None => false,
    }
}

pub(crate) unsafe fn init(config: Config) {
    debug!("enabling SYSCFG");
    // system configuration setup
    RCC.apb4hensr().write(|w| w.set_syscfgens(true));
    // delay after RCC peripheral clock enabling
    RCC.apb4hensr().read();

    debug!("setting VTOR");

    let vtor = unsafe {
        let p = cortex_m::Peripherals::steal();
        p.SCB.vtor.read()
    };

    // set default vector table location after reset or standby
    SYSCFG.initsvtorcr().write(|w| w.set_svtor_addr(vtor));
    // read back the value to ensure it is written before deactivating SYSCFG
    SYSCFG.initsvtorcr().read();

    debug!("deactivating SYSCFG");

    // deactivate SYSCFG
    RCC.apb4hensr().write(|w| w.set_syscfgens(false));

    debug!("enabling FPU");

    // enable fpu
    unsafe {
        let p = cortex_m::Peripherals::steal();
        p.SCB.cpacr.modify(|w| w | (3 << 20) | (3 << 22));
    }

    // Configure RIF/RISAF for memory access
    // RISAF register offsets: REG_CFGR=0x40, REG_STARTR=0x44, REG_ENDR=0x48, REG_CIDCFGR=0x4C (stride 0x40)
    // CRITICAL: Enable RISAF and RIFSC clocks BEFORE accessing registers
    debug!("configuring RISAF");
    RCC.ahb3ensr().write(|w| {
        w.set_risafens(true); // Enable RISAF clock
        w.set_rifscens(true); // Enable RIFSC clock
    });
    // Read-back delay after RCC peripheral clock enabling (matches ST HAL pattern)
    // Must read from AHB3ENR (status) not AHB3ENSR (set) to ensure clock is stable
    RCC.ahb3enr().read();

    // RISAF3: SRAM access for DMA
    {
        // Region 0: secure access (RW for CIDs 0-3)
        RISAF3.reg_cidcfgr(0).write(|w| {
            for i in 0..4 {
                w.set_rdenc(i, true);
                w.set_wrenc(i, true);
            }
        });
        RISAF3.reg_endr(0).write(|w| w.set_baddend(0xFFFFFFFF));
        RISAF3.reg_cfgr(0).write(|w| {
            w.set_bren(true);
            w.set_sec(true);
        });

        // Region 1: non-secure access (RW for all CIDs)
        RISAF3.reg_cidcfgr(1).write(|w| {
            for i in 0..8 {
                w.set_rdenc(i, true);
                w.set_wrenc(i, true);
            }
        });
        RISAF3.reg_endr(1).write(|w| w.set_baddend(0xFFFFFFFF));
        RISAF3.reg_cfgr(1).write(|w| {
            w.set_bren(true);
            w.set_sec(false);
        });
    }

    debug!("setting power supply config");

    power_supply_config(config.supply_config);

    // VddIO power domain configuration per STM32N6 errata ES0620
    // This must be done early in boot - set SV bits and wait for RDY
    debug!("configuring VddIO power domains");
    {
        // Enable supply valid for all VddIO domains (like ST's SystemInit)
        // PWR is always accessible on N6, no need to enable clock

        // SVMCR1: VddIO4
        PWR.svmcr1().modify(|w| {
            w.set_vddio4sv(Vddio4sv::B_0X1);
        });
        // SVMCR2: VddIO5
        PWR.svmcr2().modify(|w| {
            w.set_vddio5sv(Vddio5sv::B_0X1);
        });
        // SVMCR3: VddIO2 and VddIO3 (for XSPI1 and XSPI2)
        PWR.svmcr3().modify(|w| {
            w.set_vddio2sv(Vddio2sv::B_0X1);
            w.set_vddio2vmen(true); // Enable voltage monitoring
            w.set_vddio3sv(Vddio3sv::B_0X1);
            w.set_vddio3vmen(true); // Enable voltage monitoring
            // Set voltage range based on config
            if config.vddio2_1v8 {
                w.set_vddio2vrsel(Vddio2vrsel::B_0X1); // 1.8V mode
            }
            if config.vddio3_1v8 {
                w.set_vddio3vrsel(Vddio3vrsel::B_0X1); // 1.8V mode
            }
        });

        // Wait for VddIO domains to be ready
        while PWR.svmcr3().read().vddio2rdy() != Vddio2rdy::B_0X1 {}
        while PWR.svmcr3().read().vddio3rdy() != Vddio3rdy::B_0X1 {}

        // Debug VddIO status after configuration
        let svmcr3 = PWR.svmcr3().read();
        debug!("VddIO2 ready: {}", svmcr3.vddio2rdy() == Vddio2rdy::B_0X1);
        debug!("VddIO3 ready: {}", svmcr3.vddio3rdy() == Vddio3rdy::B_0X1);
        debug!("SVMCR3 raw = 0x{:08x}", svmcr3.0);

        // Configure compensation cells per errata ES0620
        // SYSCFG is already enabled earlier in init

        // Set compensation cell values (0x287 = ST's recommended value)
        // ransrc=7 (bits 0-3), rapsrc=8 (bits 4-7), en=1 (bit 8)
        SYSCFG.vddio2cccr().write(|w| {
            w.set_ransrc(0x7);
            w.set_rapsrc(0x8);
            w.set_en(Vddio2cccrEn::B_0X1);
        });
        SYSCFG.vddio3cccr().write(|w| {
            w.set_ransrc(0x7);
            w.set_rapsrc(0x8);
            w.set_en(Vddio3cccrEn::B_0X1);
        });
        SYSCFG.vddio4cccr().write(|w| {
            w.set_ransrc(0x7);
            w.set_rapsrc(0x8);
            w.set_en(Vddio4cccrEn::B_0X1);
        });
    }

    let osc = init_osc(config);
    let ic_freqs = [
        config.ic1,
        config.ic2,
        config.ic3,
        config.ic4,
        config.ic5,
        config.ic6,
        config.ic7,
        config.ic8,
        config.ic9,
        config.ic10,
        config.ic11,
        config.ic12,
        config.ic13,
        config.ic14,
        config.ic15,
        config.ic16,
        config.ic17,
        config.ic18,
        config.ic19,
        config.ic20,
    ]
    .map(|ic| {
        let ic_cfg = ic?;
        let pll_freq = match ic_cfg.source.to_bits() {
            0 => osc.pll1,
            1 => osc.pll2,
            2 => osc.pll3,
            3 => osc.pll4,
            _ => None,
        }?;
        let divider = (ic_cfg.divider.to_bits() as u32) + 1; // ICINT 0 = divide by 1
        Some(Hertz(pll_freq.0 / divider))
    });
    let clock_inputs = ClocksInput {
        hsi: osc.hsi,
        msi: osc.msi,
        hse: osc.hse,
        ic1: ic_freqs[0],
        ic2: ic_freqs[1],
        ic3: ic_freqs[2],
        ic4: ic_freqs[3],
        ic5: ic_freqs[4],
        ic6: ic_freqs[5],
        ic7: ic_freqs[6],
        ic8: ic_freqs[7],
        ic9: ic_freqs[8],
        ic10: ic_freqs[9],
        ic11: ic_freqs[10],
        ic12: ic_freqs[11],
        ic13: ic_freqs[12],
        ic14: ic_freqs[13],
        ic15: ic_freqs[14],
        ic16: ic_freqs[15],
        ic17: ic_freqs[16],
        ic18: ic_freqs[17],
        ic19: ic_freqs[18],
        ic20: ic_freqs[19],
    };
    let clocks = init_clocks(config, &clock_inputs);

    // TODO: sysb, sysc, sysd must have the same clock source

    config.mux.init();

    set_clocks!(
        sys: Some(clocks.sysclk),
        hsi: clock_inputs.hsi,
        hsi_div: None,
        hse: clock_inputs.hse,
        msi: clock_inputs.msi,
        lse: None,
        hclk1: Some(clocks.ahb),
        hclk2: Some(clocks.ahb),
        hclk3: Some(clocks.ahb),
        hclk4: Some(clocks.ahb),
        hclk5: Some(clocks.ahb),
        pclk1: Some(clocks.apb1),
        pclk2: Some(clocks.apb2),
        pclk1_tim: Some(clocks.pclk_tim),
        pclk2_tim: Some(clocks.pclk_tim),
        pclk4: Some(clocks.apb4),
        pclk5: Some(clocks.apb5),
        per: None,
        rtc: None,
        i2s_ckin: None,
        ic1: clock_inputs.ic1,
        ic2: clock_inputs.ic2,
        ic3: clock_inputs.ic3,
        ic4: clock_inputs.ic4,
        ic5: clock_inputs.ic5,
        ic6: clock_inputs.ic6,
        ic7: clock_inputs.ic7,
        ic8: clock_inputs.ic8,
        ic9: clock_inputs.ic9,
        ic10: clock_inputs.ic10,
        ic11: clock_inputs.ic11,
        ic12: clock_inputs.ic12,
        ic13: clock_inputs.ic13,
        ic14: clock_inputs.ic14,
        ic15: clock_inputs.ic15,
        ic16: clock_inputs.ic16,
        ic17: clock_inputs.ic17,
        ic18: clock_inputs.ic18,
        ic19: clock_inputs.ic19,
        ic20: clock_inputs.ic20,
    );
}
