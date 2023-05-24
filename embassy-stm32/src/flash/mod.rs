use embedded_storage::nor_flash::{NorFlashError, NorFlashErrorKind};

#[cfg(all(feature = "nightly", flash_f4))]
pub mod asynch;
#[cfg(flash)]
mod common;

#[cfg(flash)]
pub use common::*;

pub use crate::_generated::flash_regions::*;
pub use crate::_generated::MAX_ERASE_SIZE;
pub use crate::pac::{FLASH_BASE, FLASH_SIZE, WRITE_SIZE};

pub const READ_SIZE: usize = 1;

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
#[cfg_attr(flash_h7, path = "h7.rs")]
#[cfg_attr(
    not(any(
        flash_l0, flash_l1, flash_l4, flash_wl, flash_wb, flash_f0, flash_f3, flash_f4, flash_f7, flash_h7
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
    TryLockError,
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

pub(crate) fn get_sector(address: u32, regions: &[&FlashRegion]) -> FlashSector {
    let mut current_bank = FlashBank::Bank1;
    let mut bank_offset = 0;
    for region in regions {
        if region.bank != current_bank {
            current_bank = region.bank;
            bank_offset = 0;
        }

        if address < region.end() {
            let index_in_region = (address - region.base) / region.erase_size;
            return FlashSector {
                bank: region.bank,
                index_in_bank: bank_offset + index_in_region as u8,
                start: region.base + index_in_region * region.erase_size,
                size: region.erase_size,
            };
        }

        bank_offset += region.sectors();
    }

    panic!("Flash sector not found");
}

pub(crate) fn ensure_sector_aligned(
    start_address: u32,
    end_address: u32,
    regions: &[&FlashRegion],
) -> Result<(), Error> {
    let mut address = start_address;
    while address < end_address {
        let sector = get_sector(address, regions);
        if sector.start != address {
            return Err(Error::Unaligned);
        }
        address += sector.size;
    }
    if address != end_address {
        return Err(Error::Unaligned);
    }
    Ok(())
}
