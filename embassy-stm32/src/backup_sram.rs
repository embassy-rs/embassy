//! Battary backed SRAM

use embassy_hal_internal::Peri;

use crate::_generated::{BKPSRAM_BASE, BKPSRAM_SIZE};
use crate::peripherals::BKPSRAM;

/// Struct used to initilize backup sram
pub struct BackupMemory {
    // true if the sram was retained across last reset
    retained: bool,
}

impl BackupMemory {
    /// Setup battery backed sram
    ///
    /// Returns slice to sram and whether the sram was retained
    pub fn new(_backup_sram: Peri<'static, BKPSRAM>) -> Self {
        // Assert bksram has been enabled in rcc
        assert!(crate::pac::PWR.bdcr().read().bren() == crate::pac::pwr::vals::Retention::Preserved);

        Self {
            // SAFETY: It is safe to read this static mut in the CS
            retained: critical_section::with(|_| unsafe { crate::rcc::BKSRAM_RETAINED }),
        }
    }

    /// Returns true if the sram was retained across last reset
    pub fn is_retained(&self) -> bool {
        self.retained
    }

    /// Get raw pointer to the battery backed memory
    ///
    /// Note that this is not necesserily normal memory, so please do use volatile
    /// and aligned reads/writes unless you know what you are doing.
    pub fn as_ptr(&self) -> *mut u8 {
        BKPSRAM_BASE as *mut u8
    }

    /// Size of backup sram
    pub fn size(&self) -> usize {
        BKPSRAM_SIZE
    }

    /// Read single byte from backup sram
    ///
    /// Address is relative start of backup sram
    pub fn get(&self, address: usize) -> u8 {
        assert!(address < self.size());

        unsafe { self.as_ptr().add(address).read_volatile() }
    }

    /// Write single byte to backup sram
    ///
    /// Address is relative start of backup sram
    pub fn set(&mut self, address: usize, byte: u8) {
        assert!(address < self.size());

        unsafe { self.as_ptr().add(address).write_volatile(byte) }
    }
}
