use core::marker::PhantomData;
use core::sync::atomic::{fence, Ordering};

use embassy_hal_internal::drop::OnDrop;
use embassy_hal_internal::{into_ref, PeripheralRef};
use stm32_metapac::FLASH_BASE;

use super::{
    family, Async, Blocking, Error, FlashBank, FlashLayout, FlashRegion, FlashSector, FLASH_SIZE, MAX_ERASE_SIZE,
    READ_SIZE, WRITE_SIZE,
};
use crate::peripherals::FLASH;
use crate::Peripheral;

pub struct Flash<'d, MODE = Async> {
    pub(crate) inner: PeripheralRef<'d, FLASH>,
    pub(crate) _mode: PhantomData<MODE>,
}

impl<'d> Flash<'d, Blocking> {
    pub fn new_blocking(p: impl Peripheral<P = FLASH> + 'd) -> Self {
        into_ref!(p);

        Self {
            inner: p,
            _mode: PhantomData,
        }
    }
}

impl<'d, MODE> Flash<'d, MODE> {
    pub fn into_blocking_regions(self) -> FlashLayout<'d, Blocking> {
        assert!(family::is_default_layout());
        FlashLayout::new(self.inner)
    }

    pub fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Error> {
        blocking_read(FLASH_BASE as u32, FLASH_SIZE as u32, offset, bytes)
    }

    pub fn blocking_write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Error> {
        unsafe {
            blocking_write(
                FLASH_BASE as u32,
                FLASH_SIZE as u32,
                offset,
                bytes,
                write_chunk_unlocked,
            )
        }
    }

    pub fn blocking_erase(&mut self, from: u32, to: u32) -> Result<(), Error> {
        unsafe { blocking_erase(FLASH_BASE as u32, from, to, erase_sector_unlocked) }
    }
}

pub(super) fn blocking_read(base: u32, size: u32, offset: u32, bytes: &mut [u8]) -> Result<(), Error> {
    if offset + bytes.len() as u32 > size {
        return Err(Error::Size);
    }

    let start_address = base + offset;

    #[cfg(flash_f4)]
    family::assert_not_corrupted_read(start_address + bytes.len() as u32);

    let flash_data = unsafe { core::slice::from_raw_parts(start_address as *const u8, bytes.len()) };
    bytes.copy_from_slice(flash_data);
    Ok(())
}

pub(super) unsafe fn blocking_write(
    base: u32,
    size: u32,
    offset: u32,
    bytes: &[u8],
    write_chunk: unsafe fn(u32, &[u8]) -> Result<(), Error>,
) -> Result<(), Error> {
    if offset + bytes.len() as u32 > size {
        return Err(Error::Size);
    }
    if offset % WRITE_SIZE as u32 != 0 || bytes.len() % WRITE_SIZE != 0 {
        return Err(Error::Unaligned);
    }

    let mut address = base + offset;
    trace!("Writing {} bytes at 0x{:x}", bytes.len(), address);

    for chunk in bytes.chunks(WRITE_SIZE) {
        write_chunk(address, chunk)?;
        address += WRITE_SIZE as u32;
    }
    Ok(())
}

pub(super) unsafe fn write_chunk_unlocked(address: u32, chunk: &[u8]) -> Result<(), Error> {
    family::clear_all_err();
    fence(Ordering::SeqCst);
    family::unlock();
    fence(Ordering::SeqCst);
    family::enable_blocking_write();
    fence(Ordering::SeqCst);

    let _on_drop = OnDrop::new(|| {
        family::disable_blocking_write();
        fence(Ordering::SeqCst);
        family::lock();
    });

    family::blocking_write(address, chunk.try_into().unwrap())
}

pub(super) unsafe fn write_chunk_with_critical_section(address: u32, chunk: &[u8]) -> Result<(), Error> {
    critical_section::with(|_| write_chunk_unlocked(address, chunk))
}

pub(super) unsafe fn blocking_erase(
    base: u32,
    from: u32,
    to: u32,
    erase_sector: unsafe fn(&FlashSector) -> Result<(), Error>,
) -> Result<(), Error> {
    let start_address = base + from;
    let end_address = base + to;
    let regions = family::get_flash_regions();

    ensure_sector_aligned(start_address, end_address, regions)?;

    trace!("Erasing from 0x{:x} to 0x{:x}", start_address, end_address);

    let mut address = start_address;
    while address < end_address {
        let sector = get_sector(address, regions);
        trace!("Erasing sector: {:?}", sector);
        erase_sector(&sector)?;
        address += sector.size;
    }
    Ok(())
}

pub(super) unsafe fn erase_sector_unlocked(sector: &FlashSector) -> Result<(), Error> {
    family::clear_all_err();
    fence(Ordering::SeqCst);
    family::unlock();
    fence(Ordering::SeqCst);

    let _on_drop = OnDrop::new(|| family::lock());

    family::blocking_erase_sector(sector)
}

pub(super) unsafe fn erase_sector_with_critical_section(sector: &FlashSector) -> Result<(), Error> {
    critical_section::with(|_| erase_sector_unlocked(sector))
}

pub(super) fn get_sector(address: u32, regions: &[&FlashRegion]) -> FlashSector {
    let mut current_bank = FlashBank::Bank1;
    let mut bank_offset = 0;
    for region in regions {
        if region.bank != current_bank {
            current_bank = region.bank;
            bank_offset = 0;
        }

        if address >= region.base && address < region.end() {
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

pub(super) fn ensure_sector_aligned(
    start_address: u32,
    end_address: u32,
    regions: &[&FlashRegion],
) -> Result<(), Error> {
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
    Ok(())
}

impl<MODE> embedded_storage::nor_flash::ErrorType for Flash<'_, MODE> {
    type Error = Error;
}

impl<MODE> embedded_storage::nor_flash::ReadNorFlash for Flash<'_, MODE> {
    const READ_SIZE: usize = READ_SIZE;

    fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        self.read(offset, bytes)
    }

    fn capacity(&self) -> usize {
        FLASH_SIZE
    }
}

impl<MODE> embedded_storage::nor_flash::NorFlash for Flash<'_, MODE> {
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
        impl<MODE> crate::_generated::flash_regions::$type_name<'_, MODE> {
            pub fn blocking_read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Error> {
                blocking_read(self.0.base, self.0.size, offset, bytes)
            }
        }

        impl crate::_generated::flash_regions::$type_name<'_, Blocking> {
            pub fn blocking_write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Error> {
                unsafe { blocking_write(self.0.base, self.0.size, offset, bytes, write_chunk_with_critical_section) }
            }

            pub fn blocking_erase(&mut self, from: u32, to: u32) -> Result<(), Error> {
                unsafe { blocking_erase(self.0.base, from, to, erase_sector_with_critical_section) }
            }
        }

        impl<MODE> embedded_storage::nor_flash::ErrorType for crate::_generated::flash_regions::$type_name<'_, MODE> {
            type Error = Error;
        }

        impl<MODE> embedded_storage::nor_flash::ReadNorFlash for crate::_generated::flash_regions::$type_name<'_, MODE> {
            const READ_SIZE: usize = READ_SIZE;

            fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
                self.blocking_read(offset, bytes)
            }

            fn capacity(&self) -> usize {
                self.0.size as usize
            }
        }

        impl embedded_storage::nor_flash::NorFlash for crate::_generated::flash_regions::$type_name<'_, Blocking> {
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
