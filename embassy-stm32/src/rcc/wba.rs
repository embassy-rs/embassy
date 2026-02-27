pub use crate::pac::pwr::vals::Vos as VoltageScale;
use crate::pac::rcc::regs::Cfgr1;
#[cfg(all(peri_usb_otg_hs))]
pub use crate::pac::rcc::vals::Otghssel;
use crate::pac::rcc::vals::Pllrge;
pub use crate::pac::rcc::vals::{
    Hdiv5, Hpre as AHBPrescaler, Hpre5 as AHB5Prescaler, Hsepre as HsePrescaler, Plldiv as PllDiv, Pllm as PllPreDiv,
    Plln as PllMul, Pllsrc as PllSource, Ppre as APBPrescaler, Sai1sel, Sw as Sysclk,
};
use crate::pac::{FLASH, RCC};
#[cfg(all(peri_usb_otg_hs))]
pub use crate::pac::{SYSCFG, syscfg::vals::Usbrefcksel};
use crate::rcc::LSI_FREQ;
use crate::time::Hertz;

/// HSI speed
pub const HSI_FREQ: Hertz = Hertz(16_000_000);
// HSE speed
pub const HSE_FREQ: Hertz = Hertz(32_000_000);

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Hse {
    pub prescaler: HsePrescaler,
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

    pub frac: Option<u16>,
}

/// Clocks configuration
#[derive(Clone, Copy)]
pub struct Config {
    // base clock sources
    pub hsi: bool,
    pub hse: Option<Hse>,

    // pll
    pub pll1: Option<Pll>,

    // sysclk, buses.
    pub sys: Sysclk,
    pub ahb_pre: AHBPrescaler,
    pub ahb5_pre: AHB5Prescaler,
    pub apb1_pre: APBPrescaler,
    pub apb2_pre: APBPrescaler,
    pub apb7_pre: APBPrescaler,

    // low speed LSI/LSE/RTC
    pub ls: super::LsConfig,

    pub voltage_scale: VoltageScale,

    /// Per-peripheral kernel clock selection muxes
    pub mux: super::mux::ClockMux,
}

impl Config {
    pub const fn new() -> Self {
        Config {
            hsi: true,
            hse: None,
            pll1: None,
            sys: Sysclk::HSI,
            ahb_pre: AHBPrescaler::DIV1,
            ahb5_pre: AHB5Prescaler::DIV1,
            apb1_pre: APBPrescaler::DIV1,
            apb2_pre: APBPrescaler::DIV1,
            apb7_pre: APBPrescaler::DIV1,
            ls: crate::rcc::LsConfig::new(),
            // lsi2: crate::rcc::LsConfig::new(),
            voltage_scale: VoltageScale::RANGE2,
            mux: super::mux::ClockMux::default(),
        }
    }
}

impl Default for Config {
    fn default() -> Config {
        Self::new()
    }
}

fn hsi_enable() {
    RCC.cr().modify(|w| w.set_hsion(true));
    while !RCC.cr().read().hsirdy() {}
}

pub(crate) unsafe fn init(config: Config) {
    // Switch to HSI to prevent problems with PLL configuration.
    if !RCC.cr().read().hsion() {
        hsi_enable()
    }
    if RCC.cfgr1().read().sws() != Sysclk::HSI {
        // Set HSI as a clock source, reset prescalers.
        RCC.cfgr1().write_value(Cfgr1::default());
        // Wait for clock switch status bits to change.
        while RCC.cfgr1().read().sws() != Sysclk::HSI {}
    }

    // Set voltage scale
    crate::pac::PWR.vosr().write(|w| w.set_vos(config.voltage_scale));
    while !crate::pac::PWR.vosr().read().vosrdy() {}

    let rtc = config.ls.init();

    let hsi = config.hsi.then(|| {
        hsi_enable();

        HSI_FREQ
    });

    let hse = config.hse.map(|hse| {
        RCC.cr().write(|w| {
            w.set_hseon(true);
            w.set_hsepre(hse.prescaler);
        });
        while !RCC.cr().read().hserdy() {}

        HSE_FREQ
    });

    let pll_input = PllInput { hse, hsi };

    let pll1 = config.pll1.map_or_else(
        || {
            pll_enable(false);
            PllOutput::default()
        },
        |c| init_pll(Some(c), &pll_input, config.voltage_scale),
    );

    let sys_clk = match config.sys {
        Sysclk::HSE => hse.unwrap(),
        Sysclk::HSI => hsi.unwrap(),
        Sysclk::_RESERVED_1 => unreachable!(),
        Sysclk::PLL1_R => pll1.r.unwrap(),
    };

    assert!(sys_clk.0 <= 100_000_000);

    let hclk1 = sys_clk / config.ahb_pre;
    let hclk2 = hclk1;
    let hclk4 = hclk1;
    let (pclk1, pclk1_tim) = super::util::calc_pclk(hclk1, config.apb1_pre);
    let (pclk2, pclk2_tim) = super::util::calc_pclk(hclk1, config.apb2_pre);
    let (pclk7, _) = super::util::calc_pclk(hclk1, config.apb7_pre);

    // Set flash wait states
    let flash_latency = match config.voltage_scale {
        VoltageScale::RANGE1 => match sys_clk.0 {
            ..=32_000_000 => 0,
            ..=64_000_000 => 1,
            ..=96_000_000 => 2,
            ..=100_000_000 => 3,
            _ => 4,
        },
        VoltageScale::RANGE2 => match sys_clk.0 {
            ..=8_000_000 => 0,
            ..=16_000_000 => 1,
            _ => 2,
        },
    };

    FLASH.acr().modify(|w| w.set_latency(flash_latency));
    while FLASH.acr().read().latency() != flash_latency {}

    // Set sram wait states
    let _sram_latency = match config.voltage_scale {
        VoltageScale::RANGE1 => 0,
        VoltageScale::RANGE2 => match sys_clk.0 {
            ..=12_000_000 => 0,
            ..=16_000_000 => 1,
            _ => 2,
        },
    };
    // TODO: Set the SRAM wait states

    RCC.cfgr1().modify(|w| {
        w.set_sw(config.sys);
    });
    while RCC.cfgr1().read().sws() != config.sys {}

    RCC.cfgr2().modify(|w| {
        w.set_hpre(config.ahb_pre);
        w.set_ppre1(config.apb1_pre);
        w.set_ppre2(config.apb2_pre);
    });

    // Set AHB5 prescaler depending on sysclk source
    RCC.cfgr4().modify(|w| match config.sys {
        // When using HSI or HSE, use HDIV5 bit (0 = div1, 1 = div2)
        Sysclk::HSI | Sysclk::HSE => {
            // Only Div1 and Div2 are valid for HDIV5, enforce this
            match config.ahb5_pre {
                AHB5Prescaler::DIV1 => w.set_hdiv5(Hdiv5::DIV1),
                AHB5Prescaler::DIV2 => w.set_hdiv5(Hdiv5::DIV2),
                _ => panic!("Invalid ahb5_pre for HSI/HSE sysclk: only DIV1 and DIV2 are allowed"),
            };
        }
        // When using PLL1, use HPRE5 bits [2:0]
        Sysclk::PLL1_R => {
            w.set_hpre5(config.ahb5_pre);
        }
        _ => {}
    });

    let hclk5 = sys_clk / config.ahb5_pre;

    #[cfg(all(stm32wba, peri_usb_otg_hs))]
    let usb_refck = match config.mux.otghssel {
        Otghssel::HSE => hse,
        Otghssel::HSE_DIV_2 => hse.map(|hse_val| hse_val / 2u8),
        Otghssel::PLL1_P => pll1.p,
        Otghssel::PLL1_P_DIV_2 => pll1.p.map(|pll1p_val| pll1p_val / 2u8),
    };
    #[cfg(all(stm32wba, peri_usb_otg_hs))]
    let usb_refck_sel = match usb_refck {
        Some(clk_val) => match clk_val {
            Hertz(16_000_000) => Usbrefcksel::MHZ16,
            Hertz(19_200_000) => Usbrefcksel::MHZ19_2,
            Hertz(20_000_000) => Usbrefcksel::MHZ20,
            Hertz(24_000_000) => Usbrefcksel::MHZ24,
            Hertz(26_000_000) => Usbrefcksel::MHZ26,
            Hertz(32_000_000) => Usbrefcksel::MHZ32,
            _ => panic!(
                "cannot select OTG_HS reference clock with source frequency of {}, must be one of 16, 19.2, 20, 24, 26, 32 MHz",
                clk_val
            ),
        },
        None => Usbrefcksel::MHZ24,
    };
    #[cfg(all(stm32wba, peri_usb_otg_hs))]
    SYSCFG.otghsphycr().modify(|w| {
        w.set_clksel(usb_refck_sel);
    });

    #[cfg(sai_v4_2pdm)]
    let audioclk = match config.mux.sai1sel {
        Sai1sel::HSI => Some(HSI_FREQ),
        Sai1sel::PLL1_Q => Some(pll1.q.expect("PLL1.Q not configured")),
        Sai1sel::PLL1_P => Some(pll1.p.expect("PLL1.P not configured")),
        Sai1sel::SYS => panic!("SYS not supported yet"),
        Sai1sel::AUDIOCLK => panic!("AUDIOCLK not supported yet"),
        _ => None,
    };

    let lsi = config.ls.lsi.then_some(LSI_FREQ);

    // Disable HSI if not used
    if !config.hsi {
        RCC.cr().modify(|w| w.set_hsion(false));
    }

    config.mux.init();

    set_clocks!(
        sys: Some(sys_clk),
        hclk1: Some(hclk1),
        hclk2: Some(hclk2),
        hclk4: Some(hclk4),
        hclk5: Some(hclk5),
        pclk1: Some(pclk1),
        pclk2: Some(pclk2),
        pclk7: Some(pclk7),
        pclk1_tim: Some(pclk1_tim),
        pclk2_tim: Some(pclk2_tim),
        rtc: rtc,
        hse: hse,
        lsi: lsi,
        hsi: hsi,
        pll1_p: pll1.p,
        pll1_q: pll1.q,
        pll1_r: pll1.r,

        // TODO
        lse: None,
        #[cfg(sai_v4_2pdm)]
        audioclk: audioclk,
    );
}

pub(super) struct PllInput {
    pub hsi: Option<Hertz>,
    pub hse: Option<Hertz>,
}

#[allow(unused)]
#[derive(Default)]
pub(super) struct PllOutput {
    pub p: Option<Hertz>,
    pub q: Option<Hertz>,
    pub r: Option<Hertz>,
}

fn pll_enable(enabled: bool) {
    RCC.cr().modify(|w| w.set_pllon(enabled));
    while RCC.cr().read().pllrdy() != enabled {}
}

fn init_pll(config: Option<Pll>, input: &PllInput, voltage_range: VoltageScale) -> PllOutput {
    // Disable PLL
    pll_enable(false);

    let Some(pll) = config else { return PllOutput::default() };

    let pre_src_freq = match pll.source {
        PllSource::DISABLE => panic!("must not select PLL source as DISABLE"),
        PllSource::HSE => unwrap!(input.hse),
        PllSource::HSI => unwrap!(input.hsi),
        PllSource::_RESERVED_1 => panic!("must not select RESERVED_1 source as DISABLE"),
    };

    // Only divide by the HSE prescaler when the PLL source is HSE
    let src_freq = match pll.source {
        PllSource::HSE => {
            // read the prescaler bits and divide
            let hsepre = RCC.cr().read().hsepre();
            pre_src_freq / hsepre
        }
        _ => pre_src_freq,
    };

    // Calculate the reference clock, which is the source divided by m
    let ref_freq = src_freq / pll.prediv;
    // Check limits per RM0515 § 12.4.3
    assert!(Hertz::mhz(4) <= ref_freq && ref_freq <= Hertz::mhz(16));

    // Check PLL clocks per RM0515 § 12.4.5
    let (vco_min, vco_max, out_max) = match voltage_range {
        VoltageScale::RANGE1 => (Hertz::mhz(128), Hertz::mhz(544), Hertz::mhz(100)),
        VoltageScale::RANGE2 => panic!("PLL is unavailable in voltage range 2"),
    };

    // Calculate the PLL VCO clock
    // let vco_freq = ref_freq * pll.mul;
    // Calculate VCO frequency including fractional part: FVCO = Fref_ck × (N + FRAC/2^13)
    let numerator = (ref_freq.0 as u64) * (((pll.mul as u64) + 1 << 13) + pll.frac.unwrap_or(0) as u64);
    let vco_hz = (numerator >> 13) as u32;
    let vco_freq = Hertz(vco_hz);
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

    let divr = RCC.pll1divr();
    divr.write(|w| {
        w.set_plln(pll.mul);
        w.set_pllp(pll.divp.unwrap_or(PllDiv::DIV1));
        w.set_pllq(pll.divq.unwrap_or(PllDiv::DIV1));
        w.set_pllr(pll.divr.unwrap_or(PllDiv::DIV1));
    });
    RCC.pll1fracr().write(|w| {
        w.set_pllfracn(pll.frac.unwrap_or(0));
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
            $w.set_pllfracen(pll.frac.is_some());
            $w.set_pllm(pll.prediv);
            $w.set_pllsrc(pll.source);
            $w.set_pllrge(input_range);
        };
    }

    RCC.pll1cfgr().write(|w| {
        write_fields!(w);
    });

    // Enable PLL
    pll_enable(true);

    PllOutput { p, q, r }
}
