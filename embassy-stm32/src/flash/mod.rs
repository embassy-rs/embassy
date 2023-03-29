use embassy_hal_common::drop::OnDrop;
use embassy_hal_common::{into_ref, PeripheralRef};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::{Mutex, MutexGuard};
use embedded_storage::nor_flash::{ErrorType, NorFlash, NorFlashError, NorFlashErrorKind, ReadNorFlash};

pub use crate::_generated::flash_regions::*;
pub use crate::pac::{FLASH_BASE, FLASH_SIZE, WRITE_SIZE};
use crate::peripherals::FLASH;
use crate::Peripheral;

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
    use super::{Error, FlashSector};

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

pub struct Flash<'d> {
    inner: PeripheralRef<'d, FLASH>,
}

pub struct FlashRegionSettings {
    pub base: usize,
    pub size: usize,
    pub erase_size: usize,
    pub write_size: usize,
    pub erase_value: u8,
}

#[derive(Debug, PartialEq)]
pub struct FlashSector {
    pub index: u8,
    pub start: u32,
    pub size: u32,
}

pub trait FlashRegion {
    const SETTINGS: FlashRegionSettings;

    fn blocking_read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Error> {
        Flash::blocking_read_inner(Self::SETTINGS.base as u32 + offset, bytes)
    }

    fn blocking_write(&mut self, offset: u32, buf: &[u8]) -> Result<(), Error> {
        let start_address = Self::SETTINGS.base as u32 + offset;

        // Protect agains simultaneous write/erase to multiple regions.
        let _guard = take_lock_spin();

        unsafe { Flash::blocking_write_inner(start_address, buf) }
    }

    fn blocking_erase(&mut self, from: u32, to: u32) -> Result<(), Error> {
        let start_address = Self::SETTINGS.base as u32 + from;
        let end_address = Self::SETTINGS.base as u32 + to;

        // Protect agains simultaneous write/erase to multiple regions.
        let _guard = take_lock_spin();

        unsafe { Flash::blocking_erase_inner(start_address, end_address) }
    }
}

static REGION_LOCK: Mutex<CriticalSectionRawMutex, ()> = Mutex::new(());

impl<'d> Flash<'d> {
    pub fn new(p: impl Peripheral<P = FLASH> + 'd) -> Self {
        into_ref!(p);
        Self { inner: p }
    }

    pub fn into_regions(self) -> FlashRegions<'d> {
        let mut flash = self;
        let p = unsafe { flash.inner.clone_unchecked() };
        FlashRegions::new(p)
    }

    pub fn blocking_read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Error> {
        Self::blocking_read_inner(FLASH_BASE as u32 + offset, bytes)
    }

    fn blocking_read_inner(start_address: u32, bytes: &mut [u8]) -> Result<(), Error> {
        assert!(start_address >= FLASH_BASE as u32);
        if start_address as usize + bytes.len() > FLASH_BASE + FLASH_SIZE {
            return Err(Error::Size);
        }

        let flash_data = unsafe { core::slice::from_raw_parts(start_address as *const u8, bytes.len()) };
        bytes.copy_from_slice(flash_data);
        Ok(())
    }

    pub fn blocking_write(&mut self, offset: u32, buf: &[u8]) -> Result<(), Error> {
        let start_address = FLASH_BASE as u32 + offset;

        // No need to take lock here as we only have one mut flash reference.

        unsafe { Flash::blocking_write_inner(start_address, buf) }
    }

    unsafe fn blocking_write_inner(start_address: u32, buf: &[u8]) -> Result<(), Error> {
        assert!(start_address >= FLASH_BASE as u32);
        if start_address as usize + buf.len() > FLASH_BASE + FLASH_SIZE {
            return Err(Error::Size);
        }
        if (start_address as usize - FLASH_BASE) % WRITE_SIZE != 0 || buf.len() as usize % WRITE_SIZE != 0 {
            return Err(Error::Unaligned);
        }

        trace!("Writing {} bytes at 0x{:x}", buf.len(), start_address);

        family::clear_all_err();
        family::unlock();
        family::begin_write();

        let _ = OnDrop::new(|| {
            family::end_write();
            family::lock();
        });

        let mut address = start_address;
        for chunk in buf.chunks(WRITE_SIZE) {
            unsafe { family::blocking_write(address, chunk.try_into().unwrap())? };
            address += WRITE_SIZE as u32;
        }
        Ok(())
    }

    pub fn blocking_erase(&mut self, from: u32, to: u32) -> Result<(), Error> {
        let start_address = FLASH_BASE as u32 + from;
        let end_address = FLASH_BASE as u32 + to;

        unsafe { Flash::blocking_erase_inner(start_address, end_address) }
    }

    unsafe fn blocking_erase_inner(start_address: u32, end_address: u32) -> Result<(), Error> {
        // Test if the address range is aligned at sector base addresses
        let mut address = start_address;
        while address < end_address {
            let sector = family::get_sector(address);
            if sector.start != address {
                return Err(Error::Unaligned);
            }
            address += sector.size;
        }
        if address != end_address {
            return Err(Error::Unaligned);
        }

        trace!("Erasing from 0x{:x} to 0x{:x}", start_address, end_address);

        family::clear_all_err();
        family::unlock();

        let _ = OnDrop::new(|| {
            family::lock();
        });

        let mut address = start_address;
        while address < end_address {
            let sector = family::get_sector(address);
            family::blocking_erase_sector(&sector)?;
            address += sector.size;
        }
        Ok(())
    }
}

impl Drop for Flash<'_> {
    fn drop(&mut self) {
        unsafe { family::lock() };
    }
}

impl Drop for FlashRegions<'_> {
    fn drop(&mut self) {
        unsafe { family::lock() };
    }
}

fn take_lock_spin() -> MutexGuard<'static, CriticalSectionRawMutex, ()> {
    loop {
        if let Ok(guard) = REGION_LOCK.try_lock() {
            return guard;
        }
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

foreach_flash_region! {
    ($name:ident) => {
        impl ErrorType for crate::_generated::flash_regions::$name {
            type Error = Error;
        }

        impl ReadNorFlash for crate::_generated::flash_regions::$name {
            const READ_SIZE: usize = <crate::_generated::flash_regions::$name as FlashRegion>::SETTINGS.write_size;

            fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
                self.blocking_read(offset, bytes)
            }

            fn capacity(&self) -> usize {
                <crate::_generated::flash_regions::$name as FlashRegion>::SETTINGS.size
            }
        }

        impl NorFlash for crate::_generated::flash_regions::$name {
            const WRITE_SIZE: usize = <crate::_generated::flash_regions::$name as FlashRegion>::SETTINGS.write_size;
            const ERASE_SIZE: usize = <crate::_generated::flash_regions::$name as FlashRegion>::SETTINGS.erase_size;

            fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
                self.blocking_erase(from, to)
            }

            fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
                self.blocking_write(offset, bytes)
            }
        }
    };
}
