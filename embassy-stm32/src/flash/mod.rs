use embassy_hal_common::{into_ref, PeripheralRef};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::{Mutex, MutexGuard};
use embedded_storage::nor_flash::{ErrorType, NorFlash, NorFlashError, NorFlashErrorKind, ReadNorFlash};

pub use crate::_generated::flash_regions::*;
pub use crate::pac::{FLASH_BASE, FLASH_SIZE, WRITE_SIZE};
use crate::peripherals::FLASH;
use crate::Peripheral;

#[cfg_attr(any(flash_wl, flash_wb, flash_l0, flash_l1, flash_l4), path = "l.rs")]
#[cfg_attr(flash_f3, path = "f3.rs")]
#[cfg_attr(flash_f4, path = "f4.rs")]
#[cfg_attr(flash_f7, path = "f7.rs")]
#[cfg_attr(flash_h7, path = "h7.rs")]
mod family;

pub struct Flash<'d> {
    _inner: PeripheralRef<'d, FLASH>,
}

static REGION_LOCK: Mutex<CriticalSectionRawMutex, ()> = Mutex::new(());

impl<'d> Flash<'d> {
    pub fn new(p: impl Peripheral<P = FLASH> + 'd) -> Self {
        into_ref!(p);
        Self { _inner: p }
    }

    pub fn into_regions(self) -> FlashRegions {
        FlashRegions::take()
    }

    pub fn blocking_read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Error> {
        if offset as usize + bytes.len() > FLASH_SIZE {
            return Err(Error::Size);
        }

        let first_address = FLASH_BASE as u32 + offset;
        let flash_data = unsafe { core::slice::from_raw_parts(first_address as *const u8, bytes.len()) };
        bytes.copy_from_slice(flash_data);
        Ok(())
    }

    pub fn blocking_write(&mut self, offset: u32, buf: &[u8]) -> Result<(), Error> {
        if offset as usize + buf.len() > FLASH_SIZE {
            return Err(Error::Size);
        }
        if offset as usize % WRITE_SIZE != 0 || buf.len() as usize % WRITE_SIZE != 0 {
            return Err(Error::Unaligned);
        }

        let start_address = FLASH_BASE as u32 + offset;
        trace!("Writing {} bytes at 0x{:x}", buf.len(), start_address);

        // No need to take lock here as we only have one mut flash reference.

        unsafe {
            family::clear_all_err();
            family::unlock();
            let res = Flash::blocking_write_all(start_address, buf);
            family::lock();
            res
        }
    }

    unsafe fn blocking_write_all(start_address: u32, buf: &[u8]) -> Result<(), Error> {
        family::begin_write();
        let mut address = start_address;
        for chunk in buf.chunks(WRITE_SIZE) {
            let res = unsafe { family::blocking_write(address, chunk.try_into().unwrap()) };
            if res.is_err() {
                family::end_write();
                return res;
            }
            address += WRITE_SIZE as u32;
        }

        family::end_write();
        Ok(())
    }

    pub fn blocking_erase(&mut self, from: u32, to: u32) -> Result<(), Error> {
        if to < from || to as usize > FLASH_SIZE {
            return Err(Error::Size);
        }

        let start_address = FLASH_BASE as u32 + from;
        let end_address = FLASH_BASE as u32 + to;
        if !family::is_eraseable_range(start_address, end_address) {
            return Err(Error::Unaligned);
        }
        trace!("Erasing from 0x{:x} to 0x{:x}", start_address, end_address);

        unsafe {
            family::clear_all_err();
            family::unlock();
            let res = family::blocking_erase(start_address, end_address);
            family::lock();
            res
        }
    }
}

impl Drop for Flash<'_> {
    fn drop(&mut self) {
        unsafe { family::lock() };
    }
}

pub trait FlashRegion {
    const BASE: usize;
    const SIZE: usize;
    const ERASE_SIZE: usize;
    const WRITE_SIZE: usize;
    const ERASE_VALUE: u8;

    fn blocking_read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Error> {
        if offset as usize + bytes.len() > Self::SIZE {
            return Err(Error::Size);
        }

        let first_address = Self::BASE as u32 + offset;
        let flash_data = unsafe { core::slice::from_raw_parts(first_address as *const u8, bytes.len()) };
        bytes.copy_from_slice(flash_data);
        Ok(())
    }

    fn blocking_write(&mut self, offset: u32, buf: &[u8]) -> Result<(), Error> {
        if offset as usize + buf.len() > Self::SIZE {
            return Err(Error::Size);
        }
        if offset as usize % Self::WRITE_SIZE != 0 || buf.len() as usize % Self::WRITE_SIZE != 0 {
            return Err(Error::Unaligned);
        }

        let start_address = Self::BASE as u32 + offset;
        trace!("Writing {} bytes from 0x{:x}", buf.len(), start_address);

        // Protect agains simultaneous write/erase to multiple regions.
        let _guard = take_lock_spin();

        unsafe {
            family::clear_all_err();
            family::unlock();
            let res = Flash::blocking_write_all(start_address, buf);
            family::lock();
            res
        }
    }

    fn blocking_erase(&mut self, from: u32, to: u32) -> Result<(), Error> {
        if to < from || to as usize > Self::SIZE {
            return Err(Error::Size);
        }
        if (from as usize % Self::ERASE_SIZE) != 0 || (to as usize % Self::ERASE_SIZE) != 0 {
            return Err(Error::Unaligned);
        }

        let start_address = Self::BASE as u32 + from;
        let end_address = Self::BASE as u32 + to;
        trace!("Erasing from 0x{:x} to 0x{:x}", start_address, end_address);

        // Protect agains simultaneous write/erase to multiple regions.
        let _guard = take_lock_spin();

        unsafe {
            family::clear_all_err();
            family::unlock();
            let res = family::blocking_erase(start_address, end_address);
            family::lock();
            res
        }
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
            const READ_SIZE: usize = <crate::_generated::flash_regions::$name as FlashRegion>::WRITE_SIZE;

            fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
                self.blocking_read(offset, bytes)
            }

            fn capacity(&self) -> usize {
                <crate::_generated::flash_regions::$name as FlashRegion>::SIZE
            }
        }

        impl NorFlash for crate::_generated::flash_regions::$name {
            const WRITE_SIZE: usize = <crate::_generated::flash_regions::$name as FlashRegion>::WRITE_SIZE;
            const ERASE_SIZE: usize = <crate::_generated::flash_regions::$name as FlashRegion>::ERASE_SIZE;

            fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
                self.blocking_erase(from, to)
            }

            fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
                self.blocking_write(offset, bytes)
            }
        }
    };
}
