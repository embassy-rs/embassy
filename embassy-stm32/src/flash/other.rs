#[allow(unused)]
use super::{Error, FlashSector, WRITE_SIZE};

pub(crate) unsafe fn lock() {
    unimplemented!();
}
pub(crate) unsafe fn unlock() {
    unimplemented!();
}
pub(crate) unsafe fn begin_write() {
    unimplemented!();
}
pub(crate) unsafe fn end_write() {
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
pub(crate) fn get_sector(_address: u32) -> FlashSector {
    unimplemented!();
}
