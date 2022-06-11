use crate::peripherals::FLASH;
use crate::Unborrow;
use core::marker::PhantomData;
use embassy_hal_common::unborrow;

use embedded_storage::nor_flash::{
    ErrorType, NorFlash, NorFlashError, NorFlashErrorKind, ReadNorFlash,
};

pub use crate::pac::ERASE_SIZE;
pub use crate::pac::ERASE_VALUE;
pub use crate::pac::FLASH_BASE;
pub use crate::pac::FLASH_SIZE;
pub use crate::pac::WRITE_SIZE;
const FLASH_END: usize = FLASH_BASE + FLASH_SIZE;

#[cfg_attr(any(flash_wl, flash_wb, flash_l0, flash_l1, flash_l4), path = "l.rs")]
#[cfg_attr(flash_f3, path = "f3.rs")]
#[cfg_attr(flash_f7, path = "f7.rs")]
#[cfg_attr(flash_h7, path = "h7.rs")]
mod family;

pub struct Flash<'d> {
    _inner: FLASH,
    _phantom: PhantomData<&'d mut FLASH>,
}

impl<'d> Flash<'d> {
    pub fn new(p: impl Unborrow<Target = FLASH>) -> Self {
        unborrow!(p);
        Self {
            _inner: p,
            _phantom: PhantomData,
        }
    }

    pub fn unlock(p: impl Unborrow<Target = FLASH>) -> Self {
        let flash = Self::new(p);

        unsafe { family::unlock() };
        flash
    }

    pub fn lock(&mut self) {
        unsafe { family::lock() };
    }

    pub fn blocking_read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Error> {
        let offset = FLASH_BASE as u32 + offset;
        if offset as usize >= FLASH_END || offset as usize + bytes.len() > FLASH_END {
            return Err(Error::Size);
        }

        let flash_data = unsafe { core::slice::from_raw_parts(offset as *const u8, bytes.len()) };
        bytes.copy_from_slice(flash_data);
        Ok(())
    }

    pub fn blocking_write(&mut self, offset: u32, buf: &[u8]) -> Result<(), Error> {
        let offset = FLASH_BASE as u32 + offset;
        if offset as usize + buf.len() > FLASH_END {
            return Err(Error::Size);
        }
        if offset as usize % WRITE_SIZE != 0 || buf.len() as usize % WRITE_SIZE != 0 {
            return Err(Error::Unaligned);
        }
        trace!("Writing {} bytes at 0x{:x}", buf.len(), offset);

        self.clear_all_err();

        unsafe { family::blocking_write(offset, buf) }
    }

    pub fn blocking_erase(&mut self, from: u32, to: u32) -> Result<(), Error> {
        let from = FLASH_BASE as u32 + from;
        let to = FLASH_BASE as u32 + to;
        if to < from || to as usize > FLASH_END {
            return Err(Error::Size);
        }
        if ((to - from) as usize % ERASE_SIZE) != 0 {
            return Err(Error::Unaligned);
        }

        self.clear_all_err();

        unsafe { family::blocking_erase(from, to) }
    }

    fn clear_all_err(&mut self) {
        unsafe { family::clear_all_err() };
    }
}

impl Drop for Flash<'_> {
    fn drop(&mut self) {
        self.lock();
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

impl<'d> ErrorType for Flash<'d> {
    type Error = Error;
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

impl<'d> ReadNorFlash for Flash<'d> {
    const READ_SIZE: usize = WRITE_SIZE;

    fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        self.blocking_read(offset, bytes)
    }

    fn capacity(&self) -> usize {
        FLASH_SIZE
    }
}

impl<'d> NorFlash for Flash<'d> {
    const WRITE_SIZE: usize = WRITE_SIZE;
    const ERASE_SIZE: usize = ERASE_SIZE;

    fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
        self.blocking_erase(from, to)
    }

    fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        self.blocking_write(offset, bytes)
    }
}

/*
cfg_if::cfg_if! {
    if #[cfg(feature = "nightly")]
    {
        use embedded_storage_async::nor_flash::{AsyncNorFlash, AsyncReadNorFlash};
        use core::future::Future;

        impl<'d> AsyncNorFlash for Flash<'d> {
            const WRITE_SIZE: usize = <Self as NorFlash>::WRITE_SIZE;
            const ERASE_SIZE: usize = <Self as NorFlash>::ERASE_SIZE;

            type WriteFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a;
            fn write<'a>(&'a mut self, offset: u32, data: &'a [u8]) -> Self::WriteFuture<'a> {
                async move {
                    todo!()
                }
            }

            type EraseFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a;
            fn erase<'a>(&'a mut self, from: u32, to: u32) -> Self::EraseFuture<'a> {
                async move {
                    todo!()
                }
            }
        }

        impl<'d> AsyncReadNorFlash for Flash<'d> {
            const READ_SIZE: usize = <Self as ReadNorFlash>::READ_SIZE;
            type ReadFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a;
            fn read<'a>(&'a mut self, address: u32, data: &'a mut [u8]) -> Self::ReadFuture<'a> {
                async move {
                    todo!()
                }
            }

            fn capacity(&self) -> usize {
                FLASH_SIZE
            }
        }
    }
}
*/
