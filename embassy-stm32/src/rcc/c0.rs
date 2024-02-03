use crate::pac::flash::vals::Latency;
use crate::pac::rcc::vals::Sw;
pub use crate::pac::rcc::vals::{Hpre as AHBPrescaler, Hsidiv as HSIPrescaler, Ppre as APBPrescaler};
use crate::pac::{FLASH, RCC};
use crate::time::Hertz;

/// HSI speed
pub const HSI_FREQ: Hertz = Hertz(48_000_000);

/// System clock mux source
#[derive(Clone, Copy)]
pub enum ClockSrc {
    HSE(Hertz),
    HSI(HSIPrescaler),
    LSI,
}

/// Clocks configutation
pub struct Config {
    pub mux: ClockSrc,
    pub ahb_pre: AHBPrescaler,
    pub apb_pre: APBPrescaler,
    pub ls: super::LsConfig,
}

impl Default for Config {
    #[inline]
    fn default() -> Config {
        Config {
            mux: ClockSrc::HSI(HSIPrescaler::DIV1),
            ahb_pre: AHBPrescaler::DIV1,
            apb_pre: APBPrescaler::DIV1,
            ls: Default::default(),
        }
    }
}

pub(crate) unsafe fn init(config: Config) {
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
        ClockSrc::HSE(freq) => {
            // Enable HSE
            RCC.cr().write(|w| w.set_hseon(true));
            while !RCC.cr().read().hserdy() {}

            (freq, Sw::HSE)
        }
        ClockSrc::LSI => {
            // Enable LSI
            RCC.csr2().write(|w| w.set_lsion(true));
            while !RCC.csr2().read().lsirdy() {}
            (super::LSI_FREQ, Sw::LSI)
        }
    };

    let rtc = config.ls.init();

    // Determine the flash latency implied by the target clock speed
    // RM0454 § 3.3.4:
    let target_flash_latency = if sys_clk <= Hertz(24_000_000) {
        Latency::WS0
    } else {
        Latency::WS1
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

        // RM0490 § 3.3.4:
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

    set_clocks!(
        hsi: None,
        lse: None,
        sys: Some(sys_clk),
        hclk1: Some(ahb_freq),
        pclk1: Some(apb_freq),
        pclk1_tim: Some(apb_tim_freq),
        rtc: rtc,
    );
}
