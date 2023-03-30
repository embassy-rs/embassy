use embassy_hal_common::drop::OnDrop;
use embassy_hal_common::{into_ref, PeripheralRef};

use super::{family, Error, FlashLayout, FlashRegion, FlashSector, FLASH_BASE, FLASH_SIZE, WRITE_SIZE};
use crate::flash::FlashBank;
use crate::Peripheral;

pub struct Flash<'d> {
    inner: PeripheralRef<'d, crate::peripherals::FLASH>,
}

impl<'d> Flash<'d> {
    pub fn new(p: impl Peripheral<P = crate::peripherals::FLASH> + 'd) -> Self {
        into_ref!(p);
        Self { inner: p }
    }

    pub fn into_regions(self) -> FlashLayout<'d> {
        FlashLayout::new(self.release())
    }

    pub fn blocking_read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Error> {
        let start_address = FLASH_BASE as u32 + offset;
        blocking_read(start_address, bytes)
    }

    pub fn blocking_write(&mut self, offset: u32, buf: &[u8]) -> Result<(), Error> {
        let start_address = FLASH_BASE as u32 + offset;

        unsafe { blocking_write(start_address, buf) }
    }

    pub fn blocking_erase(&mut self, from: u32, to: u32) -> Result<(), Error> {
        let start_address = FLASH_BASE as u32 + from;
        let end_address = FLASH_BASE as u32 + to;

        unsafe { blocking_erase(start_address, end_address) }
    }

    pub(crate) fn release(self) -> PeripheralRef<'d, crate::peripherals::FLASH> {
        let mut flash = self;
        unsafe { flash.inner.clone_unchecked() }
    }
}

impl Drop for Flash<'_> {
    fn drop(&mut self) {
        unsafe { family::lock() };
    }
}

impl Drop for FlashLayout<'_> {
    fn drop(&mut self) {
        unsafe { family::lock() };
    }
}

fn blocking_read(start_address: u32, bytes: &mut [u8]) -> Result<(), Error> {
    assert!(start_address >= FLASH_BASE as u32);
    if start_address as usize + bytes.len() > FLASH_BASE + FLASH_SIZE {
        return Err(Error::Size);
    }

    let flash_data = unsafe { core::slice::from_raw_parts(start_address as *const u8, bytes.len()) };
    bytes.copy_from_slice(flash_data);
    Ok(())
}

unsafe fn blocking_write(start_address: u32, buf: &[u8]) -> Result<(), Error> {
    assert!(start_address >= FLASH_BASE as u32);
    if start_address as usize + buf.len() > FLASH_BASE + FLASH_SIZE {
        return Err(Error::Size);
    }
    if (start_address as usize - FLASH_BASE) % WRITE_SIZE != 0 || buf.len() as usize % WRITE_SIZE != 0 {
        return Err(Error::Unaligned);
    }

    trace!("Writing {} bytes at 0x{:x}", buf.len(), start_address);

    let mut address = start_address;
    for chunk in buf.chunks(WRITE_SIZE) {
        critical_section::with(|_| {
            family::clear_all_err();
            family::unlock();
            family::begin_write();
            let _ = OnDrop::new(|| {
                family::end_write();
                family::lock();
            });
            family::blocking_write(address, chunk.try_into().unwrap())
        })?;
        address += WRITE_SIZE as u32;
    }
    Ok(())
}

unsafe fn blocking_erase(start_address: u32, end_address: u32) -> Result<(), Error> {
    let regions = family::get_flash_regions();

    // Test if the address range is aligned at sector base addresses
    let mut address = start_address;
    while address < end_address {
        let sector = get_sector(address, regions);
        if sector.start != address {
            return Err(Error::Unaligned);
        }
        address += sector.size;
    }
    if address != end_address {
        return Err(Error::Unaligned);
    }

    trace!("Erasing from 0x{:x} to 0x{:x}", start_address, end_address);

    let mut address = start_address;
    while address < end_address {
        let sector = get_sector(address, regions);
        trace!("Erasing sector: {}", sector);

        critical_section::with(|_| {
            family::clear_all_err();
            family::unlock();
            let _ = OnDrop::new(|| {
                family::lock();
            });
            family::blocking_erase_sector(&sector)
        })?;
        address += sector.size;
    }
    Ok(())
}

pub(crate) fn get_sector(address: u32, regions: &[&FlashRegion]) -> FlashSector {
    let mut current_bank = FlashBank::Bank1;
    let mut bank_offset = 0;
    for region in regions {
        if region.bank != current_bank {
            current_bank = region.bank;
            bank_offset = 0;
        }

        if address < region.end() {
            let index_in_region = (address - region.base) / region.erase_size;
            return FlashSector {
                bank: region.bank,
                index_in_bank: bank_offset + index_in_region as u8,
                start: region.base + index_in_region * region.erase_size,
                size: region.erase_size,
            };
        }

        bank_offset += region.sectors();
    }

    panic!("Flash sector not found");
}

impl FlashRegion {
    pub fn blocking_read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Error> {
        let start_address = self.base + offset;
        blocking_read(start_address, bytes)
    }

    pub fn blocking_write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Error> {
        let start_address = self.base + offset;
        unsafe { blocking_write(start_address, bytes) }
    }

    pub fn blocking_erase(&mut self, from: u32, to: u32) -> Result<(), Error> {
        let start_address = self.base + from;
        let end_address = self.base + to;
        unsafe { blocking_erase(start_address, end_address) }
    }
}

foreach_flash_region! {
    ($type_name:ident, $write_size:literal, $erase_size:literal) => {
        impl crate::_generated::flash_regions::$type_name {
            pub fn blocking_read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Error> {
                let start_address = self.0.base + offset;
                blocking_read(start_address, bytes)
            }

            pub fn blocking_write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Error> {
                let start_address = self.0.base + offset;
                unsafe { blocking_write(start_address, bytes) }
            }

            pub fn blocking_erase(&mut self, from: u32, to: u32) -> Result<(), Error> {
                let start_address = self.0.base + from;
                let end_address = self.0.base + to;
                unsafe { blocking_erase(start_address, end_address) }
            }
        }

        impl embedded_storage::nor_flash::ErrorType for crate::_generated::flash_regions::$type_name {
            type Error = Error;
        }

        impl embedded_storage::nor_flash::ReadNorFlash for crate::_generated::flash_regions::$type_name {
            const READ_SIZE: usize = 1;

            fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
                self.blocking_read(offset, bytes)
            }

            fn capacity(&self) -> usize {
                self.0.size as usize
            }
        }

        impl embedded_storage::nor_flash::NorFlash for crate::_generated::flash_regions::$type_name {
            const WRITE_SIZE: usize = $write_size;
            const ERASE_SIZE: usize = $erase_size;

            fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
                self.blocking_write(offset, bytes)
            }

            fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
                self.blocking_erase(from, to)
            }
        }
    };
}
