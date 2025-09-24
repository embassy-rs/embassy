use stm32_metapac::rcc::vals::{Cpusws, Hseext, Hsitrim, Pllsel, Syssws};
pub use stm32_metapac::rcc::vals::{Hsidiv as HsiPrescaler, Hsitrim as HsiCalibration, Syssw as Sysclk};

use crate::pac::{PWR, RCC, SYSCFG};
use crate::time::Hertz;

pub const HSI_FREQ: Hertz = Hertz(64_000_000);

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
    pub calib: Hsitrim,
}

#[derive(Clone, Copy, PartialEq)]
pub enum SupplyConfig {
    Smps,
    External,
}

/// Configuration of the core clocks
#[non_exhaustive]
#[derive(Clone, Copy)]
pub struct Config {
    pub hsi: Option<Hsi>,
    pub hse: Option<Hse>,
    pub sys: Sysclk,

    pub supply_config: SupplyConfig,
}

impl Config {
    pub const fn new() -> Self {
        Self {
            hsi: None,
            hse: None,
            sys: Sysclk::HSI,

            supply_config: SupplyConfig::Smps,
        }
    }
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

fn osc_config(config: Config) -> (Option<Hertz>, Option<Hertz>) {
    let (cpu_clk_src, sys_clk_src) = {
        let cfgr = RCC.cfgr().read();
        (cfgr.cpusws(), cfgr.syssws())
    };
    let pll1_clk_src = RCC.pll1cfgr1().read().pllsel();
    let pll2_clk_src = RCC.pll2cfgr1().read().pllsel();
    let pll3_clk_src = RCC.pll3cfgr1().read().pllsel();
    let pll4_clk_src = RCC.pll4cfgr1().read().pllsel();
    let sr = RCC.sr().read();

    let hsi = match config.hsi {
        None => {
            if (cpu_clk_src == Cpusws::HSI || sys_clk_src == Syssws::HSI)
                || (pll1_clk_src == Pllsel::HSI && sr.pllrdy(0))
                || (pll2_clk_src == Pllsel::HSI && sr.pllrdy(1))
                || (pll3_clk_src == Pllsel::HSI && sr.pllrdy(2))
                || (pll4_clk_src == Pllsel::HSI && sr.pllrdy(3))
            {
                if config.hse.is_none() {
                    panic!("When the HSI is used as CPU or system bus clock source, it is not allowed to be disabled");
                }
            } else {
                // disable the HSI
                RCC.ccr().write(|w| w.set_hsionc(true));
                // wait until HSI is disabled
                while RCC.sr().read().hsirdy() {}
            }

            None
        }
        Some(hsi_config) => {
            RCC.hsicfgr().modify(|w| {
                w.set_hsidiv(hsi_config.pre);
                w.set_hsitrim(hsi_config.calib);
            });
            Some(HSI_FREQ / hsi_config.pre)
        }
    };

    let hse = match config.hse {
        None => {
            if ((cpu_clk_src == Cpusws::HSE || sys_clk_src == Syssws::HSE)
                || (pll1_clk_src == Pllsel::HSE && sr.pllrdy(0))
                || (pll2_clk_src == Pllsel::HSE && sr.pllrdy(1))
                || (pll3_clk_src == Pllsel::HSE && sr.pllrdy(2))
                || (pll4_clk_src == Pllsel::HSE && sr.pllrdy(3)))
                && config.hse.is_none()
            {
                panic!("When the HSE is used as CPU or system bus clock source, it is not allowed to be disabled");
            }

            // hse off
            RCC.csr().modify(|w| w.set_hseons(false));
            RCC.hsecfgr().modify(|w| {
                w.set_hseext(Hseext::ANALOG);
                w.set_hsebyp(false);
            });

            // wait until hse is off
            while RCC.sr().read().hserdy() {}

            None
        }
        Some(hse_config) => {
            match hse_config.mode {
                HseMode::Oscillator => RCC.csr().modify(|w| w.set_hseons(true)),
                HseMode::Bypass => {
                    RCC.hsecfgr().modify(|w| {
                        w.set_hsebyp(true);
                        w.set_hseext(Hseext::ANALOG);
                    });
                    RCC.csr().modify(|w| w.set_hseons(true));
                }
                HseMode::BypassDigital => {
                    RCC.hsecfgr().modify(|w| {
                        w.set_hsebyp(true);
                        w.set_hseext(Hseext::DIGITAL)
                    });
                }
            };

            // wait until the hse is ready
            while !RCC.sr().read().hserdy() {}

            Some(hse_config.freq)
        }
    };

    (hsi, hse)
}

pub(crate) unsafe fn init(config: Config) {
    // system configuration setup
    RCC.apb4hensr().write(|w| w.set_syscfgens(true));
    // delay after RCC peripheral clock enabling
    core::ptr::read_volatile(RCC.apb4hensr().as_ptr());

    let vtor = unsafe {
        let p = cortex_m::Peripherals::steal();
        p.SCB.vtor.read()
    };

    // set default vector table location after reset or standby
    SYSCFG.initsvtorcr().write(|w| w.set_svtor_addr(vtor));
    // read back the value to ensure it is written before deactivating SYSCFG
    core::ptr::read_volatile(SYSCFG.initsvtorcr().as_ptr());

    // deactivate SYSCFG
    RCC.apb4hensr().write(|w| w.set_syscfgens(false));

    // enable fpu
    unsafe {
        let p = cortex_m::Peripherals::steal();
        p.SCB.cpacr.modify(|w| w | (3 << 20) | (3 << 22));
    }

    power_supply_config(config.supply_config);

    let (hsi, hse) = osc_config(config);

    let sys = match config.sys {
        Sysclk::HSE => unwrap!(hse),
        Sysclk::HSI => unwrap!(hsi),
        Sysclk::MSI => todo!(),
        Sysclk::IC2 => todo!(),
    };

    // TODO: sysb, sysc, sysd must have the same clock source

    set_clocks!(
        sys: Some(sys),
        hsi: hsi,
        hsi_div: None,
        hse: hse,
        hclk1: None,
        hclk2: None,
        hclk3: None,
        hclk4: None,
        hclk5: None,
        pclk1: None,
        pclk2: None,
        pclk2_tim: None,
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
