//! Battary backed SRAM

use core::slice;

use embassy_hal_internal::Peri;

use crate::_generated::{BKPSRAM_BASE, BKPSRAM_SIZE};
use crate::peripherals::BKPSRAM;

/// Struct used to initilize backup sram
pub struct BackupMemory {}

impl BackupMemory {
    /// Setup battery backed sram
    ///
    /// Returns slice to sram and whether the sram was retained
    pub fn new(_backup_sram: Peri<'static, BKPSRAM>) -> (&'static mut [u8], bool) {
        // Assert bksram has been enabled in rcc
        assert!(crate::pac::PWR.bdcr().read().bren() == crate::pac::pwr::vals::Retention::PRESERVED);

        unsafe {
            (
                slice::from_raw_parts_mut(BKPSRAM_BASE as *mut u8, BKPSRAM_SIZE),
                critical_section::with(|_| crate::rcc::BKSRAM_RETAINED),
            )
        }
    }
}
