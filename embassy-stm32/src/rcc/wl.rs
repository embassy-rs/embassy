pub use crate::pac::pwr::vals::Vos as VoltageScale;
use crate::pac::rcc::vals::Sw;
pub use crate::pac::rcc::vals::{
    Adcsel as AdcClockSource, Hpre as AHBPrescaler, Msirange as MSIRange, Pllm, Plln, Pllp, Pllq, Pllr,
    Pllsrc as PllSource, Ppre as APBPrescaler,
};
use crate::pac::{FLASH, RCC};
use crate::rcc::{set_freqs, Clocks};
use crate::time::Hertz;

/// HSI speed
pub const HSI_FREQ: Hertz = Hertz(16_000_000);

/// HSE speed
pub const HSE_FREQ: Hertz = Hertz(32_000_000);

/// System clock mux source
#[derive(Clone, Copy)]
pub enum ClockSrc {
    MSI(MSIRange),
    HSE,
    HSI16,
}

/// Clocks configutation
pub struct Config {
    pub mux: ClockSrc,
    pub ahb_pre: AHBPrescaler,
    pub shd_ahb_pre: AHBPrescaler,
    pub apb1_pre: APBPrescaler,
    pub apb2_pre: APBPrescaler,
    pub adc_clock_source: AdcClockSource,
    pub ls: super::LsConfig,
}

impl Default for Config {
    #[inline]
    fn default() -> Config {
        Config {
            mux: ClockSrc::MSI(MSIRange::RANGE4M),
            ahb_pre: AHBPrescaler::DIV1,
            shd_ahb_pre: AHBPrescaler::DIV1,
            apb1_pre: APBPrescaler::DIV1,
            apb2_pre: APBPrescaler::DIV1,
            adc_clock_source: AdcClockSource::HSI16,
            ls: Default::default(),
        }
    }
}

pub(crate) unsafe fn init(config: Config) {
    let (sys_clk, sw, vos) = match config.mux {
        ClockSrc::HSI16 => (HSI_FREQ, Sw::HSI16, VoltageScale::RANGE2),
        ClockSrc::HSE => (HSE_FREQ, Sw::HSE, VoltageScale::RANGE1),
        ClockSrc::MSI(range) => (msirange_to_hertz(range), Sw::MSI, msirange_to_vos(range)),
    };

    let ahb_freq = sys_clk / config.ahb_pre;
    let shd_ahb_freq = sys_clk / config.shd_ahb_pre;

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

    // Adjust flash latency
    let flash_clk_src_freq = shd_ahb_freq;
    let ws = match vos {
        VoltageScale::RANGE1 => match flash_clk_src_freq.0 {
            0..=18_000_000 => 0b000,
            18_000_001..=36_000_000 => 0b001,
            _ => 0b010,
        },
        VoltageScale::RANGE2 => match flash_clk_src_freq.0 {
            0..=6_000_000 => 0b000,
            6_000_001..=12_000_000 => 0b001,
            _ => 0b010,
        },
        _ => unreachable!(),
    };

    FLASH.acr().modify(|w| {
        w.set_latency(ws);
    });

    while FLASH.acr().read().latency() != ws {}

    match config.mux {
        ClockSrc::HSI16 => {
            // Enable HSI16
            RCC.cr().write(|w| w.set_hsion(true));
            while !RCC.cr().read().hsirdy() {}
        }
        ClockSrc::HSE => {
            // Enable HSE
            RCC.cr().write(|w| {
                w.set_hsebyppwr(true);
                w.set_hseon(true);
            });
            while !RCC.cr().read().hserdy() {}
        }
        ClockSrc::MSI(range) => {
            let cr = RCC.cr().read();
            assert!(!cr.msion() || cr.msirdy());
            RCC.cr().write(|w| {
                w.set_msirgsel(true);
                w.set_msirange(range);
                w.set_msion(true);

                // If LSE is enabled, enable calibration of MSI
                w.set_msipllen(config.ls.lse.is_some());
            });
            while !RCC.cr().read().msirdy() {}
        }
    }

    RCC.extcfgr().modify(|w| {
        w.set_shdhpre(config.shd_ahb_pre);
    });

    RCC.cfgr().modify(|w| {
        w.set_sw(sw.into());
        w.set_hpre(config.ahb_pre);
        w.set_ppre1(config.apb1_pre);
        w.set_ppre2(config.apb2_pre);
    });

    // ADC clock MUX
    RCC.ccipr().modify(|w| w.set_adcsel(config.adc_clock_source));

    // TODO: switch voltage range

    let rtc = config.ls.init();

    set_freqs(Clocks {
        sys: sys_clk,
        ahb1: ahb_freq,
        ahb2: ahb_freq,
        ahb3: shd_ahb_freq,
        apb1: apb1_freq,
        apb2: apb2_freq,
        apb3: shd_ahb_freq,
        apb1_tim: apb1_tim_freq,
        apb2_tim: apb2_tim_freq,
        rtc,
    });
}

fn msirange_to_hertz(range: MSIRange) -> Hertz {
    match range {
        MSIRange::RANGE100K => Hertz(100_000),
        MSIRange::RANGE200K => Hertz(200_000),
        MSIRange::RANGE400K => Hertz(400_000),
        MSIRange::RANGE800K => Hertz(800_000),
        MSIRange::RANGE1M => Hertz(1_000_000),
        MSIRange::RANGE2M => Hertz(2_000_000),
        MSIRange::RANGE4M => Hertz(4_000_000),
        MSIRange::RANGE8M => Hertz(8_000_000),
        MSIRange::RANGE16M => Hertz(16_000_000),
        MSIRange::RANGE24M => Hertz(24_000_000),
        MSIRange::RANGE32M => Hertz(32_000_000),
        MSIRange::RANGE48M => Hertz(48_000_000),
        _ => unreachable!(),
    }
}

fn msirange_to_vos(range: MSIRange) -> VoltageScale {
    if range.to_bits() > MSIRange::RANGE16M.to_bits() {
        VoltageScale::RANGE1
    } else {
        VoltageScale::RANGE2
    }
}
