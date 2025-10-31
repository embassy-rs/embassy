//! Battary backed SRAM

use core::slice;

use crate::_generated::{BKPSRAM_BASE, BKPSRAM_SIZE};
use embassy_hal_internal::Peri;

use crate::peripherals::BKPSRAM;

/// Value used to determine whether memory has been corrupted
const MAGIC_VALUE: u32 = 0x1759_0abc;

/// Setup battery backed sram
pub fn init(_backup_sram: Peri<'static, BKPSRAM>) -> (&'static mut [u8], bool) {
    assert!(crate::pac::PWR.bdcr().read().bren() == crate::pac::pwr::vals::Retention::PRESERVED);

    let magic_ptr = BKPSRAM_BASE as *mut u32;

    let is_retained = unsafe {
        let is_retained = magic_ptr.read_volatile() == MAGIC_VALUE;
        magic_ptr.write_volatile(MAGIC_VALUE);

        is_retained
    };

    let base_ptr = unsafe { (BKPSRAM_BASE as *mut u8).add(size_of::<u32>()) };
    let bytes: &'static mut [u8] = unsafe { slice::from_raw_parts_mut(base_ptr, BKPSRAM_SIZE - size_of::<u32>()) };

    (bytes, is_retained)
}
