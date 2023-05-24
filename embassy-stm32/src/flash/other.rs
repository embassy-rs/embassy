#![allow(unused)]

use super::{Error, FlashRegion, FlashSector, FLASH_REGIONS, WRITE_SIZE};

pub const fn set_default_layout() {}

pub const fn get_flash_regions() -> &'static [&'static FlashRegion] {
    &FLASH_REGIONS
}

pub(crate) unsafe fn on_interrupt(_: *mut ()) {
    unimplemented!();
}

pub(crate) unsafe fn lock() {
    unimplemented!();
}
pub(crate) unsafe fn unlock() {
    unimplemented!();
}
pub(crate) unsafe fn enable_blocking_write() {
    unimplemented!();
}
pub(crate) unsafe fn disable_blocking_write() {
    unimplemented!();
}
pub(crate) unsafe fn write_blocking(_start_address: u32, _buf: &[u8; WRITE_SIZE]) -> Result<(), Error> {
    unimplemented!();
}
pub(crate) unsafe fn erase_sector_blocking(_sector: &FlashSector) -> Result<(), Error> {
    unimplemented!();
}
pub(crate) unsafe fn clear_all_err() {
    unimplemented!();
}
