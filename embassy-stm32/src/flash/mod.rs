use embedded_storage::nor_flash::{NorFlashError, NorFlashErrorKind};

#[cfg(flash)]
mod common;

#[cfg(flash)]
pub use common::*;

pub use crate::_generated::flash_regions::*;
pub use crate::pac::{FLASH_BASE, FLASH_SIZE, WRITE_SIZE};

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

impl FlashRegion {
    pub const fn end(&self) -> u32 {
        self.base + self.size
    }
}

#[cfg_attr(any(flash_l0, flash_l1, flash_l4, flash_wl, flash_wb), path = "l.rs")]
#[cfg_attr(flash_f3, path = "f3.rs")]
#[cfg_attr(flash_f4, path = "f4.rs")]
#[cfg_attr(flash_f7, path = "f7.rs")]
#[cfg_attr(flash_h7, path = "h7.rs")]
#[cfg_attr(
    not(any(
        flash_l0, flash_l1, flash_l4, flash_wl, flash_wb, flash_f3, flash_f4, flash_f7, flash_h7
    )),
    path = "other.rs"
)]
mod family;

#[allow(unused_imports)]
pub use family::*;

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
