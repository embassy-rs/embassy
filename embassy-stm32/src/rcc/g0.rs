pub use super::bus::{AHBPrescaler, APBPrescaler};
use crate::pac::flash::vals::Latency;
use crate::pac::rcc::vals::{self, Hsidiv, Ppre, Sw};
use crate::pac::{FLASH, PWR, RCC};
use crate::rcc::{set_freqs, Clocks};
use crate::time::Hertz;

/// HSI speed
pub const HSI_FREQ: Hertz = Hertz(16_000_000);

/// LSI speed
pub const LSI_FREQ: Hertz = Hertz(32_000);

/// System clock mux source
#[derive(Clone, Copy)]
pub enum ClockSrc {
    HSE(Hertz),
    HSI16(HSI16Prescaler),
    PLL(PllConfig),
    LSI,
}

#[derive(Clone, Copy)]
pub enum HSI16Prescaler {
    NotDivided,
    Div2,
    Div4,
    Div8,
    Div16,
    Div32,
    Div64,
    Div128,
}

impl Into<Hsidiv> for HSI16Prescaler {
    fn into(self) -> Hsidiv {
        match self {
            HSI16Prescaler::NotDivided => Hsidiv::DIV1,
            HSI16Prescaler::Div2 => Hsidiv::DIV2,
            HSI16Prescaler::Div4 => Hsidiv::DIV4,
            HSI16Prescaler::Div8 => Hsidiv::DIV8,
            HSI16Prescaler::Div16 => Hsidiv::DIV16,
            HSI16Prescaler::Div32 => Hsidiv::DIV32,
            HSI16Prescaler::Div64 => Hsidiv::DIV64,
            HSI16Prescaler::Div128 => Hsidiv::DIV128,
        }
    }
}

/// The PLL configuration.
///
/// * `VCOCLK = source / m * n`
/// * `PLLRCLK = VCOCLK / r`
/// * `PLLQCLK = VCOCLK / q`
/// * `PLLPCLK = VCOCLK / p`
#[derive(Clone, Copy)]
pub struct PllConfig {
    /// The source from which the PLL receives a clock signal
    pub source: PllSrc,
    /// The initial divisor of that clock signal
    pub m: Pllm,
    /// The PLL VCO multiplier, which must be in the range `8..=86`.
    pub n: u8,
    /// The final divisor for `PLLRCLK` output which drives the system clock
    pub r: Pllr,

    /// The divisor for the `PLLQCLK` output, if desired
    pub q: Option<Pllr>,

    /// The divisor for the `PLLPCLK` output, if desired
    pub p: Option<Pllr>,
}

impl Default for PllConfig {
    #[inline]
    fn default() -> PllConfig {
        // HSI16 / 1 * 8 / 2 = 64 MHz
        PllConfig {
            source: PllSrc::HSI16,
            m: Pllm::Div1,
            n: 8,
            r: Pllr::Div2,
            q: None,
            p: None,
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum PllSrc {
    HSI16,
    HSE(Hertz),
}

#[derive(Clone, Copy)]
pub enum Pllm {
    Div1,
    Div2,
    Div3,
    Div4,
    Div5,
    Div6,
    Div7,
    Div8,
}

impl From<Pllm> for u8 {
    fn from(v: Pllm) -> Self {
        match v {
            Pllm::Div1 => 0b000,
            Pllm::Div2 => 0b001,
            Pllm::Div3 => 0b010,
            Pllm::Div4 => 0b011,
            Pllm::Div5 => 0b100,
            Pllm::Div6 => 0b101,
            Pllm::Div7 => 0b110,
            Pllm::Div8 => 0b111,
        }
    }
}

impl From<Pllm> for u32 {
    fn from(v: Pllm) -> Self {
        match v {
            Pllm::Div1 => 1,
            Pllm::Div2 => 2,
            Pllm::Div3 => 3,
            Pllm::Div4 => 4,
            Pllm::Div5 => 5,
            Pllm::Div6 => 6,
            Pllm::Div7 => 7,
            Pllm::Div8 => 8,
        }
    }
}

#[derive(Clone, Copy)]
pub enum Pllr {
    Div2,
    Div3,
    Div4,
    Div5,
    Div6,
    Div7,
    Div8,
}

impl From<Pllr> for u8 {
    fn from(v: Pllr) -> Self {
        match v {
            Pllr::Div2 => 0b000,
            Pllr::Div3 => 0b001,
            Pllr::Div4 => 0b010,
            Pllr::Div5 => 0b011,
            Pllr::Div6 => 0b101,
            Pllr::Div7 => 0b110,
            Pllr::Div8 => 0b111,
        }
    }
}

impl From<Pllr> for u32 {
    fn from(v: Pllr) -> Self {
        match v {
            Pllr::Div2 => 2,
            Pllr::Div3 => 3,
            Pllr::Div4 => 4,
            Pllr::Div5 => 5,
            Pllr::Div6 => 6,
            Pllr::Div7 => 7,
            Pllr::Div8 => 8,
        }
    }
}

/// Clocks configutation
pub struct Config {
    pub mux: ClockSrc,
    pub ahb_pre: AHBPrescaler,
    pub apb_pre: APBPrescaler,
    pub low_power_run: bool,
}

impl Default for Config {
    #[inline]
    fn default() -> Config {
        Config {
            mux: ClockSrc::HSI16(HSI16Prescaler::NotDivided),
            ahb_pre: AHBPrescaler::NotDivided,
            apb_pre: APBPrescaler::NotDivided,
            low_power_run: false,
        }
    }
}

impl PllConfig {
    pub(crate) fn init(self) -> u32 {
        assert!(self.n >= 8 && self.n <= 86);
        let (src, input_freq) = match self.source {
            PllSrc::HSI16 => (vals::Pllsrc::HSI16, HSI_FREQ.0),
            PllSrc::HSE(freq) => (vals::Pllsrc::HSE, freq.0),
        };

        let m_freq = input_freq / u32::from(self.m);
        // RM0454 § 5.4.4:
        // > Caution: The software must set these bits so that the PLL input frequency after the
        // > /M divider is between 2.66 and 16 MHz.
        debug_assert!(m_freq >= 2_660_000 && m_freq <= 16_000_000);

        let n_freq = m_freq * self.n as u32;
        // RM0454 § 5.4.4:
        // > Caution: The software must set these bits so that the VCO output frequency is between
        // > 64 and 344 MHz.
        debug_assert!(n_freq >= 64_000_000 && n_freq <= 344_000_000);

        let r_freq = n_freq / u32::from(self.r);
        // RM0454 § 5.4.4:
        // > Caution: The software must set this bitfield so as not to exceed 64 MHz on this clock.
        debug_assert!(r_freq <= 64_000_000);

        // RM0454 § 5.2.3:
        // > To modify the PLL configuration, proceed as follows:
        // > 1. Disable the PLL by setting PLLON to 0 in Clock control register (RCC_CR).
        RCC.cr().modify(|w| w.set_pllon(false));

        // > 2. Wait until PLLRDY is cleared. The PLL is now fully stopped.
        while RCC.cr().read().pllrdy() {}

        // > 3. Change the desired parameter.
        // Enable whichever clock source we're using, and wait for it to become ready
        match self.source {
            PllSrc::HSI16 => {
                RCC.cr().write(|w| w.set_hsion(true));
                while !RCC.cr().read().hsirdy() {}
            }
            PllSrc::HSE(_) => {
                RCC.cr().write(|w| w.set_hseon(true));
                while !RCC.cr().read().hserdy() {}
            }
        }

        // Configure PLLSYSCFGR
        RCC.pllsyscfgr().modify(|w| {
            w.set_pllr(u8::from(self.r));
            w.set_pllren(false);

            if let Some(q) = self.q {
                w.set_pllq(u8::from(q));
            }
            w.set_pllqen(false);

            if let Some(p) = self.p {
                w.set_pllp(u8::from(p));
            }
            w.set_pllpen(false);

            w.set_plln(self.n);

            w.set_pllm(self.m as u8);

            w.set_pllsrc(src)
        });

        // > 4. Enable the PLL again by setting PLLON to 1.
        RCC.cr().modify(|w| w.set_pllon(true));

        // Wait for the PLL to become ready
        while !RCC.cr().read().pllrdy() {}

        // > 5. Enable the desired PLL outputs by configuring PLLPEN, PLLQEN, and PLLREN in PLL
        // > configuration register (RCC_PLLCFGR).
        RCC.pllsyscfgr().modify(|w| {
            // We'll use R for system clock, so enable that unconditionally
            w.set_pllren(true);

            // We may also use Q or P
            w.set_pllqen(self.q.is_some());
            w.set_pllpen(self.p.is_some());
        });

        r_freq
    }
}

pub(crate) unsafe fn init(config: Config) {
    let (sys_clk, sw) = match config.mux {
        ClockSrc::HSI16(div) => {
            // Enable HSI16
            let div: Hsidiv = div.into();
            RCC.cr().write(|w| {
                w.set_hsidiv(div);
                w.set_hsion(true)
            });
            while !RCC.cr().read().hsirdy() {}

            (HSI_FREQ.0 >> div.to_bits(), Sw::HSI)
        }
        ClockSrc::HSE(freq) => {
            // Enable HSE
            RCC.cr().write(|w| w.set_hseon(true));
            while !RCC.cr().read().hserdy() {}

            (freq.0, Sw::HSE)
        }
        ClockSrc::PLL(pll) => {
            let freq = pll.init();
            (freq, Sw::PLLRCLK)
        }
        ClockSrc::LSI => {
            // Enable LSI
            RCC.csr().write(|w| w.set_lsion(true));
            while !RCC.csr().read().lsirdy() {}
            (LSI_FREQ.0, Sw::LSI)
        }
    };

    // Determine the flash latency implied by the target clock speed
    // RM0454 § 3.3.4:
    let target_flash_latency = if sys_clk <= 24_000_000 {
        Latency::WS0
    } else if sys_clk <= 48_000_000 {
        Latency::WS1
    } else {
        Latency::WS2
    };

    // Increase the number of cycles we wait for flash if the new value is higher
    // There's no harm in waiting a little too much before the clock change, but we'll
    // crash immediately if we don't wait enough after the clock change
    let mut set_flash_latency_after = false;
    FLASH.acr().modify(|w| {
        // Is the current flash latency less than what we need at the new SYSCLK?
        if w.latency().to_bits() <= target_flash_latency.to_bits() {
            // We must increase the number of wait states now
            w.set_latency(target_flash_latency)
        } else {
            // We may decrease the number of wait states later
            set_flash_latency_after = true;
        }

        // RM0454 § 3.3.5:
        // > Prefetch is enabled by setting the PRFTEN bit of the FLASH access control register
        // > (FLASH_ACR). This feature is useful if at least one wait state is needed to access the
        // > Flash memory.
        //
        // Enable flash prefetching if we have at least one wait state, and disable it otherwise.
        w.set_prften(target_flash_latency.to_bits() > 0);
    });

    if !set_flash_latency_after {
        // Spin until the effective flash latency is compatible with the clock change
        while FLASH.acr().read().latency().to_bits() < target_flash_latency.to_bits() {}
    }

    // Configure SYSCLK source, HCLK divisor, and PCLK divisor all at once
    let (sw, hpre, ppre) = (sw.into(), config.ahb_pre.into(), config.apb_pre.into());
    RCC.cfgr().modify(|w| {
        w.set_sw(sw);
        w.set_hpre(hpre);
        w.set_ppre(ppre);
    });

    if set_flash_latency_after {
        // We can make the flash require fewer wait states
        // Spin until the SYSCLK changes have taken effect
        loop {
            let cfgr = RCC.cfgr().read();
            if cfgr.sw() == sw && cfgr.hpre() == hpre && cfgr.ppre() == ppre {
                break;
            }
        }

        // Set the flash latency to require fewer wait states
        FLASH.acr().modify(|w| w.set_latency(target_flash_latency));
    }

    let ahb_freq = Hertz(sys_clk) / config.ahb_pre;

    let (apb_freq, apb_tim_freq) = match config.apb_pre {
        APBPrescaler::NotDivided => (ahb_freq.0, ahb_freq.0),
        pre => {
            let pre: Ppre = pre.into();
            let pre: u8 = 1 << (pre.to_bits() - 3);
            let freq = ahb_freq.0 / pre as u32;
            (freq, freq * 2)
        }
    };

    if config.low_power_run {
        assert!(sys_clk <= 2_000_000);
        PWR.cr1().modify(|w| w.set_lpr(true));
    }

    set_freqs(Clocks {
        sys: Hertz(sys_clk),
        ahb1: ahb_freq,
        apb1: Hertz(apb_freq),
        apb1_tim: Hertz(apb_tim_freq),
    });
}
