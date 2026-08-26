use embassy_hal_internal::drop::OnDrop;
use stm32_metapac::{flash::vals::Bksel, metadata::ir::Block};

use super::{Blocking, Error, Flash, family};
use crate::pac;

const EDATA_BANK1_BASE: u32 = 0x0900_0000;
const EDATA_BANK2_BASE: u32 = 0x0900_6000;

const EDATA_BANK_SIZE: u32 = 0x6000;
const EDATA_PAGE_SIZE: u32 = 0x600;
const EDATA_PAGE_COUNT: u8 = 16;
const EDATA_WRITE_SIZE: usize = 16;

#[derive(Clone, Copy)]
pub enum EDataBank {
    Bank1,
    Bank2
}

impl EDataBank {
    fn base(self) -> u32 {
        match self {
            Self::Bank1 => EDATA_BANK1_BASE,
            Self::Bank2 => EDATA_BANK2_BASE
        }
    }

    fn selector(self) -> Bksel {
        match self {
            Self::Bank1 => Bksel::Bank1,
            Self::Bank2 => Bksel::Bank2
        }
    }
}

impl<'d> Flash<'d, Blocking> {
    pub fn edata_is_enabled(&self) -> bool {
        pac::FLASH.optsr_cur().read().edata_en()
    }

    pub fn edata_read(
        &self,
        bank: EDataBank,
        offset: u32,
        buf: &mut [u8]
    ) -> Result<(), Error> {
        let end = offset.checked_add(buf.len() as u32).ok_or(Error::Size)?;

        if end > EDATA_BANK_SIZE {
            return Err(Error::Size);
        }

        let address = bank.base() + offset;

        unsafe {
            let source = core::slice::from_raw_parts(address as *const u8, buf.len());
            buf.copy_from_slice(source);
        }

        Ok(())
    }

    pub fn edata_write(
        &mut self,
        bank: EDataBank,
        offset: u32,
        data: &[u8]
    ) -> Result<(), Error> {
        if !self.edata_is_enabled() {
            return Err(Error::Unsupported);
        }

        if offset % EDATA_WRITE_SIZE as u32 != 0 || data.len() % EDATA_WRITE_SIZE != 0 {
            return Err(Error::Unaligned);
        }

        let end = offset.checked_add(data.len() as u32).ok_or(Error::Size)?;

        if end > EDATA_BANK_SIZE {
            return Err(Error::Size);
        }

        unsafe {
            family::unlock();
            let _lock = OnDrop::new(|| family::lock());

            for (index, chunk) in data.chunks_exact(EDATA_WRITE_SIZE).enumerate() {
                let block: &[u8; EDATA_WRITE_SIZE] = chunk.try_into().unwrap();

                family::blocking_write(bank.base() + offset + index as u32 * EDATA_WRITE_SIZE as u32, block)?;
            }
        }

        Ok(())
    }

    pub fn edata_erase_page(&mut self, bank: EDataBank, page: u8) -> Result<(), Error> {
        if !self.edata_is_enabled() {
            return Err(Error::Unsupported);
        }

        if page >= EDATA_PAGE_COUNT {
            return Err(Error::Size);
        }

        unsafe {
            family::unlock();
            let _lock = OnDrop::new(|| family::lock());

            family::blocking_erase_edata_page(bank.selector(), page)
        }
    }
}