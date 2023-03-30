use embedded_storage::nor_flash::{NorFlashError, NorFlashErrorKind};

#[cfg_attr(any(flash_l0, flash_l1, flash_l4, flash_wl, flash_wb), path = "l.rs")]
#[cfg_attr(flash_f3, path = "f3.rs")]
#[cfg_attr(flash_f4, path = "f4.rs")]
#[cfg_attr(flash_f7, path = "f7.rs")]
#[cfg_attr(flash_h7, path = "h7.rs")]
mod family;

#[cfg(not(any(
    flash_l0, flash_l1, flash_l4, flash_wl, flash_wb, flash_f3, flash_f4, flash_f7, flash_h7
)))]
mod family {
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
}

#[cfg(flash)]
mod common;

#[cfg(flash)]
pub use common::*;

pub struct FlashRegion {
    pub base: u32,
    pub size: u32,
    pub erase_size: u32,
    pub write_size: u32,
    pub erase_value: u8,
}

#[derive(Debug, PartialEq)]
pub struct FlashSector {
    pub index: u8,
    pub start: u32,
    pub size: u32,
}

impl Drop for FlashLayout<'_> {
    fn drop(&mut self) {
        unsafe { family::lock() };
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    Prog,
    Size,
    Miss,
    Seq,
    Protected,
    Unaligned,
    Parallelism,
}

impl NorFlashError for Error {
    fn kind(&self) -> NorFlashErrorKind {
        match self {
            Self::Size => NorFlashErrorKind::OutOfBounds,
            Self::Unaligned => NorFlashErrorKind::NotAligned,
            _ => NorFlashErrorKind::Other,
        }
    }
}
