//! NorFlash trait implementation wrapping the XSPI flash driver.
//!
//! Allows embassy-boot to use the external NOR flash for swap/state operations.

use embassy_stm32::xspi::Instance;
use embedded_storage::nor_flash::{ErrorType, NorFlash, NorFlashError, NorFlashErrorKind, ReadNorFlash};

use crate::xspi_flash::SpiFlashMemory;

/// NorFlash wrapper around the XSPI flash driver.
pub struct XspiNorFlash<I: Instance> {
    flash: SpiFlashMemory<I>,
}

#[derive(Debug)]
pub struct XspiFlashError;

impl NorFlashError for XspiFlashError {
    fn kind(&self) -> NorFlashErrorKind {
        NorFlashErrorKind::Other
    }
}

impl<I: Instance> XspiNorFlash<I> {
    pub fn new(flash: SpiFlashMemory<I>) -> Self {
        Self { flash }
    }

    /// Consume the wrapper and return the inner SpiFlashMemory driver.
    pub fn free(self) -> SpiFlashMemory<I> {
        self.flash
    }
}

impl<I: Instance> ErrorType for XspiNorFlash<I> {
    type Error = XspiFlashError;
}

impl<I: Instance> ReadNorFlash for XspiNorFlash<I> {
    const READ_SIZE: usize = 1;

    fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        self.flash.read_memory(offset, bytes);
        Ok(())
    }

    fn capacity(&self) -> usize {
        #[cfg(feature = "dk")]
        {
            128 * 1024 * 1024
        }
        #[cfg(feature = "nucleo")]
        {
            64 * 1024 * 1024
        }
    }
}

impl<I: Instance> NorFlash for XspiNorFlash<I> {
    const WRITE_SIZE: usize = 4;
    const ERASE_SIZE: usize = 4096; // 4K sector erase

    fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        self.flash.write_memory(offset, bytes);
        Ok(())
    }

    fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
        let mut addr = from;
        while addr < to {
            self.flash.erase_sector(addr);
            addr += Self::ERASE_SIZE as u32;
        }
        Ok(())
    }
}
