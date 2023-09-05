pub use super::bus::{AHBPrescaler, APBPrescaler, VoltageScale};
use crate::pac::rcc::vals::Adcsel;
use crate::pac::{FLASH, PWR, RCC};
use crate::rcc::bd::{BackupDomain, RtcClockSource};
use crate::rcc::{set_freqs, Clocks};
use crate::time::Hertz;

/// Most of clock setup is copied from stm32l0xx-hal, and adopted to the generated PAC,
/// and with the addition of the init function to configure a system clock.

/// Only the basic setup using the HSE and HSI clocks are supported as of now.

/// HSI speed
pub const HSI_FREQ: Hertz = Hertz(16_000_000);

/// LSI speed
pub const LSI_FREQ: Hertz = Hertz(32_000);

/// HSE32 speed
pub const HSE32_FREQ: Hertz = Hertz(32_000_000);

/// System clock mux source
#[derive(Clone, Copy)]
pub enum ClockSrc {
    MSI(MSIRange),
    HSE32,
    HSI16,
}

#[derive(Clone, Copy, PartialOrd, PartialEq)]
pub enum MSIRange {
    /// Around 100 kHz
    Range0,
    /// Around 200 kHz
    Range1,
    /// Around 400 kHz
    Range2,
    /// Around 800 kHz
    Range3,
    /// Around 1 MHz
    Range4,
    /// Around 2 MHz
    Range5,
    /// Around 4 MHz (reset value)
    Range6,
    /// Around 8 MHz
    Range7,
    /// Around 16 MHz
    Range8,
    /// Around 24 MHz
    Range9,
    /// Around 32 MHz
    Range10,
    /// Around 48 MHz
    Range11,
}

impl MSIRange {
    fn freq(&self) -> u32 {
        match self {
            MSIRange::Range0 => 100_000,
            MSIRange::Range1 => 200_000,
            MSIRange::Range2 => 400_000,
            MSIRange::Range3 => 800_000,
            MSIRange::Range4 => 1_000_000,
            MSIRange::Range5 => 2_000_000,
            MSIRange::Range6 => 4_000_000,
            MSIRange::Range7 => 8_000_000,
            MSIRange::Range8 => 16_000_000,
            MSIRange::Range9 => 24_000_000,
            MSIRange::Range10 => 32_000_000,
            MSIRange::Range11 => 48_000_000,
        }
    }

    fn vos(&self) -> VoltageScale {
        if self > &MSIRange::Range8 {
            VoltageScale::Scale0
        } else {
            VoltageScale::Scale1
        }
    }
}

impl Default for MSIRange {
    fn default() -> MSIRange {
        MSIRange::Range6
    }
}

impl Into<u8> for MSIRange {
    fn into(self) -> u8 {
        match self {
            MSIRange::Range0 => 0b0000,
            MSIRange::Range1 => 0b0001,
            MSIRange::Range2 => 0b0010,
            MSIRange::Range3 => 0b0011,
            MSIRange::Range4 => 0b0100,
            MSIRange::Range5 => 0b0101,
            MSIRange::Range6 => 0b0110,
            MSIRange::Range7 => 0b0111,
            MSIRange::Range8 => 0b1000,
            MSIRange::Range9 => 0b1001,
            MSIRange::Range10 => 0b1010,
            MSIRange::Range11 => 0b1011,
        }
    }
}

#[derive(Clone, Copy)]
pub enum AdcClockSource {
    HSI16,
    PLLPCLK,
    SYSCLK,
}

impl AdcClockSource {
    pub fn adcsel(&self) -> Adcsel {
        match self {
            AdcClockSource::HSI16 => Adcsel::HSI16,
            AdcClockSource::PLLPCLK => Adcsel::PLLPCLK,
            AdcClockSource::SYSCLK => Adcsel::SYSCLK,
        }
    }
}

impl Default for AdcClockSource {
    fn default() -> Self {
        Self::HSI16
    }
}

/// Clocks configutation
pub struct Config {
    pub mux: ClockSrc,
    pub ahb_pre: AHBPrescaler,
    pub shd_ahb_pre: AHBPrescaler,
    pub apb1_pre: APBPrescaler,
    pub apb2_pre: APBPrescaler,
    pub enable_lsi: bool,
    pub enable_rtc_apb: bool,
    pub rtc_mux: RtcClockSource,
    pub adc_clock_source: AdcClockSource,
}

impl Default for Config {
    #[inline]
    fn default() -> Config {
        Config {
            mux: ClockSrc::MSI(MSIRange::default()),
            ahb_pre: AHBPrescaler::NotDivided,
            shd_ahb_pre: AHBPrescaler::NotDivided,
            apb1_pre: APBPrescaler::NotDivided,
            apb2_pre: APBPrescaler::NotDivided,
            enable_lsi: false,
            enable_rtc_apb: false,
            rtc_mux: RtcClockSource::LSI,
            adc_clock_source: AdcClockSource::default(),
        }
    }
}

#[repr(u8)]
pub enum Lsedrv {
    Low = 0,
    MediumLow = 1,
    MediumHigh = 2,
    High = 3,
}

pub(crate) unsafe fn init(config: Config) {
    let (sys_clk, sw, vos) = match config.mux {
        ClockSrc::HSI16 => (HSI_FREQ.0, 0x01, VoltageScale::Scale1),
        ClockSrc::HSE32 => (HSE32_FREQ.0, 0x02, VoltageScale::Scale0),
        ClockSrc::MSI(range) => (range.freq(), 0x00, range.vos()),
    };

    let ahb_freq: u32 = match config.ahb_pre {
        AHBPrescaler::NotDivided => sys_clk,
        pre => {
            let pre: u8 = pre.into();
            let pre = 1 << (pre as u32 - 7);
            sys_clk / pre
        }
    };

    let shd_ahb_freq: u32 = match config.shd_ahb_pre {
        AHBPrescaler::NotDivided => sys_clk,
        pre => {
            let pre: u8 = pre.into();
            let pre = 1 << (pre as u32 - 7);
            sys_clk / pre
        }
    };

    let (apb1_freq, apb1_tim_freq) = match config.apb1_pre {
        APBPrescaler::NotDivided => (ahb_freq, ahb_freq),
        pre => {
            let pre: u8 = pre.into();
            let pre: u8 = 1 << (pre - 3);
            let freq = ahb_freq / pre as u32;
            (freq, freq * 2)
        }
    };

    let (apb2_freq, apb2_tim_freq) = match config.apb2_pre {
        APBPrescaler::NotDivided => (ahb_freq, ahb_freq),
        pre => {
            let pre: u8 = pre.into();
            let pre: u8 = 1 << (pre - 3);
            let freq = ahb_freq / pre as u32;
            (freq, freq * 2)
        }
    };

    // Adjust flash latency
    let flash_clk_src_freq: u32 = shd_ahb_freq;
    let ws = match vos {
        VoltageScale::Scale0 => match flash_clk_src_freq {
            0..=18_000_000 => 0b000,
            18_000_001..=36_000_000 => 0b001,
            _ => 0b010,
        },
        VoltageScale::Scale1 => match flash_clk_src_freq {
            0..=6_000_000 => 0b000,
            6_000_001..=12_000_000 => 0b001,
            _ => 0b010,
        },
    };

    FLASH.acr().modify(|w| {
        w.set_latency(ws);
    });

    while FLASH.acr().read().latency() != ws {}

    match config.rtc_mux {
        RtcClockSource::LSE => {
            // 1. Unlock the backup domain
            PWR.cr1().modify(|w| w.set_dbp(true));

            // 2. Setup the LSE
            RCC.bdcr().modify(|w| {
                // Enable LSE
                w.set_lseon(true);
                // Max drive strength
                // TODO: should probably be settable
                w.set_lsedrv(Lsedrv::High as u8); //---// PAM - should not be commented
            });

            // Wait until LSE is running
            while !RCC.bdcr().read().lserdy() {}

            BackupDomain::set_rtc_clock_source(RtcClockSource::LSE);
        }
        RtcClockSource::LSI => {
            // Turn on the internal 32 kHz LSI oscillator
            RCC.csr().modify(|w| w.set_lsion(true));

            // Wait until LSI is running
            while !RCC.csr().read().lsirdy() {}

            BackupDomain::set_rtc_clock_source(RtcClockSource::LSI);
        }
        _ => unreachable!(),
    }

    match config.mux {
        ClockSrc::HSI16 => {
            // Enable HSI16
            RCC.cr().write(|w| w.set_hsion(true));
            while !RCC.cr().read().hsirdy() {}
        }
        ClockSrc::HSE32 => {
            // Enable HSE32
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
                w.set_msirange(range.into());
                w.set_msion(true);

                if let RtcClockSource::LSE = config.rtc_mux {
                    // If LSE is enabled, enable calibration of MSI
                    w.set_msipllen(true);
                } else {
                    w.set_msipllen(false);
                }
            });
            while !RCC.cr().read().msirdy() {}
        }
    }

    if config.enable_rtc_apb {
        // enable peripheral clock for communication
        crate::pac::RCC.apb1enr1().modify(|w| w.set_rtcapben(true));

        // read to allow the pwr clock to enable
        crate::pac::PWR.cr1().read();
    }

    RCC.extcfgr().modify(|w| {
        if config.shd_ahb_pre == AHBPrescaler::NotDivided {
            w.set_shdhpre(0);
        } else {
            w.set_shdhpre(config.shd_ahb_pre.into());
        }
    });

    RCC.cfgr().modify(|w| {
        w.set_sw(sw.into());
        if config.ahb_pre == AHBPrescaler::NotDivided {
            w.set_hpre(0);
        } else {
            w.set_hpre(config.ahb_pre.into());
        }
        w.set_ppre1(config.apb1_pre.into());
        w.set_ppre2(config.apb2_pre.into());
    });

    // ADC clock MUX
    RCC.ccipr().modify(|w| w.set_adcsel(config.adc_clock_source.adcsel()));

    // TODO: switch voltage range

    if config.enable_lsi {
        let csr = RCC.csr().read();
        if !csr.lsion() {
            RCC.csr().modify(|w| w.set_lsion(true));
            while !RCC.csr().read().lsirdy() {}
        }
    }

    set_freqs(Clocks {
        sys: Hertz(sys_clk),
        ahb1: Hertz(ahb_freq),
        ahb2: Hertz(ahb_freq),
        ahb3: Hertz(shd_ahb_freq),
        apb1: Hertz(apb1_freq),
        apb2: Hertz(apb2_freq),
        apb3: Hertz(shd_ahb_freq),
        apb1_tim: Hertz(apb1_tim_freq),
        apb2_tim: Hertz(apb2_tim_freq),
    });
}
