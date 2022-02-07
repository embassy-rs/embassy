//! Nvmcerature sensor interface.

use crate::pac;
use crate::peripherals::NVMC;

use core::marker::PhantomData;
use core::ptr;
use core::slice;
use embassy::util::Unborrow;
use embassy_hal_common::unborrow;
use embedded_storage::nor_flash::{
    ErrorType, MultiwriteNorFlash, NorFlash, NorFlashError, NorFlashErrorKind, ReadNorFlash,
};

pub const PAGE_SIZE: usize = 4096;
pub const FLASH_SIZE: usize = crate::chip::FLASH_SIZE;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    OutOfBounds,
    Unaligned,
}

impl NorFlashError for Error {
    fn kind(&self) -> NorFlashErrorKind {
        match self {
            Self::OutOfBounds => NorFlashErrorKind::OutOfBounds,
            Self::Unaligned => NorFlashErrorKind::NotAligned,
        }
    }
}

pub struct Nvmc<'d> {
    _p: PhantomData<&'d NVMC>,
}

impl<'d> Nvmc<'d> {
    pub fn new(_p: impl Unborrow<Target = NVMC> + 'd) -> Self {
        unborrow!(_p);

        Self { _p: PhantomData }
    }

    fn regs() -> &'static pac::nvmc::RegisterBlock {
        unsafe { &*pac::NVMC::ptr() }
    }

    fn wait_ready(&mut self) {
        let p = Self::regs();
        while p.ready.read().ready().is_busy() {}
    }
}

impl<'d> MultiwriteNorFlash for Nvmc<'d> {}

impl<'d> ErrorType for Nvmc<'d> {
    type Error = Error;
}

impl<'d> ReadNorFlash for Nvmc<'d> {
    const READ_SIZE: usize = 1;

    fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        if offset as usize >= FLASH_SIZE || offset as usize + bytes.len() > FLASH_SIZE {
            return Err(Error::OutOfBounds);
        }

        let flash_data = unsafe { slice::from_raw_parts(offset as *const u8, bytes.len()) };
        bytes.copy_from_slice(flash_data);
        Ok(())
    }

    fn capacity(&self) -> usize {
        FLASH_SIZE
    }
}

impl<'d> NorFlash for Nvmc<'d> {
    const WRITE_SIZE: usize = 4;
    const ERASE_SIZE: usize = PAGE_SIZE;

    fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
        if to < from || to as usize > FLASH_SIZE {
            return Err(Error::OutOfBounds);
        }
        if from as usize % PAGE_SIZE != 0 || to as usize % PAGE_SIZE != 0 {
            return Err(Error::Unaligned);
        }

        let p = Self::regs();

        p.config.write(|w| w.wen().een());
        self.wait_ready();

        for page in (from..to).step_by(PAGE_SIZE) {
            p.erasepage().write(|w| unsafe { w.bits(page) });
            self.wait_ready();
        }

        p.config.reset();
        self.wait_ready();

        Ok(())
    }

    fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        if offset as usize + bytes.len() > FLASH_SIZE {
            return Err(Error::OutOfBounds);
        }
        if offset as usize % 4 != 0 || bytes.len() as usize % 4 != 0 {
            return Err(Error::Unaligned);
        }

        let p = Self::regs();

        p.config.write(|w| w.wen().wen());
        self.wait_ready();

        unsafe {
            let p_src = bytes.as_ptr() as *const u32;
            let p_dst = offset as *mut u32;
            let words = bytes.len() / 4;
            for i in 0..words {
                let w = ptr::read_unaligned(p_src.add(i));
                ptr::write_volatile(p_dst.add(i), w);
                self.wait_ready();
            }
        }

        p.config.reset();
        self.wait_ready();

        Ok(())
    }
}
