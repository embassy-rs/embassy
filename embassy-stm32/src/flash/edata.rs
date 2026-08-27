use core::ptr::read_volatile;

use embassy_hal_internal::drop::OnDrop;
use stm32_metapac::flash::vals::Bksel;

use super::{Blocking, Error, Flash, family};
use crate::pac;

const EDATA_BASE: u32 = 0x0900_0000;

const EDATA_BANK_SIZE: u32 = 0x6000;
const EDATA_PAGE_SIZE: u32 = 0x600;
const EDATA_WRITE_SIZE: usize = 2;
const EDATA_PAGE_COUNT: u8 = (EDATA_BANK_SIZE / EDATA_PAGE_SIZE) as u8;
const EDATA_BANK2_BASE: u32 = EDATA_BASE + EDATA_BANK_SIZE;

/// A physical EDATA flash bank.
///
/// The address used to access a bank follows the current `SWAP_BANK` mapping,
/// while erase operations always select the physical bank.
#[derive(Clone, Copy)]
pub enum EDataBank {
    /// Physical EDATA bank 1.
    Bank1,
    /// Physical EDATA bank 2.
    Bank2,
}

impl EDataBank {
    fn base(self) -> u32 {
        match (self, family::banks_swapped()) {
            (Self::Bank1, false) | (Self::Bank2, true) => EDATA_BASE,
            (Self::Bank2, false) | (Self::Bank1, true) => EDATA_BANK2_BASE,
        }
    }

    fn selector(self) -> Bksel {
        match self {
            Self::Bank1 => Bksel::Bank1,
            Self::Bank2 => Bksel::Bank2,
        }
    }
}

impl<'d> Flash<'d, Blocking> {
    /// Returns whether the EDATA area is enabled by the option bytes.
    pub fn edata_is_enabled(&self) -> bool {
        pac::FLASH.optsr_cur().read().edata_en()
    }

    fn check_edata_enabled(&self) -> Result<(), Error> {
        if self.edata_is_enabled() {
            Ok(())
        } else {
            Err(Error::EdataDisabled)
        }
    }

    /// Reads a 16-bit value from an EDATA bank.
    ///
    /// `offset` is a byte offset from the beginning of the selected physical
    /// bank and must be aligned to 2 bytes.
    pub fn edata_read_u16(&self, bank: EDataBank, offset: u32) -> Result<u16, Error> {
        self.check_edata_enabled()?;

        if offset % 2 != 0 {
            return Err(Error::Unaligned);
        }

        let address = checked_address(bank, offset, 2)?;

        Ok(unsafe { read_volatile(address as *const u16) })
    }

    /// Reads 16-bit values from an EDATA bank into `output`.
    ///
    /// `offset` is a byte offset from the beginning of the selected physical
    /// bank and must be aligned to 2 bytes.
    pub fn edata_read_u16_slice(&self, bank: EDataBank, offset: u32, output: &mut [u16]) -> Result<(), Error> {
        self.check_edata_enabled()?;

        if offset % 2 != 0 {
            return Err(Error::Unaligned);
        }

        let byte_len = output.len().checked_mul(2).ok_or(Error::Size)?;

        let mut address = checked_address(bank, offset, byte_len)?;

        for value in output {
            *value = unsafe { read_volatile(address as *const u16) };

            address += 2;
        }

        Ok(())
    }

    /// Writes a 16-bit value to an EDATA bank.
    ///
    /// `offset` is a byte offset from the beginning of the selected physical
    /// bank and must be aligned to 2 bytes.
    pub fn edata_write_u16(&mut self, bank: EDataBank, offset: u32, value: u16) -> Result<(), Error> {
        self.check_edata_enabled()?;

        if offset % 2 != 0 {
            return Err(Error::Unaligned);
        }

        let address = checked_address(bank, offset, 2)?;

        unsafe { family::unlock() };
        let _lock = OnDrop::new(|| unsafe { family::lock() });

        unsafe { family::blocking_write_edata_u16(address, value) }
    }

    /// Writes 16-bit values to an EDATA bank.
    ///
    /// `offset` is a byte offset from the beginning of the selected physical
    /// bank and must be aligned to 2 bytes.
    pub fn edata_write_u16_slice(&mut self, bank: EDataBank, offset: u32, values: &[u16]) -> Result<(), Error> {
        self.check_edata_enabled()?;

        if offset % 2 != 0 {
            return Err(Error::Unaligned);
        }

        let byte_len = values.len().checked_mul(2).ok_or(Error::Size)?;

        let address = checked_address(bank, offset, byte_len)?;

        unsafe { family::unlock() };
        let _lock = OnDrop::new(|| unsafe { family::lock() });

        unsafe { family::blocking_write_edata_u16_slice(address, values) }
    }

    /// Writes a 32-bit value to an EDATA bank.
    ///
    /// `offset` is a byte offset from the beginning of the selected physical
    /// bank and must be aligned to 4 bytes.
    pub fn edata_write_u32(&mut self, bank: EDataBank, offset: u32, value: u32) -> Result<(), Error> {
        self.check_edata_enabled()?;

        if offset % 4 != 0 {
            return Err(Error::Unaligned);
        }

        let address = checked_address(bank, offset, 4)?;

        unsafe { family::unlock() };
        let _lock = OnDrop::new(|| unsafe { family::lock() });

        unsafe { family::blocking_write_edata_u32(address, value) }
    }

    /// Writes bytes to an EDATA bank using 16-bit programming operations.
    ///
    /// `offset` is a byte offset from the beginning of the selected physical
    /// bank. Both `offset` and `data.len()` must be aligned to 2 bytes.
    pub fn edata_write(&mut self, bank: EDataBank, offset: u32, data: &[u8]) -> Result<(), Error> {
        self.check_edata_enabled()?;

        if offset % EDATA_WRITE_SIZE as u32 != 0 || data.len() % EDATA_WRITE_SIZE != 0 {
            return Err(Error::Unaligned);
        }

        let end = offset.checked_add(data.len() as u32).ok_or(Error::Size)?;

        if end > EDATA_BANK_SIZE {
            return Err(Error::Size);
        }

        let address = checked_address(bank, offset, data.len())?;

        unsafe { family::unlock() };
        let _lock = OnDrop::new(|| unsafe { family::lock() });

        unsafe { family::blocking_write_edata(address, data) }
    }

    /// Erases one page in an EDATA bank.
    ///
    /// `page` is a zero-based physical page index within the selected bank.
    pub fn edata_erase_page(&mut self, bank: EDataBank, page: u8) -> Result<(), Error> {
        self.check_edata_enabled()?;

        if page >= EDATA_PAGE_COUNT {
            return Err(Error::Size);
        }

        unsafe { family::unlock() };
        let _lock = OnDrop::new(|| unsafe { family::lock() });
        unsafe { family::blocking_erase_edata_page(bank.selector(), page) }
    }
}

fn checked_address(bank: EDataBank, offset: u32, size: usize) -> Result<u32, Error> {
    let end = offset.checked_add(size as u32).ok_or(Error::Size)?;

    if end > EDATA_BANK_SIZE {
        return Err(Error::Size);
    }

    bank.base().checked_add(offset).ok_or(Error::Size)
}
