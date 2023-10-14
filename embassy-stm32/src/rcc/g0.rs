use crate::pac::flash::vals::Latency;
use crate::pac::rcc::vals::{self, Sw};
pub use crate::pac::rcc::vals::{
    Hpre as AHBPrescaler, Hsidiv as HSI16Prescaler, Pllm, Plln, Pllp, Pllq, Pllr, Ppre as APBPrescaler,
};
use crate::pac::{FLASH, PWR, RCC};
use crate::rcc::{set_freqs, Clocks};
use crate::time::Hertz;

/// HSI speed
pub const HSI_FREQ: Hertz = Hertz(16_000_000);

/// System clock mux source
#[derive(Clone, Copy)]
pub enum ClockSrc {
    HSE(Hertz),
    HSI16(HSI16Prescaler),
    PLL(PllConfig),
    LSI,
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
    pub n: Plln,
    /// The final divisor for `PLLRCLK` output which drives the system clock
    pub r: Pllr,

    /// The divisor for the `PLLQCLK` output, if desired
    pub q: Option<Pllq>,

    /// The divisor for the `PLLPCLK` output, if desired
    pub p: Option<Pllp>,
}

impl Default for PllConfig {
    #[inline]
    fn default() -> PllConfig {
        // HSI16 / 1 * 8 / 2 = 64 MHz
        PllConfig {
            source: PllSrc::HSI16,
            m: Pllm::DIV1,
            n: Plln::MUL8,
            r: Pllr::DIV2,
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

/// Clocks configutation
pub struct Config {
    pub mux: ClockSrc,
    pub ahb_pre: AHBPrescaler,
    pub apb_pre: APBPrescaler,
    pub low_power_run: bool,
    pub ls: super::LsConfig,
}

impl Default for Config {
    #[inline]
    fn default() -> Config {
        Config {
            mux: ClockSrc::HSI16(HSI16Prescaler::DIV1),
            ahb_pre: AHBPrescaler::DIV1,
            apb_pre: APBPrescaler::DIV1,
            low_power_run: false,
            ls: Default::default(),
        }
    }
}

impl PllConfig {
    pub(crate) fn init(self) -> Hertz {
        let (src, input_freq) = match self.source {
            PllSrc::HSI16 => (vals::Pllsrc::HSI16, HSI_FREQ),
            PllSrc::HSE(freq) => (vals::Pllsrc::HSE, freq),
        };

        let m_freq = input_freq / self.m;
        // RM0454 § 5.4.4:
        // > Caution: The software must set these bits so that the PLL input frequency after the
        // > /M divider is between 2.66 and 16 MHz.
        debug_assert!(m_freq.0 >= 2_660_000 && m_freq.0 <= 16_000_000);

        let n_freq = m_freq * self.n as u32;
        // RM0454 § 5.4.4:
        // > Caution: The software must set these bits so that the VCO output frequency is between
        // > 64 and 344 MHz.
        debug_assert!(n_freq.0 >= 64_000_000 && n_freq.0 <= 344_000_000);

        let r_freq = n_freq / self.r;
        // RM0454 § 5.4.4:
        // > Caution: The software must set this bitfield so as not to exceed 64 MHz on this clock.
        debug_assert!(r_freq.0 <= 64_000_000);

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

        // Configure PLLCFGR
        RCC.pllcfgr().modify(|w| {
            w.set_pllr(self.r);
            w.set_pllren(false);
            w.set_pllq(self.q.unwrap_or(Pllq::DIV2));
            w.set_pllqen(false);
            w.set_pllp(self.p.unwrap_or(Pllp::DIV2));
            w.set_pllpen(false);
            w.set_plln(self.n);
            w.set_pllm(self.m);
            w.set_pllsrc(src)
        });

        // > 4. Enable the PLL again by setting PLLON to 1.
        RCC.cr().modify(|w| w.set_pllon(true));

        // Wait for the PLL to become ready
        while !RCC.cr().read().pllrdy() {}

        // > 5. Enable the desired PLL outputs by configuring PLLPEN, PLLQEN, and PLLREN in PLL
        // > configuration register (RCC_PLLCFGR).
        RCC.pllcfgr().modify(|w| {
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
            RCC.cr().write(|w| {
                w.set_hsidiv(div);
                w.set_hsion(true)
            });
            while !RCC.cr().read().hsirdy() {}

            (HSI_FREQ / div, Sw::HSI)
        }
        ClockSrc::HSE(freq) => {
            // Enable HSE
            RCC.cr().write(|w| w.set_hseon(true));
            while !RCC.cr().read().hserdy() {}

            (freq, Sw::HSE)
        }
        ClockSrc::PLL(pll) => {
            let freq = pll.init();
            (freq, Sw::PLLRCLK)
        }
        ClockSrc::LSI => {
            // Enable LSI
            RCC.csr().write(|w| w.set_lsion(true));
            while !RCC.csr().read().lsirdy() {}
            (super::LSI_FREQ, Sw::LSI)
        }
    };

    // Determine the flash latency implied by the target clock speed
    // RM0454 § 3.3.4:
    let target_flash_latency = if sys_clk.0 <= 24_000_000 {
        Latency::WS0
    } else if sys_clk.0 <= 48_000_000 {
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
    let (sw, hpre, ppre) = (sw.into(), config.ahb_pre, config.apb_pre);
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

    let ahb_freq = sys_clk / config.ahb_pre;

    let (apb_freq, apb_tim_freq) = match config.apb_pre {
        APBPrescaler::DIV1 => (ahb_freq, ahb_freq),
        pre => {
            let freq = ahb_freq / pre;
            (freq, freq * 2u32)
        }
    };

    if config.low_power_run {
        assert!(sys_clk.0 <= 2_000_000);
        PWR.cr1().modify(|w| w.set_lpr(true));
    }

    let rtc = config.ls.init();

    set_freqs(Clocks {
        sys: sys_clk,
        ahb1: ahb_freq,
        apb1: apb_freq,
        apb1_tim: apb_tim_freq,
        rtc,
    });
}
