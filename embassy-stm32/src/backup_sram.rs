//! Battary backed SRAM

use core::{ptr, slice};

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
        #[cfg(not(stm32h7))]
        assert!(crate::pac::PWR.bdcr().read().bren() == crate::pac::pwr::vals::Retention::Preserved);
        #[cfg(stm32h7)]
        assert!(crate::pac::PWR.cr2().read().bren() == crate::pac::pwr::vals::Retention::Preserved);

        Self {
            // SAFETY: It is safe to read this static mut in the CS
            retained: critical_section::with(|_| unsafe { crate::rcc::BKSRAM_RETAINED }),
        }
    }

    /// Returns true if the sram was retained across last reset
    pub const fn is_retained(&self) -> bool {
        self.retained
    }

    /// Get raw pointer to the battery backed memory
    ///
    /// Note that this is not necesserily normal memory, so please do use volatile
    /// and aligned reads/writes unless you know what you are doing.
    pub const fn as_ptr(&self) -> *mut u8 {
        BKPSRAM_BASE as *mut u8
    }

    /// Size of backup sram
    pub const fn size(&self) -> usize {
        BKPSRAM_SIZE
    }

    /// Write bytes to backup sram
    ///
    /// Address is relative start of backup sram
    #[inline]
    pub fn read(&mut self, address: usize, dst: &mut [u8]) {
        assert!(address + dst.len() <= self.size());
        unsafe {
            let (start, mid, end) = slice::from_raw_parts(self.as_ptr().add(address), dst.len()).align_to::<u64>();

            let (buf, dst) = dst.split_at_mut(size_of_val(start));
            for (src, dst) in start.iter().zip(buf) {
                *dst = ptr::read_volatile(src);
            }

            let (buf, dst) = dst.split_at_mut(size_of_val(mid));
            for (src, dst) in mid.iter().zip(buf.chunks_mut(size_of::<u64>())) {
                dst.copy_from_slice(&ptr::read_volatile(src).to_le_bytes());
            }

            let buf = dst;
            for (src, dst) in end.iter().zip(buf) {
                *dst = ptr::read_volatile(src);
            }
        };
    }

    /// Write single byte to backup sram
    ///
    /// Address is relative start of backup sram
    #[inline]
    pub fn write(&mut self, address: usize, src: &[u8]) {
        assert!(address + src.len() <= self.size());
        unsafe {
            let (start, mid, end) =
                slice::from_raw_parts_mut(self.as_ptr().add(address), src.len()).align_to_mut::<u64>();

            let (buf, src) = src.split_at(size_of_val(start));
            for (dst, src) in start.iter_mut().zip(buf) {
                ptr::write_volatile(dst, *src);
            }

            let (buf, src) = src.split_at(size_of_val(mid));
            for (dst, src) in mid.iter_mut().zip(buf.chunks(size_of::<u64>())) {
                ptr::write_volatile(dst, u64::from_le_bytes(src.try_into().unwrap()));
            }

            let buf = src;
            for (dst, src) in end.iter_mut().zip(buf) {
                ptr::write_volatile(dst, *src);
            }
        };
    }
}
