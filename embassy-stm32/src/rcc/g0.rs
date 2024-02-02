use crate::pac::flash::vals::Latency;
use crate::pac::rcc::vals::{self, Sw};
pub use crate::pac::rcc::vals::{
    Hpre as AHBPrescaler, Hsidiv as HSIPrescaler, Pllm, Plln, Pllp, Pllq, Pllr, Ppre as APBPrescaler,
};
use crate::pac::{FLASH, PWR, RCC};
use crate::time::Hertz;

/// HSI speed
pub const HSI_FREQ: Hertz = Hertz(16_000_000);

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum HseMode {
    /// crystal/ceramic oscillator (HSEBYP=0)
    Oscillator,
    /// external analog clock (low swing) (HSEBYP=1)
    Bypass,
}

/// System clock mux source
#[derive(Clone, Copy)]
pub enum ClockSrc {
    HSE(Hertz, HseMode),
    HSI(HSIPrescaler),
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
    pub source: PllSource,
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
        // HSI / 1 * 8 / 2 = 64 MHz
        PllConfig {
            source: PllSource::HSI,
            m: Pllm::DIV1,
            n: Plln::MUL8,
            r: Pllr::DIV2,
            q: None,
            p: None,
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum PllSource {
    HSI,
    HSE(Hertz, HseMode),
}

/// Sets the source for the 48MHz clock to the USB peripheral.
#[cfg(any(stm32g0b1, stm32g0c1, stm32g0b0))]
pub enum UsbSrc {
    /// Use the High Speed Internal Oscillator. The CRS must be used to calibrate the
    /// oscillator to comply with the USB specification for oscillator tolerance.
    #[cfg(any(stm32g0b1, stm32g0c1))]
    Hsi48(super::Hsi48Config),
    /// Use the PLLQ output. The PLL must be configured to output a 48MHz clock. The
    /// PLL needs to be using the HSE source to comply with the USB specification for oscillator
    /// tolerance.
    PllQ,
    /// Use the HSE source directly.  The HSE must be a 48MHz source.  The HSE source must comply
    /// with the USB specification for oscillator tolerance.
    HSE,
}

/// Clocks configutation
pub struct Config {
    pub mux: ClockSrc,
    pub ahb_pre: AHBPrescaler,
    pub apb_pre: APBPrescaler,
    pub low_power_run: bool,
    pub ls: super::LsConfig,
    #[cfg(any(stm32g0b1, stm32g0c1, stm32g0b0))]
    pub usb_src: Option<UsbSrc>,
}

impl Default for Config {
    #[inline]
    fn default() -> Config {
        Config {
            mux: ClockSrc::HSI(HSIPrescaler::DIV1),
            ahb_pre: AHBPrescaler::DIV1,
            apb_pre: APBPrescaler::DIV1,
            low_power_run: false,
            ls: Default::default(),
            #[cfg(any(stm32g0b1, stm32g0c1, stm32g0b0))]
            usb_src: None,
        }
    }
}

impl PllConfig {
    pub(crate) fn init(self) -> (Hertz, Option<Hertz>, Option<Hertz>) {
        let (src, input_freq) = match self.source {
            PllSource::HSI => (vals::Pllsrc::HSI, HSI_FREQ),
            PllSource::HSE(freq, _) => (vals::Pllsrc::HSE, freq),
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

        let q_freq = self.q.map(|q| n_freq / q);
        let p_freq = self.p.map(|p| n_freq / p);

        // RM0454 § 5.2.3:
        // > To modify the PLL configuration, proceed as follows:
        // > 1. Disable the PLL by setting PLLON to 0 in Clock control register (RCC_CR).
        RCC.cr().modify(|w| w.set_pllon(false));

        // > 2. Wait until PLLRDY is cleared. The PLL is now fully stopped.
        while RCC.cr().read().pllrdy() {}

        // > 3. Change the desired parameter.
        // Enable whichever clock source we're using, and wait for it to become ready
        match self.source {
            PllSource::HSI => {
                RCC.cr().write(|w| w.set_hsion(true));
                while !RCC.cr().read().hsirdy() {}
            }
            PllSource::HSE(_, mode) => {
                RCC.cr().write(|w| {
                    w.set_hsebyp(mode != HseMode::Oscillator);
                    w.set_hseon(true);
                });
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

        (r_freq, q_freq, p_freq)
    }
}

pub(crate) unsafe fn init(config: Config) {
    let mut pll1_q_freq = None;
    let mut pll1_p_freq = None;

    let (sys_clk, sw) = match config.mux {
        ClockSrc::HSI(div) => {
            // Enable HSI
            RCC.cr().write(|w| {
                w.set_hsidiv(div);
                w.set_hsion(true)
            });
            while !RCC.cr().read().hsirdy() {}

            (HSI_FREQ / div, Sw::HSI)
        }
        ClockSrc::HSE(freq, mode) => {
            // Enable HSE
            RCC.cr().write(|w| {
                w.set_hseon(true);
                w.set_hsebyp(mode != HseMode::Oscillator);
            });
            while !RCC.cr().read().hserdy() {}

            (freq, Sw::HSE)
        }
        ClockSrc::PLL(pll) => {
            let (r_freq, q_freq, p_freq) = pll.init();

            pll1_q_freq = q_freq;
            pll1_p_freq = p_freq;

            (r_freq, Sw::PLL1_R)
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
    let lse_freq = config.ls.lse.map(|lse| lse.frequency);

    let hsi_freq = (sw == Sw::HSI).then_some(HSI_FREQ);
    let hsi_div_8_freq = hsi_freq.map(|f| f / 8u32);
    let lsi_freq = (sw == Sw::LSI).then_some(super::LSI_FREQ);
    let hse_freq = (sw == Sw::HSE).then_some(sys_clk);

    #[cfg(any(stm32g0b1, stm32g0c1, stm32g0b0))]
    let hsi48_freq = config.usb_src.and_then(|config| {
        match config {
            UsbSrc::PllQ => {
                // Make sure the PLLQ is enabled and running at 48Mhz
                assert!(pll1_q_freq.is_some() && pll1_q_freq.unwrap().0 == 48_000_000);
                RCC.ccipr2()
                    .modify(|w| w.set_usbsel(crate::pac::rcc::vals::Usbsel::PLL1_Q));
                None
            }
            UsbSrc::HSE => {
                // Make sure the HSE is enabled and running at 48Mhz
                assert!(hse_freq.is_some() && hse_freq.unwrap().0 == 48_000_000);
                RCC.ccipr2()
                    .modify(|w| w.set_usbsel(crate::pac::rcc::vals::Usbsel::HSE));
                None
            }
            #[cfg(any(stm32g0b1, stm32g0c1))]
            UsbSrc::Hsi48(config) => {
                let freq = super::init_hsi48(config);
                RCC.ccipr2()
                    .modify(|w| w.set_usbsel(crate::pac::rcc::vals::Usbsel::HSI48));
                Some(freq)
            }
        }
    });
    #[cfg(not(any(stm32g0b1, stm32g0c1, stm32g0b0)))]
    let hsi48_freq: Option<Hertz> = None;

    set_clocks!(
        sys: Some(sys_clk),
        hclk1: Some(ahb_freq),
        pclk1: Some(apb_freq),
        pclk1_tim: Some(apb_tim_freq),
        hsi: hsi_freq,
        hsi48: hsi48_freq,
        hsi_div_8: hsi_div_8_freq,
        hse: hse_freq,
        lse: lse_freq,
        lsi: lsi_freq,
        pll1_q: pll1_q_freq,
        pll1_p: pll1_p_freq,
        rtc: rtc,
    );
}
