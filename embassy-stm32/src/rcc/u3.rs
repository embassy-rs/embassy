use stm32_metapac::rcc::vals::{Msikdiv, Msisdiv, Msissel};

// pub use crate::pac::pwr::vals::Vosr as VoltageScale;
pub use crate::pac::rcc::vals::{Hpre as AHBPrescaler, Ppre as APBPrescaler, Sw as Sysclk};
use crate::pac::rcc::vals::{Hseext, Msipllsel, Msirgsel};
use crate::pac::{FLASH, PWR, RCC};
use crate::rcc::LSI_FREQ;
use crate::time::Hertz;

/// HSI speed
pub const HSI_FREQ: Hertz = Hertz(16_000_000);

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum VoltageScale {
    /// High performance range (system clock freq up to 96MHz)
    RANGE1,
    /// Low power range (system clock freq up to 48MHz)
    RANGE2,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum MSIRange {
    /// MSIRC0
    /// Range 0: 96 MHz
    RANGE0_96MHZ,
    /// Range 1: 48 MHz
    RANGE1_48MHZ,
    /// Range 2/4: 24 MHz
    RANGE2_24MHZ,
    /// Range 3/5: 12 MHz
    RANGE3_12MHZ,
    /// MSIRC1
    /// Range 4: 24 MHz
    RANGE4_24MHZ,
    /// Range 5: 12 MHz
    RANGE5_12MHZ,
    /// Range 6: 6 MHz
    RANGE6_6MHZ,
    /// Range 7: 3 MHz
    RANGE7_3MHZ,
}

impl From<MSIRange> for Msisdiv {
    fn from(range: MSIRange) -> Self {
        match range {
            MSIRange::RANGE0_96MHZ => Msisdiv::DIV1,
            MSIRange::RANGE1_48MHZ => Msisdiv::DIV2,
            MSIRange::RANGE2_24MHZ => Msisdiv::DIV4,
            MSIRange::RANGE3_12MHZ => Msisdiv::DIV8,
            MSIRange::RANGE4_24MHZ => Msisdiv::DIV1,
            MSIRange::RANGE5_12MHZ => Msisdiv::DIV2,
            MSIRange::RANGE6_6MHZ => Msisdiv::DIV4,
            MSIRange::RANGE7_3MHZ => Msisdiv::DIV8,
        }
    }
}

impl From<MSIRange> for Msikdiv {
    fn from(range: MSIRange) -> Self {
        match range {
            MSIRange::RANGE0_96MHZ => Msikdiv::DIV1,
            MSIRange::RANGE1_48MHZ => Msikdiv::DIV2,
            MSIRange::RANGE2_24MHZ => Msikdiv::DIV4,
            MSIRange::RANGE3_12MHZ => Msikdiv::DIV8,
            MSIRange::RANGE4_24MHZ => Msikdiv::DIV1,
            MSIRange::RANGE5_12MHZ => Msikdiv::DIV2,
            MSIRange::RANGE6_6MHZ => Msikdiv::DIV4,
            MSIRange::RANGE7_3MHZ => Msikdiv::DIV8,
        }
    }
}

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

    // sysclk, buses.
    pub sys: Sysclk,
    pub ahb_pre: AHBPrescaler,
    pub apb1_pre: APBPrescaler,
    pub apb2_pre: APBPrescaler,
    pub apb3_pre: APBPrescaler,

    /// The voltage range influences the maximum clock frequencies for different parts of the
    /// device. In particular, system clocks exceeding 48 MHz require `RANGE1`.
    ///
    /// See RM0487 § 10.2.3 for clock source frequency limits.
    pub voltage_range: VoltageScale,
    pub ls: super::LsConfig,

    /// Per-peripheral kernel clock selection muxes
    pub mux: super::mux::ClockMux,
    pub auto_calibration: MsiAutoCalibration,
}

impl Config {
    pub const fn new() -> Self {
        Self {
            msis: Some(MSIRange::RANGE5_12MHZ),
            msik: Some(MSIRange::RANGE5_12MHZ),
            hse: None,
            hsi: false,
            hsi48: Some(crate::rcc::Hsi48Config::new()),
            sys: Sysclk::MSIS,
            ahb_pre: AHBPrescaler::DIV1,
            apb1_pre: APBPrescaler::DIV1,
            apb2_pre: APBPrescaler::DIV1,
            apb3_pre: APBPrescaler::DIV1,
            voltage_range: VoltageScale::RANGE1,
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
    // Set the requested power mode
    // Voltage range enable bits must be at opposite values and can
    // only be set once the current range is ready. See RM0487 § 9.5.4.
    match config.voltage_range {
        VoltageScale::RANGE1 => {
            if PWR.vosr().read().r2en() {
                while !PWR.vosr().read().r2rdy() {}
            }
            PWR.vosr().modify(|w| {
                w.set_r1en(true);
                w.set_r2en(false);
            });
            while !PWR.vosr().read().r1rdy() {}
        }
        VoltageScale::RANGE2 => {
            if PWR.vosr().read().r1en() {
                while !PWR.vosr().read().r1rdy() {}
            }
            PWR.vosr().modify(|w| {
                w.set_r2en(true);
                w.set_r1en(false);
            });
            while !PWR.vosr().read().r2rdy() {}
        }
    }

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
        // Check MSI output per RM0487 § 10.2.3 Table 98
        match config.voltage_range {
            VoltageScale::RANGE2 => {
                assert!(msirange_to_hertz(range).0 <= 48_000_000);
            }
            _ => {}
        }

        // RM0487 § 10.5.2: spin until MSIS is off or MSIS is ready before setting its range
        loop {
            let cr = RCC.cr().read();
            if cr.msison() == false || cr.msisrdy() == true {
                break;
            }
        }

        // Use MSIRC0 or MSIRC1 depending on requested range.
        let msissel = if msirange_to_hertz(range).0 <= 24_000_000 {
            Msissel::MSIRC1_24MHZ
        } else {
            Msissel::MSIRC0_96MHZ
        };
        RCC.icscr1().modify(|w| {
            w.set_msissel(msissel);
            w.set_msisdiv(range.into());
            w.set_msirgsel(Msirgsel::RCC_ICSCR1);
        });
        RCC.cr().write(|w| {
            // Out of reset MSIPLLxEN and MSIPLLSEL are 0
            // and msison is true
            w.set_msipll0en(false);
            w.set_msipll1en(false);
            w.set_msison(true);
        });
        let msis = if let (Some(freq), MsiAutoCalibration::MSIS) =
            (lse_calibration_freq, config.auto_calibration.base_mode())
        {
            // Enable the MSIS auto-calibration feature
            if msissel == Msissel::MSIRC0_96MHZ {
                RCC.icscr1().modify(|w| w.set_msipll0sel(Msipllsel::LSE));
                RCC.cr().modify(|w| w.set_msipll0en(true));
            } else {
                RCC.icscr1().modify(|w| w.set_msipll1sel(Msipllsel::LSE));
                RCC.cr().modify(|w| w.set_msipll1en(true));
            }
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
            VoltageScale::RANGE2 => {
                assert!(msirange_to_hertz(range).0 <= 48_000_000);
            }
            _ => {}
        }

        // RM0456 § 11.8.2: spin until MSIK is off or MSIK is ready before setting its range
        loop {
            let cr = RCC.cr().read();
            if cr.msikon() == false || cr.msikrdy() == true {
                break;
            }
        }

        RCC.icscr1().modify(|w| {
            w.set_msikdiv(range.into());
            w.set_msirgsel(Msirgsel::RCC_ICSCR1);
        });
        RCC.cr().modify(|w| {
            w.set_msikon(true);
        });
        let msik = if let (Some(freq), MsiAutoCalibration::MSIK) =
            (lse_calibration_freq, config.auto_calibration.base_mode())
        {
            // Enable the MSIK auto-calibration feature

            // RCC.cr().modify(|w| w.set_msipllsel(Msipllsel::MSIK));
            // RCC.cr().modify(|w| w.set_msipllen(true));
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
            if msis_range == msik_range {
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
                // TODO clean this up to select the correct MSIPLLxFAST bit
                w.set_msipll0fast(true);
                // w.set_msipllfast(Msipllfast::FAST);
            });
        }
    }

    let hsi = config.hsi.then(|| {
        RCC.cr().modify(|w| w.set_hsion(true));
        while !RCC.cr().read().hsirdy() {}

        HSI_FREQ
    });

    let hse = config.hse.map(|hse| {
        // Check frequency limits per RM456 § 11.4.10
        match config.voltage_range {
            VoltageScale::RANGE1 => {
                assert!(hse.freq.0 <= 50_000_000);
            }
            VoltageScale::RANGE2 => {
                assert!(hse.freq.0 <= 48_000_000);
            }
        }

        // Enable HSE, and wait for it to stabilize
        RCC.cr().modify(|w| {
            w.set_hseon(true);
            w.set_hsebyp(hse.mode != HseMode::Oscillator);
            w.set_hseext(match hse.mode {
                HseMode::Oscillator | HseMode::Bypass => Hseext::ANALOG,
                HseMode::BypassDigital => Hseext::DIGITAL,
            });
        });
        while !RCC.cr().read().hserdy() {}

        hse.freq
    });

    let hsi48 = config.hsi48.map(super::init_hsi48);

    // let pll_input = PllInput { hse, hsi, msi: msis };
    // let pll1 = init_pll(PllInstance::Pll1, config.pll1, &pll_input, config.voltage_range);
    // let pll2 = init_pll(PllInstance::Pll2, config.pll2, &pll_input, config.voltage_range);
    // let pll3 = init_pll(PllInstance::Pll3, config.pll3, &pll_input, config.voltage_range);

    let sys_clk = match config.sys {
        Sysclk::HSE => hse.unwrap(),
        Sysclk::HSI16 => hsi.unwrap(),
        Sysclk::MSIS => msis.unwrap(),
        Sysclk::_RESERVED_3 => unreachable!(),
        // Sysclk:: => pll1.r.unwrap(),
    };

    // Do we need the EPOD booster to reach the target clock speed per § 10.5.4?
    if sys_clk >= Hertz::mhz(24) {
        // Enable the booster
        PWR.vosr().modify(|w| w.set_boosten(true));
        while !PWR.vosr().read().boostrdy() {}
    }

    // The clock source is ready
    // Calculate and set the flash wait states
    // TODO: add wait states for low power modes
    let wait_states = match config.voltage_range {
        // VOS 1 range VCORE 1.26V - 1.40V
        VoltageScale::RANGE1 => match sys_clk.0 {
            ..=32_000_000 => 0,
            ..=64_000_000 => 1,
            ..=96_000_000 => 2,
            _ => 3,
        },
        // VOS 2 range VCORE 1.15V - 1.26V
        VoltageScale::RANGE2 => match sys_clk.0 {
            ..=16_000_000 => 0,
            ..=32_000_000 => 1,
            ..=48_000_000 => 2,
            _ => 3,
        },
    };
    FLASH.acr().modify(|w| {
        w.set_latency(wait_states);
    });

    // Switch the system clock source
    RCC.cfgr1().modify(|w| w.set_sw(config.sys));
    while RCC.cfgr1().read().sw() != config.sys {}

    // Configure the bus prescalers
    RCC.cfgr2().modify(|w| {
        w.set_hpre(config.ahb_pre);
        w.set_ppre1(config.apb1_pre);
        w.set_ppre2(config.apb2_pre);
    });
    RCC.cfgr3().modify(|w| {
        w.set_ppre3(config.apb3_pre);
    });

    let hclk = sys_clk / config.ahb_pre;

    let hclk_max = match config.voltage_range {
        VoltageScale::RANGE1 => Hertz::mhz(96),
        VoltageScale::RANGE2 => Hertz::mhz(48),
    };
    assert!(hclk <= hclk_max);

    let (pclk1, pclk1_tim) = super::util::calc_pclk(hclk, config.apb1_pre);
    let (pclk2, pclk2_tim) = super::util::calc_pclk(hclk, config.apb2_pre);
    let (pclk3, _) = super::util::calc_pclk(hclk, config.apb3_pre);

    let rtc = config.ls.init();
    let lse = config.ls.lse.map(|l| l.frequency);
    let lsi = config.ls.lsi.then_some(LSI_FREQ);

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

        // TODO
        audioclk: None,
        shsi: None,
    );
}

fn msirange_to_hertz(range: MSIRange) -> Hertz {
    match range {
        MSIRange::RANGE0_96MHZ => Hertz(96_000_000),
        MSIRange::RANGE1_48MHZ => Hertz(48_000_000),
        MSIRange::RANGE2_24MHZ => Hertz(24_000_000),
        MSIRange::RANGE3_12MHZ => Hertz(12_000_000),
        MSIRange::RANGE4_24MHZ => Hertz(12_000_000),
        MSIRange::RANGE5_12MHZ => Hertz(12_000_000),
        MSIRange::RANGE6_6MHZ => Hertz(6_000_000),
        MSIRange::RANGE7_3MHZ => Hertz(3_000_000),
    }
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

fn get_msi_calibration_fraction(range: MSIRange) -> MsiFraction {
    // Exploiting the MSIx internals to make calculations compact
    let denominator = (range as u32 & 0x03) + 1;
    // Base multipliers are deduced from Table 82: MSI oscillator characteristics in data sheet
    let numerator = [1465, 122, 94, 12][range as usize >> 2];

    MsiFraction::new(numerator, denominator)
}

/// Calculate the calibrated MSI frequency for a given range and LSE frequency
fn calculate_calibrated_msi_frequency(range: MSIRange, lse_freq: Hertz) -> Hertz {
    let fraction = get_msi_calibration_fraction(range);
    fraction.calculate_frequency(lse_freq)
}
