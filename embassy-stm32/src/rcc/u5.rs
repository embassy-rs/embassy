pub use crate::pac::pwr::vals::Vos as VoltageScale;
#[cfg(all(peri_usb_otg_hs))]
pub use crate::pac::rcc::vals::Otghssel;
pub use crate::pac::rcc::vals::{
    Hpre as AHBPrescaler, Msirange, Msirange as MSIRange, Plldiv as PllDiv, Pllm as PllPreDiv, Plln as PllMul,
    Pllsrc as PllSource, Ppre as APBPrescaler, Sw as Sysclk,
};
use crate::pac::rcc::vals::{Hseext, Msipllfast, Msipllsel, Msirgsel, Pllmboost, Pllrge};
use crate::pac::{FLASH, PWR, RCC};
#[cfg(all(peri_usb_otg_hs))]
pub use crate::pac::{SYSCFG, syscfg::vals::Usbrefcksel};
use crate::rcc::LSI_FREQ;
#[cfg(dsihost)]
use crate::rcc::dsi;
#[cfg(dsihost)]
pub use crate::rcc::dsi::{DsiHostPllConfig, DsiPllInput, DsiPllNdiv, DsiPllOutput};
use crate::time::Hertz;

/// HSI speed
pub const HSI_FREQ: Hertz = Hertz(16_000_000);

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

#[derive(Clone, Copy)]
pub struct Pll {
    /// The clock source for the PLL.
    pub source: PllSource,
    /// The PLL pre-divider.
    ///
    /// The clock speed of the `source` divided by `m` must be between 4 and 16 MHz.
    pub prediv: PllPreDiv,
    /// The PLL multiplier.
    ///
    /// The multiplied clock – `source` divided by `m` times `n` – must be between 128 and 544
    /// MHz. The upper limit may be lower depending on the `Config { voltage_range }`.
    pub mul: PllMul,
    /// The divider for the P output.
    ///
    /// The P output is one of several options
    /// that can be used to feed the SAI/MDF/ADF Clock mux's.
    pub divp: Option<PllDiv>,
    /// The divider for the Q output.
    ///
    /// The Q ouput is one of severals options that can be used to feed the 48MHz clocks
    /// and the OCTOSPI clock. It may also be used on the MDF/ADF clock mux's.
    pub divq: Option<PllDiv>,
    /// The divider for the R output.
    ///
    /// When used to drive the system clock, `source` divided by `m` times `n` divided by `r`
    /// must not exceed 160 MHz. System clocks above 55 MHz require a non-default
    /// `Config { voltage_range }`.
    pub divr: Option<PllDiv>,
}

#[derive(Clone, Copy, PartialEq)]
pub enum MsiAutoCalibration {
    /// MSI auto-calibration is disabled
    Disabled,
    /// MSIS is given priority for auto-calibration
    MSIS,
    /// MSIK is given priority for auto-calibration
    MSIK,
    /// MSIS with fast mode (always on)
    MsisFast,
    /// MSIK with fast mode (always on)
    MsikFast,
}

impl MsiAutoCalibration {
    const fn default() -> Self {
        MsiAutoCalibration::Disabled
    }

    fn base_mode(&self) -> Self {
        match self {
            MsiAutoCalibration::Disabled => MsiAutoCalibration::Disabled,
            MsiAutoCalibration::MSIS => MsiAutoCalibration::MSIS,
            MsiAutoCalibration::MSIK => MsiAutoCalibration::MSIK,
            MsiAutoCalibration::MsisFast => MsiAutoCalibration::MSIS,
            MsiAutoCalibration::MsikFast => MsiAutoCalibration::MSIK,
        }
    }

    fn is_fast(&self) -> bool {
        matches!(self, MsiAutoCalibration::MsisFast | MsiAutoCalibration::MsikFast)
    }
}

impl Default for MsiAutoCalibration {
    fn default() -> Self {
        Self::default()
    }
}

#[derive(Clone, Copy)]
pub struct Config {
    // base clock sources
    pub msis: Option<MSIRange>,
    pub msik: Option<MSIRange>,
    pub hsi: bool,
    pub hse: Option<Hse>,
    pub hsi48: Option<super::Hsi48Config>,

    // pll
    pub pll1: Option<Pll>,
    pub pll2: Option<Pll>,
    pub pll3: Option<Pll>,

    // sysclk, buses.
    pub sys: Sysclk,
    pub ahb_pre: AHBPrescaler,
    pub apb1_pre: APBPrescaler,
    pub apb2_pre: APBPrescaler,
    pub apb3_pre: APBPrescaler,

    #[cfg(dsihost)]
    pub dsi: Option<DsiHostPllConfig>,

    /// The voltage range influences the maximum clock frequencies for different parts of the
    /// device. In particular, system clocks exceeding 110 MHz require `RANGE1`, and system clocks
    /// exceeding 55 MHz require at least `RANGE2`.
    ///
    /// See RM0456 § 10.5.4 for a general overview and § 11.4.10 for clock source frequency limits.
    pub voltage_range: VoltageScale,
    pub ls: super::LsConfig,

    /// Per-peripheral kernel clock selection muxes
    pub mux: super::mux::ClockMux,
    pub auto_calibration: MsiAutoCalibration,
}

impl Config {
    pub const fn new() -> Self {
        Self {
            msis: Some(Msirange::Range4mhz),
            msik: Some(Msirange::Range4mhz),
            hse: None,
            hsi: false,
            hsi48: Some(crate::rcc::Hsi48Config::new()),
            pll1: None,
            pll2: None,
            pll3: None,
            sys: Sysclk::Msis,
            ahb_pre: AHBPrescaler::Div1,
            apb1_pre: APBPrescaler::Div1,
            apb2_pre: APBPrescaler::Div1,
            apb3_pre: APBPrescaler::Div1,
            #[cfg(dsihost)]
            dsi: None,
            voltage_range: VoltageScale::Range1,
            ls: crate::rcc::LsConfig::new(),
            mux: super::mux::ClockMux::default(),
            auto_calibration: MsiAutoCalibration::default(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

pub(crate) unsafe fn init(config: Config) {
    // Configure the clock to a safe default state before starting configuration:

    // 1 - Set power mode to Range1
    PWR.vosr().modify(|w| w.set_vos(VoltageScale::Range1));
    while !PWR.vosr().read().vosrdy() {}

    //2 - set flash WS to 4
    FLASH.acr().modify(|w| {
        w.set_latency(4);
    });

    // 3 - enable HSI
    // HSI is the preferred source for a safe clock setup since its value is fixed and it is available on all MCus
    RCC.cr().modify(|w| w.set_hsion(true));
    while !RCC.cr().read().hsirdy() {}

    // 4 - set sysclock to HSI
    RCC.cfgr1().modify(|w| w.set_sw(Sysclk::Hsi));
    while RCC.cfgr1().read().sws() != Sysclk::Hsi {}

    // 5 - set HPRE to div1 (not strictly necessary, but at this point it is a safe operation and there is no need to keep AHB prescalers)
    RCC.cfgr2().modify(|w| {
        w.set_hpre(AHBPrescaler::Div1);
    });

    // now configuration can proceed without issues:

    let lse_calibration_freq = if config.auto_calibration != MsiAutoCalibration::Disabled {
        // LSE must be configured and peripherals clocked for MSI auto-calibration
        let lse_config = config
            .ls
            .lse
            .clone()
            .expect("LSE must be configured for MSI auto-calibration");
        assert!(lse_config.peripherals_clocked);

        // Expect less than +/- 5% deviation for LSE frequency
        if (31_100..=34_400).contains(&lse_config.frequency.0) {
            // Check that the calibration is applied to an active clock
            match (
                config.auto_calibration.base_mode(),
                config.msis.is_some(),
                config.msik.is_some(),
            ) {
                (MsiAutoCalibration::MSIS, true, _) => {
                    // MSIS is active and using LSE for auto-calibration
                    Some(lse_config.frequency)
                }
                (MsiAutoCalibration::MSIK, _, true) => {
                    // MSIK is active and using LSE for auto-calibration
                    Some(lse_config.frequency)
                }
                // improper configuration
                _ => panic!("MSIx auto-calibration is enabled for a source that has not been configured."),
            }
        } else {
            panic!("LSE frequency more than 5% off from 32.768 kHz, cannot use for MSI auto-calibration");
        }
    } else {
        None
    };

    let mut msis = config.msis.map(|range| {
        // Check MSI output per RM0456 § 11.4.10
        match config.voltage_range {
            VoltageScale::Range4 => {
                assert!(msirange_to_hertz(range).0 <= 24_000_000);
            }
            _ => {}
        }

        // RM0456 § 11.8.2: spin until MSIS is off or MSIS is ready before setting its range
        loop {
            let cr = RCC.cr().read();
            if cr.msison() == false || cr.msisrdy() == true {
                break;
            }
        }

        RCC.icscr1().modify(|w| {
            w.set_msisrange(range);
            w.set_msirgsel(Msirgsel::Icscr1);
        });
        RCC.cr().write(|w| {
            w.set_msipllen(false);
            w.set_msison(true);
        });
        let msis = if let (Some(freq), MsiAutoCalibration::MSIS) =
            (lse_calibration_freq, config.auto_calibration.base_mode())
        {
            // Enable the MSIS auto-calibration feature
            RCC.cr().modify(|w| w.set_msipllsel(Msipllsel::Msis));
            RCC.cr().modify(|w| w.set_msipllen(true));
            calculate_calibrated_msi_frequency(range, freq)
        } else {
            msirange_to_hertz(range)
        };
        while !RCC.cr().read().msisrdy() {}
        msis
    });

    let mut msik = config.msik.map(|range| {
        // Check MSI output per RM0456 § 11.4.10
        match config.voltage_range {
            VoltageScale::Range4 => {
                assert!(msirange_to_hertz(range).0 <= 24_000_000);
            }
            _ => {}
        }

        // RM0456 § 11.8.2: spin until MSIS is off or MSIS is ready before setting its range
        loop {
            let cr = RCC.cr().read();
            if cr.msikon() == false || cr.msikrdy() == true {
                break;
            }
        }

        RCC.icscr1().modify(|w| {
            w.set_msikrange(range);
            w.set_msirgsel(Msirgsel::Icscr1);
        });
        RCC.cr().modify(|w| {
            w.set_msikon(true);
        });
        let msik = if let (Some(freq), MsiAutoCalibration::MSIK) =
            (lse_calibration_freq, config.auto_calibration.base_mode())
        {
            // Enable the MSIK auto-calibration feature
            RCC.cr().modify(|w| w.set_msipllsel(Msipllsel::Msik));
            RCC.cr().modify(|w| w.set_msipllen(true));
            calculate_calibrated_msi_frequency(range, freq)
        } else {
            msirange_to_hertz(range)
        };
        while !RCC.cr().read().msikrdy() {}
        msik
    });

    if let Some(lse_freq) = lse_calibration_freq {
        // If both MSIS and MSIK are enabled, we need to check if they are using the same internal source.
        if let (Some(msis_range), Some(msik_range)) = (config.msis, config.msik) {
            if (msis_range as u8 >> 2) == (msik_range as u8 >> 2) {
                // Clock source is shared, both will be auto calibrated, recalculate other frequency
                match config.auto_calibration.base_mode() {
                    MsiAutoCalibration::MSIS => {
                        msik = Some(calculate_calibrated_msi_frequency(msik_range, lse_freq));
                    }
                    MsiAutoCalibration::MSIK => {
                        msis = Some(calculate_calibrated_msi_frequency(msis_range, lse_freq));
                    }
                    _ => {}
                }
            }
        }
        // Check if Fast mode should be used
        if config.auto_calibration.is_fast() {
            RCC.cr().modify(|w| {
                w.set_msipllfast(Msipllfast::Fast);
            });
        }
    }

    let hsi = config.hsi.then(|| HSI_FREQ);

    let hse = config.hse.map(|hse| {
        // Check frequency limits per RM456 § 11.4.10
        match config.voltage_range {
            VoltageScale::Range1 | VoltageScale::Range2 | VoltageScale::Range3 => {
                assert!(hse.freq.0 <= 50_000_000);
            }
            VoltageScale::Range4 => {
                assert!(hse.freq.0 <= 25_000_000);
            }
        }

        // Enable HSE, and wait for it to stabilize
        RCC.cr().modify(|w| {
            w.set_hseon(true);
            w.set_hsebyp(hse.mode != HseMode::Oscillator);
            w.set_hseext(match hse.mode {
                HseMode::Oscillator | HseMode::Bypass => Hseext::Analog,
                HseMode::BypassDigital => Hseext::Digital,
            });
        });
        while !RCC.cr().read().hserdy() {}

        hse.freq
    });

    let hsi48 = config.hsi48.map(super::init_hsi48);

    let pll_input = PllInput { hse, hsi, msi: msis };
    let pll1 = config.pll1.map_or_else(
        || {
            pll_enable(PllInstance::Pll1, false);
            PllOutput::default()
        },
        |c| init_pll(PllInstance::Pll1, Some(c), &pll_input, config.voltage_range),
    );
    let pll2 = config.pll2.map_or_else(
        || {
            pll_enable(PllInstance::Pll2, false);
            PllOutput::default()
        },
        |c| init_pll(PllInstance::Pll2, Some(c), &pll_input, config.voltage_range),
    );
    let pll3 = config.pll3.map_or_else(
        || {
            pll_enable(PllInstance::Pll3, false);
            PllOutput::default()
        },
        |c| init_pll(PllInstance::Pll3, Some(c), &pll_input, config.voltage_range),
    );

    // Verify that sysclk is valid before attempting to change the clock source
    // This ensures that, even in case of an error, the clock remains in a safe state
    let sys_clk = match config.sys {
        Sysclk::Hse => hse.unwrap(),
        Sysclk::Hsi => hsi.unwrap(),
        Sysclk::Msis => msis.unwrap(),
        Sysclk::Pll1R => pll1.r.unwrap(),
    };

    let hclk = sys_clk / config.ahb_pre;

    let hclk_max = match config.voltage_range {
        VoltageScale::Range1 => Hertz::mhz(160),
        VoltageScale::Range2 => Hertz::mhz(110),
        VoltageScale::Range3 => Hertz::mhz(55),
        VoltageScale::Range4 => Hertz::mhz(25),
    };
    assert!(hclk <= hclk_max);

    // Do we need the EPOD booster to reach the target clock speed per § 10.5.4?
    if sys_clk >= Hertz::mhz(55) {
        // Enable the booster
        PWR.vosr().modify(|w| w.set_boosten(true));
        while !PWR.vosr().read().boostrdy() {}
    }

    // modifying flash WS and VOS here is safe because the clock has already been set to HSI
    // Set the requested power mode
    PWR.vosr().modify(|w| w.set_vos(config.voltage_range));
    while !PWR.vosr().read().vosrdy() {}

    // The clock source is ready
    // Calculate and set the flash wait states
    let wait_states = match config.voltage_range {
        // VOS 1 range VCORE 1.26V - 1.40V
        VoltageScale::Range1 => match sys_clk.0 {
            ..=32_000_000 => 0,
            ..=64_000_000 => 1,
            ..=96_000_000 => 2,
            ..=128_000_000 => 3,
            _ => 4,
        },
        // VOS 2 range VCORE 1.15V - 1.26V
        VoltageScale::Range2 => match sys_clk.0 {
            ..=30_000_000 => 0,
            ..=60_000_000 => 1,
            ..=90_000_000 => 2,
            _ => 3,
        },
        // VOS 3 range VCORE 1.05V - 1.15V
        VoltageScale::Range3 => match sys_clk.0 {
            ..=24_000_000 => 0,
            ..=48_000_000 => 1,
            _ => 2,
        },
        // VOS 4 range VCORE 0.95V - 1.05V
        VoltageScale::Range4 => match sys_clk.0 {
            ..=12_000_000 => 0,
            _ => 1,
        },
    };

    FLASH.acr().modify(|w| {
        w.set_latency(wait_states);
    });

    // Configure the bus prescalers
    RCC.cfgr2().modify(|w| {
        w.set_hpre(config.ahb_pre);
        w.set_ppre1(config.apb1_pre);
        w.set_ppre2(config.apb2_pre);
    });

    RCC.cfgr3().modify(|w| {
        w.set_ppre3(config.apb3_pre);
    });

    // now that flash WS, VOS and HPRE are configured, the system can switch the clock source
    RCC.cfgr1().modify(|w| w.set_sw(config.sys));
    while RCC.cfgr1().read().sws() != config.sys {}

    let (pclk1, pclk1_tim) = super::util::calc_pclk(hclk, config.apb1_pre);
    let (pclk2, pclk2_tim) = super::util::calc_pclk(hclk, config.apb2_pre);
    let (pclk3, _) = super::util::calc_pclk(hclk, config.apb3_pre);

    let rtc = config.ls.init();

    #[cfg(all(stm32u5, peri_usb_otg_hs))]
    let usb_refck = match config.mux.otghssel {
        Otghssel::Hse => hse,
        Otghssel::HseDiv2 => hse.map(|hse_val| hse_val / 2u8),
        Otghssel::Pll1P => pll1.p,
        Otghssel::Pll1PDiv2 => pll1.p.map(|pll1p_val| pll1p_val / 2u8),
    };
    #[cfg(all(stm32u5, peri_usb_otg_hs))]
    let usb_refck_sel = match usb_refck {
        Some(clk_val) => match clk_val {
            Hertz(16_000_000) => Usbrefcksel::Mhz16,
            Hertz(19_200_000) => Usbrefcksel::Mhz192,
            Hertz(20_000_000) => Usbrefcksel::Mhz20,
            Hertz(24_000_000) => Usbrefcksel::Mhz24,
            Hertz(26_000_000) => Usbrefcksel::Mhz26,
            Hertz(32_000_000) => Usbrefcksel::Mhz32,
            _ => panic!(
                "cannot select OTG_HS reference clock with source frequency of {}, must be one of 16, 19.2, 20, 24, 26, 32 MHz",
                clk_val
            ),
        },
        None => Usbrefcksel::Mhz24,
    };
    #[cfg(all(stm32u5, peri_usb_otg_hs))]
    SYSCFG.otghsphycr().modify(|w| {
        w.set_clksel(usb_refck_sel);
    });

    let lse = config.ls.lse.map(|l| l.frequency);
    let lsi = config.ls.lsi.then_some(LSI_FREQ);

    // Disable HSI if not used
    if !config.hsi {
        RCC.cr().modify(|w| w.set_hsion(false));
    }

    // Disable the HSI48, if not used
    #[cfg(crs)]
    if config.hsi48.is_none() {
        super::disable_hsi48();
    }

    config.mux.init();

    set_clocks!(
        sys: Some(sys_clk),
        hclk1: Some(hclk),
        hclk2: Some(hclk),
        hclk3: Some(hclk),
        pclk1: Some(pclk1),
        pclk2: Some(pclk2),
        pclk3: Some(pclk3),
        pclk1_tim: Some(pclk1_tim),
        pclk2_tim: Some(pclk2_tim),
        msik: msik,
        hsi48: hsi48,
        rtc: rtc,
        lse: lse,
        lsi: lsi,
        hse: hse,
        hsi: hsi,
        pll1_p: pll1.p,
        pll1_q: pll1.q,
        pll1_r: pll1.r,
        pll2_p: pll2.p,
        pll2_q: pll2.q,
        pll2_r: pll2.r,
        pll3_p: pll3.p,
        pll3_q: pll3.q,
        pll3_r: pll3.r,

        #[cfg(dsihost)]
        dsi_phy: config.dsi.map(|config| dsi::configure_pll(hse, config)),

        // TODO
        audioclk: None,
        shsi: None,
    );
}

fn msirange_to_hertz(range: Msirange) -> Hertz {
    match range {
        Msirange::Range48mhz => Hertz(48_000_000),
        Msirange::Range24mhz => Hertz(24_000_000),
        Msirange::Range16mhz => Hertz(16_000_000),
        Msirange::Range12mhz => Hertz(12_000_000),
        Msirange::Range4mhz => Hertz(4_000_000),
        Msirange::Range2mhz => Hertz(2_000_000),
        Msirange::Range133mhz => Hertz(1_330_000),
        Msirange::Range1mhz => Hertz(1_000_000),
        Msirange::Range3072mhz => Hertz(3_072_000),
        Msirange::Range1536mhz => Hertz(1_536_000),
        Msirange::Range1024mhz => Hertz(1_024_000),
        Msirange::Range768khz => Hertz(768_000),
        Msirange::Range400khz => Hertz(400_000),
        Msirange::Range200khz => Hertz(200_000),
        Msirange::Range133khz => Hertz(133_000),
        Msirange::Range100khz => Hertz(100_000),
    }
}

pub(super) struct PllInput {
    pub hsi: Option<Hertz>,
    pub hse: Option<Hertz>,
    pub msi: Option<Hertz>,
}

#[allow(unused)]
#[derive(Default)]
pub(super) struct PllOutput {
    pub p: Option<Hertz>,
    pub q: Option<Hertz>,
    pub r: Option<Hertz>,
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum PllInstance {
    Pll1 = 0,
    Pll2 = 1,
    Pll3 = 2,
}

fn pll_enable(instance: PllInstance, enabled: bool) {
    RCC.cr().modify(|w| w.set_pllon(instance as _, enabled));
    while RCC.cr().read().pllrdy(instance as _) != enabled {}
}

fn init_pll(instance: PllInstance, config: Option<Pll>, input: &PllInput, voltage_range: VoltageScale) -> PllOutput {
    // Disable PLL
    pll_enable(instance, false);

    let Some(pll) = config else { return PllOutput::default() };

    let src_freq = match pll.source {
        PllSource::Disable => panic!("must not select PLL source as DISABLE"),
        PllSource::Hse => unwrap!(input.hse),
        PllSource::Hsi => unwrap!(input.hsi),
        PllSource::Msis => unwrap!(input.msi),
    };

    // Calculate the reference clock, which is the source divided by m
    let ref_freq = src_freq / pll.prediv;
    // Check limits per RM0456 § 11.4.6
    assert!(Hertz::mhz(4) <= ref_freq && ref_freq <= Hertz::mhz(16));

    // Check PLL clocks per RM0456 § 11.4.10
    let (vco_min, vco_max, out_max) = match voltage_range {
        VoltageScale::Range1 => (Hertz::mhz(128), Hertz::mhz(544), Hertz::mhz(208)),
        VoltageScale::Range2 => (Hertz::mhz(128), Hertz::mhz(544), Hertz::mhz(110)),
        VoltageScale::Range3 => (Hertz::mhz(128), Hertz::mhz(330), Hertz::mhz(55)),
        VoltageScale::Range4 => panic!("PLL is unavailable in voltage range 4"),
    };

    // Calculate the PLL VCO clock
    let vco_freq = ref_freq * pll.mul;
    assert!(vco_freq >= vco_min && vco_freq <= vco_max);

    // Calculate output clocks.
    let p = pll.divp.map(|div| vco_freq / div);
    let q = pll.divq.map(|div| vco_freq / div);
    let r = pll.divr.map(|div| vco_freq / div);
    for freq in [p, q, r] {
        if let Some(freq) = freq {
            assert!(freq <= out_max);
        }
    }

    let divr = match instance {
        PllInstance::Pll1 => RCC.pll1divr(),
        PllInstance::Pll2 => RCC.pll2divr(),
        PllInstance::Pll3 => RCC.pll3divr(),
    };
    divr.write(|w| {
        w.set_plln(pll.mul);
        w.set_pllp(pll.divp.unwrap_or(PllDiv::Div1));
        w.set_pllq(pll.divq.unwrap_or(PllDiv::Div1));
        w.set_pllr(pll.divr.unwrap_or(PllDiv::Div1));
    });

    let input_range = match ref_freq.0 {
        ..=8_000_000 => Pllrge::Freq4to8mhz,
        _ => Pllrge::Freq8to16mhz,
    };

    macro_rules! write_fields {
        ($w:ident) => {
            $w.set_pllpen(pll.divp.is_some());
            $w.set_pllqen(pll.divq.is_some());
            $w.set_pllren(pll.divr.is_some());
            $w.set_pllm(pll.prediv);
            $w.set_pllsrc(pll.source);
            $w.set_pllrge(input_range);
        };
    }

    match instance {
        PllInstance::Pll1 => RCC.pll1cfgr().write(|w| {
            // § 10.5.4: if we're targeting >= 55 MHz, we must configure PLL1MBOOST to a prescaler
            // value that results in an output between 4 and 16 MHz for the PWR EPOD boost
            if r.unwrap() >= Hertz::mhz(55) {
                // source_clk can be up to 50 MHz, so there's just a few cases:
                let mboost = match src_freq.0 {
                    ..=16_000_000 => Pllmboost::Div1, // Bypass, giving EPOD 4-16 MHz
                    ..=32_000_000 => Pllmboost::Div2, // Divide by 2, giving EPOD 8-16 MHz
                    _ => Pllmboost::Div4,             // Divide by 4, giving EPOD 8-12.5 MHz
                };
                w.set_pllmboost(mboost);
            }
            write_fields!(w);
        }),
        PllInstance::Pll2 => RCC.pll2cfgr().write(|w| {
            write_fields!(w);
        }),
        PllInstance::Pll3 => RCC.pll3cfgr().write(|w| {
            write_fields!(w);
        }),
    }

    // Enable PLL
    pll_enable(instance, true);

    PllOutput { p, q, r }
}

/// Fraction structure for MSI auto-calibration
/// Represents the multiplier as numerator/denominator that LSE frequency is multiplied by
#[derive(Debug, Clone, Copy)]
struct MsiFraction {
    numerator: u32,
    denominator: u32,
}

impl MsiFraction {
    const fn new(numerator: u32, denominator: u32) -> Self {
        Self { numerator, denominator }
    }

    /// Calculate the calibrated frequency given an LSE frequency
    fn calculate_frequency(&self, lse_freq: Hertz) -> Hertz {
        Hertz(lse_freq.0 * self.numerator / self.denominator)
    }
}

fn get_msi_calibration_fraction(range: Msirange) -> MsiFraction {
    // Exploiting the MSIx internals to make calculations compact
    let denominator = (range as u32 & 0x03) + 1;
    // Base multipliers are deduced from Table 82: MSI oscillator characteristics in data sheet
    let numerator = [1465, 122, 94, 12][range as usize >> 2];

    MsiFraction::new(numerator, denominator)
}

/// Calculate the calibrated MSI frequency for a given range and LSE frequency
fn calculate_calibrated_msi_frequency(range: Msirange, lse_freq: Hertz) -> Hertz {
    let fraction = get_msi_calibration_fraction(range);
    fraction.calculate_frequency(lse_freq)
}
