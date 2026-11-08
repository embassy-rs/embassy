use core::marker::PhantomData;
use core::sync::atomic::{Ordering, fence};

use super::{
    Config, Error, FLASH_SIZE, FlashLayout, FlashRegion, FlashSector, MAX_ERASE_SIZE, READ_SIZE, WRITE_SIZE, cache,
    family, get_flash_regions,
};
use crate::_generated::{FLASH_BASE, FLASH_CONTIGUOUS_REGIONS};
use crate::Peri;
use crate::mode::{Blocking, Mode};
use crate::peripherals::FLASH;

/// Internal flash memory driver.
pub struct Flash<'d, M: Mode> {
    pub(crate) inner: Peri<'d, FLASH>,
    pub(crate) config: Config,
    pub(crate) _mode: PhantomData<M>,
}

impl<'d> Flash<'d, Blocking> {
    /// Create a new flash driver, usable in blocking mode.
    pub fn new_blocking(p: Peri<'d, FLASH>, config: Config) -> Self {
        Self::new_inner(p, config)
    }
}

impl<'d, M: Mode> Flash<'d, M> {
    pub(super) fn new_inner(p: Peri<'d, FLASH>, config: Config) -> Self {
        #[cfg(bank_setup_configurable)]
        super::check_bank_setup();

        Self {
            inner: p,
            config,
            _mode: PhantomData,
        }
    }

    /// Split this flash driver into one blocking driver per flash memory region.
    ///
    /// See module-level documentation for details on how memory regions work.
    pub fn into_blocking_regions(self) -> FlashLayout<'d, Blocking> {
        FlashLayout::new(self.inner, self.config)
    }

    /// Blocking read.
    ///
    /// NOTE: `offset` is an offset from the flash start, NOT an absolute address.
    /// For example, to read address `0x0800_1234` you have to use offset `0x1234`.
    pub fn blocking_read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Error> {
        let (base, size, offset) = check_flash_address(FLASH_BASE as u32, offset)?;
        blocking_read(base, size, offset, bytes)
    }

    /// Blocking write.
    ///
    /// NOTE: `offset` is an offset from the flash start, NOT an absolute address.
    /// For example, to write address `0x0800_1234` you have to use offset `0x1234`.
    pub fn blocking_write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Error> {
        let (base, size, offset) = check_flash_address(FLASH_BASE as u32, offset)?;
        blocking_write(base, size, offset, bytes, false)
    }

    /// Blocking erase.
    ///
    /// NOTE: `from` and `to` are offsets from the flash start, NOT an absolute address.
    /// For example, to erase address `0x0801_0000` you have to use offset `0x1_0000`.
    pub fn blocking_erase(&mut self, from: u32, to: u32) -> Result<(), Error> {
        blocking_erase(FLASH_BASE as u32, from, to, &self.config, false)
    }
}

pub(super) fn blocking_read(base: u32, size: u32, offset: u32, bytes: &mut [u8]) -> Result<(), Error> {
    check_range(offset, bytes.len(), size)?;

    let start_address = base + offset;

    #[cfg(flash_v2_f4)]
    family::assert_not_corrupted_read(start_address, bytes.len() as u32);

    let flash_data = unsafe { core::slice::from_raw_parts(start_address as *const u8, bytes.len()) };
    bytes.copy_from_slice(flash_data);
    Ok(())
}

/// Checks that `offset..offset+len` is within `0..size`.
pub(super) fn check_range(offset: u32, len: usize, size: u32) -> Result<(), Error> {
    let len = u32::try_from(len).map_err(|_| Error::OutOfBounds)?;
    match offset.checked_add(len) {
        Some(end) if end <= size => Ok(()),
        _ => Err(Error::OutOfBounds),
    }
}

pub(super) fn check_write(size: u32, offset: u32, len: usize) -> Result<(), Error> {
    check_range(offset, len, size)?;
    if !offset.is_multiple_of(WRITE_SIZE as u32) || !len.is_multiple_of(WRITE_SIZE) {
        return Err(Error::Unaligned);
    }
    Ok(())
}

pub(super) fn blocking_write(
    base: u32,
    size: u32,
    offset: u32,
    bytes: &[u8],
    critical_section: bool,
) -> Result<(), Error> {
    check_write(size, offset, bytes.len())?;

    let mut address = base + offset;
    trace!("Writing {} bytes at 0x{:x}", bytes.len(), address);

    for chunk in bytes.chunks(WRITE_SIZE) {
        let chunk = unwrap!(chunk.try_into());
        with_cs(critical_section, || blocking_op(Op::Write(address, chunk)))?;
        address += WRITE_SIZE as u32;
    }
    Ok(())
}

pub(super) fn blocking_erase(
    base: u32,
    from: u32,
    to: u32,
    config: &Config,
    critical_section: bool,
) -> Result<(), Error> {
    let start_address = base + from;
    let end_address = base + to;
    let regions = get_flash_regions();

    ensure_sector_aligned(start_address, end_address, regions)?;

    trace!("Erasing from 0x{:x} to 0x{:x}", start_address, end_address);

    let mut address = start_address;
    while address < end_address {
        let sector = get_sector(address, regions)?;
        trace!("Erasing sector: {:?}", sector);
        with_cs(critical_section, || blocking_op(Op::Erase(&sector, config)))?;
        address += sector.size;
    }
    Ok(())
}

fn with_cs<R>(critical_section: bool, f: impl FnOnce() -> R) -> R {
    if critical_section {
        critical_section::with(|_| f())
    } else {
        f()
    }
}

/// A single program or erase operation.
#[derive(Clone, Copy)]
pub(super) enum Op<'a> {
    /// Program one [`WRITE_SIZE`] unit at the given address.
    Write(u32, &'a [u8; WRITE_SIZE]),
    /// Erase one sector.
    Erase(&'a FlashSector, &'a Config),
}

impl Op<'_> {
    /// Checks for accesses that the hardware rejects with a bus fault rather than an error flag.
    pub(super) fn check(&self) -> Result<(), Error> {
        #[cfg(all(any(flash_v4_wb, flash_v4_wl), not(feature = "_core-cm0p")))]
        {
            let (start, len) = self.range();
            family::check_secure_area(start, start + len)?;
        }
        Ok(())
    }

    /// Address range touched by the operation.
    fn range(&self) -> (u32, u32) {
        match *self {
            Op::Write(address, _) => (address, WRITE_SIZE as u32),
            Op::Erase(sector, _) => (sector.start, sector.size),
        }
    }
}

/// Runs a program or erase operation from start to finish: unlock, start, wait, check, lock.
///
/// Must not be called concurrently with another operation.
pub(super) fn blocking_op(op: Op<'_>) -> Result<(), Error> {
    op.check()?;
    let _guard = OpGuard::start(op, false);
    while family::busy() {}
    family::finish()
}

/// Brings the controller back to an idle, locked state when an operation ends, also if it
/// is abandoned halfway (dropped async future).
pub(super) struct OpGuard {
    cache: cache::State,
    range: (u32, u32),
}

impl OpGuard {
    /// Unlocks the controller and starts `op`. With `interrupt`, the completion interrupt is
    /// enabled.
    pub(super) fn start(op: Op<'_>, interrupt: bool) -> Self {
        let cache = cache::before_op();
        fence(Ordering::SeqCst);
        unsafe {
            // The control register must not be written while an operation is ongoing.
            family::wait_idle();
            family::unlock();
            // Clear flags left over by a previous operation, they would block this one.
            let _ = family::finish();
            match op {
                Op::Write(address, data) => family::start_write(address, data, interrupt),
                Op::Erase(sector, config) => family::start_erase(sector, config, interrupt),
            }
        }
        Self {
            cache,
            range: op.range(),
        }
    }
}

impl Drop for OpGuard {
    fn drop(&mut self) {
        // If the operation was abandoned, let it complete before touching the controller.
        family::wait_idle();
        let _ = family::finish();
        unsafe { family::lock() };
        fence(Ordering::SeqCst);
        cache::after_op(&self.cache, self.range);
    }
}

/// Returns (base, size, offset) for the contiguous region specified by the offset
pub(super) fn check_flash_address(base: u32, offset: u32) -> Result<(u32, u32, u32), Error> {
    let addr = base.checked_add(offset).ok_or(Error::OutOfBounds)?;
    for (base, size) in FLASH_CONTIGUOUS_REGIONS {
        let (base, size) = (base as u32, size as u32);
        let end = base + size;

        if addr >= base && addr < end {
            return Ok((base, end - base, addr - base));
        }
    }

    Err(Error::OutOfBounds)
}

pub(super) fn get_sector(address: u32, regions: &[&FlashRegion]) -> Result<FlashSector, Error> {
    let mut current_bank = None;
    let mut bank_offset = 0;
    for region in regions {
        if current_bank != Some(region.bank) {
            current_bank = Some(region.bank);
            bank_offset = 0;
        }

        if region.erase_size != 0 && address >= region.base() && address < region.end() {
            let index_in_region = (address - region.base()) / region.erase_size;
            return Ok(FlashSector {
                bank: region.bank,
                index_in_bank: bank_offset + index_in_region,
                start: region.base() + index_in_region * region.erase_size,
                size: region.erase_size,
            });
        }

        if region.erase_size != 0 {
            bank_offset += region.sectors();
        }
    }

    Err(Error::OutOfBounds)
}

pub(super) fn ensure_sector_aligned(
    start_address: u32,
    end_address: u32,
    regions: &[&FlashRegion],
) -> Result<(), Error> {
    if end_address < start_address {
        return Err(Error::OutOfBounds);
    }
    let mut address = start_address;
    while address < end_address {
        let sector = get_sector(address, regions)?;
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

impl<M: Mode> embedded_storage::nor_flash::ErrorType for Flash<'_, M> {
    type Error = Error;
}

impl<M: Mode> embedded_storage::nor_flash::ReadNorFlash for Flash<'_, M> {
    const READ_SIZE: usize = READ_SIZE;

    fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        self.blocking_read(offset, bytes)
    }

    fn capacity(&self) -> usize {
        FLASH_SIZE
    }
}

impl<M: Mode> embedded_storage::nor_flash::NorFlash for Flash<'_, M> {
    const WRITE_SIZE: usize = WRITE_SIZE;
    const ERASE_SIZE: usize = MAX_ERASE_SIZE;

    fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        self.blocking_write(offset, bytes)
    }

    fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
        self.blocking_erase(from, to)
    }
}

foreach_flash_region! {
    ($type_name:ident, $write_size:literal, $erase_size:literal) => {
        impl<M: Mode> crate::_generated::flash_regions::$type_name<'_, M> {
            /// Blocking read.
            ///
            /// NOTE: `offset` is an offset from the region start, NOT an absolute address.
            pub fn blocking_read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Error> {
                blocking_read(self.0.base(), self.0.size, offset, bytes)
            }
        }

        impl crate::_generated::flash_regions::$type_name<'_, Blocking> {
            /// Blocking write.
            ///
            /// NOTE: `offset` is an offset from the region start, NOT an absolute address.
            pub fn blocking_write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Error> {
                blocking_write(self.0.base(), self.0.size, offset, bytes, true)
            }

            /// Blocking erase.
            ///
            /// NOTE: `from` and `to` are offsets from the region start, NOT absolute addresses.
            pub fn blocking_erase(&mut self, from: u32, to: u32) -> Result<(), Error> {
                if to > self.0.size {
                    return Err(Error::OutOfBounds);
                }
                blocking_erase(self.0.base(), from, to, &self.3, true)
            }
        }

        impl<M: Mode> embedded_storage::nor_flash::ErrorType for crate::_generated::flash_regions::$type_name<'_, M> {
            type Error = Error;
        }

        impl<M: Mode> embedded_storage::nor_flash::ReadNorFlash for crate::_generated::flash_regions::$type_name<'_, M> {
            const READ_SIZE: usize = READ_SIZE;

            fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
                self.blocking_read(offset, bytes)
            }

            fn capacity(&self) -> usize {
                self.0.size as usize
            }
        }

        impl embedded_storage::nor_flash::NorFlash for crate::_generated::flash_regions::$type_name<'_, Blocking> {
            const WRITE_SIZE: usize = $write_size;
            const ERASE_SIZE: usize = $erase_size;

            fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
                self.blocking_write(offset, bytes)
            }

            fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
                self.blocking_erase(from, to)
            }
        }
    };
}
