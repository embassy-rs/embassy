use embassy_hal_common::{into_ref, PeripheralRef};
use embedded_storage::nor_flash::{ErrorType, NorFlash, NorFlashError, NorFlashErrorKind, ReadNorFlash};

pub use crate::_generated::flash_regions::*;
pub use crate::pac::{FLASH_BASE, FLASH_SIZE};
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
        if offset as usize % family::MAX_WRITE_SIZE != 0 || buf.len() as usize % family::MAX_WRITE_SIZE != 0 {
            return Err(Error::Unaligned);
        }

        let first_address = FLASH_BASE as u32 + offset;
        trace!("Writing {} bytes at 0x{:x}", buf.len(), first_address);

        unsafe {
            family::clear_all_err();

            family::unlock();
            let res = family::blocking_write(first_address, buf);
            family::lock();
            res
        }
    }

    pub fn blocking_erase(&mut self, from: u32, to: u32) -> Result<(), Error> {
        if to < from || to as usize > FLASH_SIZE {
            return Err(Error::Size);
        }
        if (from as usize % family::MAX_ERASE_SIZE) != 0 || (to as usize % family::MAX_ERASE_SIZE) != 0 {
            return Err(Error::Unaligned);
        }

        let from_address = FLASH_BASE as u32 + from;
        let to_address = FLASH_BASE as u32 + to;

        unsafe {
            family::clear_all_err();

            family::unlock();
            let res = family::blocking_erase(from_address, to_address);
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

        let first_address = Self::BASE as u32 + offset;
        trace!("Writing {} bytes from 0x{:x}", buf.len(), first_address);

        critical_section::with(|_| unsafe {
            family::clear_all_err();

            family::unlock();
            let res = family::blocking_write(first_address, buf);
            family::lock();
            res
        })
    }

    fn blocking_erase(&mut self, from: u32, to: u32) -> Result<(), Error> {
        if to < from || to as usize > Self::SIZE {
            return Err(Error::Size);
        }
        if (from as usize % Self::ERASE_SIZE) != 0 || (to as usize % Self::ERASE_SIZE) != 0 {
            return Err(Error::Unaligned);
        }

        let from_address = Self::BASE as u32 + from;
        let to_address = Self::BASE as u32 + to;
        trace!("Erasing from 0x{:x} to 0x{:x}", from_address, to_address);

        critical_section::with(|_| unsafe {
            family::clear_all_err();

            family::unlock();
            let res = family::blocking_erase(from_address, to_address);
            family::lock();
            res
        })
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
