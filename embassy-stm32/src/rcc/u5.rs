use stm32_metapac::rcc::vals::{Msirange, Msirgsel, Pllm, Pllmboost, Pllrge, Pllsrc, Sw};

pub use super::bus::{AHBPrescaler, APBPrescaler};
use crate::pac::{FLASH, PWR, RCC};
use crate::rcc::{set_freqs, Clocks};
use crate::time::Hertz;

/// HSI speed
pub const HSI_FREQ: Hertz = Hertz(16_000_000);

/// LSI speed
pub const LSI_FREQ: Hertz = Hertz(32_000);

pub use crate::pac::pwr::vals::Vos as VoltageScale;

#[derive(Copy, Clone)]
pub enum ClockSrc {
    /// Use an internal medium speed oscillator (MSIS) as the system clock.
    MSI(MSIRange),
    /// Use the external high speed clock as the system clock.
    ///
    /// HSE clocks faster than 25 MHz require at least `VoltageScale::RANGE3`, and HSE clocks must
    /// never exceed 50 MHz.
    HSE(Hertz),
    /// Use the 16 MHz internal high speed oscillator as the system clock.
    HSI16,
    /// Use PLL1 as the system clock.
    PLL1R(PllConfig),
}

impl Default for ClockSrc {
    fn default() -> Self {
        // The default system clock source is MSIS @ 4 MHz, per RM0456 § 11.4.9
        ClockSrc::MSI(MSIRange::Range4mhz)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PllConfig {
    /// The clock source for the PLL.
    pub source: PllSrc,
    /// The PLL prescaler.
    ///
    /// The clock speed of the `source` divided by `m` must be between 4 and 16 MHz.
    pub m: PllM,
    /// The PLL multiplier.
    ///
    /// The multiplied clock – `source` divided by `m` times `n` – must be between 128 and 544
    /// MHz. The upper limit may be lower depending on the `Config { voltage_range }`.
    pub n: PllN,
    /// The divider for the R output.
    ///
    /// When used to drive the system clock, `source` divided by `m` times `n` divided by `r`
    /// must not exceed 160 MHz. System clocks above 55 MHz require a non-default
    /// `Config { voltage_range }`.
    pub r: PllClkDiv,
}

impl PllConfig {
    /// A configuration for HSI16 / 1 * 10 / 1 = 160 MHz
    pub const fn hsi16_160mhz() -> Self {
        PllConfig {
            source: PllSrc::HSI16,
            m: PllM::NotDivided,
            n: PllN::Mul10,
            r: PllClkDiv::NotDivided,
        }
    }

    /// A configuration for MSIS @ 48 MHz / 3 * 10 / 1 = 160 MHz
    pub const fn msis_160mhz() -> Self {
        PllConfig {
            source: PllSrc::MSIS(MSIRange::Range48mhz),
            m: PllM::Div3,
            n: PllN::Mul10,
            r: PllClkDiv::NotDivided,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum PllSrc {
    /// Use an internal medium speed oscillator as the PLL source.
    MSIS(MSIRange),
    /// Use the external high speed clock as the system PLL source.
    ///
    /// HSE clocks faster than 25 MHz require at least `VoltageScale::RANGE3`, and HSE clocks must
    /// never exceed 50 MHz.
    HSE(Hertz),
    /// Use the 16 MHz internal high speed oscillator as the PLL source.
    HSI16,
}

impl Into<Pllsrc> for PllSrc {
    fn into(self) -> Pllsrc {
        match self {
            PllSrc::MSIS(..) => Pllsrc::MSIS,
            PllSrc::HSE(..) => Pllsrc::HSE,
            PllSrc::HSI16 => Pllsrc::HSI16,
        }
    }
}

seq_macro::seq!(N in 2..=128 {
    #[derive(Copy, Clone, Debug)]
    pub enum PllClkDiv {
        NotDivided = 1,
        #(
            Div~N = N,
        )*
    }

    impl PllClkDiv {
        fn to_div(&self) -> u8 {
            match self {
                PllClkDiv::NotDivided => 0,
                #(
                    PllClkDiv::Div~N => N - 1,
                )*
            }
        }
    }
});

seq_macro::seq!(N in 4..=512 {
    #[derive(Copy, Clone, Debug)]
    pub enum PllN {
        NotMultiplied = 1,
        #(
            Mul~N = N,
        )*
    }

    impl PllN {
        fn to_mul(&self) -> u16 {
            match self {
                PllN::NotMultiplied => 0,
                #(
                    PllN::Mul~N => N - 1,
                )*
            }
        }
    }
});

// Pre-division
#[derive(Copy, Clone, Debug)]
pub enum PllM {
    NotDivided = 0b0000,
    Div2 = 0b0001,
    Div3 = 0b0010,
    Div4 = 0b0011,
    Div5 = 0b0100,
    Div6 = 0b0101,
    Div7 = 0b0110,
    Div8 = 0b0111,
    Div9 = 0b1000,
    Div10 = 0b1001,
    Div11 = 0b1010,
    Div12 = 0b1011,
    Div13 = 0b1100,
    Div14 = 0b1101,
    Div15 = 0b1110,
    Div16 = 0b1111,
}

impl Into<Pllm> for PllM {
    fn into(self) -> Pllm {
        Pllm::from_bits(self as u8)
    }
}

impl Into<Sw> for ClockSrc {
    fn into(self) -> Sw {
        match self {
            ClockSrc::MSI(..) => Sw::MSIS,
            ClockSrc::HSE(..) => Sw::HSE,
            ClockSrc::HSI16 => Sw::HSI16,
            ClockSrc::PLL1R(..) => Sw::PLL1_R,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum MSIRange {
    /// The 48 MHz MSI speed is unavailable in `VoltageScale::RANGE4`.
    Range48mhz = 48_000_000,
    Range24mhz = 24_000_000,
    Range16mhz = 16_000_000,
    Range12mhz = 12_000_000,
    Range4mhz = 4_000_000,
    Range2mhz = 2_000_000,
    Range1_33mhz = 1_330_000,
    Range1mhz = 1_000_000,
    Range3_072mhz = 3_072_000,
    Range1_536mhz = 1_536_000,
    Range1_024mhz = 1_024_000,
    Range768khz = 768_000,
    Range400khz = 400_000,
    Range200khz = 200_000,
    Range133khz = 133_000,
    Range100khz = 100_000,
}

impl Into<u32> for MSIRange {
    fn into(self) -> u32 {
        self as u32
    }
}

impl Into<Msirange> for MSIRange {
    fn into(self) -> Msirange {
        match self {
            MSIRange::Range48mhz => Msirange::RANGE_48MHZ,
            MSIRange::Range24mhz => Msirange::RANGE_24MHZ,
            MSIRange::Range16mhz => Msirange::RANGE_16MHZ,
            MSIRange::Range12mhz => Msirange::RANGE_12MHZ,
            MSIRange::Range4mhz => Msirange::RANGE_4MHZ,
            MSIRange::Range2mhz => Msirange::RANGE_2MHZ,
            MSIRange::Range1_33mhz => Msirange::RANGE_1_33MHZ,
            MSIRange::Range1mhz => Msirange::RANGE_1MHZ,
            MSIRange::Range3_072mhz => Msirange::RANGE_3_072MHZ,
            MSIRange::Range1_536mhz => Msirange::RANGE_1_536MHZ,
            MSIRange::Range1_024mhz => Msirange::RANGE_1_024MHZ,
            MSIRange::Range768khz => Msirange::RANGE_768KHZ,
            MSIRange::Range400khz => Msirange::RANGE_400KHZ,
            MSIRange::Range200khz => Msirange::RANGE_200KHZ,
            MSIRange::Range133khz => Msirange::RANGE_133KHZ,
            MSIRange::Range100khz => Msirange::RANGE_100KHZ,
        }
    }
}

#[derive(Copy, Clone)]
pub struct Config {
    pub mux: ClockSrc,
    pub ahb_pre: AHBPrescaler,
    pub apb1_pre: APBPrescaler,
    pub apb2_pre: APBPrescaler,
    pub apb3_pre: APBPrescaler,
    pub hsi48: bool,
    /// The voltage range influences the maximum clock frequencies for different parts of the
    /// device. In particular, system clocks exceeding 110 MHz require `RANGE1`, and system clocks
    /// exceeding 55 MHz require at least `RANGE2`.
    ///
    /// See RM0456 § 10.5.4 for a general overview and § 11.4.10 for clock source frequency limits.
    pub voltage_range: VoltageScale,
}

impl Config {
    unsafe fn init_hsi16(&self) -> Hertz {
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

    unsafe fn init_msis(&self, range: MSIRange) -> Hertz {
        // Check MSI output per RM0456 § 11.4.10
        match self.voltage_range {
            VoltageScale::RANGE4 => {
                assert!(range as u32 <= 24_000_000);
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
            let bits: Msirange = range.into();
            w.set_msisrange(bits);
            w.set_msirgsel(Msirgsel::RCC_ICSCR1);
        });
        RCC.cr().write(|w| {
            w.set_msipllen(false);
            w.set_msison(true);
        });
        while !RCC.cr().read().msisrdy() {}
        Hertz(range as u32)
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
            hsi48: false,
            voltage_range: VoltageScale::RANGE3,
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
        ClockSrc::HSI16 => config.init_hsi16(),
        ClockSrc::PLL1R(pll) => {
            // Configure the PLL source
            let source_clk = match pll.source {
                PllSrc::MSIS(range) => config.init_msis(range),
                PllSrc::HSE(hertz) => config.init_hse(hertz),
                PllSrc::HSI16 => config.init_hsi16(),
            };

            // Calculate the reference clock, which is the source divided by m
            let reference_clk = source_clk / (pll.m as u8 as u32 + 1);

            // Check limits per RM0456 § 11.4.6
            assert!(Hertz::mhz(4) <= reference_clk && reference_clk <= Hertz::mhz(16));

            // Calculate the PLL1 VCO clock and PLL1 R output clock
            let pll1_clk = reference_clk * (pll.n as u8 as u32);
            let pll1r_clk = pll1_clk / (pll.r as u8 as u32);

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
                    Pllmboost::BYPASS
                }
            } else {
                // Nothing to do
                Pllmboost::BYPASS
            };

            // Disable the PLL, and wait for it to disable
            RCC.cr().modify(|w| w.set_pllon(0, false));
            while RCC.cr().read().pllrdy(0) {}

            // Configure the PLL
            RCC.pll1cfgr().write(|w| {
                // Configure PLL1 source and prescaler
                w.set_pllsrc(pll.source.into());
                w.set_pllm(pll.m.into());

                // Configure PLL1 input frequncy range
                let input_range = if reference_clk <= Hertz::mhz(8) {
                    Pllrge::FREQ_4TO8MHZ
                } else {
                    Pllrge::FREQ_8TO16MHZ
                };
                w.set_pllrge(input_range);

                // Set the prescaler for PWR EPOD
                w.set_pllmboost(mboost);

                // Enable PLL1R output
                w.set_pllren(true);
            });

            // Configure the PLL divisors
            RCC.pll1divr().modify(|w| {
                // Set the VCO multiplier
                w.set_plln(pll.n.to_mul());
                // Set the R output divisor
                w.set_pllr(pll.r.to_div());
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
    }
    .0;

    if config.hsi48 {
        RCC.cr().modify(|w| w.set_hsi48on(true));
        while !RCC.cr().read().hsi48rdy() {}
    }

    // The clock source is ready
    // Calculate and set the flash wait states
    let wait_states = match config.voltage_range {
        // VOS 1 range VCORE 1.26V - 1.40V
        VoltageScale::RANGE1 => {
            if sys_clk < 32_000_000 {
                0
            } else if sys_clk < 64_000_000 {
                1
            } else if sys_clk < 96_000_000 {
                2
            } else if sys_clk < 128_000_000 {
                3
            } else {
                4
            }
        }
        // VOS 2 range VCORE 1.15V - 1.26V
        VoltageScale::RANGE2 => {
            if sys_clk < 30_000_000 {
                0
            } else if sys_clk < 60_000_000 {
                1
            } else if sys_clk < 90_000_000 {
                2
            } else {
                3
            }
        }
        // VOS 3 range VCORE 1.05V - 1.15V
        VoltageScale::RANGE3 => {
            if sys_clk < 24_000_000 {
                0
            } else if sys_clk < 48_000_000 {
                1
            } else {
                2
            }
        }
        // VOS 4 range VCORE 0.95V - 1.05V
        VoltageScale::RANGE4 => {
            if sys_clk < 12_000_000 {
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
        w.set_hpre(config.ahb_pre.into());
        w.set_ppre1(config.apb1_pre.into());
        w.set_ppre2(config.apb2_pre.into());
    });
    RCC.cfgr3().modify(|w| {
        w.set_ppre3(config.apb3_pre.into());
    });

    let ahb_freq: u32 = match config.ahb_pre {
        AHBPrescaler::DIV1 => sys_clk,
        pre => {
            let pre: u8 = pre.into();
            let pre = 1 << (pre as u32 - 7);
            sys_clk / pre
        }
    };

    let (apb1_freq, apb1_tim_freq) = match config.apb1_pre {
        APBPrescaler::DIV1 => (ahb_freq, ahb_freq),
        pre => {
            let pre: u8 = pre.into();
            let pre: u8 = 1 << (pre - 3);
            let freq = ahb_freq / pre as u32;
            (freq, freq * 2)
        }
    };

    let (apb2_freq, apb2_tim_freq) = match config.apb2_pre {
        APBPrescaler::DIV1 => (ahb_freq, ahb_freq),
        pre => {
            let pre: u8 = pre.into();
            let pre: u8 = 1 << (pre - 3);
            let freq = ahb_freq / pre as u32;
            (freq, freq * 2)
        }
    };

    let (apb3_freq, _apb3_tim_freq) = match config.apb3_pre {
        APBPrescaler::DIV1 => (ahb_freq, ahb_freq),
        pre => {
            let pre: u8 = pre.into();
            let pre: u8 = 1 << (pre - 3);
            let freq = ahb_freq / pre as u32;
            (freq, freq * 2)
        }
    };

    set_freqs(Clocks {
        sys: Hertz(sys_clk),
        ahb1: Hertz(ahb_freq),
        ahb2: Hertz(ahb_freq),
        ahb3: Hertz(ahb_freq),
        apb1: Hertz(apb1_freq),
        apb2: Hertz(apb2_freq),
        apb3: Hertz(apb3_freq),
        apb1_tim: Hertz(apb1_tim_freq),
        apb2_tim: Hertz(apb2_tim_freq),
    });
}
