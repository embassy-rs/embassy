pub use super::bus::{AHBPrescaler, APBPrescaler};
use crate::pac::flash::vals::Latency;
use crate::pac::rcc::vals::{Hsidiv, Ppre, Sw};
use crate::pac::{FLASH, RCC};
use crate::rcc::{set_freqs, Clocks};
use crate::time::Hertz;

/// HSI speed
pub const HSI_FREQ: Hertz = Hertz(48_000_000);

/// LSI speed
pub const LSI_FREQ: Hertz = Hertz(32_000);

/// System clock mux source
#[derive(Clone, Copy)]
pub enum ClockSrc {
    HSE(Hertz),
    HSI(HSIPrescaler),
    LSI,
}

#[derive(Clone, Copy)]
pub enum HSIPrescaler {
    NotDivided,
    Div2,
    Div4,
    Div8,
    Div16,
    Div32,
    Div64,
    Div128,
}

impl Into<Hsidiv> for HSIPrescaler {
    fn into(self) -> Hsidiv {
        match self {
            HSIPrescaler::NotDivided => Hsidiv::DIV1,
            HSIPrescaler::Div2 => Hsidiv::DIV2,
            HSIPrescaler::Div4 => Hsidiv::DIV4,
            HSIPrescaler::Div8 => Hsidiv::DIV8,
            HSIPrescaler::Div16 => Hsidiv::DIV16,
            HSIPrescaler::Div32 => Hsidiv::DIV32,
            HSIPrescaler::Div64 => Hsidiv::DIV64,
            HSIPrescaler::Div128 => Hsidiv::DIV128,
        }
    }
}

/// Clocks configutation
pub struct Config {
    pub mux: ClockSrc,
    pub ahb_pre: AHBPrescaler,
    pub apb_pre: APBPrescaler,
}

impl Default for Config {
    #[inline]
    fn default() -> Config {
        Config {
            mux: ClockSrc::HSI(HSIPrescaler::NotDivided),
            ahb_pre: AHBPrescaler::NotDivided,
            apb_pre: APBPrescaler::NotDivided,
        }
    }
}

pub(crate) unsafe fn init(config: Config) {
    let (sys_clk, sw) = match config.mux {
        ClockSrc::HSI(div) => {
            // Enable HSI
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
        ClockSrc::LSI => {
            // Enable LSI
            RCC.csr2().write(|w| w.set_lsion(true));
            while !RCC.csr2().read().lsirdy() {}
            (LSI_FREQ.0, Sw::LSI)
        }
    };

    // Determine the flash latency implied by the target clock speed
    // RM0454 § 3.3.4:
    let target_flash_latency = if sys_clk <= 24_000_000 {
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

    let ahb_div = match config.ahb_pre {
        AHBPrescaler::NotDivided => 1,
        AHBPrescaler::Div2 => 2,
        AHBPrescaler::Div4 => 4,
        AHBPrescaler::Div8 => 8,
        AHBPrescaler::Div16 => 16,
        AHBPrescaler::Div64 => 64,
        AHBPrescaler::Div128 => 128,
        AHBPrescaler::Div256 => 256,
        AHBPrescaler::Div512 => 512,
    };
    let ahb_freq = sys_clk / ahb_div;

    let (apb_freq, apb_tim_freq) = match config.apb_pre {
        APBPrescaler::NotDivided => (ahb_freq, ahb_freq),
        pre => {
            let pre: Ppre = pre.into();
            let pre: u8 = 1 << (pre.to_bits() - 3);
            let freq = ahb_freq / pre as u32;
            (freq, freq * 2)
        }
    };

    set_freqs(Clocks {
        sys: Hertz(sys_clk),
        ahb1: Hertz(ahb_freq),
        apb1: Hertz(apb_freq),
        apb1_tim: Hertz(apb_tim_freq),
    });
}
