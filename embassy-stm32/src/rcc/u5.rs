pub use crate::pac::rcc::vals::{Hpre as AHBPrescaler, Msirange, Plldiv, Pllm, Plln, Ppre as APBPrescaler};
use crate::pac::rcc::vals::{Msirgsel, Pllmboost, Pllrge, Pllsrc, Sw};
use crate::pac::{FLASH, PWR, RCC};
use crate::time::Hertz;

/// HSI speed
pub const HSI_FREQ: Hertz = Hertz(16_000_000);

pub use crate::pac::pwr::vals::Vos as VoltageScale;

#[derive(Copy, Clone)]
#[allow(non_camel_case_types)]
pub enum ClockSrc {
    /// Use an internal medium speed oscillator (MSIS) as the system clock.
    MSI(Msirange),
    /// Use the external high speed clock as the system clock.
    ///
    /// HSE clocks faster than 25 MHz require at least `VoltageScale::RANGE3`, and HSE clocks must
    /// never exceed 50 MHz.
    HSE(Hertz),
    /// Use the 16 MHz internal high speed oscillator as the system clock.
    HSI,
    /// Use PLL1 as the system clock.
    PLL1_R(PllConfig),
}

impl Default for ClockSrc {
    fn default() -> Self {
        // The default system clock source is MSIS @ 4 MHz, per RM0456 § 11.4.9
        ClockSrc::MSI(Msirange::RANGE_4MHZ)
    }
}

#[derive(Clone, Copy)]
pub struct PllConfig {
    /// The clock source for the PLL.
    pub source: PllSource,
    /// The PLL prescaler.
    ///
    /// The clock speed of the `source` divided by `m` must be between 4 and 16 MHz.
    pub m: Pllm,
    /// The PLL multiplier.
    ///
    /// The multiplied clock – `source` divided by `m` times `n` – must be between 128 and 544
    /// MHz. The upper limit may be lower depending on the `Config { voltage_range }`.
    pub n: Plln,
    /// The divider for the P output.
    ///
    /// The P output is one of several options
    /// that can be used to feed the SAI/MDF/ADF Clock mux's.
    pub p: Plldiv,
    /// The divider for the Q output.
    ///
    /// The Q ouput is one of severals options that can be used to feed the 48MHz clocks
    /// and the OCTOSPI clock. It may also be used on the MDF/ADF clock mux's.
    pub q: Plldiv,
    /// The divider for the R output.
    ///
    /// When used to drive the system clock, `source` divided by `m` times `n` divided by `r`
    /// must not exceed 160 MHz. System clocks above 55 MHz require a non-default
    /// `Config { voltage_range }`.
    pub r: Plldiv,
}

impl PllConfig {
    /// A configuration for HSI / 1 * 10 / 1 = 160 MHz
    pub const fn hsi_160mhz() -> Self {
        PllConfig {
            source: PllSource::HSI,
            m: Pllm::DIV1,
            n: Plln::MUL10,
            p: Plldiv::DIV3,
            q: Plldiv::DIV2,
            r: Plldiv::DIV1,
        }
    }

    /// A configuration for MSIS @ 48 MHz / 3 * 10 / 1 = 160 MHz
    pub const fn msis_160mhz() -> Self {
        PllConfig {
            source: PllSource::MSIS(Msirange::RANGE_48MHZ),
            m: Pllm::DIV3,
            n: Plln::MUL10,
            p: Plldiv::DIV3,
            q: Plldiv::DIV2,
            r: Plldiv::DIV1,
        }
    }
}

#[derive(Clone, Copy)]
pub enum PllSource {
    /// Use an internal medium speed oscillator as the PLL source.
    MSIS(Msirange),
    /// Use the external high speed clock as the system PLL source.
    ///
    /// HSE clocks faster than 25 MHz require at least `VoltageScale::RANGE3`, and HSE clocks must
    /// never exceed 50 MHz.
    HSE(Hertz),
    /// Use the 16 MHz internal high speed oscillator as the PLL source.
    HSI,
}

impl Into<Pllsrc> for PllSource {
    fn into(self) -> Pllsrc {
        match self {
            PllSource::MSIS(..) => Pllsrc::MSIS,
            PllSource::HSE(..) => Pllsrc::HSE,
            PllSource::HSI => Pllsrc::HSI,
        }
    }
}

impl Into<Sw> for ClockSrc {
    fn into(self) -> Sw {
        match self {
            ClockSrc::MSI(..) => Sw::MSIS,
            ClockSrc::HSE(..) => Sw::HSE,
            ClockSrc::HSI => Sw::HSI,
            ClockSrc::PLL1_R(..) => Sw::PLL1_R,
        }
    }
}

pub struct Config {
    pub mux: ClockSrc,
    pub ahb_pre: AHBPrescaler,
    pub apb1_pre: APBPrescaler,
    pub apb2_pre: APBPrescaler,
    pub apb3_pre: APBPrescaler,
    pub hsi48: Option<super::Hsi48Config>,
    /// The voltage range influences the maximum clock frequencies for different parts of the
    /// device. In particular, system clocks exceeding 110 MHz require `RANGE1`, and system clocks
    /// exceeding 55 MHz require at least `RANGE2`.
    ///
    /// See RM0456 § 10.5.4 for a general overview and § 11.4.10 for clock source frequency limits.
    pub voltage_range: VoltageScale,
    pub ls: super::LsConfig,
}

impl Config {
    unsafe fn init_hsi(&self) -> Hertz {
        RCC.cr().write(|w| w.set_hsion(true));
        while !RCC.cr().read().hsirdy() {}

        HSI_FREQ
    }

    unsafe fn init_hse(&self, frequency: Hertz) -> Hertz {
        // Check frequency limits per RM456 § 11.4.10
        match self.voltage_range {
            VoltageScale::RANGE1 | VoltageScale::RANGE2 | VoltageScale::RANGE3 => {
                assert!(frequency.0 <= 50_000_000);
            }
            VoltageScale::RANGE4 => {
                assert!(frequency.0 <= 25_000_000);
            }
        }

        // Enable HSE, and wait for it to stabilize
        RCC.cr().write(|w| w.set_hseon(true));
        while !RCC.cr().read().hserdy() {}

        frequency
    }

    unsafe fn init_msis(&self, range: Msirange) -> Hertz {
        // Check MSI output per RM0456 § 11.4.10
        match self.voltage_range {
            VoltageScale::RANGE4 => {
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
            w.set_msirgsel(Msirgsel::ICSCR1);
        });
        RCC.cr().write(|w| {
            w.set_msipllen(false);
            w.set_msison(true);
        });
        while !RCC.cr().read().msisrdy() {}
        msirange_to_hertz(range)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mux: ClockSrc::default(),
            ahb_pre: AHBPrescaler::DIV1,
            apb1_pre: APBPrescaler::DIV1,
            apb2_pre: APBPrescaler::DIV1,
            apb3_pre: APBPrescaler::DIV1,
            hsi48: Some(Default::default()),
            voltage_range: VoltageScale::RANGE3,
            ls: Default::default(),
        }
    }
}

pub(crate) unsafe fn init(config: Config) {
    // Ensure PWR peripheral clock is enabled
    RCC.ahb3enr().modify(|w| {
        w.set_pwren(true);
    });
    RCC.ahb3enr().read(); // synchronize

    // Set the requested power mode
    PWR.vosr().modify(|w| {
        w.set_vos(config.voltage_range);
    });
    while !PWR.vosr().read().vosrdy() {}

    let sys_clk = match config.mux {
        ClockSrc::MSI(range) => config.init_msis(range),
        ClockSrc::HSE(freq) => config.init_hse(freq),
        ClockSrc::HSI => config.init_hsi(),
        ClockSrc::PLL1_R(pll) => {
            // Configure the PLL source
            let source_clk = match pll.source {
                PllSource::MSIS(range) => config.init_msis(range),
                PllSource::HSE(hertz) => config.init_hse(hertz),
                PllSource::HSI => config.init_hsi(),
            };

            // Calculate the reference clock, which is the source divided by m
            let reference_clk = source_clk / pll.m;

            // Check limits per RM0456 § 11.4.6
            assert!(Hertz::mhz(4) <= reference_clk && reference_clk <= Hertz::mhz(16));

            // Calculate the PLL1 VCO clock and PLL1 R output clock
            let pll1_clk = reference_clk * pll.n;
            let pll1r_clk = pll1_clk / pll.r;

            // Check system clock per RM0456 § 11.4.9
            assert!(pll1r_clk <= Hertz::mhz(160));

            // Check PLL clocks per RM0456 § 11.4.10
            match config.voltage_range {
                VoltageScale::RANGE1 => {
                    assert!(pll1_clk >= Hertz::mhz(128) && pll1_clk <= Hertz::mhz(544));
                    assert!(pll1r_clk <= Hertz::mhz(208));
                }
                VoltageScale::RANGE2 => {
                    assert!(pll1_clk >= Hertz::mhz(128) && pll1_clk <= Hertz::mhz(544));
                    assert!(pll1r_clk <= Hertz::mhz(110));
                }
                VoltageScale::RANGE3 => {
                    assert!(pll1_clk >= Hertz::mhz(128) && pll1_clk <= Hertz::mhz(330));
                    assert!(pll1r_clk <= Hertz::mhz(55));
                }
                VoltageScale::RANGE4 => {
                    panic!("PLL is unavailable in voltage range 4");
                }
            }

            // § 10.5.4: if we're targeting >= 55 MHz, we must configure PLL1MBOOST to a prescaler
            // value that results in an output between 4 and 16 MHz for the PWR EPOD boost
            let mboost = if pll1r_clk >= Hertz::mhz(55) {
                // source_clk can be up to 50 MHz, so there's just a few cases:
                if source_clk > Hertz::mhz(32) {
                    // Divide by 4, giving EPOD 8-12.5 MHz
                    Pllmboost::DIV4
                } else if source_clk > Hertz::mhz(16) {
                    // Divide by 2, giving EPOD 8-16 MHz
                    Pllmboost::DIV2
                } else {
                    // Bypass, giving EPOD 4-16 MHz
                    Pllmboost::DIV1
                }
            } else {
                // Nothing to do
                Pllmboost::DIV1
            };

            // Disable the PLL, and wait for it to disable
            RCC.cr().modify(|w| w.set_pllon(0, false));
            while RCC.cr().read().pllrdy(0) {}

            // Configure the PLL
            RCC.pll1cfgr().write(|w| {
                // Configure PLL1 source and prescaler
                w.set_pllsrc(pll.source.into());
                w.set_pllm(pll.m);

                // Configure PLL1 input frequncy range
                let input_range = if reference_clk <= Hertz::mhz(8) {
                    Pllrge::FREQ_4TO8MHZ
                } else {
                    Pllrge::FREQ_8TO16MHZ
                };
                w.set_pllrge(input_range);

                // Set the prescaler for PWR EPOD
                w.set_pllmboost(mboost);

                // Enable PLL1_R output
                w.set_pllren(true);
            });

            // Configure the PLL divisors
            RCC.pll1divr().modify(|w| {
                // Set the VCO multiplier
                w.set_plln(pll.n);
                w.set_pllp(pll.p);
                w.set_pllq(pll.q);
                // Set the R output divisor
                w.set_pllr(pll.r);
            });

            // Do we need the EPOD booster to reach the target clock speed per § 10.5.4?
            if pll1r_clk >= Hertz::mhz(55) {
                // Enable the booster
                PWR.vosr().modify(|w| {
                    w.set_boosten(true);
                });
                while !PWR.vosr().read().boostrdy() {}
            }

            // Enable the PLL
            RCC.cr().modify(|w| w.set_pllon(0, true));
            while !RCC.cr().read().pllrdy(0) {}

            pll1r_clk
        }
    };

    let hsi48 = config.hsi48.map(super::init_hsi48);

    // The clock source is ready
    // Calculate and set the flash wait states
    let wait_states = match config.voltage_range {
        // VOS 1 range VCORE 1.26V - 1.40V
        VoltageScale::RANGE1 => {
            if sys_clk.0 < 32_000_000 {
                0
            } else if sys_clk.0 < 64_000_000 {
                1
            } else if sys_clk.0 < 96_000_000 {
                2
            } else if sys_clk.0 < 128_000_000 {
                3
            } else {
                4
            }
        }
        // VOS 2 range VCORE 1.15V - 1.26V
        VoltageScale::RANGE2 => {
            if sys_clk.0 < 30_000_000 {
                0
            } else if sys_clk.0 < 60_000_000 {
                1
            } else if sys_clk.0 < 90_000_000 {
                2
            } else {
                3
            }
        }
        // VOS 3 range VCORE 1.05V - 1.15V
        VoltageScale::RANGE3 => {
            if sys_clk.0 < 24_000_000 {
                0
            } else if sys_clk.0 < 48_000_000 {
                1
            } else {
                2
            }
        }
        // VOS 4 range VCORE 0.95V - 1.05V
        VoltageScale::RANGE4 => {
            if sys_clk.0 < 12_000_000 {
                0
            } else {
                1
            }
        }
    };
    FLASH.acr().modify(|w| {
        w.set_latency(wait_states);
    });

    // Switch the system clock source
    RCC.cfgr1().modify(|w| {
        w.set_sw(config.mux.into());
    });

    // RM0456 § 11.4.9 specifies maximum bus frequencies per voltage range, but the maximum bus
    // frequency for each voltage range exactly matches the maximum permitted PLL output frequency.
    // Given that:
    //
    //   1. Any bus frequency can never exceed the system clock frequency;
    //   2. We checked the PLL output frequency if we're using it as a system clock;
    //   3. The maximum HSE frequencies at each voltage range are lower than the bus limits, and
    //      we checked the HSE frequency if configured as a system clock; and
    //   4. The maximum frequencies from the other clock sources are lower than the lowest bus
    //      frequency limit
    //
    // ...then we do not need to perform additional bus-related frequency checks.

    // Configure the bus prescalers
    RCC.cfgr2().modify(|w| {
        w.set_hpre(config.ahb_pre);
        w.set_ppre1(config.apb1_pre);
        w.set_ppre2(config.apb2_pre);
    });
    RCC.cfgr3().modify(|w| {
        w.set_ppre3(config.apb3_pre);
    });

    let ahb_freq = sys_clk / config.ahb_pre;

    let (apb1_freq, apb1_tim_freq) = match config.apb1_pre {
        APBPrescaler::DIV1 => (ahb_freq, ahb_freq),
        pre => {
            let freq = ahb_freq / pre;
            (freq, freq * 2u32)
        }
    };

    let (apb2_freq, apb2_tim_freq) = match config.apb2_pre {
        APBPrescaler::DIV1 => (ahb_freq, ahb_freq),
        pre => {
            let freq = ahb_freq / pre;
            (freq, freq * 2u32)
        }
    };

    let (apb3_freq, _apb3_tim_freq) = match config.apb3_pre {
        APBPrescaler::DIV1 => (ahb_freq, ahb_freq),
        pre => {
            let freq = ahb_freq / pre;
            (freq, freq * 2u32)
        }
    };

    let rtc = config.ls.init();

    set_clocks!(
        sys: Some(sys_clk),
        hclk1: Some(ahb_freq),
        hclk2: Some(ahb_freq),
        hclk3: Some(ahb_freq),
        pclk1: Some(apb1_freq),
        pclk2: Some(apb2_freq),
        pclk3: Some(apb3_freq),
        pclk1_tim: Some(apb1_tim_freq),
        pclk2_tim: Some(apb2_tim_freq),
        hsi48: hsi48,
        rtc: rtc,

        // TODO
        hse: None,
        hsi: None,
        audioclk: None,
        hsi48_div_2: None,
        lse: None,
        lsi: None,
        msik: None,
        pll1_p: None,
        pll1_q: None,
        pll1_r: None,
        pll2_p: None,
        pll2_q: None,
        pll2_r: None,
        pll3_p: None,
        pll3_q: None,
        pll3_r: None,
    );
}

fn msirange_to_hertz(range: Msirange) -> Hertz {
    match range {
        Msirange::RANGE_48MHZ => Hertz(48_000_000),
        Msirange::RANGE_24MHZ => Hertz(24_000_000),
        Msirange::RANGE_16MHZ => Hertz(16_000_000),
        Msirange::RANGE_12MHZ => Hertz(12_000_000),
        Msirange::RANGE_4MHZ => Hertz(4_000_000),
        Msirange::RANGE_2MHZ => Hertz(2_000_000),
        Msirange::RANGE_1_33MHZ => Hertz(1_330_000),
        Msirange::RANGE_1MHZ => Hertz(1_000_000),
        Msirange::RANGE_3_072MHZ => Hertz(3_072_000),
        Msirange::RANGE_1_536MHZ => Hertz(1_536_000),
        Msirange::RANGE_1_024MHZ => Hertz(1_024_000),
        Msirange::RANGE_768KHZ => Hertz(768_000),
        Msirange::RANGE_400KHZ => Hertz(400_000),
        Msirange::RANGE_200KHZ => Hertz(200_000),
        Msirange::RANGE_133KHZ => Hertz(133_000),
        Msirange::RANGE_100KHZ => Hertz(100_000),
    }
}
