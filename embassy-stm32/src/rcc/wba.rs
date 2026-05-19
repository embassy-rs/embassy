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
use crate::rcc::{LSI_FREQ, LsConfig, LseConfig, LseDrive, LseMode, RtcClockSource, mux};
use crate::time::Hertz;

/// HSI speed
pub const HSI_FREQ: Hertz = Hertz(16_000_000);
// HSE speed
pub const HSE_FREQ: Hertz = Hertz(32_000_000);

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Hse {
    pub prescaler: HsePrescaler,
    pub trim: Option<u8>,
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
            sys: Sysclk::Hsi,
            ahb_pre: AHBPrescaler::Div1,
            ahb5_pre: AHB5Prescaler::Div1,
            apb1_pre: APBPrescaler::Div1,
            apb2_pre: APBPrescaler::Div1,
            apb7_pre: APBPrescaler::Div1,
            ls: crate::rcc::LsConfig::new(),
            // lsi2: crate::rcc::LsConfig::new(),
            voltage_scale: VoltageScale::Range2,
            mux: super::mux::ClockMux::default(),
        }
    }

    pub const fn new_wpan() -> Self {
        let mut rcc = Self::new();

        // Enable HSE (32 MHz external crystal) - REQUIRED for BLE radio
        rcc.hse = Some(Hse {
            prescaler: HsePrescaler::Div1,
            trim: Some(0x0C),
        });

        // Enable LSE (32.768 kHz external crystal) - REQUIRED for BLE radio sleep timer
        rcc.ls = LsConfig {
            rtc: RtcClockSource::Lse,
            lsi: false,
            lse: Some(LseConfig {
                frequency: Hertz(32_768),
                mode: LseMode::Oscillator(LseDrive::MediumLow),
                peripherals_clocked: true,
            }),
        };

        // Configure PLL1 from HSE for system clock
        // HSE = 32MHz (fixed for WBA), prediv /2 gives 16MHz to PLL input (must be 4-16MHz)
        // VCO = 16MHz * 12 = 192MHz, PLLR = 192 / 2 = 96MHz system clock
        rcc.pll1 = Some(Pll {
            source: PllSource::Hse,
            prediv: PllPreDiv::Div2,  // 32MHz / 2 = 16MHz to PLL input
            mul: PllMul::Mul12,       // 16MHz * 12 = 192MHz VCO
            divr: Some(PllDiv::Div2), // 192MHz / 2 = 96MHz system clock
            divq: None,
            divp: Some(PllDiv::Div12), // 192MHz / 12 = 16MHz for peripherals
            frac: Some(0),
        });

        rcc.ahb_pre = AHBPrescaler::Div1;
        rcc.apb1_pre = APBPrescaler::Div1;
        rcc.apb2_pre = APBPrescaler::Div1;
        rcc.apb7_pre = APBPrescaler::Div1;
        rcc.ahb5_pre = AHB5Prescaler::Div4; // Radio bus: 96MHz / 4 = 24MHz
        rcc.voltage_scale = VoltageScale::Range1;
        rcc.sys = Sysclk::Pll1R;
        rcc.mux.rngsel = mux::Rngsel::Hsi; // RNG clock from HSI (16 MHz)
        rcc.mux.radiostsel = mux::Radiostsel::Lse;

        rcc
    }
}

impl Default for Config {
    fn default() -> Config {
        Self::new()
    }
}

/// SRAM page power-down configuration for Stop modes (Stop 0, Stop 1).
///
/// Each field controls whether a particular SRAM region is powered down
/// (content lost) or retained when the MCU enters a Stop mode. Powering
/// down unused SRAM pages reduces Stop-mode current consumption.
///
/// All pages default to retained (`false`), preserving backward compatibility.
#[derive(Clone, Copy)]
pub struct StopModeSramConfig {
    /// SRAM1 page 0 power-down in Stop modes.
    pub sram1_page0: bool,
    /// SRAM1 page 1 power-down in Stop modes.
    pub sram1_page1: bool,
    /// SRAM1 page 2 power-down in Stop modes.
    pub sram1_page2: bool,
    /// SRAM1 page 3 power-down in Stop modes.
    pub sram1_page3: bool,
    /// SRAM2 power-down in Stop modes.
    pub sram2: bool,
    /// SRAM1 pages 5-7 (192KB) power-down in Stop modes.
    /// Only present on WBA6x variants with 256KB SRAM.
    pub sram1_pages567: bool,
    /// ICACHE SRAM power-down in Stop modes.
    pub icache_sram: bool,
    /// OTG (USB) SRAM power-down in Stop modes.
    pub otg_sram: bool,
    /// PKA SRAM power-down in Stop modes.
    pub pka_sram: bool,
}

impl Default for StopModeSramConfig {
    fn default() -> Self {
        Self {
            sram1_page0: false,
            sram1_page1: false,
            sram1_page2: false,
            sram1_page3: false,
            sram2: false,
            sram1_pages567: false,
            icache_sram: false,
            otg_sram: false,
            pka_sram: false,
        }
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
    if RCC.cfgr1().read().sws() != Sysclk::Hsi {
        // Set HSI as a clock source, reset prescalers.
        RCC.cfgr1().write_value(Cfgr1::default());
        // Wait for clock switch status bits to change.
        while RCC.cfgr1().read().sws() != Sysclk::Hsi {}
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

        hse.trim.map(|trim| {
            RCC.ecscr1().modify(|w| w.set_hsetrim(trim));
        });

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
        Sysclk::Hse => hse.unwrap(),
        Sysclk::Hsi => hsi.unwrap(),
        Sysclk::_RESERVED_1 => unreachable!(),
        Sysclk::Pll1R => pll1.r.unwrap(),
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
        VoltageScale::Range1 => match sys_clk.0 {
            ..=32_000_000 => 0,
            ..=64_000_000 => 1,
            ..=96_000_000 => 2,
            ..=100_000_000 => 3,
            _ => 4,
        },
        VoltageScale::Range2 => match sys_clk.0 {
            ..=8_000_000 => 0,
            ..=16_000_000 => 1,
            _ => 2,
        },
    };

    FLASH.acr().modify(|w| w.set_latency(flash_latency));
    while FLASH.acr().read().latency() != flash_latency {}

    // Set sram wait states
    let sram_latency = match config.voltage_scale {
        VoltageScale::Range1 => 0,
        VoltageScale::Range2 => match sys_clk.0 {
            ..=12_000_000 => 0,
            ..=16_000_000 => 1,
            _ => 2,
        },
    };
    crate::pac::RCC.ahb1enr().modify(|w| w.set_ramcfgen(true));
    cortex_m::asm::dsb();
    crate::pac::RAMCFG.m1cr().modify(|w| w.set_wsc(sram_latency));
    crate::pac::RAMCFG.m2cr().modify(|w| w.set_wsc(sram_latency));

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
        Sysclk::Hsi | Sysclk::Hse => {
            // Only Div1 and Div2 are valid for HDIV5, enforce this
            match config.ahb5_pre {
                AHB5Prescaler::Div1 => w.set_hdiv5(Hdiv5::Div1),
                AHB5Prescaler::Div2 => w.set_hdiv5(Hdiv5::Div2),
                _ => panic!("Invalid ahb5_pre for HSI/HSE sysclk: only DIV1 and DIV2 are allowed"),
            };
        }
        // When using PLL1, use HPRE5 bits [2:0]
        Sysclk::Pll1R => {
            w.set_hpre5(config.ahb5_pre);
        }
        _ => {}
    });

    let hclk5 = sys_clk / config.ahb5_pre;

    #[cfg(all(stm32wba, peri_usb_otg_hs))]
    let usb_refck = match config.mux.otghssel {
        Otghssel::Hse => hse,
        Otghssel::HseDiv2 => hse.map(|hse_val| hse_val / 2u8),
        Otghssel::Pll1P => pll1.p,
        Otghssel::Pll1PDiv2 => pll1.p.map(|pll1p_val| pll1p_val / 2u8),
    };
    #[cfg(all(stm32wba, peri_usb_otg_hs))]
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
    #[cfg(all(stm32wba, peri_usb_otg_hs))]
    SYSCFG.otghsphycr().modify(|w| {
        w.set_clksel(usb_refck_sel);
    });

    #[cfg(sai_v4_2pdm)]
    let audioclk = match config.mux.sai1sel {
        Sai1sel::Hsi => Some(HSI_FREQ),
        Sai1sel::Pll1Q => Some(pll1.q.expect("PLL1.Q not configured")),
        Sai1sel::Pll1P => Some(pll1.p.expect("PLL1.P not configured")),
        Sai1sel::Sys => panic!("SYS not supported yet"),
        Sai1sel::Audioclk => panic!("AUDIOCLK not supported yet"),
        _ => None,
    };

    let lsi = config.ls.lsi.then_some(LSI_FREQ);
    let lse = config.ls.lse.map(|c| c.frequency);

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
        lse: lse,
        hsi: hsi,
        pll1_p: pll1.p,
        pll1_q: pll1.q,
        pll1_r: pll1.r,

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
        PllSource::Disable => panic!("must not select PLL source as DISABLE"),
        PllSource::Hse => unwrap!(input.hse),
        PllSource::Hsi => unwrap!(input.hsi),
        PllSource::_RESERVED_1 => panic!("must not select RESERVED_1 source as DISABLE"),
    };

    // Only divide by the HSE prescaler when the PLL source is HSE
    let src_freq = match pll.source {
        PllSource::Hse => {
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
        VoltageScale::Range1 => (Hertz::mhz(128), Hertz::mhz(544), Hertz::mhz(100)),
        VoltageScale::Range2 => panic!("PLL is unavailable in voltage range 2"),
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
        w.set_pllp(pll.divp.unwrap_or(PllDiv::Div1));
        w.set_pllq(pll.divq.unwrap_or(PllDiv::Div1));
        w.set_pllr(pll.divr.unwrap_or(PllDiv::Div1));
    });
    RCC.pll1fracr().write(|w| {
        w.set_pllfracn(pll.frac.unwrap_or(0));
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
