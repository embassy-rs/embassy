pub use crate::pac::pwr::vals::Vos as VoltageScale;
#[cfg(all(peri_usb_otg_hs))]
pub use crate::pac::rcc::vals::Otghssel;
pub use crate::pac::rcc::vals::{
    Hpre as AHBPrescaler, Msirange, Msirange as MSIRange, Plldiv as PllDiv, Pllm as PllPreDiv, Plln as PllMul,
    Pllsrc as PllSource, Ppre as APBPrescaler, Sw as Sysclk,
};
use crate::pac::rcc::vals::{Hseext, Msirgsel, Pllmboost, Pllrge};
#[cfg(all(peri_usb_otg_hs))]
pub use crate::pac::{syscfg::vals::Usbrefcksel, SYSCFG};
use crate::pac::{FLASH, PWR, RCC};
use crate::rcc::LSI_FREQ;
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

    /// The voltage range influences the maximum clock frequencies for different parts of the
    /// device. In particular, system clocks exceeding 110 MHz require `RANGE1`, and system clocks
    /// exceeding 55 MHz require at least `RANGE2`.
    ///
    /// See RM0456 § 10.5.4 for a general overview and § 11.4.10 for clock source frequency limits.
    pub voltage_range: VoltageScale,
    pub ls: super::LsConfig,

    /// Per-peripheral kernel clock selection muxes
    pub mux: super::mux::ClockMux,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            msis: Some(Msirange::RANGE_4MHZ),
            msik: Some(Msirange::RANGE_4MHZ),
            hse: None,
            hsi: false,
            hsi48: Some(Default::default()),
            pll1: None,
            pll2: None,
            pll3: None,
            sys: Sysclk::MSIS,
            ahb_pre: AHBPrescaler::DIV1,
            apb1_pre: APBPrescaler::DIV1,
            apb2_pre: APBPrescaler::DIV1,
            apb3_pre: APBPrescaler::DIV1,
            voltage_range: VoltageScale::RANGE1,
            ls: Default::default(),
            mux: Default::default(),
        }
    }
}

pub(crate) unsafe fn init(config: Config) {
    // Set the requested power mode
    PWR.vosr().modify(|w| w.set_vos(config.voltage_range));
    while !PWR.vosr().read().vosrdy() {}

    let msis = config.msis.map(|range| {
        // Check MSI output per RM0456 § 11.4.10
        match config.voltage_range {
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
    });

    let msik = config.msik.map(|range| {
        // Check MSI output per RM0456 § 11.4.10
        match config.voltage_range {
            VoltageScale::RANGE4 => {
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
            w.set_msirgsel(Msirgsel::ICSCR1);
        });
        RCC.cr().write(|w| {
            w.set_msikon(true);
        });
        while !RCC.cr().read().msikrdy() {}
        msirange_to_hertz(range)
    });

    let hsi = config.hsi.then(|| {
        RCC.cr().write(|w| w.set_hsion(true));
        while !RCC.cr().read().hsirdy() {}

        HSI_FREQ
    });

    let hse = config.hse.map(|hse| {
        // Check frequency limits per RM456 § 11.4.10
        match config.voltage_range {
            VoltageScale::RANGE1 | VoltageScale::RANGE2 | VoltageScale::RANGE3 => {
                assert!(hse.freq.0 <= 50_000_000);
            }
            VoltageScale::RANGE4 => {
                assert!(hse.freq.0 <= 25_000_000);
            }
        }

        // Enable HSE, and wait for it to stabilize
        RCC.cr().write(|w| {
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

    let pll_input = PllInput { hse, hsi, msi: msis };
    let pll1 = init_pll(PllInstance::Pll1, config.pll1, &pll_input, config.voltage_range);
    let pll2 = init_pll(PllInstance::Pll2, config.pll2, &pll_input, config.voltage_range);
    let pll3 = init_pll(PllInstance::Pll3, config.pll3, &pll_input, config.voltage_range);

    let sys_clk = match config.sys {
        Sysclk::HSE => hse.unwrap(),
        Sysclk::HSI => hsi.unwrap(),
        Sysclk::MSIS => msis.unwrap(),
        Sysclk::PLL1_R => pll1.r.unwrap(),
    };

    // Do we need the EPOD booster to reach the target clock speed per § 10.5.4?
    if sys_clk >= Hertz::mhz(55) {
        // Enable the booster
        PWR.vosr().modify(|w| w.set_boosten(true));
        while !PWR.vosr().read().boostrdy() {}
    }

    // The clock source is ready
    // Calculate and set the flash wait states
    let wait_states = match config.voltage_range {
        // VOS 1 range VCORE 1.26V - 1.40V
        VoltageScale::RANGE1 => match sys_clk.0 {
            ..=32_000_000 => 0,
            ..=64_000_000 => 1,
            ..=96_000_000 => 2,
            ..=128_000_000 => 3,
            _ => 4,
        },
        // VOS 2 range VCORE 1.15V - 1.26V
        VoltageScale::RANGE2 => match sys_clk.0 {
            ..=30_000_000 => 0,
            ..=60_000_000 => 1,
            ..=90_000_000 => 2,
            _ => 3,
        },
        // VOS 3 range VCORE 1.05V - 1.15V
        VoltageScale::RANGE3 => match sys_clk.0 {
            ..=24_000_000 => 0,
            ..=48_000_000 => 1,
            _ => 2,
        },
        // VOS 4 range VCORE 0.95V - 1.05V
        VoltageScale::RANGE4 => match sys_clk.0 {
            ..=12_000_000 => 0,
            _ => 1,
        },
    };
    FLASH.acr().modify(|w| {
        w.set_latency(wait_states);
    });

    // Switch the system clock source
    RCC.cfgr1().modify(|w| w.set_sw(config.sys));
    while RCC.cfgr1().read().sws() != config.sys {}

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
        VoltageScale::RANGE1 => Hertz::mhz(160),
        VoltageScale::RANGE2 => Hertz::mhz(110),
        VoltageScale::RANGE3 => Hertz::mhz(55),
        VoltageScale::RANGE4 => Hertz::mhz(25),
    };
    assert!(hclk <= hclk_max);

    let (pclk1, pclk1_tim) = super::util::calc_pclk(hclk, config.apb1_pre);
    let (pclk2, pclk2_tim) = super::util::calc_pclk(hclk, config.apb2_pre);
    let (pclk3, _) = super::util::calc_pclk(hclk, config.apb3_pre);

    let rtc = config.ls.init();

    #[cfg(all(stm32u5, peri_usb_otg_hs))]
    let usb_refck = match config.mux.otghssel {
        Otghssel::HSE => hse,
        Otghssel::HSE_DIV_2 => hse.map(|hse_val| hse_val / 2u8),
        Otghssel::PLL1_P => pll1.p,
        Otghssel::PLL1_P_DIV_2 => pll1.p.map(|pll1p_val| pll1p_val / 2u8),
    };
    #[cfg(all(stm32u5, peri_usb_otg_hs))]
    let usb_refck_sel = match usb_refck {
        Some(clk_val) => match clk_val {
            Hertz(16_000_000) => Usbrefcksel::MHZ16,
            Hertz(19_200_000) => Usbrefcksel::MHZ19_2,
            Hertz(20_000_000) => Usbrefcksel::MHZ20,
            Hertz(24_000_000) => Usbrefcksel::MHZ24,
            Hertz(26_000_000) => Usbrefcksel::MHZ26,
            Hertz(32_000_000) => Usbrefcksel::MHZ32,
            _ => panic!("cannot select OTG_HS reference clock with source frequency of {}, must be one of 16, 19.2, 20, 24, 26, 32 MHz", clk_val),
        },
        None => Usbrefcksel::MHZ24,
    };
    #[cfg(all(stm32u5, peri_usb_otg_hs))]
    SYSCFG.otghsphycr().modify(|w| {
        w.set_clksel(usb_refck_sel);
    });

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
        dsi_phy: None, // DSI PLL clock not supported, don't call `RccPeripheral::frequency()` in the drivers

        // TODO
        audioclk: None,
        shsi: None,
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
        PllSource::DISABLE => panic!("must not select PLL source as DISABLE"),
        PllSource::HSE => unwrap!(input.hse),
        PllSource::HSI => unwrap!(input.hsi),
        PllSource::MSIS => unwrap!(input.msi),
    };

    // Calculate the reference clock, which is the source divided by m
    let ref_freq = src_freq / pll.prediv;
    // Check limits per RM0456 § 11.4.6
    assert!(Hertz::mhz(4) <= ref_freq && ref_freq <= Hertz::mhz(16));

    // Check PLL clocks per RM0456 § 11.4.10
    let (vco_min, vco_max, out_max) = match voltage_range {
        VoltageScale::RANGE1 => (Hertz::mhz(128), Hertz::mhz(544), Hertz::mhz(208)),
        VoltageScale::RANGE2 => (Hertz::mhz(128), Hertz::mhz(544), Hertz::mhz(110)),
        VoltageScale::RANGE3 => (Hertz::mhz(128), Hertz::mhz(330), Hertz::mhz(55)),
        VoltageScale::RANGE4 => panic!("PLL is unavailable in voltage range 4"),
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
        w.set_pllp(pll.divp.unwrap_or(PllDiv::DIV1));
        w.set_pllq(pll.divq.unwrap_or(PllDiv::DIV1));
        w.set_pllr(pll.divr.unwrap_or(PllDiv::DIV1));
    });

    let input_range = match ref_freq.0 {
        ..=8_000_000 => Pllrge::FREQ_4TO8MHZ,
        _ => Pllrge::FREQ_8TO16MHZ,
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
                    ..=16_000_000 => Pllmboost::DIV1, // Bypass, giving EPOD 4-16 MHz
                    ..=32_000_000 => Pllmboost::DIV2, // Divide by 2, giving EPOD 8-16 MHz
                    _ => Pllmboost::DIV4,             // Divide by 4, giving EPOD 8-12.5 MHz
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
