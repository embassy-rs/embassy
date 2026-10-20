//! Flash memory (FLASH)
//!
//! [`Flash`] gives access to the whole internal flash, addressed by offsets from [`FLASH_BASE`].
//! [`Flash::into_regions`] and [`Flash::into_blocking_regions`] split it into one driver per
//! flash region (bank, or group of equally sized sectors), which can be handed to different
//! parts of the application.
//!
//! The driver is shared by all families. Writes are done in units of [`WRITE_SIZE`] bytes,
//! which is the native program unit of the chip (a half-word on F0/F1/F3, a 256-bit flash
//! word on H7, ...), and erases are done per sector/page.
use embedded_storage::nor_flash::{NorFlashError, NorFlashErrorKind};

mod asynch;
mod cache;
mod common;
#[cfg(flash_v5_c5)]
mod edata;
#[cfg(eeprom)]
mod eeprom;

#[cfg_attr(flash_v1, path = "v1.rs")]
#[cfg_attr(flash_v2, path = "v2.rs")]
#[cfg_attr(flash_v3, path = "v3.rs")]
#[cfg_attr(flash_v4, path = "v4.rs")]
#[cfg_attr(flash_v5, path = "v5.rs")]
mod family;

pub use asynch::InterruptHandler;
pub use common::*;
#[cfg(flash_v5_c5)]
pub use edata::*;
#[cfg(all(flash_dual_bank, any(flash_v5_h5, flash_v5_h50, flash_v5_c5)))]
pub use family::perform_bank_swap;

#[cfg(flash_dual_bank)]
use crate::_generated::FLASH_BANK2_BASE;
pub use crate::_generated::flash_regions::*;
#[cfg(eeprom)]
pub use crate::_generated::{EEPROM_BASE, EEPROM_SIZE};
use crate::_generated::{FLASH_BANK1_BASE, FLASH_OTP_BASE};
pub use crate::_generated::{FLASH_BASE, FLASH_SIZE, MAX_ERASE_SIZE, WRITE_SIZE};

/// Get all flash regions.
pub fn get_flash_regions() -> &'static [&'static FlashRegion] {
    &FLASH_REGIONS
}

/// Read size (always 1)
pub const READ_SIZE: usize = 1;

/// Flash driver configuration.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Config {
    /// Parallelism used for erase operations.
    ///
    /// Higher parallelism makes erases faster but draws more current. On STM32F2/F4/F7 the
    /// supply must meet the requirements of the selected parallelism during the erase (x32
    /// needs 2.7 V or more, x64 needs an external VPP), see the reference manual. The driver
    /// does not check the supply voltage.
    #[cfg(any(flash_v2, flash_v5_h7))]
    pub erase_parallelism: EraseParallelism,
}

/// Parallelism used for flash erase operations.
#[cfg(any(flash_v2, flash_v5_h7))]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum EraseParallelism {
    /// Erase 8 bits in parallel.
    X8,
    /// Erase 16 bits in parallel.
    X16,
    /// Erase 32 bits in parallel.
    #[default]
    X32,
    /// Erase 64 bits in parallel.
    X64,
}

/// Flash memory region
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct FlashRegion {
    /// Bank number.
    pub bank: FlashBank,
    /// Offset from bank base.
    pub offset: u32,
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
    /// Absolute base address.
    pub fn base(&self) -> u32 {
        self.bank.base() + self.offset
    }

    /// Absolute end address.
    pub fn end(&self) -> u32 {
        self.base() + self.size
    }

    /// Number of sectors in the region.
    pub const fn sectors(&self) -> u32 {
        self.size / self.erase_size
    }
}

/// Flash sector.
#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct FlashSector {
    /// Bank number.
    pub bank: FlashBank,
    /// Sector number within the bank.
    pub index_in_bank: u32,
    /// Absolute start address.
    pub start: u32,
    /// Size in bytes.
    pub size: u32,
}

/// Flash bank.
///
/// This is the physical bank. On chips that can swap the banks (`SWAP_BANK`/`FB_MODE`),
/// [`FlashBank::base`] returns the address where the bank is currently mapped.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum FlashBank {
    /// Bank 1
    Bank1,
    /// Bank 2
    Bank2,
    /// OTP region
    Otp,
}

impl FlashBank {
    /// Absolute base address where the bank is currently mapped.
    pub fn base(&self) -> u32 {
        match self {
            FlashBank::Otp => unwrap!(FLASH_OTP_BASE, "OTP not present"),
            #[cfg(flash_dual_bank)]
            bank => {
                if (*bank == FlashBank::Bank1) != banks_swapped() {
                    FLASH_BANK1_BASE
                } else {
                    FLASH_BANK2_BASE
                }
            }
            #[cfg(not(flash_dual_bank))]
            FlashBank::Bank1 => FLASH_BANK1_BASE,
            #[cfg(not(flash_dual_bank))]
            FlashBank::Bank2 => panic!("Bank 2 not present"),
        }
    }
}

/// Returns whether flash banks 1 and 2 are currently swapped in the memory map.
///
/// Only available when the flash is in dual-bank mode: on chips with a selectable bank mode,
/// this needs the `dual-bank` feature. Always `false` on dual-bank chips that can't swap the
/// banks (F1 XL-density, L1).
#[cfg(flash_dual_bank)]
pub fn banks_swapped() -> bool {
    // Host unit tests can't read the hardware.
    !cfg!(test) && family::banks_swapped()
}

#[cfg(bank_setup_configurable)]
fn check_bank_setup() {
    let dual = family::dual_bank_mode();
    if cfg!(feature = "single-bank") && dual {
        panic!(
            "embassy-stm32 is configured as single-bank, but the flash is in dual-bank mode. Change the dual-bank option bit in the option bytes or enable the `dual-bank` feature."
        );
    }
    if cfg!(feature = "dual-bank") && !dual {
        panic!(
            "embassy-stm32 is configured as dual-bank, but the flash is in single-bank mode. Change the dual-bank option bit in the option bytes or enable the `single-bank` feature."
        );
    }
}

/// Flash error
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// The address range is outside the flash memory or region.
    OutOfBounds,
    /// The address or length is not a multiple of the write or erase size.
    Unaligned,
    /// The target area is write-protected (`WRPERR`).
    Protected,
    /// The target location was not erased before programming (`PGERR`, `PROGERR`, `NOTZEROERR`).
    NotErased,
    /// The hardware reported a programming or erase error, for example a sequence, size,
    /// alignment or parallelism error. See the status register flags in the reference manual.
    Hardware,
    /// The EDATA area is not enabled in the option bytes.
    #[cfg(flash_v5_c5)]
    EdataDisabled,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let msg = match self {
            Self::OutOfBounds => "address out of bounds",
            Self::Unaligned => "unaligned address or length",
            Self::Protected => "write protected",
            Self::NotErased => "location not erased",
            Self::Hardware => "flash hardware error",
            #[cfg(flash_v5_c5)]
            Self::EdataDisabled => "EDATA area disabled",
        };
        f.write_str(msg)
    }
}

impl core::error::Error for Error {}

impl NorFlashError for Error {
    fn kind(&self) -> NorFlashErrorKind {
        match self {
            Self::OutOfBounds => NorFlashErrorKind::OutOfBounds,
            Self::Unaligned => NorFlashErrorKind::NotAligned,
            _ => NorFlashErrorKind::Other,
        }
    }
}
