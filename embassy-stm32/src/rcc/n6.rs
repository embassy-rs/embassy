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
    Hse,
    Ic1 { source: Icsel, divider: Icint },
    Msi,
    Hsi,
}

impl CpuClk {
    const fn to_bits(self) -> u8 {
        match self {
            Self::Hsi => 0x0,
            Self::Msi => 0x1,
            Self::Hse => 0x2,
            Self::Ic1 { .. } => 0x3,
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
    Hse,
    Ic2 {
        ic2: IcConfig,
        ic6: IcConfig,
        ic11: IcConfig,
    },
    Msi,
    Hsi,
}

impl SysClk {
    const fn to_bits(self) -> u8 {
        match self {
            Self::Hsi => 0x0,
            Self::Msi => 0x1,
            Self::Hse => 0x2,
            Self::Ic2 { .. } => 0x3,
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

    pub sys: SysClk,
    pub cpu: CpuClk,

    pub pll1: Option<Pll>,
    pub pll2: Option<Pll>,
    pub pll3: Option<Pll>,
    pub pll4: Option<Pll>,

    /// IC3 kernel clock configuration (used by XSPI1)
    pub ic3: Option<IcConfig>,
    /// IC4 kernel clock configuration (used by XSPI2)
    pub ic4: Option<IcConfig>,

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

    /// XSPI1 kernel clock source selection
    /// Default: HCLK5
    pub xspi1_clk_src: XspiClkSrc,
    /// XSPI2 kernel clock source selection
    /// Default: HCLK5
    pub xspi2_clk_src: XspiClkSrc,
    /// XSPI3 kernel clock source selection
    /// Default: HCLK5
    pub xspi3_clk_src: XspiClkSrc,
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
            sys: SysClk::Hsi,
            cpu: CpuClk::Hsi,
            pll1: Some(Pll::Bypass { source: Pllsel::HSI }),
            pll2: Some(Pll::Bypass { source: Pllsel::HSI }),
            pll3: Some(Pll::Bypass { source: Pllsel::HSI }),
            pll4: Some(Pll::Bypass { source: Pllsel::HSI }),

            ic3: None,
            ic4: None,

            ahb: AhbPrescaler::DIV2,
            apb1: ApbPrescaler::DIV1,
            apb2: ApbPrescaler::DIV1,
            apb4: ApbPrescaler::DIV1,
            apb5: ApbPrescaler::DIV1,

            supply_config: SupplyConfig::Smps,

            vddio2_1v8: false, // Default to 3.3V
            vddio3_1v8: false, // Default to 3.3V

            xspi1_clk_src: XspiClkSrc::HCLK5,
            xspi2_clk_src: XspiClkSrc::HCLK5,
            xspi3_clk_src: XspiClkSrc::HCLK5,
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
    pll1: Option<Hertz>,
    pll2: Option<Hertz>,
    pll3: Option<Hertz>,
    pll4: Option<Hertz>,
}

fn init_clocks(config: Config, input: &ClocksInput) -> ClocksOutput {
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
        CpuClk::Hse if !RCC.sr().read().hserdy() => panic!("HSE is not ready to be selected as CPU clock source"),
        CpuClk::Ic1 { source, divider } => {
            if !pll_sources_ready(RCC.iccfgr(0).read().icsel().to_bits(), source.to_bits()) {
                panic!("ICx clock switch requires both origin and destination clock source to be active")
            }

            RCC.iccfgr(0).write(|w| {
                w.set_icsel(source);
                w.set_icint(divider);
            });
            RCC.divensr().modify(|w| w.set_ic1ens(true));
        }
        CpuClk::Msi if !RCC.sr().read().msirdy() => panic!("MSI is not ready to be selected as CPU clock source"),
        CpuClk::Hsi if !RCC.sr().read().hsirdy() => panic!("HSI is not ready to be selected as CPU clock source"),
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
        SysClk::Hse if !RCC.sr().read().hserdy() => panic!("HSE is not ready to be selected as CPU clock source"),
        SysClk::Ic2 { ic2, ic6, ic11 } => {
            if !pll_sources_ready(RCC.iccfgr(1).read().icsel().to_bits(), ic2.source.to_bits()) {
                panic!("IC2 clock switch requires both origin and destination clock source to be active")
            }
            if !pll_sources_ready(RCC.iccfgr(5).read().icsel().to_bits(), ic6.source.to_bits()) {
                panic!("IC6 clock switch requires both origin and destination clock source to be active")
            }
            if !pll_sources_ready(RCC.iccfgr(10).read().icsel().to_bits(), ic11.source.to_bits()) {
                panic!("IC11 clock switch requires both origin and destination clock source to be active")
            }

            RCC.iccfgr(1).write(|w| {
                w.set_icsel(ic2.source);
                w.set_icint(ic2.divider);
            });
            RCC.iccfgr(5).write(|w| {
                w.set_icsel(ic6.source);
                w.set_icint(ic6.divider);
            });
            RCC.iccfgr(10).write(|w| {
                w.set_icsel(ic11.source);
                w.set_icint(ic11.divider);
            });
            RCC.divensr().modify(|w| {
                w.set_ic2ens(true);
                w.set_ic6ens(true);
                w.set_ic11ens(true);
            });
        }
        SysClk::Msi if !RCC.sr().read().msirdy() => panic!("MSI is not ready to be selected as CPU clock source"),
        SysClk::Hsi if !RCC.sr().read().hsirdy() => panic!("HSI is not ready to be selected as CPU clock source"),
        _ => {}
    }
    // switch the system bus clock
    let syssw = Syssw::from_bits(config.sys.to_bits());
    RCC.cfgr().modify(|w| w.set_syssw(syssw));
    // wait for changes to be applied
    while RCC.cfgr().read().syssws() != Syssws::from_bits(config.sys.to_bits()) {}

    // IC3 configuration (XSPI1 kernel clock)
    debug!("configuring IC3");
    if let Some(ic3) = config.ic3 {
        if !pll_sources_ready(RCC.iccfgr(2).read().icsel().to_bits(), ic3.source.to_bits()) {
            panic!("IC3 clock switch requires both origin and destination clock source to be active")
        }

        RCC.iccfgr(2).write(|w| {
            w.set_icsel(ic3.source);
            w.set_icint(ic3.divider);
        });
        RCC.divensr().modify(|w| w.set_ic3ens(true));
    }

    // IC4 configuration (XSPI2 kernel clock)
    debug!("configuring IC4");
    if let Some(ic4) = config.ic4 {
        if !pll_sources_ready(RCC.iccfgr(3).read().icsel().to_bits(), ic4.source.to_bits()) {
            panic!("IC4 clock switch requires both origin and destination clock source to be active")
        }

        RCC.iccfgr(3).write(|w| {
            w.set_icsel(ic4.source);
            w.set_icint(ic4.divider);
        });
        RCC.divensr().modify(|w| w.set_ic4ens(true));
    }

    // XSPI clock source configuration
    debug!("configuring XSPI clock sources");
    RCC.ccipr6().modify(|w| {
        w.set_xspi1sel(config.xspi1_clk_src);
        w.set_xspi2sel(config.xspi2_clk_src);
        w.set_xspi3sel(config.xspi3_clk_src);
    });

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
        CpuClk::Ic1 { source, divider } => {
            let src_freq = match source {
                Icsel::PLL1 => unwrap!(input.pll1),
                Icsel::PLL2 => unwrap!(input.pll2),
                Icsel::PLL3 => unwrap!(input.pll3),
                Icsel::PLL4 => unwrap!(input.pll4),
            };
            let div = (divider.to_bits() as u32) + 1;
            Hertz(src_freq.0 / div)
        }
    };

    let sysclk = match config.sys {
        SysClk::Hsi => unwrap!(input.hsi),
        SysClk::Msi => unwrap!(input.msi),
        SysClk::Hse => unwrap!(input.hse),
        SysClk::Ic2 { ic2, .. } => {
            let src_freq = match ic2.source {
                Icsel::PLL1 => unwrap!(input.pll1),
                Icsel::PLL2 => unwrap!(input.pll2),
                Icsel::PLL3 => unwrap!(input.pll3),
                Icsel::PLL4 => unwrap!(input.pll4),
            };
            let div = (ic2.divider.to_bits() as u32) + 1;
            Hertz(src_freq.0 / div)
        }
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
        sysclk,
        cpuclk,
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
        0x0 if !RCC.sr().read().pllrdy(0) && !RCC.pllcfgr1(0).read().pllbyp() => false,
        0x1 if !RCC.sr().read().pllrdy(1) && !RCC.pllcfgr1(1).read().pllbyp() => false,
        0x2 if !RCC.sr().read().pllrdy(2) && !RCC.pllcfgr1(2).read().pllbyp() => false,
        0x3 if !RCC.sr().read().pllrdy(3) && !RCC.pllcfgr1(3).read().pllbyp() => false,
        _ => true,
    }
}

fn pll_sources_ready(source1: u8, source2: u8) -> bool {
    pll_source_ready(source1) && pll_source_ready(source2)
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
    let clock_inputs = ClocksInput {
        hsi: osc.hsi,
        msi: osc.msi,
        hse: osc.hse,
        pll1: osc.pll1,
        pll2: osc.pll2,
        pll3: osc.pll3,
        pll4: osc.pll4,
    };
    let clocks = init_clocks(config, &clock_inputs);

    // Calculate IC3 clock frequency (XSPI1 kernel clock)
    let ic3_freq = config
        .ic3
        .map(|ic_cfg| {
            let pll_freq = match ic_cfg.source.to_bits() {
                0 => osc.pll1,
                1 => osc.pll2,
                2 => osc.pll3,
                3 => osc.pll4,
                _ => None,
            };
            pll_freq.map(|freq| {
                let divider = (ic_cfg.divider.to_bits() as u32) + 1; // ICINT 0 = divide by 1
                Hertz(freq.0 / divider)
            })
        })
        .flatten();

    // Calculate IC4 clock frequency (XSPI2 kernel clock)
    let ic4_freq = config
        .ic4
        .map(|ic_cfg| {
            let pll_freq = match ic_cfg.source.to_bits() {
                0 => osc.pll1,
                1 => osc.pll2,
                2 => osc.pll3,
                3 => osc.pll4,
                _ => None,
            };
            pll_freq.map(|freq| {
                let divider = (ic_cfg.divider.to_bits() as u32) + 1; // ICINT 0 = divide by 1
                Hertz(freq.0 / divider)
            })
        })
        .flatten();

    // TODO: sysb, sysc, sysd must have the same clock source

    set_clocks!(
        sys: Some(clocks.sysclk),
        hsi: osc.hsi,
        hsi_div: None,
        hse: osc.hse,
        msi: osc.msi,
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
        ic3: ic3_freq,
        ic4: ic4_freq,
        ic8: None,
        ic9: None,
        ic10: None,
        ic14: None,
        ic15: None,
        ic17: None,
        ic20: None,
    );
}
