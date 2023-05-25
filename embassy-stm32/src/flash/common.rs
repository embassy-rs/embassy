use atomic_polyfill::{fence, Ordering};
use embassy_cortex_m::interrupt::{Interrupt, InterruptExt};
use embassy_hal_common::drop::OnDrop;
use embassy_hal_common::{into_ref, PeripheralRef};
use stm32_metapac::FLASH_BASE;

use super::{
    family, Blocking, Error, FlashBank, FlashLayout, FlashRegion, FlashSector, FLASH_SIZE, MAX_ERASE_SIZE, READ_SIZE,
    WRITE_SIZE,
};
use crate::peripherals::FLASH;
use crate::{interrupt, Peripheral};

pub struct Flash<'d> {
    pub(crate) inner: PeripheralRef<'d, FLASH>,
    #[cfg(all(feature = "nightly", flash_f4))]
    pub(crate) blocking_only: bool,
}

impl<'d> Flash<'d> {
    pub fn new(
        p: impl Peripheral<P = FLASH> + 'd,
        _irq: impl interrupt::Binding<crate::interrupt::FLASH, InterruptHandler> + 'd,
    ) -> Self {
        into_ref!(p);

        let flash_irq = unsafe { crate::interrupt::FLASH::steal() };
        flash_irq.unpend();
        flash_irq.enable();

        Self {
            inner: p,
            #[cfg(all(feature = "nightly", flash_f4))]
            blocking_only: false,
        }
    }

    pub fn new_blocking_only(p: impl Peripheral<P = FLASH> + 'd) -> Self {
        into_ref!(p);

        Self {
            inner: p,
            #[cfg(all(feature = "nightly", flash_f4))]
            blocking_only: true,
        }
    }

    pub fn into_blocking_regions(self) -> FlashLayout<'d, Blocking> {
        family::set_default_layout();
        FlashLayout::new(self.inner)
    }

    pub fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Error> {
        read_blocking(FLASH_BASE as u32, FLASH_SIZE as u32, offset, bytes)
    }

    pub fn write_blocking(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Error> {
        unsafe {
            write_blocking(
                FLASH_BASE as u32,
                FLASH_SIZE as u32,
                offset,
                bytes,
                write_chunk_unlocked,
            )
        }
    }

    pub fn erase_blocking(&mut self, from: u32, to: u32) -> Result<(), Error> {
        unsafe { erase_blocking(FLASH_BASE as u32, from, to, erase_sector_unlocked) }
    }
}

/// Interrupt handler
pub struct InterruptHandler;

impl interrupt::Handler<crate::interrupt::FLASH> for InterruptHandler {
    unsafe fn on_interrupt() {
        family::on_interrupt();
    }
}

pub(super) fn read_blocking(base: u32, size: u32, offset: u32, bytes: &mut [u8]) -> Result<(), Error> {
    if offset + bytes.len() as u32 > size {
        return Err(Error::Size);
    }

    #[cfg(flash_f4)]
    family::assert_not_corrupted_read();

    let start_address = base + offset;
    let flash_data = unsafe { core::slice::from_raw_parts(start_address as *const u8, bytes.len()) };
    bytes.copy_from_slice(flash_data);
    Ok(())
}

pub(super) unsafe fn write_blocking(
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

    family::write_blocking(address, chunk.try_into().unwrap())
}

pub(super) unsafe fn write_chunk_with_critical_section(address: u32, chunk: &[u8]) -> Result<(), Error> {
    critical_section::with(|_| write_chunk_unlocked(address, chunk))
}

pub(super) unsafe fn erase_blocking(
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

    family::erase_sector_blocking(&sector)
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

impl embedded_storage::nor_flash::ErrorType for Flash<'_> {
    type Error = Error;
}

impl embedded_storage::nor_flash::ReadNorFlash for Flash<'_> {
    const READ_SIZE: usize = READ_SIZE;

    fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        self.read(offset, bytes)
    }

    fn capacity(&self) -> usize {
        FLASH_SIZE
    }
}

impl embedded_storage::nor_flash::NorFlash for Flash<'_> {
    const WRITE_SIZE: usize = WRITE_SIZE;
    const ERASE_SIZE: usize = MAX_ERASE_SIZE;

    fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        self.write_blocking(offset, bytes)
    }

    fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
        self.erase_blocking(from, to)
    }
}

foreach_flash_region! {
    ($type_name:ident, $write_size:literal, $erase_size:literal) => {
        impl<'d> crate::_generated::flash_regions::$type_name<'d, Blocking> {
            pub fn read_blocking(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Error> {
                read_blocking(self.0.base, self.0.size, offset, bytes)
            }

            pub fn write_blocking(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Error> {
                unsafe { write_blocking(self.0.base, self.0.size, offset, bytes, write_chunk_with_critical_section) }
            }

            pub fn erase_blocking(&mut self, from: u32, to: u32) -> Result<(), Error> {
                unsafe { erase_blocking(self.0.base, from, to, erase_sector_with_critical_section) }
            }
        }

        impl<MODE> embedded_storage::nor_flash::ErrorType for crate::_generated::flash_regions::$type_name<'_, MODE> {
            type Error = Error;
        }

        impl embedded_storage::nor_flash::ReadNorFlash for crate::_generated::flash_regions::$type_name<'_, Blocking> {
            const READ_SIZE: usize = READ_SIZE;

            fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
                self.read_blocking(offset, bytes)
            }

            fn capacity(&self) -> usize {
                self.0.size as usize
            }
        }

        impl embedded_storage::nor_flash::NorFlash for crate::_generated::flash_regions::$type_name<'_, Blocking> {
            const WRITE_SIZE: usize = $write_size;
            const ERASE_SIZE: usize = $erase_size;

            fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
                self.write_blocking(offset, bytes)
            }

            fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
                self.erase_blocking(from, to)
            }
        }
    };
}
