use core::ops::RangeInclusive;

#[cfg(stm32h7rs)]
use stm32_metapac::rcc::vals::Xspisel;

use crate::pac;
#[cfg(stm32h7rs)]
use crate::pac::flash::regs::Optkeyr;
#[cfg(stm32h7rs)]
pub use crate::pac::rcc::vals::Plldivst as PllDivSt;
pub use crate::pac::rcc::vals::{
    Hsidiv as HSIPrescaler, Plldiv as PllDiv, Pllm as PllPreDiv, Plln as PllMul, Pllsrc as PllSource, Sw as Sysclk,
};
use crate::pac::rcc::vals::{Pllrge, Pllvcosel, Timpre};
use crate::pac::{FLASH, PWR, RCC};
#[cfg(dsihost)]
use crate::rcc::dsi;
#[cfg(dsihost)]
pub use crate::rcc::dsi::{DsiHostPllConfig, DsiPllInput, DsiPllNdiv, DsiPllOutput};
use crate::time::Hertz;

/// HSI speed
pub const HSI_FREQ: Hertz = Hertz(64_000_000);

/// CSI speed
pub const CSI_FREQ: Hertz = Hertz(4_000_000);

const VCO_RANGE: RangeInclusive<Hertz> = Hertz(150_000_000)..=Hertz(420_000_000);
#[cfg(any(stm32h5, pwr_h7rm0455))]
const VCO_WIDE_RANGE: RangeInclusive<Hertz> = Hertz(128_000_000)..=Hertz(560_000_000);
#[cfg(pwr_h7rm0468)]
const VCO_WIDE_RANGE: RangeInclusive<Hertz> = Hertz(192_000_000)..=Hertz(836_000_000);
#[cfg(any(pwr_h7rm0399, pwr_h7rm0433))]
const VCO_WIDE_RANGE: RangeInclusive<Hertz> = Hertz(192_000_000)..=Hertz(960_000_000);
#[cfg(any(stm32h7rs))]
const VCO_WIDE_RANGE: RangeInclusive<Hertz> = Hertz(384_000_000)..=Hertz(1672_000_000);

pub use crate::pac::rcc::vals::{Hpre as AHBPrescaler, Ppre as APBPrescaler};

#[cfg(any(stm32h5, stm32h7))]
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum VoltageScale {
    Scale0,
    Scale1,
    Scale2,
    Scale3,
}
#[cfg(stm32h7rs)]
pub use crate::pac::pwr::vals::Vos as VoltageScale;
#[cfg(all(stm32h7rs, peri_usb_otg_hs))]
pub use crate::pac::rcc::vals::{Usbphycsel, Usbrefcksel};

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum HseMode {
    /// crystal/ceramic oscillator (HSEBYP=0)
    Oscillator,
    /// external analog clock (low swing) (HSEBYP=1, HSEEXT=0)
    Bypass,
    /// external digital clock (full swing) (HSEBYP=1, HSEEXT=1)
    #[cfg(any(rcc_h5, rcc_h50, rcc_h7rs))]
    BypassDigital,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Hse {
    /// HSE frequency.
    pub freq: Hertz,
    /// HSE mode.
    pub mode: HseMode,
}

#[derive(Clone, Copy)]
pub struct Pll {
    /// Source clock selection.
    pub source: PllSource,

    /// PLL pre-divider (DIVM).
    pub prediv: PllPreDiv,

    /// PLL multiplication factor.
    pub mul: PllMul,

    #[cfg(any(stm32h743, stm32h730))]
    /// PLL Fractional multiplier.
    pub fracn: Option<u16>,

    /// PLL P division factor. If None, PLL P output is disabled.
    /// On PLL1, it must be even for most series (in particular,
    /// it cannot be 1 in series other than stm32h7, stm32h7rs23/733,
    /// stm32h7, stm32h7rs25/735 and stm32h7, stm32h7rs30.)
    pub divp: Option<PllDiv>,
    /// PLL Q division factor. If None, PLL Q output is disabled.
    pub divq: Option<PllDiv>,
    /// PLL R division factor. If None, PLL R output is disabled.
    pub divr: Option<PllDiv>,
    #[cfg(stm32h7rs)]
    /// PLL S division factor. If None, PLL S output is disabled.
    pub divs: Option<PllDivSt>,
    #[cfg(stm32h7rs)]
    /// PLL T division factor. If None, PLL T output is disabled.
    pub divt: Option<PllDivSt>,
}

fn apb_div_tim(apb: &APBPrescaler, clk: Hertz, tim: TimerPrescaler) -> Hertz {
    match (tim, apb) {
        (TimerPrescaler::DefaultX2, APBPrescaler::Div1) => clk,
        (TimerPrescaler::DefaultX2, APBPrescaler::Div2) => clk,
        (TimerPrescaler::DefaultX2, APBPrescaler::Div4) => clk / 2u32,
        (TimerPrescaler::DefaultX2, APBPrescaler::Div8) => clk / 4u32,
        (TimerPrescaler::DefaultX2, APBPrescaler::Div16) => clk / 8u32,

        (TimerPrescaler::DefaultX4, APBPrescaler::Div1) => clk,
        (TimerPrescaler::DefaultX4, APBPrescaler::Div2) => clk,
        (TimerPrescaler::DefaultX4, APBPrescaler::Div4) => clk,
        (TimerPrescaler::DefaultX4, APBPrescaler::Div8) => clk / 2u32,
        (TimerPrescaler::DefaultX4, APBPrescaler::Div16) => clk / 4u32,

        _ => unreachable!(),
    }
}

/// Timer prescaler
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum TimerPrescaler {
    /// The timers kernel clock is equal to hclk if PPREx corresponds to a
    /// division by 1 or 2, else it is equal to 2*pclk
    DefaultX2,

    /// The timers kernel clock is equal to hclk if PPREx corresponds to a
    /// division by 1, 2 or 4, else it is equal to 4*pclk
    DefaultX4,
}

impl From<TimerPrescaler> for Timpre {
    fn from(value: TimerPrescaler) -> Self {
        match value {
            TimerPrescaler::DefaultX2 => Timpre::DefaultX2,
            TimerPrescaler::DefaultX4 => Timpre::DefaultX4,
        }
    }
}

/// Power supply configuration
/// See RM0433 Rev 4 7.4
#[cfg(any(pwr_h7rm0399, pwr_h7rm0455, pwr_h7rm0468, pwr_h7rs))]
#[derive(Clone, Copy, PartialEq)]
pub enum SupplyConfig {
    /// Default power supply configuration.
    /// V CORE Power Domains are supplied from the LDO according to VOS.
    /// SMPS step-down converter enabled at 1.2V, may be used to supply the LDO.
    Default,

    /// Power supply configuration using the LDO.
    /// V CORE Power Domains are supplied from the LDO according to VOS.
    /// LDO power mode (Main, LP, Off) will follow system low-power modes.
    /// SMPS step-down converter disabled.
    LDO,

    /// Power supply configuration directly from the SMPS step-down converter.
    /// V CORE Power Domains are supplied from SMPS step-down converter according to VOS.
    /// LDO bypassed.
    /// SMPS step-down converter power mode (MR, LP, Off) will follow system low-power modes.
    DirectSMPS,

    /// Power supply configuration from the SMPS step-down converter, that supplies the LDO.
    /// V CORE Power Domains are supplied from the LDO according to VOS
    /// LDO power mode (Main, LP, Off) will follow system low-power modes.
    /// SMPS step-down converter enabled according to SDLEVEL, and supplies the LDO.
    /// SMPS step-down converter power mode (MR, LP, Off) will follow system low-power modes.
    SMPSLDO(SMPSSupplyVoltage),

    /// Power supply configuration from SMPS supplying external circuits and potentially the LDO.
    /// V CORE Power Domains are supplied from voltage regulator according to VOS
    /// LDO power mode (Main, LP, Off) will follow system low-power modes.
    /// SMPS step-down converter enabled according to SDLEVEL used to supply external circuits and may supply the LDO.
    /// SMPS step-down converter forced ON in MR mode.
    SMPSExternalLDO(SMPSSupplyVoltage),

    /// Power supply configuration from SMPS supplying external circuits and bypassing the LDO.
    /// V CORE supplied from external source
    /// SMPS step-down converter enabled according to SDLEVEL used to supply external circuits and may supply the external source for V CORE .
    /// SMPS step-down converter forced ON in MR mode.
    SMPSExternalLDOBypass(SMPSSupplyVoltage),

    /// Power supply configuration from an external source, SMPS disabled and the LDO bypassed.
    /// V CORE supplied from external source
    /// SMPS step-down converter disabled and LDO bypassed, voltage monitoring still active.
    SMPSDisabledLDOBypass,
}

/// SMPS step-down converter voltage output level.
/// This is only used in certain power supply configurations:
/// SMPSLDO, SMPSExternalLDO, SMPSExternalLDOBypass.
#[cfg(any(pwr_h7rm0399, pwr_h7rm0455, pwr_h7rm0468, pwr_h7rs))]
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum SMPSSupplyVoltage {
    /// 1.8v
    V1_8,
    /// 2.5v
    #[cfg(not(pwr_h7rs))]
    V2_5,
}

/// Configuration of the core clocks
#[non_exhaustive]
#[derive(Clone, Copy)]
pub struct Config {
    pub hsi: Option<HSIPrescaler>,
    pub hse: Option<Hse>,
    pub csi: bool,
    pub hsi48: Option<super::Hsi48Config>,
    pub sys: Sysclk,

    pub pll1: Option<Pll>,
    pub pll2: Option<Pll>,
    #[cfg(any(rcc_h5, stm32h7, stm32h7rs))]
    pub pll3: Option<Pll>,

    #[cfg(any(stm32h7, stm32h7rs))]
    pub d1c_pre: AHBPrescaler,
    pub ahb_pre: AHBPrescaler,
    pub apb1_pre: APBPrescaler,
    pub apb2_pre: APBPrescaler,
    #[cfg(not(stm32h7rs))]
    pub apb3_pre: APBPrescaler,
    #[cfg(any(stm32h7, stm32h7rs))]
    pub apb4_pre: APBPrescaler,
    #[cfg(stm32h7rs)]
    pub apb5_pre: APBPrescaler,

    #[cfg(dsihost)]
    pub dsi: Option<DsiHostPllConfig>,

    pub timer_prescaler: TimerPrescaler,
    pub voltage_scale: VoltageScale,
    pub ls: super::LsConfig,

    #[cfg(any(pwr_h7rm0399, pwr_h7rm0455, pwr_h7rm0468, pwr_h7rs))]
    pub supply_config: SupplyConfig,

    /// Per-peripheral kernel clock selection muxes
    pub mux: super::mux::ClockMux,

    /// Enable HSLV mode for XSPI1.
    /// CAUTION: enabling when VDD_XSPI1 > 2.7 V may be destructive!
    #[cfg(stm32h7rs)]
    pub hslv_xspi1: bool,
    /// Enable HSLV mode for XSPI2.
    /// CAUTION: enabling when VDD_XSPI2 > 2.7 V may be destructive!
    #[cfg(stm32h7rs)]
    pub hslv_xspi2: bool,
    /// Enable HSLV mode for I/O pins.
    /// CAUTION: enabling when VDD > 2.7 V may be destructive!
    #[cfg(stm32h7rs)]
    pub hslv_io: bool,

    /// Enable the compensation cell for XSPI1.
    /// Enabling with no active device connected will fail with time-out.
    #[cfg(stm32h7rs)]
    pub comp_xspi1: bool,
    /// Enable the compensation cell for XSPI2.
    /// Enabling with no active device connected will fail with time-out.
    #[cfg(stm32h7rs)]
    pub comp_xspi2: bool,
}

impl Config {
    pub const fn new() -> Self {
        Self {
            hsi: Some(HSIPrescaler::Div1),
            hse: None,
            csi: false,
            hsi48: Some(crate::rcc::Hsi48Config::new()),
            sys: Sysclk::Hsi,
            pll1: None,
            pll2: None,
            #[cfg(any(rcc_h5, stm32h7, stm32h7rs))]
            pll3: None,

            #[cfg(any(stm32h7, stm32h7rs))]
            d1c_pre: AHBPrescaler::Div1,
            ahb_pre: AHBPrescaler::Div1,
            apb1_pre: APBPrescaler::Div1,
            apb2_pre: APBPrescaler::Div1,
            #[cfg(not(stm32h7rs))]
            apb3_pre: APBPrescaler::Div1,
            #[cfg(any(stm32h7, stm32h7rs))]
            apb4_pre: APBPrescaler::Div1,
            #[cfg(stm32h7rs)]
            apb5_pre: APBPrescaler::Div1,

            #[cfg(dsihost)]
            dsi: None,

            timer_prescaler: TimerPrescaler::DefaultX2,
            #[cfg(not(rcc_h7rs))]
            voltage_scale: VoltageScale::Scale0,
            #[cfg(rcc_h7rs)]
            voltage_scale: VoltageScale::High,
            ls: crate::rcc::LsConfig::new(),

            #[cfg(any(pwr_h7rm0399, pwr_h7rm0455, pwr_h7rm0468, pwr_h7rs))]
            supply_config: SupplyConfig::LDO,

            mux: super::mux::ClockMux::default(),

            #[cfg(stm32h7rs)]
            hslv_xspi1: false,
            #[cfg(stm32h7rs)]
            hslv_xspi2: false,
            #[cfg(stm32h7rs)]
            hslv_io: false,

            #[cfg(stm32h7rs)]
            comp_xspi1: false,
            #[cfg(stm32h7rs)]
            comp_xspi2: false,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

pub(crate) unsafe fn init(config: Config) {
    #[cfg(any(stm32h7))]
    let pwr_reg = PWR.cr3();
    #[cfg(any(stm32h7rs))]
    let pwr_reg = PWR.csr2();

    // NB. The lower bytes of CR3 can only be written once after
    // POR, and must be written with a valid combination. Refer to
    // RM0433 Rev 7 6.8.4. This is partially enforced by dropping
    // `self` at the end of this method, but of course we cannot
    // know what happened between the previous POR and here.
    #[cfg(pwr_h7rm0433)]
    pwr_reg.modify(|w| {
        w.set_scuen(true);
        w.set_ldoen(true);
        w.set_bypass(false);
    });

    #[cfg(any(pwr_h7rm0399, pwr_h7rm0455, pwr_h7rm0468, pwr_h7rs))]
    {
        use pac::pwr::vals::Sdlevel;
        match config.supply_config {
            SupplyConfig::Default => {
                pwr_reg.modify(|w| {
                    w.set_sdlevel(Sdlevel::Reset);
                    w.set_sdexthp(false);
                    w.set_sden(true);
                    w.set_ldoen(true);
                    w.set_bypass(false);
                });
            }
            SupplyConfig::LDO => {
                pwr_reg.modify(|w| {
                    w.set_sden(false);
                    w.set_ldoen(true);
                    w.set_bypass(false);
                });
            }
            SupplyConfig::DirectSMPS => {
                pwr_reg.modify(|w| {
                    w.set_sdexthp(false);
                    w.set_sden(true);
                    w.set_ldoen(false);
                    w.set_bypass(false);
                });
            }
            SupplyConfig::SMPSLDO(sdlevel)
            | SupplyConfig::SMPSExternalLDO(sdlevel)
            | SupplyConfig::SMPSExternalLDOBypass(sdlevel) => {
                let sdlevel = match sdlevel {
                    SMPSSupplyVoltage::V1_8 => Sdlevel::V18,
                    #[cfg(not(pwr_h7rs))]
                    SMPSSupplyVoltage::V2_5 => Sdlevel::V25,
                };
                pwr_reg.modify(|w| {
                    w.set_sdlevel(sdlevel);
                    w.set_sdexthp(matches!(
                        config.supply_config,
                        SupplyConfig::SMPSExternalLDO(_) | SupplyConfig::SMPSExternalLDOBypass(_)
                    ));
                    w.set_sden(true);
                    w.set_ldoen(matches!(
                        config.supply_config,
                        SupplyConfig::SMPSLDO(_) | SupplyConfig::SMPSExternalLDO(_)
                    ));
                    w.set_bypass(matches!(config.supply_config, SupplyConfig::SMPSExternalLDOBypass(_)));
                });
            }
            SupplyConfig::SMPSDisabledLDOBypass => {
                pwr_reg.modify(|w| {
                    w.set_sden(false);
                    w.set_ldoen(false);
                    w.set_bypass(true);
                });
            }
        }
    }

    // Validate the supply configuration. If you are stuck here, it is
    // because the voltages on your board do not match those specified
    // in the D3CR.VOS and CR3.SDLEVEL fields. By default after reset
    // VOS = Scale 3, so check that the voltage on the VCAP pins =
    // 1.0V.
    #[cfg(any(stm32h7))]
    while !PWR.csr1().read().actvosrdy() {}
    #[cfg(any(stm32h7rs))]
    while !PWR.sr1().read().actvosrdy() {}

    // Configure voltage scale.
    #[cfg(any(pwr_h5, pwr_h50))]
    {
        PWR.voscr().modify(|w| {
            w.set_vos(match config.voltage_scale {
                VoltageScale::Scale0 => crate::pac::pwr::vals::Vos::Scale0,
                VoltageScale::Scale1 => crate::pac::pwr::vals::Vos::Scale1,
                VoltageScale::Scale2 => crate::pac::pwr::vals::Vos::Scale2,
                VoltageScale::Scale3 => crate::pac::pwr::vals::Vos::Scale3,
            })
        });
        while !PWR.vossr().read().vosrdy() {}
    }
    #[cfg(syscfg_h7)]
    {
        // in chips without the overdrive bit, we can go from any scale to any scale directly.
        PWR.d3cr().modify(|w| {
            w.set_vos(match config.voltage_scale {
                VoltageScale::Scale0 => crate::pac::pwr::vals::Vos::Scale0,
                VoltageScale::Scale1 => crate::pac::pwr::vals::Vos::Scale1,
                VoltageScale::Scale2 => crate::pac::pwr::vals::Vos::Scale2,
                VoltageScale::Scale3 => crate::pac::pwr::vals::Vos::Scale3,
            })
        });
        while !PWR.d3cr().read().vosrdy() {}
    }
    #[cfg(pwr_h7rs)]
    {
        PWR.csr4().modify(|w| w.set_vos(config.voltage_scale));
        while !PWR.csr4().read().vosrdy() {}
    }

    #[cfg(syscfg_h7od)]
    {
        match config.voltage_scale {
            VoltageScale::Scale0 => {
                // to go to scale0, we must go to Scale1 first...
                PWR.d3cr().modify(|w| w.set_vos(crate::pac::pwr::vals::Vos::Scale1));
                while !PWR.d3cr().read().vosrdy() {}

                // Then enable overdrive.
                critical_section::with(|_| pac::SYSCFG.pwrcr().modify(|w| w.set_oden(1)));
                while !PWR.d3cr().read().vosrdy() {}
            }
            _ => {
                // for all other scales, we can go directly.
                PWR.d3cr().modify(|w| {
                    w.set_vos(match config.voltage_scale {
                        VoltageScale::Scale0 => unreachable!(),
                        VoltageScale::Scale1 => crate::pac::pwr::vals::Vos::Scale1,
                        VoltageScale::Scale2 => crate::pac::pwr::vals::Vos::Scale2,
                        VoltageScale::Scale3 => crate::pac::pwr::vals::Vos::Scale3,
                    })
                });
                while !PWR.d3cr().read().vosrdy() {}
            }
        }
    }

    // Turn on the HSI
    match config.hsi {
        None => RCC.cr().modify(|w| w.set_hsion(true)),
        Some(hsidiv) => RCC.cr().modify(|w| {
            w.set_hsidiv(hsidiv);
            w.set_hsion(true);
        }),
    }
    while !RCC.cr().read().hsirdy() {}

    #[cfg(stm32h7rs)]
    {
        // Switch the XSPI clock source so it will use HSI
        RCC.ahbperckselr().modify(|w| w.set_xspi1sel(Xspisel::Hclk5));
        RCC.ahbperckselr().modify(|w| w.set_xspi2sel(Xspisel::Hclk5));
    };

    // Use the HSI clock as system clock during the actual clock setup
    RCC.cfgr().modify(|w| w.set_sw(Sysclk::Hsi));
    while RCC.cfgr().read().sws() != Sysclk::Hsi {}

    // Configure HSI
    let hsi = match config.hsi {
        None => None,
        Some(hsidiv) => Some(HSI_FREQ / hsidiv),
    };

    // Configure HSE
    let hse = match config.hse {
        None => {
            RCC.cr().modify(|w| w.set_hseon(false));
            None
        }
        Some(hse) => {
            RCC.cr().modify(|w| {
                w.set_hsebyp(hse.mode != HseMode::Oscillator);
                #[cfg(any(rcc_h5, rcc_h50, rcc_h7rs))]
                w.set_hseext(match hse.mode {
                    HseMode::Oscillator | HseMode::Bypass => pac::rcc::vals::Hseext::Analog,
                    HseMode::BypassDigital => pac::rcc::vals::Hseext::Digital,
                });
            });
            RCC.cr().modify(|w| w.set_hseon(true));
            while !RCC.cr().read().hserdy() {}
            Some(hse.freq)
        }
    };

    // Configure HSI48.
    let hsi48 = config.hsi48.map(super::init_hsi48);

    // Configure CSI.
    RCC.cr().modify(|w| w.set_csion(config.csi));
    let csi = match config.csi {
        false => None,
        true => {
            while !RCC.cr().read().csirdy() {}
            Some(CSI_FREQ)
        }
    };

    // H7 has shared PLLSRC, check it's equal in all PLLs.
    #[cfg(any(stm32h7, stm32h7rs))]
    {
        let plls = [&config.pll1, &config.pll2, &config.pll3];
        if !super::util::all_equal(plls.into_iter().flatten().map(|p| p.source)) {
            panic!("Source must be equal across all enabled PLLs.")
        };
    }

    // Configure PLLs.
    let pll_input = PllInput { csi, hse, hsi };
    let pll1 = config.pll1.map_or_else(
        || {
            disable_pll(0);
            PllOutput::default()
        },
        |c| init_pll(0, Some(c), &pll_input),
    );
    let pll2 = config.pll2.map_or_else(
        || {
            disable_pll(1);
            PllOutput::default()
        },
        |c| init_pll(1, Some(c), &pll_input),
    );
    #[cfg(any(rcc_h5, stm32h7, stm32h7rs))]
    let pll3 = config.pll3.map_or_else(
        || {
            disable_pll(2);
            PllOutput::default()
        },
        |c| init_pll(2, Some(c), &pll_input),
    );

    // Configure sysclk
    let sys = match config.sys {
        Sysclk::Hsi => unwrap!(hsi),
        Sysclk::Hse => unwrap!(hse),
        Sysclk::Csi => unwrap!(csi),
        Sysclk::Pll1P => unwrap!(pll1.p),
        _ => unreachable!(),
    };

    // Check limits.
    #[cfg(stm32h5)]
    let (hclk_max, pclk_max) = match config.voltage_scale {
        VoltageScale::Scale0 => (Hertz(250_000_000), Hertz(250_000_000)),
        VoltageScale::Scale1 => (Hertz(200_000_000), Hertz(200_000_000)),
        VoltageScale::Scale2 => (Hertz(150_000_000), Hertz(150_000_000)),
        VoltageScale::Scale3 => (Hertz(100_000_000), Hertz(100_000_000)),
    };
    #[cfg(pwr_h7rm0455)]
    let (d1cpre_clk_max, hclk_max, pclk_max) = match config.voltage_scale {
        VoltageScale::Scale0 => (Hertz(280_000_000), Hertz(280_000_000), Hertz(140_000_000)),
        VoltageScale::Scale1 => (Hertz(225_000_000), Hertz(225_000_000), Hertz(112_500_000)),
        VoltageScale::Scale2 => (Hertz(160_000_000), Hertz(160_000_000), Hertz(80_000_000)),
        VoltageScale::Scale3 => (Hertz(88_000_000), Hertz(88_000_000), Hertz(44_000_000)),
    };
    #[cfg(pwr_h7rm0468)]
    let (d1cpre_clk_max, hclk_max, pclk_max) = match config.voltage_scale {
        VoltageScale::Scale0 => {
            let d1cpre_clk_max = if pac::SYSCFG.ur18().read().cpu_freq_boost() {
                550_000_000
            } else {
                520_000_000
            };
            (Hertz(d1cpre_clk_max), Hertz(275_000_000), Hertz(137_500_000))
        }
        VoltageScale::Scale1 => (Hertz(400_000_000), Hertz(200_000_000), Hertz(100_000_000)),
        VoltageScale::Scale2 => (Hertz(300_000_000), Hertz(150_000_000), Hertz(75_000_000)),
        VoltageScale::Scale3 => (Hertz(170_000_000), Hertz(85_000_000), Hertz(42_500_000)),
    };
    #[cfg(all(stm32h7, not(any(pwr_h7rm0455, pwr_h7rm0468))))]
    let (d1cpre_clk_max, hclk_max, pclk_max) = match config.voltage_scale {
        VoltageScale::Scale0 => (Hertz(480_000_000), Hertz(240_000_000), Hertz(120_000_000)),
        VoltageScale::Scale1 => (Hertz(400_000_000), Hertz(200_000_000), Hertz(100_000_000)),
        VoltageScale::Scale2 => (Hertz(300_000_000), Hertz(150_000_000), Hertz(75_000_000)),
        VoltageScale::Scale3 => (Hertz(200_000_000), Hertz(100_000_000), Hertz(50_000_000)),
    };
    #[cfg(stm32h7rs)]
    let (d1cpre_clk_max, hclk_max, pclk_max) = match config.voltage_scale {
        VoltageScale::High => (Hertz(600_000_000), Hertz(300_000_000), Hertz(150_000_000)),
        VoltageScale::Low => (Hertz(400_000_000), Hertz(200_000_000), Hertz(100_000_000)),
    };

    #[cfg(any(stm32h7, stm32h7rs))]
    let hclk = {
        let d1cpre_clk = sys / config.d1c_pre;
        assert!(d1cpre_clk <= d1cpre_clk_max);
        d1cpre_clk / config.ahb_pre
    };
    #[cfg(stm32h5)]
    let hclk = sys / config.ahb_pre;
    assert!(hclk <= hclk_max);

    let apb1 = hclk / config.apb1_pre;
    let apb1_tim = apb_div_tim(&config.apb1_pre, hclk, config.timer_prescaler);
    assert!(apb1 <= pclk_max);
    let apb2 = hclk / config.apb2_pre;
    let apb2_tim = apb_div_tim(&config.apb2_pre, hclk, config.timer_prescaler);
    assert!(apb2 <= pclk_max);
    #[cfg(not(stm32h7rs))]
    let apb3 = hclk / config.apb3_pre;
    #[cfg(not(stm32h7rs))]
    assert!(apb3 <= pclk_max);
    #[cfg(any(stm32h7, stm32h7rs))]
    let apb4 = hclk / config.apb4_pre;
    #[cfg(any(stm32h7, stm32h7rs))]
    assert!(apb4 <= pclk_max);
    #[cfg(stm32h7rs)]
    let apb5 = hclk / config.apb5_pre;
    #[cfg(stm32h7rs)]
    assert!(apb5 <= pclk_max);

    flash_setup(hclk, config.voltage_scale);

    let rtc = config.ls.init();

    #[cfg(all(stm32h7rs, peri_usb_otg_hs))]
    let usb_refck = match config.mux.usbphycsel {
        Usbphycsel::Hse => hse,
        Usbphycsel::HseDiv2 => hse.map(|hse_val| hse_val / 2u8),
        Usbphycsel::Pll3Q => pll3.q,
        _ => None,
    };
    #[cfg(all(stm32h7rs, peri_usb_otg_hs))]
    let usb_refck_sel = match usb_refck {
        Some(clk_val) => match clk_val {
            Hertz(16_000_000) => Usbrefcksel::Mhz16,
            Hertz(19_200_000) => Usbrefcksel::Mhz192,
            Hertz(20_000_000) => Usbrefcksel::Mhz20,
            Hertz(24_000_000) => Usbrefcksel::Mhz24,
            Hertz(26_000_000) => Usbrefcksel::Mhz26,
            Hertz(32_000_000) => Usbrefcksel::Mhz32,
            _ => panic!(
                "cannot select USBPHYC reference clock with source frequency of {}, must be one of 16, 19.2, 20, 24, 26, 32 MHz",
                clk_val
            ),
        },
        None => Usbrefcksel::Mhz24,
    };

    #[cfg(stm32h7)]
    {
        RCC.d1cfgr().modify(|w| {
            w.set_d1cpre(config.d1c_pre);
            w.set_d1ppre(config.apb3_pre);
            w.set_hpre(config.ahb_pre);
        });
        // Ensure core prescaler value is valid before future lower core voltage
        while RCC.d1cfgr().read().d1cpre() != config.d1c_pre {}

        RCC.d2cfgr().modify(|w| {
            w.set_d2ppre1(config.apb1_pre);
            w.set_d2ppre2(config.apb2_pre);
        });
        RCC.d3cfgr().modify(|w| {
            w.set_d3ppre(config.apb4_pre);
        });
    }
    #[cfg(stm32h7rs)]
    {
        RCC.cdcfgr().write(|w| {
            w.set_cpre(config.d1c_pre);
        });
        while RCC.cdcfgr().read().cpre() != config.d1c_pre {}

        RCC.bmcfgr().write(|w| {
            w.set_bmpre(config.ahb_pre);
        });
        while RCC.bmcfgr().read().bmpre() != config.ahb_pre {}

        RCC.apbcfgr().modify(|w| {
            w.set_ppre1(config.apb1_pre);
            w.set_ppre2(config.apb2_pre);
            w.set_ppre4(config.apb4_pre);
            w.set_ppre5(config.apb5_pre);
        });

        #[cfg(peri_usb_otg_hs)]
        RCC.ahbperckselr().modify(|w| {
            w.set_usbrefcksel(usb_refck_sel);
        });
    }
    #[cfg(stm32h5)]
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

    RCC.cfgr().modify(|w| w.set_timpre(config.timer_prescaler.into()));

    RCC.cfgr().modify(|w| w.set_sw(config.sys));
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

    // IO compensation cell(s) - Requires CSI clock and SYSCFG
    #[cfg(any(stm32h7))] // TODO h5
    if csi.is_some() {
        // Enable the compensation cell, using back-bias voltage code
        // provide by the cell.
        critical_section::with(|_| {
            pac::SYSCFG.cccsr().modify(|w| {
                w.set_en(true);
                w.set_cs(false);
                w.set_hslv(false);
            })
        });
        while !pac::SYSCFG.cccsr().read().rdy() {}
    }
    #[cfg(any(stm32h7rs))]
    if csi.is_some() {
        // The CSI oscillator must be enabled for this to work.
        // (RM0477, p.362, Ch. 7.5.2 "Oscillators description".)

        // SBS peripheral clock is required by the compensation cells.
        RCC.apb4enr().modify(|w| w.set_syscfgen(true));

        // Unlock flash OPTCR, when required, to be able to configure HSLV
        // (high-speed, low-voltage) mode for the required I/O domains.
        // The original state of the lock bits will be restored when done.
        // Note: this also avoids double-unlocking, which is not permitted.
        // (RM0477, p.581, Ch. 10.3.16: High-speed low-voltage mode)
        let original_lock_state = FLASH.optcr().read().optlock();
        let original_opt_pg_state = FLASH.optcr().read().pg_opt();
        if original_lock_state {
            FLASH.optkeyr().write_value(Optkeyr(0x08192A3B));
            FLASH.optkeyr().write_value(Optkeyr(0x4C5D6E7F));
        }
        if !original_opt_pg_state {
            // Enable write access to the option bits.
            FLASH.optcr().modify(|w| w.set_pg_opt(true));
            while FLASH.sr().read().busy() {}
        }

        // Set the HSLV option bits to enable HSLV mode configuratrion for all
        // three domains. (RM0477, p.272, 5.9.33 "FLASH option byte word 1
        // status register programming")
        FLASH.obw1srp().modify(|w| {
            w.set_octo1_hslv(true);
            w.set_octo2_hslv(true);
            w.set_vddio_hslv(true);
        });
        while FLASH.sr().read().busy() {}

        // Restore the original state of the optlock and pg_opt bits. This is
        // effectively a no-op if no state change was originally required.
        FLASH.optcr().modify(|w| {
            w.set_pg_opt(original_opt_pg_state);
            w.set_optlock(original_lock_state);
        });
        while FLASH.sr().read().busy() {}

        // Configure the HSLV mode domains and I/O compensation cells.
        pac::SYSCFG.cccsr().modify(|w| {
            w.set_octo1_iohslv(config.hslv_xspi1);
            w.set_octo2_iohslv(config.hslv_xspi2);
            w.set_iohslv(config.hslv_io);

            // Enable the compensation cells, using automatic compensation at
            // first. Note that an errata applies, which is handled below.
            w.set_octo1_comp_codesel(false);
            w.set_octo2_comp_codesel(false);
            w.set_comp_codesel(false);

            w.set_octo1_comp_en(config.comp_xspi1);
            w.set_octo2_comp_en(config.comp_xspi2);
            w.set_comp_en(true);
        });

        // Wait for the compensation cells to stabilize.
        // Note: this may never happen for XSPI ports that have no active
        //       device connected to them. A basic polling time-out is added
        //       as fall-back, with a diagnostic message.
        const COMP_POLL_MAX: u32 = 10000; // Enough for maxed-out clocks.
        let mut poll_cnt = 0;
        while config.comp_xspi1 && !pac::SYSCFG.cccsr().read().octo1_comp_rdy() && poll_cnt < COMP_POLL_MAX {
            poll_cnt += 1;
        }
        if poll_cnt == COMP_POLL_MAX {
            warn!("XSPI1 compensation cell ready-flag time-out.");
        }
        poll_cnt = 0;
        while config.comp_xspi2 && !pac::SYSCFG.cccsr().read().octo2_comp_rdy() && poll_cnt < COMP_POLL_MAX {
            poll_cnt += 1
        }
        if poll_cnt == COMP_POLL_MAX {
            warn!("XSPI2 compensation cell ready-flag time-out.");
        }
        while !pac::SYSCFG.cccsr().read().comp_rdy() {}

        // Now the I/O compensation and HSLV mode are configured, there are
        // still issues with the automatic tuning of the I/O compensation, when
        // operating over a wider temperature range. A work-around is required,
        // as explained in the errata:
        // - Read the auto-tune values at around 30°C ambient and store in non-
        //   volatile storage. The values should have a correction applied.
        // - Apply these stored, correct values at boot.
        //
        // The fix is applied here, assuming a sufficiently-constant ambient
        // temperature, but the user should still do the calibration step and
        // apply the correct values, before attempting high-speed peripheral
        // access in real-world applications.
        //
        // (ES0596, p. 12, Ch 2.2.15 "I/O compensation could alter duty-cycle of high-frequency output signal")
        // <https://community.st.com/t5/stm32-mcus-products/stm32h7s7l8h6h-xspi-instability/td-p/749315>
        //
        // Note: applying the errata to the GPIO compensation cell, as is done
        //       here, seems to improve this as well, judging by a signal on
        //       the MCO output pin, although it is not explicitly stated in
        //       the errata.
        let ccv = get_corrected_comp_vals();
        set_and_enable_comp_vals(&ccv);
    }

    config.mux.init();

    set_clocks!(
        sys: Some(sys),
        hclk1: Some(hclk),
        hclk2: Some(hclk),
        hclk3: Some(hclk),
        hclk4: Some(hclk),
        #[cfg(stm32h7rs)]
        hclk5: Some(hclk),
        pclk1: Some(apb1),
        pclk2: Some(apb2),
        #[cfg(not(stm32h7rs))]
        pclk3: Some(apb3),
        #[cfg(any(stm32h7, stm32h7rs))]
        pclk4: Some(apb4),
        #[cfg(stm32h7rs)]
        pclk5: Some(apb5),

        pclk1_tim: Some(apb1_tim),
        pclk2_tim: Some(apb2_tim),
        rtc: rtc,

        hsi: hsi,
        hsi48: hsi48,
        csi: csi,
        hse: hse,

        lse: None,
        lsi: None,

        pll1_q: pll1.q,
        pll2_p: pll2.p,
        pll2_q: pll2.q,
        pll2_r: pll2.r,
        #[cfg(stm32h7rs)]
        pll2_s: pll2.s,
        #[cfg(stm32h7rs)]
        pll2_t: pll2.t,
        #[cfg(any(rcc_h5, stm32h7, stm32h7rs))]
        pll3_p: pll3.p,
        #[cfg(any(rcc_h5, stm32h7, stm32h7rs))]
        pll3_q: pll3.q,
        #[cfg(any(rcc_h5, stm32h7, stm32h7rs))]
        pll3_r: pll3.r,

        #[cfg(rcc_h50)]
        pll3_p: None,
        #[cfg(rcc_h50)]
        pll3_q: None,
        #[cfg(rcc_h50)]
        pll3_r: None,

        #[cfg(dsihost)]
        dsi_phy: config.dsi.map(|config| dsi::configure_pll(hse, config)),

        #[cfg(stm32h5)]
        audioclk: None,
        i2s_ckin: None,
        #[cfg(stm32h7rs)]
        spdifrx_symb: None, // TODO
        #[cfg(stm32h7rs)]
        clk48mohci: None, // TODO
        #[cfg(stm32h7rs)]
        usb: Some(Hertz(48_000_000)),
        #[cfg(stm32h5)]
        hse_div_rtcpre: None, // TODO
    );
}

struct PllInput {
    hsi: Option<Hertz>,
    hse: Option<Hertz>,
    csi: Option<Hertz>,
}

#[derive(Default)]
struct PllOutput {
    p: Option<Hertz>,
    #[allow(dead_code)]
    q: Option<Hertz>,
    #[allow(dead_code)]
    r: Option<Hertz>,
    #[cfg(stm32h7rs)]
    #[allow(dead_code)]
    s: Option<Hertz>,
    #[cfg(stm32h7rs)]
    #[allow(dead_code)]
    t: Option<Hertz>,
}

fn disable_pll(num: usize) {
    // Stop PLL
    RCC.cr().modify(|w| w.set_pllon(num, false));
    while RCC.cr().read().pllrdy(num) {}

    // "To save power when PLL1 is not used, the value of PLL1M must be set to 0.""
    #[cfg(any(stm32h7, stm32h7rs))]
    RCC.pllckselr().write(|w| w.set_divm(num, PllPreDiv::from_bits(0)));
    #[cfg(stm32h5)]
    RCC.pllcfgr(num).write(|w| w.set_divm(PllPreDiv::from_bits(0)));
}

fn init_pll(num: usize, config: Option<Pll>, input: &PllInput) -> PllOutput {
    let Some(config) = config else {
        disable_pll(num);

        return PllOutput::default();
    };

    let in_clk = match config.source {
        PllSource::Disable => panic!("must not set PllSource::Disable"),
        PllSource::Hsi => unwrap!(input.hsi),
        PllSource::Hse => unwrap!(input.hse),
        PllSource::Csi => unwrap!(input.csi),
    };

    let ref_clk = in_clk / config.prediv as u32;

    let ref_range = match ref_clk.0 {
        ..=1_999_999 => Pllrge::Range1,
        ..=3_999_999 => Pllrge::Range2,
        ..=7_999_999 => Pllrge::Range4,
        ..=16_000_000 => Pllrge::Range8,
        x => panic!("pll ref_clk out of range: {} hz", x),
    };

    // The smaller range (150 to 420 MHz) must
    // be chosen when the reference clock frequency is lower than 2 MHz.
    let wide_allowed = ref_range != Pllrge::Range1;

    #[cfg(any(stm32h743, stm32h730))]
    let vco_clk = match config.fracn {
        Some(fracn) => {
            Hertz::hz((ref_clk.0 as f32 * ((config.mul.to_bits() + 1) as f32 + (fracn as f32 / 8192.0))) as u32)
        }
        None => ref_clk * config.mul,
    };
    #[cfg(not(any(stm32h743, stm32h730)))]
    let vco_clk = ref_clk * config.mul;

    let vco_range = if VCO_RANGE.contains(&vco_clk) {
        Pllvcosel::MediumVco
    } else if wide_allowed && VCO_WIDE_RANGE.contains(&vco_clk) {
        Pllvcosel::WideVco
    } else {
        panic!("pll vco_clk out of range: {}", vco_clk)
    };

    let p = config.divp.map(|div| {
        if num == 0 {
            // on PLL1, DIVP must be even for most series.
            // The enum value is 1 less than the divider, so check it's odd.
            #[cfg(not(any(pwr_h7rm0468, stm32h7rs)))]
            assert!(div.to_bits() % 2 == 1);
            #[cfg(pwr_h7rm0468)]
            assert!(div.to_bits() % 2 == 1 || div.to_bits() == 0);
        }

        vco_clk / div
    });
    let q = config.divq.map(|div| vco_clk / div);
    let r = config.divr.map(|div| vco_clk / div);
    #[cfg(stm32h7rs)]
    let s = config.divs.map(|div| vco_clk / div);
    #[cfg(stm32h7rs)]
    let t = config.divt.map(|div| vco_clk / div);

    #[cfg(stm32h5)]
    RCC.pllcfgr(num).write(|w| {
        w.set_pllsrc(config.source);
        w.set_divm(config.prediv);
        w.set_pllvcosel(vco_range);
        w.set_pllrge(ref_range);
        w.set_pllfracen(false);
        w.set_pllpen(p.is_some());
        w.set_pllqen(q.is_some());
        w.set_pllren(r.is_some());
    });

    #[cfg(any(stm32h7, stm32h7rs))]
    {
        RCC.pllckselr().modify(|w| {
            w.set_divm(num, config.prediv);
            w.set_pllsrc(config.source);
        });

        #[cfg(any(stm32h743, stm32h730))]
        if let Some(fracn) = config.fracn {
            RCC.pllfracr(num).modify(|w| w.set_fracn(fracn))
        }

        RCC.pllcfgr().modify(|w| {
            w.set_pllvcosel(num, vco_range);
            w.set_pllrge(num, ref_range);

            #[cfg(any(stm32h743, stm32h730))]
            if config.fracn.is_some() {
                w.set_pllfracen(num, true);
            } else {
                w.set_pllfracen(num, false);
            }

            #[cfg(not(any(stm32h743, stm32h730)))]
            w.set_pllfracen(num, false);

            w.set_divpen(num, p.is_some());
            w.set_divqen(num, q.is_some());
            w.set_divren(num, r.is_some());
            #[cfg(stm32h7rs)]
            w.set_divsen(num, s.is_some());
            #[cfg(stm32h7rs)]
            w.set_divten(num, t.is_some());
        });
    }

    RCC.plldivr(num).write(|w| {
        w.set_plln(config.mul);
        w.set_pllp(config.divp.unwrap_or(PllDiv::Div2));
        w.set_pllq(config.divq.unwrap_or(PllDiv::Div2));
        w.set_pllr(config.divr.unwrap_or(PllDiv::Div2));
    });

    #[cfg(stm32h7rs)]
    RCC.plldivr2(num).write(|w| {
        w.set_plls(config.divs.unwrap_or(PllDivSt::Div2));
        w.set_pllt(config.divt.unwrap_or(PllDivSt::Div2));
    });

    RCC.cr().modify(|w| w.set_pllon(num, true));
    while !RCC.cr().read().pllrdy(num) {}

    PllOutput {
        p,
        q,
        r,
        #[cfg(stm32h7rs)]
        s,
        #[cfg(stm32h7rs)]
        t,
    }
}

fn flash_setup(clk: Hertz, vos: VoltageScale) {
    // RM0481 Rev 1, table 37
    // LATENCY  WRHIGHFREQ  VOS3           VOS2            VOS1            VOS0
    //      0           0   0 to 20 MHz    0 to 30 MHz     0 to 34 MHz     0 to 42 MHz
    //      1           0   20 to 40 MHz   30 to 60 MHz    34 to 68 MHz    42 to 84 MHz
    //      2           1   40 to 60 MHz   60 to 90 MHz    68 to 102 MHz   84 to 126 MHz
    //      3           1   60 to 80 MHz   90 to 120 MHz   102 to 136 MHz  126 to 168 MHz
    //      4           2   80 to 100 MHz  120 to 150 MHz  136 to 170 MHz  168 to 210 MHz
    //      5           2                                  170 to 200 MHz  210 to 250 MHz
    #[cfg(stm32h5)]
    let (latency, wrhighfreq) = match (vos, clk.0) {
        (VoltageScale::Scale0, ..=42_000_000) => (0, 0),
        (VoltageScale::Scale0, ..=84_000_000) => (1, 0),
        (VoltageScale::Scale0, ..=126_000_000) => (2, 1),
        (VoltageScale::Scale0, ..=168_000_000) => (3, 1),
        (VoltageScale::Scale0, ..=210_000_000) => (4, 2),
        (VoltageScale::Scale0, ..=250_000_000) => (5, 2),

        (VoltageScale::Scale1, ..=34_000_000) => (0, 0),
        (VoltageScale::Scale1, ..=68_000_000) => (1, 0),
        (VoltageScale::Scale1, ..=102_000_000) => (2, 1),
        (VoltageScale::Scale1, ..=136_000_000) => (3, 1),
        (VoltageScale::Scale1, ..=170_000_000) => (4, 2),
        (VoltageScale::Scale1, ..=200_000_000) => (5, 2),

        (VoltageScale::Scale2, ..=30_000_000) => (0, 0),
        (VoltageScale::Scale2, ..=60_000_000) => (1, 0),
        (VoltageScale::Scale2, ..=90_000_000) => (2, 1),
        (VoltageScale::Scale2, ..=120_000_000) => (3, 1),
        (VoltageScale::Scale2, ..=150_000_000) => (4, 2),

        (VoltageScale::Scale3, ..=20_000_000) => (0, 0),
        (VoltageScale::Scale3, ..=40_000_000) => (1, 0),
        (VoltageScale::Scale3, ..=60_000_000) => (2, 1),
        (VoltageScale::Scale3, ..=80_000_000) => (3, 1),
        (VoltageScale::Scale3, ..=100_000_000) => (4, 2),

        _ => unreachable!(),
    };

    #[cfg(all(flash_h7, not(pwr_h7rm0468)))]
    let (latency, wrhighfreq) = match (vos, clk.0) {
        // VOS 0 range VCORE 1.26V - 1.40V
        (VoltageScale::Scale0, ..=70_000_000) => (0, 0),
        (VoltageScale::Scale0, ..=140_000_000) => (1, 1),
        (VoltageScale::Scale0, ..=185_000_000) => (2, 1),
        (VoltageScale::Scale0, ..=210_000_000) => (2, 2),
        (VoltageScale::Scale0, ..=225_000_000) => (3, 2),
        (VoltageScale::Scale0, ..=240_000_000) => (4, 2),
        // VOS 1 range VCORE 1.15V - 1.26V
        (VoltageScale::Scale1, ..=70_000_000) => (0, 0),
        (VoltageScale::Scale1, ..=140_000_000) => (1, 1),
        (VoltageScale::Scale1, ..=185_000_000) => (2, 1),
        (VoltageScale::Scale1, ..=210_000_000) => (2, 2),
        (VoltageScale::Scale1, ..=225_000_000) => (3, 2),
        // VOS 2 range VCORE 1.05V - 1.15V
        (VoltageScale::Scale2, ..=55_000_000) => (0, 0),
        (VoltageScale::Scale2, ..=110_000_000) => (1, 1),
        (VoltageScale::Scale2, ..=165_000_000) => (2, 1),
        (VoltageScale::Scale2, ..=224_000_000) => (3, 2),
        // VOS 3 range VCORE 0.95V - 1.05V
        (VoltageScale::Scale3, ..=45_000_000) => (0, 0),
        (VoltageScale::Scale3, ..=90_000_000) => (1, 1),
        (VoltageScale::Scale3, ..=135_000_000) => (2, 1),
        (VoltageScale::Scale3, ..=180_000_000) => (3, 2),
        (VoltageScale::Scale3, ..=224_000_000) => (4, 2),
        _ => unreachable!(),
    };

    // See RM0468 Rev 3 Table 16. FLASH recommended number of wait
    // states and programming delay
    #[cfg(all(flash_h7, pwr_h7rm0468))]
    let (latency, wrhighfreq) = match (vos, clk.0) {
        // VOS 0 range VCORE 1.26V - 1.40V
        (VoltageScale::Scale0, ..=70_000_000) => (0, 0),
        (VoltageScale::Scale0, ..=140_000_000) => (1, 1),
        (VoltageScale::Scale0, ..=210_000_000) => (2, 2),
        (VoltageScale::Scale0, ..=275_000_000) => (3, 3),
        // VOS 1 range VCORE 1.15V - 1.26V
        (VoltageScale::Scale1, ..=67_000_000) => (0, 0),
        (VoltageScale::Scale1, ..=133_000_000) => (1, 1),
        (VoltageScale::Scale1, ..=200_000_000) => (2, 2),
        // VOS 2 range VCORE 1.05V - 1.15V
        (VoltageScale::Scale2, ..=50_000_000) => (0, 0),
        (VoltageScale::Scale2, ..=100_000_000) => (1, 1),
        (VoltageScale::Scale2, ..=150_000_000) => (2, 2),
        // VOS 3 range VCORE 0.95V - 1.05V
        (VoltageScale::Scale3, ..=35_000_000) => (0, 0),
        (VoltageScale::Scale3, ..=70_000_000) => (1, 1),
        (VoltageScale::Scale3, ..=85_000_000) => (2, 2),
        _ => unreachable!(),
    };

    // See RM0455 Rev 10 Table 16. FLASH recommended number of wait
    // states and programming delay
    #[cfg(flash_h7ab)]
    let (latency, wrhighfreq) = match (vos, clk.0) {
        // VOS 0 range VCORE 1.25V - 1.35V
        (VoltageScale::Scale0, ..=42_000_000) => (0, 0),
        (VoltageScale::Scale0, ..=84_000_000) => (1, 0),
        (VoltageScale::Scale0, ..=126_000_000) => (2, 1),
        (VoltageScale::Scale0, ..=168_000_000) => (3, 1),
        (VoltageScale::Scale0, ..=210_000_000) => (4, 2),
        (VoltageScale::Scale0, ..=252_000_000) => (5, 2),
        (VoltageScale::Scale0, ..=280_000_000) => (6, 3),
        // VOS 1 range VCORE 1.15V - 1.25V
        (VoltageScale::Scale1, ..=38_000_000) => (0, 0),
        (VoltageScale::Scale1, ..=76_000_000) => (1, 0),
        (VoltageScale::Scale1, ..=114_000_000) => (2, 1),
        (VoltageScale::Scale1, ..=152_000_000) => (3, 1),
        (VoltageScale::Scale1, ..=190_000_000) => (4, 2),
        (VoltageScale::Scale1, ..=225_000_000) => (5, 2),
        // VOS 2 range VCORE 1.05V - 1.15V
        (VoltageScale::Scale2, ..=34) => (0, 0),
        (VoltageScale::Scale2, ..=68) => (1, 0),
        (VoltageScale::Scale2, ..=102) => (2, 1),
        (VoltageScale::Scale2, ..=136) => (3, 1),
        (VoltageScale::Scale2, ..=160) => (4, 2),
        // VOS 3 range VCORE 0.95V - 1.05V
        (VoltageScale::Scale3, ..=22) => (0, 0),
        (VoltageScale::Scale3, ..=44) => (1, 0),
        (VoltageScale::Scale3, ..=66) => (2, 1),
        (VoltageScale::Scale3, ..=88) => (3, 1),
        _ => unreachable!(),
    };
    #[cfg(flash_h7rs)]
    let (latency, wrhighfreq) = match (vos, clk.0) {
        // VOS high range VCORE 1.30V - 1.40V
        (VoltageScale::High, ..=40_000_000) => (0, 0),
        (VoltageScale::High, ..=80_000_000) => (1, 0),
        (VoltageScale::High, ..=120_000_000) => (2, 1),
        (VoltageScale::High, ..=160_000_000) => (3, 1),
        (VoltageScale::High, ..=200_000_000) => (4, 2),
        (VoltageScale::High, ..=240_000_000) => (5, 2),
        (VoltageScale::High, ..=280_000_000) => (6, 3),
        (VoltageScale::High, ..=320_000_000) => (7, 3),
        // VOS low range VCORE 1.15V - 1.26V
        (VoltageScale::Low, ..=36_000_000) => (0, 0),
        (VoltageScale::Low, ..=72_000_000) => (1, 0),
        (VoltageScale::Low, ..=108_000_000) => (2, 1),
        (VoltageScale::Low, ..=144_000_000) => (3, 1),
        (VoltageScale::Low, ..=180_000_000) => (4, 2),
        (VoltageScale::Low, ..=216_000_000) => (5, 2),
        _ => unreachable!(),
    };

    debug!("flash: latency={} wrhighfreq={}", latency, wrhighfreq);

    FLASH.acr().write(|w| {
        w.set_wrhighfreq(wrhighfreq);
        w.set_latency(latency);
    });
    while FLASH.acr().read().latency() != latency {}
}

/// Compensation cell calibration values. The N-MOS and P-MOS transistors slew
/// rate compensation factors are stored for the three different compensation
/// cells available in the STM32H7RS family.
#[cfg(stm32h7rs)]
pub struct CompVals {
    /// XSPI1 N-MOS transistors slew-rate compensation value [0-15].
    pub octo1_nsrc: u8,
    /// XSPI1 P-MOS transistors slew-rate compensation value [0-15].
    pub octo1_psrc: u8,
    /// XSPI2 N-MOS transistors slew-rate compensation value [0-15].
    pub octo2_nsrc: u8,
    /// XSPI2 P-MOS transistors slew-rate compensation value [0-15].
    pub octo2_psrc: u8,
    /// GPIO N-MOS transistors slew-rate compensation value [0-15].
    pub io_nsrc: u8,
    /// GPIO P-MOS transistors slew-rate compensation value [0-15].
    pub io_psrc: u8,
}

/// Obtain the auto-tuned, slew-rate compensation values for the different
/// compensation cells. The errata corrections are applied. Following the
/// errata, these values should be obtained once during production, around
/// 30°C MCU temperature, for each individual board, and stored in non-volatile
/// memory for future use. The stored values should then be applied at power-up
/// to guarantee stable, high-speed operation of the XSPI busses, and other
/// high-speed I/O. While ST does not discuss the application to the GPIO pins
/// in general in the errata, applying the errata compensation to those as well
/// seems to improve the waveform symmetry (eg: MCO).
/// (ES0596, p. 12, Ch 2.2.15 "I/O compensation could alter duty-cycle of
/// high-frequency output signal")
#[cfg(stm32h7rs)]
pub fn get_corrected_comp_vals() -> CompVals {
    let ccvalr = pac::SYSCFG.ccvalr().read();
    return CompVals {
        octo1_nsrc: ccvalr.octo1_nsrc().saturating_add(2),
        octo1_psrc: ccvalr.octo1_psrc().saturating_sub(2),
        octo2_nsrc: ccvalr.octo2_nsrc().saturating_add(2),
        octo2_psrc: ccvalr.octo2_psrc().saturating_sub(2),
        io_nsrc: ccvalr.octo2_nsrc().saturating_add(2),
        io_psrc: ccvalr.octo2_psrc().saturating_sub(2),
    };
}

/// Apply static slew-rate compensation values to all compensation cells, and
/// enable them. These should be the corrected values outlined in the errata.
/// (ES0596, p. 12, Ch 2.2.15 "I/O compensation could alter duty-cycle of
/// high-frequency output signal")
#[cfg(stm32h7rs)]
pub fn set_and_enable_comp_vals(cv: &CompVals) {
    pac::SYSCFG.ccswvalr().modify(|w| {
        // Set the corrected, constant compensation values manually.
        w.set_octo1_sw_nsrc(cv.octo1_nsrc);
        w.set_octo1_sw_psrc(cv.octo1_psrc);
        w.set_octo2_sw_nsrc(cv.octo2_nsrc);
        w.set_octo2_sw_psrc(cv.octo2_psrc);
        w.set_sw_nsrc(cv.io_nsrc);
        w.set_sw_psrc(cv.io_psrc);
    });
    pac::SYSCFG.cccsr().modify(|w| {
        // Switch to use the constant, manual compensation values.
        w.set_octo1_comp_codesel(true);
        w.set_octo2_comp_codesel(true);
        w.set_comp_codesel(true);
    });
}

/// CPU Reset Sources
///
/// The STM32 RCC peripheral implements the ability for the CPU to detect why a reset
/// occurred.
#[cfg(rcc_h7rm0433)]
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ResetReason {
    PowerOnReset,
    PinReset,
    BrownoutReset,
    SysReset,
    CpuReset,
    Wwdg1Reset,
    Iwdg1Reset,
    D1ExitStandby,
    D2ExitStandby,
    D1ErroneousStandby,
    Unknown(u32),
}

#[cfg(rcc_h7rm0433)]
impl ResetReason {
    /// Read and clear the reason the core thinks the reset occurred.
    pub fn read_clear() -> ResetReason {
        let rsr = RCC.rsr().read();
        RCC.rsr().modify(|w| w.set_rmvf(true));

        // Refer to Reference Manual (RM0433, Rev 8, Table 56).
        match (
            rsr.lpwrrstf(),
            rsr.wwdg1rstf(),
            rsr.iwdg1rstf(),
            rsr.sftrstf(),
            rsr.porrstf(),
            rsr.pinrstf(),
            rsr.borrstf(),
            rsr.d2rstf(),
            rsr.d1rstf(),
            rsr.cpurstf(),
        ) {
            (false, false, false, false, true, true, true, true, true, true) => ResetReason::PowerOnReset,
            (false, false, false, false, false, true, false, false, false, true) => ResetReason::PinReset,
            (false, false, false, false, false, true, true, false, false, true) => ResetReason::BrownoutReset,
            (false, false, false, true, false, true, false, false, false, true) => ResetReason::SysReset,
            (false, false, false, false, false, false, false, false, false, true) => ResetReason::CpuReset,
            (false, true, false, false, false, true, false, false, false, true) => ResetReason::Wwdg1Reset,
            (false, false, true, false, false, true, false, false, false, true) => ResetReason::Iwdg1Reset,
            (false, false, false, false, false, false, false, false, true, false) => ResetReason::D1ExitStandby,
            (false, false, false, false, false, false, false, true, false, false) => ResetReason::D2ExitStandby,
            (true, false, false, false, false, true, false, false, false, true) => ResetReason::D1ErroneousStandby,
            _ => ResetReason::Unknown(rsr.0),
        }
    }
}
