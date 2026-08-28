use core::ops::Range;
use core::ptr::read_volatile;

use embassy_hal_internal::drop::OnDrop;
use stm32_metapac::flash::vals::Bksel;

use super::{Blocking, Error, Flash, family};
use crate::pac;

/// Size in bytes of one physical EDATA bank.
pub const EDATA_BANK_SIZE: u32 = 0x6000;
/// Size in bytes of one erasable EDATA page.
pub const EDATA_PAGE_SIZE: u32 = 0x600;
/// Minimum EDATA write size and required write alignment, in bytes.
pub const EDATA_WRITE_SIZE: usize = 2;

const EDATA_BASE: u32 = 0x0900_0000;
const EDATA_PAGE_COUNT: u8 = (EDATA_BANK_SIZE / EDATA_PAGE_SIZE) as u8;
const EDATA_BANK2_BASE: u32 = EDATA_BASE + EDATA_BANK_SIZE;

/// A physical EDATA flash bank.
///
/// The address used to access a bank follows the current `SWAP_BANK` mapping,
/// while erase operations always select the physical bank.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

    /// Reads `output.len()` bytes from an EDATA bank.
    ///
    /// `offset` is a byte offset from the beginning of the selected physical
    /// bank and does not need to be aligned to 2 bytes.
    ///
    /// # Hardware behavior
    ///
    /// EDATA associates ECC information with each aligned 16-bit word.
    /// Although this function accepts unaligned byte ranges, it reads the
    /// aligned 16-bit word containing each requested byte.
    ///
    /// Reading a virgin EDATA word, such as a word that has not been programmed
    /// since the page was erased, raises a double ECC error. Therefore, callers
    /// should ensure that every 16-bit word touched by the requested range has
    /// been programmed before reading it.
    ///
    /// See RM0522, "Error correction codes (ECC)."
    pub fn edata_read(&self, bank: EDataBank, offset: u32, output: &mut [u8]) -> Result<(), Error> {
        self.check_edata_enabled()?;
        let mut address = checked_address(bank, offset, output.len())?;

        let mut i = 0;

        if address % 2 != 0 && i < output.len() {
            let value = unsafe { read_volatile((address - 1) as *const u16) };

            output[i] = value.to_le_bytes()[1];

            address += 1;
            i += 1;
        }

        while i + 1 < output.len() {
            let value = unsafe { read_volatile(address as *const u16) };

            let bytes = value.to_le_bytes();
            output[i] = bytes[0];
            output[i + 1] = bytes[1];

            address += 2;
            i += 2;
        }

        if i < output.len() {
            let value = unsafe { read_volatile(address as *const u16) };

            output[i] = value.to_le_bytes()[0];
        }

        Ok(())
    }

    /// Reads a 16-bit value from an EDATA bank.
    ///
    /// `offset` is a byte offset from the beginning of the selected physical
    /// bank and must be aligned to 2 bytes.
    ///
    /// # Hardware behavior
    ///
    /// EDATA associates ECC information with each aligned 16-bit word.
    /// Reading a virgin EDATA word, such as a word that has not been programmed
    /// since the page was erased, raises a double ECC error. Callers should
    /// ensure that the requested word has been programmed before reading it.
    ///
    /// See RM0522, "Error correction codes (ECC)."
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
    ///
    /// # Hardware behavior
    ///
    /// EDATA associates ECC information with each aligned 16-bit word.
    /// Reading a virgin EDATA word, such as a word that has not been programmed
    /// since the page was erased, raises a double ECC error. Callers should
    /// ensure that every requested word has been programmed before reading it.
    ///
    /// See RM0522, "Error correction codes (ECC)."
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
    let range = checked_range(offset, size, EDATA_BANK_SIZE)?;
    bank.base().checked_add(range.start).ok_or(Error::Size)
}

fn checked_range(offset: u32, len: usize, bank_size: u32) -> Result<Range<u32>, Error> {
    let len = u32::try_from(len).map_err(|_| Error::Size)?;
    let end = offset.checked_add(len).ok_or(Error::Size)?;

    if end > bank_size {
        return Err(Error::Size);
    }

    Ok(offset..end)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checked_range_accepts_valid_ranges() {
        assert_eq!(checked_range(0, 0, 16), Ok(0..0));
        assert_eq!(checked_range(0, 2, 16), Ok(0..2));
        assert_eq!(checked_range(1, 1, 16), Ok(1..2));
        assert_eq!(checked_range(15, 1, 16), Ok(15..16));
        assert_eq!(checked_range(0, 16, 16), Ok(0..16));
    }

    #[test]
    fn checked_range_rejects_out_of_bounds() {
        assert_eq!(checked_range(15, 2, 16), Err(Error::Size));
        assert_eq!(checked_range(u32::MAX, 1, u32::MAX), Err(Error::Size));
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn checked_range_rejects_lengths_larger_than_u32() {
        assert_eq!(checked_range(0, u32::MAX as usize + 1, u32::MAX), Err(Error::Size));
    }
}
