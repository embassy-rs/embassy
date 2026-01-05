//! Flash memory (FLASH)
use embedded_storage::nor_flash::{NorFlashError, NorFlashErrorKind};

#[cfg(any(flash_f4, flash_h7, flash_h7ab))]
mod asynch;
#[cfg(flash)]
mod common;
#[cfg(eeprom)]
mod eeprom;

#[cfg(any(flash_f4, flash_h7, flash_h7ab))]
pub use asynch::InterruptHandler;
#[cfg(flash)]
pub use common::*;
#[cfg(eeprom)]
#[allow(unused_imports)]
pub use eeprom::*;

pub use crate::_generated::flash_regions::*;
#[cfg(eeprom)]
pub use crate::_generated::{EEPROM_BASE, EEPROM_SIZE};
pub use crate::_generated::{FLASH_BASE, FLASH_SIZE, MAX_ERASE_SIZE, WRITE_SIZE};

/// Get all flash regions.
pub fn get_flash_regions() -> &'static [&'static FlashRegion] {
    &FLASH_REGIONS
}

/// Read size (always 1)
pub const READ_SIZE: usize = 1;

/// Blocking flash mode typestate.
pub enum Blocking {}
/// Async flash mode typestate.
pub enum Async {}

/// Flash memory region
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct FlashRegion {
    /// Bank number.
    pub bank: FlashBank,
    /// Absolute base address.
    pub base: u32,
    /// Size in bytes.
    pub size: u32,
    /// Erase size (sector size).
    pub erase_size: u32,
    /// Minimum write size.
    pub write_size: u32,
    /// Erase value (usually `0xFF`, but is `0x00` in some chips)
    pub erase_value: u8,
    pub(crate) _ensure_internal: (),
}

impl FlashRegion {
    /// Absolute end address.
    pub const fn end(&self) -> u32 {
        self.base + self.size
    }

    /// Number of sectors in the region.
    pub const fn sectors(&self) -> u8 {
        (self.size / self.erase_size) as u8
    }
}

/// Flash sector.
#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct FlashSector {
    /// Bank number.
    pub bank: FlashBank,
    /// Sector number within the bank.
    pub index_in_bank: u8,
    /// Absolute start address.
    pub start: u32,
    /// Size in bytes.
    pub size: u32,
}

/// Flash bank.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum FlashBank {
    /// Bank 1
    Bank1 = 0,
    /// Bank 2
    Bank2 = 1,
    /// OTP region,
    Otp,
}
#[cfg(all(eeprom, not(any(flash_l0, flash_l1))))]
compile_error!("The 'eeprom' cfg is enabled for a non-L0/L1 chip family. This is an unsupported configuration.");
#[cfg_attr(any(flash_l0, flash_l1, flash_l4, flash_l5, flash_wl, flash_wb), path = "l.rs")]
#[cfg_attr(flash_f0, path = "f0.rs")]
#[cfg_attr(any(flash_f1, flash_f3), path = "f1f3.rs")]
#[cfg_attr(flash_f2, path = "f2.rs")]
#[cfg_attr(flash_f4, path = "f4.rs")]
#[cfg_attr(flash_f7, path = "f7.rs")]
#[cfg_attr(any(flash_g0x0, flash_g0x1, flash_g4c2, flash_g4c3, flash_g4c4), path = "g.rs")]
#[cfg_attr(flash_c0, path = "c.rs")]
#[cfg_attr(flash_h7, path = "h7.rs")]
#[cfg_attr(flash_h7ab, path = "h7.rs")]
#[cfg_attr(any(flash_u5, flash_wba), path = "u5.rs")]
#[cfg_attr(flash_h5, path = "h5.rs")]
#[cfg_attr(flash_h50, path = "h50.rs")]
#[cfg_attr(flash_u0, path = "u0.rs")]
#[cfg_attr(
    not(any(
        flash_l0, flash_l1, flash_l4, flash_l5, flash_wl, flash_wb, flash_f0, flash_f1, flash_f2, flash_f3, flash_f4,
        flash_f7, flash_g0x0, flash_g0x1, flash_g4c2, flash_g4c3, flash_g4c4, flash_c0, flash_h7, flash_h7ab, flash_u5,
        flash_wba, flash_h50, flash_u0, flash_h5,
    )),
    path = "other.rs"
)]
mod family;

#[allow(unused_imports)]
pub use family::*;

/// Flash error
///
/// See STM32 Reference Manual for your chip for details.
#[allow(missing_docs)]
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
