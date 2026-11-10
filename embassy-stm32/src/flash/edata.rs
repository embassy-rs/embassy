use core::marker::PhantomData;

use embassy_hal_internal::Peri;

use super::asynch::{REGION_ACCESS, op};
use super::common::{Op, blocking_op, check_range};
use super::{Config, Error, FlashBank, FlashSector, READ_SIZE, ecc};
use crate::mode::{Async, Blocking, Mode};
use crate::pac;
use crate::peripherals::FLASH;

/// Address of the EDATA area of the bank that is mapped first.
pub(super) const BASE: u32 = 0x0900_0000;

/// Number of EDATA sectors of a bank.
#[cfg(flash_v5_c5)]
const SECTORS: u32 = 16;
#[cfg(flash_v5_h5)]
const SECTORS: u32 = 8;

/// Size of an EDATA erase unit in bytes: a 1.5 KB page on C5, a 6 KB sector on H5.
#[cfg(flash_v5_c5)]
pub const EDATA_ERASE_SIZE: usize = 1536;
/// Size of an EDATA erase unit in bytes: a 1.5 KB page on C5, a 6 KB sector on H5.
#[cfg(flash_v5_h5)]
pub const EDATA_ERASE_SIZE: usize = 6 * 1024;

/// Size of an EDATA write unit in bytes.
pub const EDATA_WRITE_SIZE: usize = 2;

const SECTOR_SIZE: u32 = EDATA_ERASE_SIZE as u32;

/// Driver for the EDATA area of a flash bank.
///
/// The EDATA area is flash with 16-bit words, each with its own ECC, and a higher endurance
/// than the rest of the flash. It's written in units of [`EDATA_WRITE_SIZE`] bytes, and erased
/// in units of [`EDATA_ERASE_SIZE`] bytes. It implements [`NorFlash`], so it can store data with
/// crates like `sequential-storage`.
///
/// The EDATA area must be enabled in the option bytes:
///
/// - C5: `EDATA_EN` enables 16 pages in each bank.
/// - H5: `EDATAx_EN` enables the last `EDATAx_STRT + 1` EDATA sectors of bank x. On the parts
///   with the largest flash of the line, they are the last user flash sectors of the bank, which
///   then can't be used as user flash: reading them through the user flash is a bus error.
///
/// [`capacity()`](Self::capacity) is the size of the area that is enabled, 0 when it's
/// disabled. Offsets are relative to the start of the area.
///
/// Locations that were not written since they were erased read as `0xFF`. A read with an
/// uncorrectable ECC error returns [`Error::Ecc`].
///
/// The reads are done with the instruction cache (ICACHE) disabled, as the EDATA area must not be
/// cached.
///
/// Programming or erasing the bank the code runs from stalls the CPU until it's done. Use the
/// EDATA area of the other bank to avoid it.
///
/// [`NorFlash`]: embedded_storage::nor_flash::NorFlash
pub struct Edata<'d, M: Mode> {
    bank: FlashBank,
    _flash: Peri<'d, FLASH>,
    _mode: PhantomData<M>,
}

impl<'d, M: Mode> Edata<'d, M> {
    pub(crate) fn new(flash: Peri<'d, FLASH>, bank: FlashBank) -> Self {
        Self {
            bank,
            _flash: flash,
            _mode: PhantomData,
        }
    }

    /// Size in bytes of the EDATA area of the bank enabled in the option bytes. 0 if it's
    /// disabled.
    pub fn capacity(&self) -> usize {
        (enabled_sectors(self.bank) * SECTOR_SIZE) as usize
    }

    /// Blocking read.
    ///
    /// NOTE: `offset` is an offset from the start of the EDATA area of the bank.
    ///
    /// An uncorrectable ECC error in the data returns [`Error::Ecc`]. The contents of `bytes`
    /// are then unspecified.
    pub fn blocking_read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Error> {
        let address = self.address(offset, bytes.len())?;
        ecc::read_half_words(address, bytes)
    }

    /// Address of `offset`, after checking that the `len` bytes from it are in the area.
    fn address(&self, offset: u32, len: usize) -> Result<u32, Error> {
        let sectors = enabled_sectors(self.bank);
        check_range(offset, len, sectors * SECTOR_SIZE)?;
        Ok(base(self.bank) + (SECTORS - sectors) * SECTOR_SIZE + offset)
    }

    fn check_write(&self, offset: u32, len: usize) -> Result<u32, Error> {
        let address = self.address(offset, len)?;
        if !offset.is_multiple_of(EDATA_WRITE_SIZE as u32) || !len.is_multiple_of(EDATA_WRITE_SIZE) {
            return Err(Error::Unaligned);
        }
        Ok(address)
    }

    /// The sectors to erase for `from..to`.
    fn sectors(&self, from: u32, to: u32) -> Result<impl Iterator<Item = FlashSector>, Error> {
        if to < from {
            return Err(Error::OutOfBounds);
        }
        let start = self.address(from, (to - from) as usize)?;
        if !from.is_multiple_of(SECTOR_SIZE) || !to.is_multiple_of(SECTOR_SIZE) {
            return Err(Error::Unaligned);
        }
        let bank = self.bank;
        let first = (start - base(bank)) / SECTOR_SIZE;
        Ok((0..(to - from) / SECTOR_SIZE).map(move |i| FlashSector {
            bank,
            index_in_bank: sector_number(first + i),
            start: start + i * SECTOR_SIZE,
            size: SECTOR_SIZE,
        }))
    }
}

impl<'d> Edata<'d, Blocking> {
    /// Blocking write.
    ///
    /// NOTE: `offset` is an offset from the start of the EDATA area of the bank.
    ///
    /// `offset` and the length of `bytes` must be multiples of [`EDATA_WRITE_SIZE`]. The 16-bit
    /// words written must not have been written since they were erased: writing a word twice
    /// would corrupt its ECC. If one was, the write returns [`Error::NotErased`] without writing
    /// anything.
    pub fn blocking_write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Error> {
        let address = self.check_write(offset, bytes.len())?;
        check_erased(address, bytes.len())?;
        for (address, data) in units(address, bytes) {
            critical_section::with(|_| blocking_op(Op::WriteEdata(address, data)))?;
        }
        Ok(())
    }

    /// Blocking erase.
    ///
    /// NOTE: `from` and `to` are offsets from the start of the EDATA area of the bank. They
    /// must be multiples of [`EDATA_ERASE_SIZE`].
    pub fn blocking_erase(&mut self, from: u32, to: u32) -> Result<(), Error> {
        for sector in self.sectors(from, to)? {
            critical_section::with(|_| blocking_op(Op::Erase(&sector, &Config::default())))?;
        }
        Ok(())
    }
}

impl<'d> Edata<'d, Async> {
    /// Async read.
    ///
    /// Note: reading from flash can't actually block, so this is the same as `blocking_read`.
    pub async fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Error> {
        self.blocking_read(offset, bytes)
    }

    /// Async write.
    ///
    /// See [`blocking_write`](Edata::blocking_write).
    pub async fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Error> {
        let address = self.check_write(offset, bytes.len())?;
        let _guard = REGION_ACCESS.lock().await;
        check_erased(address, bytes.len())?;
        for (address, data) in units(address, bytes) {
            op(Op::WriteEdata(address, data)).await?;
        }
        Ok(())
    }

    /// Async erase.
    ///
    /// See [`blocking_erase`](Edata::blocking_erase).
    pub async fn erase(&mut self, from: u32, to: u32) -> Result<(), Error> {
        let sectors = self.sectors(from, to)?;
        let _guard = REGION_ACCESS.lock().await;
        for sector in sectors {
            op(Op::Erase(&sector, &Config::default())).await?;
        }
        Ok(())
    }
}

/// The units to program to write `bytes` at `address`: 32-bit words where they are aligned,
/// 16-bit words elsewhere.
fn units(mut address: u32, mut bytes: &[u8]) -> impl Iterator<Item = (u32, &[u8])> {
    core::iter::from_fn(move || {
        let n = if address.is_multiple_of(4) && bytes.len() >= 4 {
            4
        } else {
            2
        };
        let (unit, rest) = bytes.split_at_checked(n)?;
        let item = (address, unit);
        address += n as u32;
        bytes = rest;
        Some(item)
    })
}

/// Checks that the `len` bytes at `address` were not written since they were erased.
fn check_erased(address: u32, len: usize) -> Result<(), Error> {
    ecc::half_word_reads(|| {
        for word in (address..address + len as u32).step_by(2) {
            if !matches!(ecc::read_half_word(word), Ok(None)) {
                return Err(Error::NotErased);
            }
        }
        Ok(())
    })
}

/// Address of the EDATA area of `bank`. Like the banks, the EDATA areas are swapped with
/// `SWAP_BANK`.
fn base(bank: FlashBank) -> u32 {
    #[cfg(flash_dual_bank)]
    let swapped = super::banks_swapped();
    #[cfg(not(flash_dual_bank))]
    let swapped = false;
    if (bank == FlashBank::Bank1) != swapped {
        BASE
    } else {
        BASE + SECTORS * SECTOR_SIZE
    }
}

/// Number of the EDATA sectors of `bank` enabled in the option bytes, which are the last ones.
#[cfg(flash_v5_c5)]
fn enabled_sectors(_bank: FlashBank) -> u32 {
    if pac::FLASH.optsr_cur().read().edata_en() {
        SECTORS
    } else {
        0
    }
}

/// Number of the EDATA sectors of `bank` enabled in the option bytes, which are the last ones.
#[cfg(flash_v5_h5)]
fn enabled_sectors(bank: FlashBank) -> u32 {
    let (enabled, start) = match bank {
        FlashBank::Bank1 => {
            let r = pac::FLASH.edata1r_cur().read();
            (r.edata1_en(), r.edata1_strt())
        }
        _ => {
            let r = pac::FLASH.edata2r_cur().read();
            (r.edata2_en(), r.edata2_strt())
        }
    };
    if enabled { start as u32 + 1 } else { 0 }
}

/// The number used to erase the EDATA sector `i` of a bank: its page number on C5, the user flash
/// sector it's made of on H5.
fn sector_number(i: u32) -> u32 {
    #[cfg(flash_v5_c5)]
    return i;
    #[cfg(flash_v5_h5)]
    return crate::_generated::EDATA_FIRST_SECTOR as u32 + i;
}

impl<M: Mode> embedded_storage::nor_flash::ErrorType for Edata<'_, M> {
    type Error = Error;
}

impl<M: Mode> embedded_storage::nor_flash::ReadNorFlash for Edata<'_, M> {
    const READ_SIZE: usize = READ_SIZE;

    fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        self.blocking_read(offset, bytes)
    }

    fn capacity(&self) -> usize {
        Edata::capacity(self)
    }
}

impl embedded_storage::nor_flash::NorFlash for Edata<'_, Blocking> {
    const WRITE_SIZE: usize = EDATA_WRITE_SIZE;
    const ERASE_SIZE: usize = EDATA_ERASE_SIZE;

    fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        self.blocking_write(offset, bytes)
    }

    fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
        self.blocking_erase(from, to)
    }
}

impl embedded_storage_async::nor_flash::ReadNorFlash for Edata<'_, Async> {
    const READ_SIZE: usize = READ_SIZE;

    async fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        self.read(offset, bytes).await
    }

    fn capacity(&self) -> usize {
        Edata::capacity(self)
    }
}

impl embedded_storage_async::nor_flash::NorFlash for Edata<'_, Async> {
    const WRITE_SIZE: usize = EDATA_WRITE_SIZE;
    const ERASE_SIZE: usize = EDATA_ERASE_SIZE;

    async fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        self.write(offset, bytes).await
    }

    async fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
        self.erase(from, to).await
    }
}
