#![allow(unused)]

use crate::pac::crs::vals::Syncsrc;
use crate::pac::{CRS, RCC};
use crate::rcc::sealed::RccPeripheral;
use crate::time::Hertz;

/// HSI48 speed
pub const HSI48_FREQ: Hertz = Hertz(48_000_000);

/// Configuration for the HSI48 clock
#[derive(Clone, Copy, Debug)]
pub struct Hsi48Config {
    /// Enable CRS Sync from USB Start Of Frame (SOF) events.
    /// Required if HSI48 is going to be used as USB clock.
    ///
    /// Other use cases of CRS are not supported yet.
    pub sync_from_usb: bool,
}

impl Default for Hsi48Config {
    fn default() -> Self {
        Self { sync_from_usb: false }
    }
}

pub(crate) fn init_hsi48(config: Hsi48Config) -> Hertz {
    // Enable VREFINT reference for HSI48 oscillator
    #[cfg(stm32l0)]
    crate::pac::SYSCFG.cfgr3().modify(|w| {
        w.set_enref_hsi48(true);
        w.set_en_vrefint(true);
    });

    // Enable HSI48
    #[cfg(not(any(stm32u5, stm32g0, stm32h5, stm32h7, stm32u5, stm32wba, stm32f0)))]
    let r = RCC.crrcr();
    #[cfg(any(stm32u5, stm32g0, stm32h5, stm32h7, stm32u5, stm32wba))]
    let r = RCC.cr();
    #[cfg(any(stm32f0))]
    let r = RCC.cr2();

    r.modify(|w| w.set_hsi48on(true));
    while r.read().hsi48rdy() == false {}

    if config.sync_from_usb {
        crate::peripherals::CRS::enable_and_reset();

        CRS.cfgr().modify(|w| {
            w.set_syncsrc(Syncsrc::USB);
        });

        // These are the correct settings for standard USB operation. If other settings
        // are needed there will need to be additional config options for the CRS.
        crate::pac::CRS.cr().modify(|w| {
            w.set_autotrimen(true);
            w.set_cen(true);
        });
    }

    HSI48_FREQ
}
