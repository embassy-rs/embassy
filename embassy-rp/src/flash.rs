use embedded_storage::nor_flash::{
    ErrorType, MultiwriteNorFlash, NorFlash, NorFlashError, NorFlashErrorKind, ReadNorFlash,
};

/// Error type for NVMC operations.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// Opration using a location not in flash.
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

pub struct Flash<const FLASH_SIZE: usize>;

impl<const FLASH_SIZE: usize> ErrorType for Flash<FLASH_SIZE> {
    type Error = Error;
}

impl<const FLASH_SIZE: usize> MultiwriteNorFlash for Flash<FLASH_SIZE> {}

impl<const FLASH_SIZE: usize> ReadNorFlash for Flash<FLASH_SIZE> {
    const READ_SIZE: usize = 1;

    fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        if offset as usize >= FLASH_SIZE || offset as usize + bytes.len() > FLASH_SIZE {
            return Err(Error::OutOfBounds);
        }

        let flash_data = unsafe { core::slice::from_raw_parts(offset as *const u8, bytes.len()) };
        bytes.copy_from_slice(flash_data);
        Ok(())
    }

    fn capacity(&self) -> usize {
        FLASH_SIZE
    }
}

impl<const FLASH_SIZE: usize> NorFlash for Flash<FLASH_SIZE> {
    const WRITE_SIZE: usize = 4;

    const ERASE_SIZE: usize = 4096;

    fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
        if to < from || to as usize > FLASH_SIZE {
            return Err(Error::OutOfBounds);
        }
        if from as usize % Self::ERASE_SIZE != 0 || to as usize % Self::ERASE_SIZE != 0 {
            return Err(Error::Unaligned);
        }

        let len = to - from;

        // Make sure to uphold the contract point with rp2040-flash.
        // - interrupts must be disabled
        // - DMA must not access flash memory
        // FIXME: Pause all DMA channels for the duration of the flash_write?

        critical_section::with(|_| {
            unsafe { rp2040_flash::flash::flash_range_erase(from, len, true) };
        });

        // Re-enable DMA channels

        Ok(())
    }

    fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        if offset as usize + bytes.len() > FLASH_SIZE {
            return Err(Error::OutOfBounds);
        }
        if offset as usize % 4 != 0 || bytes.len() as usize % 4 != 0 {
            return Err(Error::Unaligned);
        }

        // Make sure to uphold the contract point with rp2040-flash.
        // - interrupts must be disabled
        // - DMA must not access flash memory
        // FIXME: Pause all DMA channels for the duration of the flash_write?

        critical_section::with(|_| {
            unsafe { rp2040_flash::flash::flash_range_program(offset, bytes, true) };
        });

        // Re-enable DMA channels

        Ok(())
    }
}
