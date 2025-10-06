//! Resistive Random-Access Memory Controller driver.

use core::{ptr, slice};

use embedded_storage::nor_flash::{
    ErrorType, MultiwriteNorFlash, NorFlash, NorFlashError, NorFlashErrorKind, ReadNorFlash,
};

use crate::peripherals::RRAMC;
use crate::{Peri, pac};

//
// Export Nvmc alias and page size for downstream compatibility
//
/// RRAM-backed `Nvmc` compatibile driver.
pub type Nvmc<'d> = Rramc<'d>;
/// Emulated page size.  RRAM does not use pages. This exists only for downstream compatibility.
pub const PAGE_SIZE: usize = 4096;

// In bytes, one line is 128 bits
const WRITE_LINE_SIZE: usize = 16;

/// Size of RRAM flash in bytes.
pub const FLASH_SIZE: usize = crate::chip::FLASH_SIZE;

/// Error type for RRAMC operations.
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

/// Resistive Random-Access Memory Controller (RRAMC) that implements the `embedded-storage`
/// traits.
pub struct Rramc<'d> {
    _p: Peri<'d, RRAMC>,
}

impl<'d> Rramc<'d> {
    /// Create Rramc driver.
    pub fn new(_p: Peri<'d, RRAMC>) -> Self {
        Self { _p }
    }

    fn regs() -> pac::rramc::Rramc {
        pac::RRAMC
    }

    fn wait_ready(&mut self) {
        let p = Self::regs();
        while !p.ready().read().ready() {}
    }

    fn wait_ready_write(&mut self) {
        let p = Self::regs();
        while !p.readynext().read().readynext() {}
        while !p.bufstatus().writebufempty().read().empty() {}
    }

    fn enable_read(&self) {
        Self::regs().config().write(|w| w.set_wen(false));
    }

    fn enable_write(&self) {
        Self::regs().config().write(|w| w.set_wen(true));
    }
}

//
// RRAM is not NOR flash, but many crates require embedded-storage NorFlash traits. We therefore
// implement the traits for downstream compatibility.
//

impl<'d> MultiwriteNorFlash for Rramc<'d> {}

impl<'d> ErrorType for Rramc<'d> {
    type Error = Error;
}

impl<'d> ReadNorFlash for Rramc<'d> {
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

impl<'d> NorFlash for Rramc<'d> {
    const WRITE_SIZE: usize = WRITE_LINE_SIZE;
    const ERASE_SIZE: usize = PAGE_SIZE;

    // RRAM can overwrite in-place, so emulate page erases by overwriting the page bytes with 0xFF.
    fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
        if to < from || to as usize > FLASH_SIZE {
            return Err(Error::OutOfBounds);
        }
        if from as usize % Self::ERASE_SIZE != 0 || to as usize % Self::ERASE_SIZE != 0 {
            return Err(Error::Unaligned);
        }

        self.enable_write();
        self.wait_ready();

        // Treat each emulated page separately so callers can rely on post‑erase read‑back
        // returning 0xFF just like on real NOR flash.
        let buf = [0xFFu8; Self::WRITE_SIZE];
        for page_addr in (from..to).step_by(Self::ERASE_SIZE) {
            let page_end = page_addr + Self::ERASE_SIZE as u32;
            for line_addr in (page_addr..page_end).step_by(Self::WRITE_SIZE) {
                unsafe {
                    let src = buf.as_ptr() as *const u32;
                    let dst = line_addr as *mut u32;
                    for i in 0..(Self::WRITE_SIZE / 4) {
                        core::ptr::write_volatile(dst.add(i), core::ptr::read_unaligned(src.add(i)));
                    }
                }
                self.wait_ready_write();
            }
        }

        self.enable_read();
        self.wait_ready();
        Ok(())
    }

    fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        if offset as usize + bytes.len() > FLASH_SIZE {
            return Err(Error::OutOfBounds);
        }
        if offset as usize % Self::WRITE_SIZE != 0 || bytes.len() % Self::WRITE_SIZE != 0 {
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
                if (i + 1) % (Self::WRITE_SIZE / 4) == 0 {
                    self.wait_ready_write();
                }
            }
        }

        self.enable_read();
        self.wait_ready();
        Ok(())
    }
}
