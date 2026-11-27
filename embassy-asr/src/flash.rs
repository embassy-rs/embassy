//! Embedded flash controller (EFC).
//!
//! Sequences follow the vendor `tremo_flash` driver and the local flash
//! algorithm used by probe-rs.

use core::ptr::{read_volatile, write_volatile};

use embedded_storage::nor_flash::{
    ErrorType, MultiwriteNorFlash, NorFlash, NorFlashError, NorFlashErrorKind, ReadNorFlash,
};

use crate::sec::flash_access;
use crate::{Peri, pac, peripherals};

const FLASH_BASE: u32 = 0x0800_0000;
const FLASH_SIZE_TAG_ADDR: u32 = 0x1000_2010;
const PAGE_SIZE: u32 = 0x1000;
const WRITE_SIZE: u32 = 8;
const PROTECT_SEQ0: u32 = 0x8c9d_aebf;
const PROTECT_SEQ1: u32 = 0x1314_1516;

/// Flash driver error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// Address is outside the main flash array or misaligned.
    Address,
    /// Length is invalid for the requested operation.
    Length,
    /// Flash security policy rejected the operation.
    Security,
}

impl NorFlashError for Error {
    fn kind(&self) -> NorFlashErrorKind {
        match self {
            Self::Address | Self::Length => NorFlashErrorKind::OutOfBounds,
            Self::Security => NorFlashErrorKind::Other,
        }
    }
}

/// Owned flash controller for the main array at `0x0800_0000`.
pub struct Flash<'d> {
    _peri: Peri<'d, peripherals::EFC>,
    size: u32,
}

impl<'d> Flash<'d> {
    /// Create a flash driver.
    ///
    /// Flash size is derived from the vendor INFO tag at `0x1000_2010`, matching
    /// the SDK / flash algorithm: bits `[1:0] == 0` means 128 KiB, otherwise
    /// 256 KiB.
    pub fn new(peri: Peri<'d, peripherals::EFC>) -> Self {
        let tag = unsafe { read_volatile(FLASH_SIZE_TAG_ADDR as *const u32) } & 0x3;
        let size = if tag == 0 { 0x2_0000 } else { 0x4_0000 };
        Self { _peri: peri, size }
    }

    /// Main flash capacity in bytes.
    pub fn capacity(&self) -> u32 {
        self.size
    }

    /// Erase one 4 KiB page.
    pub fn erase_page(&mut self, offset: u32) -> Result<(), Error> {
        if offset % PAGE_SIZE != 0 || offset >= self.size {
            return Err(Error::Address);
        }
        let addr = FLASH_BASE + offset;
        flash_access::clear_error();
        let efc = Self::regs();
        let ecc_disable = efc.cr().read().ecc_disable().bit_is_set();
        unlock(&efc);
        unsafe {
            efc.cr().write_with_zero(|w| {
                w.page_erase_en().set_bit();
                w.prefetch_en().set_bit();
                if ecc_disable {
                    w.ecc_disable().set_bit();
                }
                w
            });
        }
        lock(&efc);
        unsafe {
            write_volatile(addr as *mut u32, 0xffff_ffff);
        }
        if flash_access::take_error() {
            return Err(Error::Security);
        }
        wait_done(&efc);
        Ok(())
    }

    /// Erase the entire main flash array page by page.
    pub fn erase_all(&mut self) -> Result<(), Error> {
        let mut offset = 0;
        while offset < self.size {
            self.erase_page(offset)?;
            offset += PAGE_SIZE;
        }
        Ok(())
    }

    /// Program `data` starting at byte `offset`.
    ///
    /// The offset must be 8-byte aligned. Trailing bytes that do not fill an
    /// 8-byte unit are padded with `0xFF`, matching the vendor driver.
    pub fn program(&mut self, offset: u32, data: &[u8]) -> Result<(), Error> {
        if offset % WRITE_SIZE != 0 || offset >= self.size {
            return Err(Error::Address);
        }
        if data.is_empty() {
            return Err(Error::Length);
        }
        if offset + data.len() as u32 > self.size {
            return Err(Error::Length);
        }

        flash_access::clear_error();
        let efc = Self::regs();
        let ecc_disable = efc.cr().read().ecc_disable().bit_is_set();
        unlock(&efc);
        unsafe {
            efc.cr().write_with_zero(|w| {
                w.prog_en().set_bit();
                w.prefetch_en().set_bit();
                if ecc_disable {
                    w.ecc_disable().set_bit();
                }
                w
            });
        }
        lock(&efc);

        let mut i = 0;
        while i < data.len() {
            let mut chunk = [0xffu8; 8];
            let remaining = data.len() - i;
            let copy = remaining.min(8);
            chunk[..copy].copy_from_slice(&data[i..i + copy]);
            let d0 = u32::from_le_bytes(chunk[0..4].try_into().unwrap());
            let d1 = u32::from_le_bytes(chunk[4..8].try_into().unwrap());
            unsafe {
                efc.program_data0().write_with_zero(|w| w.bits(d0));
                efc.program_data1().write_with_zero(|w| w.bits(d1));
                write_volatile((FLASH_BASE + offset + i as u32) as *mut u32, 0xffff_ffff);
            }
            if flash_access::take_error() {
                return Err(Error::Security);
            }
            wait_done(&efc);
            i += 8;
        }
        Ok(())
    }

    /// Read flash contents into `buf`.
    pub fn read(&self, offset: u32, buf: &mut [u8]) -> Result<(), Error> {
        if offset >= self.size {
            return Err(Error::Address);
        }
        if offset + buf.len() as u32 > self.size {
            return Err(Error::Length);
        }
        let src = (FLASH_BASE + offset) as *const u8;
        for (i, byte) in buf.iter_mut().enumerate() {
            *byte = unsafe { read_volatile(src.add(i)) };
        }
        Ok(())
    }

    #[inline]
    fn regs() -> pac::Efc {
        unsafe { pac::Efc::steal() }
    }
}

fn unlock(efc: &pac::Efc) {
    unsafe {
        efc.protect_seq().write_with_zero(|w| w.bits(PROTECT_SEQ0));
        efc.protect_seq().write_with_zero(|w| w.bits(PROTECT_SEQ1));
    }
}

fn lock(efc: &pac::Efc) {
    unsafe {
        efc.protect_seq().write_with_zero(|w| w.bits(PROTECT_SEQ0));
        efc.protect_seq().write_with_zero(|w| w.bits(0));
    }
}

fn wait_done(efc: &pac::Efc) {
    while efc.sr().read().operation_done().bit_is_clear() {}
    let status = efc.sr().read().bits();
    unsafe {
        efc.sr().write_with_zero(|w| w.bits(status));
    }
}

impl ErrorType for Flash<'_> {
    type Error = Error;
}

impl ReadNorFlash for Flash<'_> {
    const READ_SIZE: usize = 1;

    fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        Flash::read(self, offset, bytes)
    }

    fn capacity(&self) -> usize {
        self.size as usize
    }
}

impl NorFlash for Flash<'_> {
    const WRITE_SIZE: usize = WRITE_SIZE as usize;
    const ERASE_SIZE: usize = PAGE_SIZE as usize;

    fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
        if from % PAGE_SIZE != 0 || to % PAGE_SIZE != 0 || from >= to || to > self.size {
            return Err(Error::Address);
        }
        let mut offset = from;
        while offset < to {
            self.erase_page(offset)?;
            offset += PAGE_SIZE;
        }
        Ok(())
    }

    fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        if bytes.len() % WRITE_SIZE as usize != 0 {
            return Err(Error::Length);
        }
        self.program(offset, bytes)
    }
}

impl MultiwriteNorFlash for Flash<'_> {}
