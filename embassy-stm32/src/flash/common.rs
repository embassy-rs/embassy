use atomic_polyfill::{fence, Ordering};
use embassy_hal_common::drop::OnDrop;
use embassy_hal_common::{into_ref, PeripheralRef};

use super::{family, Error, FlashLayout, FlashRegion, FlashSector, FLASH_BASE, FLASH_SIZE, MAX_ERASE_SIZE, WRITE_SIZE};
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
        family::set_default_layout();
        FlashLayout::new(self.release())
    }

    pub fn blocking_read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Error> {
        blocking_read(FLASH_BASE as u32, FLASH_SIZE as u32, offset, bytes)
    }

    pub fn blocking_write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Error> {
        unsafe { blocking_write_chunked(FLASH_BASE as u32, FLASH_SIZE as u32, offset, bytes) }
    }

    pub fn blocking_erase(&mut self, from: u32, to: u32) -> Result<(), Error> {
        unsafe { blocking_erase_sectored(FLASH_BASE as u32, from, to) }
    }

    pub(crate) fn release(self) -> PeripheralRef<'d, crate::peripherals::FLASH> {
        unsafe { self.inner.clone_unchecked() }
    }
}

pub(super) fn blocking_read(base: u32, size: u32, offset: u32, bytes: &mut [u8]) -> Result<(), Error> {
    if offset + bytes.len() as u32 > size {
        return Err(Error::Size);
    }

    let start_address = base + offset;
    let flash_data = unsafe { core::slice::from_raw_parts(start_address as *const u8, bytes.len()) };
    bytes.copy_from_slice(flash_data);
    Ok(())
}

pub(super) unsafe fn blocking_write_chunked(base: u32, size: u32, offset: u32, bytes: &[u8]) -> Result<(), Error> {
    if offset + bytes.len() as u32 > size {
        return Err(Error::Size);
    }
    if offset % WRITE_SIZE as u32 != 0 || bytes.len() % WRITE_SIZE != 0 {
        return Err(Error::Unaligned);
    }

    let mut address = base + offset;
    trace!("Writing {} bytes at 0x{:x}", bytes.len(), address);

    for chunk in bytes.chunks(WRITE_SIZE) {
        critical_section::with(|_| {
            family::clear_all_err();
            fence(Ordering::SeqCst);
            family::unlock();
            fence(Ordering::SeqCst);
            family::begin_write();
            fence(Ordering::SeqCst);

            let _on_drop = OnDrop::new(|| {
                family::end_write();
                fence(Ordering::SeqCst);
                family::lock();
            });

            family::blocking_write(address, chunk.try_into().unwrap())
        })?;
        address += WRITE_SIZE as u32;
    }
    Ok(())
}

pub(super) unsafe fn blocking_erase_sectored(base: u32, from: u32, to: u32) -> Result<(), Error> {
    let start_address = base + from;
    let end_address = base + to;
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
        trace!("Erasing sector: {:?}", sector);

        critical_section::with(|_| {
            family::clear_all_err();
            fence(Ordering::SeqCst);
            family::unlock();
            fence(Ordering::SeqCst);

            let _on_drop = OnDrop::new(|| {
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
        blocking_read(self.base, self.size, offset, bytes)
    }

    pub fn blocking_write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Error> {
        unsafe { blocking_write_chunked(self.base, self.size, offset, bytes) }
    }

    pub fn blocking_erase(&mut self, from: u32, to: u32) -> Result<(), Error> {
        unsafe { blocking_erase_sectored(self.base, from, to) }
    }
}

impl embedded_storage::nor_flash::ErrorType for Flash<'_> {
    type Error = Error;
}

impl embedded_storage::nor_flash::ReadNorFlash for Flash<'_> {
    const READ_SIZE: usize = 1;

    fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        self.blocking_read(offset, bytes)
    }

    fn capacity(&self) -> usize {
        FLASH_SIZE
    }
}

impl embedded_storage::nor_flash::NorFlash for Flash<'_> {
    const WRITE_SIZE: usize = WRITE_SIZE;
    const ERASE_SIZE: usize = MAX_ERASE_SIZE;

    fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        self.blocking_write(offset, bytes)
    }

    fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
        self.blocking_erase(from, to)
    }
}

foreach_flash_region! {
    ($type_name:ident, $write_size:literal, $erase_size:literal) => {
        impl crate::_generated::flash_regions::$type_name<'_> {
            pub fn blocking_read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Error> {
                blocking_read(self.0.base, self.0.size, offset, bytes)
            }

            pub fn blocking_write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Error> {
                unsafe { blocking_write_chunked(self.0.base, self.0.size, offset, bytes) }
            }

            pub fn blocking_erase(&mut self, from: u32, to: u32) -> Result<(), Error> {
                unsafe { blocking_erase_sectored(self.0.base, from, to) }
            }
        }

        impl embedded_storage::nor_flash::ErrorType for crate::_generated::flash_regions::$type_name<'_> {
            type Error = Error;
        }

        impl embedded_storage::nor_flash::ReadNorFlash for crate::_generated::flash_regions::$type_name<'_> {
            const READ_SIZE: usize = 1;

            fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
                self.blocking_read(offset, bytes)
            }

            fn capacity(&self) -> usize {
                self.0.size as usize
            }
        }

        impl embedded_storage::nor_flash::NorFlash for crate::_generated::flash_regions::$type_name<'_> {
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
