use stm32_metapac::rcc::vals::{
    Cpusw, Cpusws, Hseext, Hsitrim, Icint, Icsel, Msifreqsel, Plldivm, Pllmodssdis, Pllpdiv, Pllsel, Syssw, Syssws,
};
pub use stm32_metapac::rcc::vals::{
    Hpre as AhbPrescaler, Hsidiv as HsiPrescaler, Hsitrim as HsiCalibration, Ppre as ApbPrescaler,
};

use crate::pac::{PWR, RCC, SYSCFG};
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
    Pll { source: Icsel, divider: Icint },
    Msi,
    Hsi,
}

impl CpuClk {
    const fn to_bits(self) -> u8 {
        match self {
            Self::Hsi => 0x0,
            Self::Msi => 0x1,
            Self::Hse => 0x2,
            Self::Pll { .. } => 0x3,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct IcConfig {
    source: Icsel,
    divider: Icint,
}

#[derive(Clone, Copy, PartialEq)]
pub enum SysClk {
    Hse,
    Pll {
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
            Self::Pll { .. } => 0x3,
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

    pub ahb: AhbPrescaler,
    pub apb1: ApbPrescaler,
    pub apb2: ApbPrescaler,
    pub apb4: ApbPrescaler,
    pub apb5: ApbPrescaler,

    pub supply_config: SupplyConfig,
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
            pll1: None,
            pll2: None,
            pll3: None,
            pll4: None,

            ahb: AhbPrescaler::DIV1,
            apb1: ApbPrescaler::DIV1,
            apb2: ApbPrescaler::DIV1,
            apb4: ApbPrescaler::DIV1,
            apb5: ApbPrescaler::DIV1,

            supply_config: SupplyConfig::Smps,
        }
    }
}

fn init_clocks(config: Config) {
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
        CpuClk::Pll { source, divider } => {
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
        SysClk::Pll { ic2, ic6, ic11 } => {
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

fn init_pll(pll_config: Option<Pll>, pll_index: usize) {
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
            RCC.ccr().write(|w| w.set_pllonc(pll_index, true));
            while RCC.sr().read().pllrdy(pll_index) {}

            // ensure PLLxMODSSDIS=1
            cfgr3.modify(|w| w.set_pllmodssdis(Pllmodssdis::FRACTIONAL_DIVIDE));
            // clear bypass mode
            cfgr1.modify(|w| w.set_pllbyp(false));
            // configure the pll clock source, mul and div factors
            cfgr1.modify(|w| {
                w.set_pllsel(source);
                w.set_plldivm(divm);
                w.set_plldivn(divn);
            });
            cfgr3.modify(|w| {
                w.set_pllpdiv1(divp1);
                w.set_pllpdiv2(divp2);
            });
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
            RCC.csr().write(|w| w.pllons(pll_index));
            // wait until ready
            while RCC.sr().read().pllrdy(pll_index) {}
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
            })
        }
        None => {
            cfgr3.modify(|w| w.set_pllpdiven(false));
            RCC.ccr().write(|w| w.set_pllonc(pll_index, true));
            // wait till disabled
            while RCC.sr().read().pllrdy(pll_index) {}

            // clear bypass mode
            cfgr1.modify(|w| w.set_pllbyp(false));
        }
    }
}

fn init_osc(config: Config) {
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
        panic!("When the HSE is used as cpu/system bus clock or clock source for any PLL, it is not allowed to be disabled");
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
        panic!("When the HSI is used as cpu/system bus clock or clock source for any PLL, it is not allowed to be disabled");
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
        panic!("When the MSI is used as cpu/system bus clock or clock source for any PLL, it is not allowed to be disabled");
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

    // pll1,2,3,4 config
    let pll_configs = [config.pll1, config.pll2, config.pll3, config.pll4];
    for (n, &pll) in pll_configs.iter().enumerate() {
        debug!("configuring PLL{}", n + 1);
        let pll_ready = RCC.sr().read().pllrdy(n);

        if is_new_pll_config(pll, 0) {
            let ic1_src = RCC.iccfgr(0).read().icsel();
            let ic2_src = RCC.iccfgr(1).read().icsel();
            let ic6_src = RCC.iccfgr(5).read().icsel();
            let ic11_src = RCC.iccfgr(10).read().icsel();

            let this_pll = Icsel::from_bits(n as u8);

            if cpu_src == Cpusws::IC1 && ic1_src == this_pll {
                panic!("PLL should not be disabled / reconfigured if used for IC1 (cpuclksrc)")
            }

            if sys_src == Syssws::IC2 && (ic2_src == this_pll || ic6_src == this_pll || ic11_src == this_pll) {
                panic!("PLL should not be disabled / reconfigured if used for IC2, IC6 or IC11 (sysclksrc)")
            }

            init_pll(pll, 0);
        } else if pll.is_some() && !pll_ready {
            debug!("PLL{} off", n + 1);
            RCC.csr().write(|w| w.pllons(n));
            while !RCC.sr().read().pllrdy(n) {}
        }
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
    core::ptr::read_volatile(RCC.apb4hensr().as_ptr());

    debug!("setting VTOR");

    let vtor = unsafe {
        let p = cortex_m::Peripherals::steal();
        p.SCB.vtor.read()
    };

    // set default vector table location after reset or standby
    SYSCFG.initsvtorcr().write(|w| w.set_svtor_addr(vtor));
    // read back the value to ensure it is written before deactivating SYSCFG
    core::ptr::read_volatile(SYSCFG.initsvtorcr().as_ptr());

    debug!("deactivating SYSCFG");

    // deactivate SYSCFG
    RCC.apb4hensr().write(|w| w.set_syscfgens(false));

    debug!("enabling FPU");

    // enable fpu
    unsafe {
        let p = cortex_m::Peripherals::steal();
        p.SCB.cpacr.modify(|w| w | (3 << 20) | (3 << 22));
    }

    debug!("setting power supply config");

    power_supply_config(config.supply_config);

    init_osc(config);
    init_clocks(config);

    let sys = match config.sys {
        SysClk::Hse => todo!(),
        SysClk::Hsi => Hertz(64_000_000),
        SysClk::Msi => todo!(),
        SysClk::Pll { .. } => todo!(),
    };

    // TODO: sysb, sysc, sysd must have the same clock source

    set_clocks!(
        sys: Some(sys),
        hsi: None,
        hsi_div: None,
        hse: None,
        hclk1: None,
        hclk2: None,
        hclk3: None,
        hclk4: None,
        hclk5: None,
        pclk1: None,
        pclk2: None,
        pclk2_tim: Some(Hertz(1_000_000)), // FIXME: what is this??
        pclk4: None,
        pclk5: None,
        per: None,
        rtc: None,
        msi: None,
        i2s_ckin: None,
        ic8: None,
        ic9: None,
        ic14: None,
        ic17: None,
        ic20: None,
    );
}
