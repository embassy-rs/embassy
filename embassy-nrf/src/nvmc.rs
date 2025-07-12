//! Non-Volatile Memory Controller (NVMC, AKA internal flash) driver.

use core::{ptr, slice};

use embedded_storage::nor_flash::{
    ErrorType, MultiwriteNorFlash, NorFlash, NorFlashError, NorFlashErrorKind, ReadNorFlash,
};

use crate::pac::nvmc::vals;
use crate::peripherals::NVMC;
use crate::{pac, Peri};

#[cfg(not(feature = "_nrf5340-net"))]
/// Erase size of NVMC flash in bytes.
pub const PAGE_SIZE: usize = 4096;
#[cfg(feature = "_nrf5340-net")]
/// Erase size of NVMC flash in bytes.
pub const PAGE_SIZE: usize = 2048;

/// Size of NVMC flash in bytes.
pub const FLASH_SIZE: usize = crate::chip::FLASH_SIZE;

/// Error type for NVMC operations.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// Operation using a location not in flash.
    OutOfBounds,
    /// Unaligned operation or using unaligned buffers.
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

/// Non-Volatile Memory Controller (NVMC) that implements the `embedded-storage` traits.
pub struct Nvmc<'d> {
    _p: Peri<'d, NVMC>,
}

impl<'d> Nvmc<'d> {
    /// Create Nvmc driver.
    pub fn new(_p: Peri<'d, NVMC>) -> Self {
        Self { _p }
    }

    fn regs() -> pac::nvmc::Nvmc {
        pac::NVMC
    }

    fn wait_ready(&mut self) {
        let p = Self::regs();
        while !p.ready().read().ready() {}
    }

    #[cfg(not(any(feature = "_nrf91", feature = "_nrf5340")))]
    fn wait_ready_write(&mut self) {
        self.wait_ready();
    }

    #[cfg(any(feature = "_nrf91", feature = "_nrf5340"))]
    fn wait_ready_write(&mut self) {
        let p = Self::regs();
        while !p.readynext().read().readynext() {}
    }

    #[cfg(not(any(feature = "_nrf91", feature = "_nrf5340")))]
    fn erase_page(&mut self, page_addr: u32) {
        Self::regs().erasepage().write_value(page_addr);
    }

    #[cfg(any(feature = "_nrf91", feature = "_nrf5340"))]
    fn erase_page(&mut self, page_addr: u32) {
        let first_page_word = page_addr as *mut u32;
        unsafe {
            first_page_word.write_volatile(0xFFFF_FFFF);
        }
    }

    fn enable_erase(&self) {
        #[cfg(not(feature = "_ns"))]
        Self::regs().config().write(|w| w.set_wen(vals::Wen::EEN));
        #[cfg(feature = "_ns")]
        Self::regs().configns().write(|w| w.set_wen(vals::ConfignsWen::EEN));
    }

    fn enable_read(&self) {
        #[cfg(not(feature = "_ns"))]
        Self::regs().config().write(|w| w.set_wen(vals::Wen::REN));
        #[cfg(feature = "_ns")]
        Self::regs().configns().write(|w| w.set_wen(vals::ConfignsWen::REN));
    }

    fn enable_write(&self) {
        #[cfg(not(feature = "_ns"))]
        Self::regs().config().write(|w| w.set_wen(vals::Wen::WEN));
        #[cfg(feature = "_ns")]
        Self::regs().configns().write(|w| w.set_wen(vals::ConfignsWen::WEN));
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

        self.enable_erase();
        self.wait_ready();

        for page_addr in (from..to).step_by(PAGE_SIZE) {
            self.erase_page(page_addr);
            self.wait_ready();
        }

        self.enable_read();
        self.wait_ready();

        Ok(())
    }

    fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        if offset as usize + bytes.len() > FLASH_SIZE {
            return Err(Error::OutOfBounds);
        }
        if offset as usize % 4 != 0 || bytes.len() % 4 != 0 {
            return Err(Error::Unaligned);
        }

        self.enable_write();
        self.wait_ready();

        unsafe {
            let p_src = bytes.as_ptr() as *const u32;
            let p_dst = offset as *mut u32;
            let words = bytes.len() / 4;
            for i in 0..words {
                let w = ptr::read_unaligned(p_src.add(i));
                ptr::write_volatile(p_dst.add(i), w);
                self.wait_ready_write();
            }
        }

        self.enable_read();
        self.wait_ready();

        Ok(())
    }
}
