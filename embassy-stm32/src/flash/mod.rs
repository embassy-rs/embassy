use embedded_storage::nor_flash::{NorFlashError, NorFlashErrorKind};

#[cfg(flash_f4)]
mod asynch;
#[cfg(flash)]
mod common;

#[cfg(flash_f4)]
pub use asynch::InterruptHandler;
#[cfg(flash)]
pub use common::*;

pub use crate::_generated::flash_regions::*;
pub use crate::_generated::MAX_ERASE_SIZE;
pub use crate::pac::{FLASH_BASE, FLASH_SIZE, WRITE_SIZE};

pub const READ_SIZE: usize = 1;

pub struct Blocking;
pub struct Async;

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct FlashRegion {
    pub bank: FlashBank,
    pub base: u32,
    pub size: u32,
    pub erase_size: u32,
    pub write_size: u32,
    pub erase_value: u8,
    pub(crate) _ensure_internal: (),
}

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct FlashSector {
    pub bank: FlashBank,
    pub index_in_bank: u8,
    pub start: u32,
    pub size: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum FlashBank {
    Bank1 = 0,
    Bank2 = 1,
    Otp,
}

impl FlashRegion {
    pub const fn end(&self) -> u32 {
        self.base + self.size
    }

    pub const fn sectors(&self) -> u8 {
        (self.size / self.erase_size) as u8
    }
}

#[cfg_attr(any(flash_l0, flash_l1, flash_l4, flash_wl, flash_wb), path = "l.rs")]
#[cfg_attr(flash_f0, path = "f0.rs")]
#[cfg_attr(flash_f3, path = "f3.rs")]
#[cfg_attr(flash_f4, path = "f4.rs")]
#[cfg_attr(flash_f7, path = "f7.rs")]
#[cfg_attr(flash_g0, path = "g0.rs")]
#[cfg_attr(flash_h7, path = "h7.rs")]
#[cfg_attr(flash_h7ab, path = "h7.rs")]
#[cfg_attr(
    not(any(
        flash_l0, flash_l1, flash_l4, flash_wl, flash_wb, flash_f0, flash_f3, flash_f4, flash_f7, flash_g0, flash_h7,
        flash_h7ab
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
