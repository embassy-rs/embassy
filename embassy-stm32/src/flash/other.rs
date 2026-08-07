#![allow(unused)]

use super::{Error, FlashSector, WRITE_SIZE};

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
pub(crate) unsafe fn blocking_write(_start_address: u32, _buf: &[u8; WRITE_SIZE]) -> Result<(), Error> {
    unimplemented!();
}
pub(crate) unsafe fn blocking_erase_sector(_sector: &FlashSector) -> Result<(), Error> {
    unimplemented!();
}
pub(crate) unsafe fn clear_all_err() {
    unimplemented!();
}
