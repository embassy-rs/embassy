use atomic_polyfill::{fence, Ordering};
use embassy_cortex_m::interrupt::InterruptExt;
use embassy_futures::block_on;
use embassy_hal_common::drop::OnDrop;
use embassy_hal_common::{into_ref, PeripheralRef};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use stm32_metapac::FLASH_BASE;

use super::{
    family, Error, FlashLayout, FlashRegion, FLASH_SIZE, MAX_ERASE_SIZE, READ_SIZE,
    WRITE_SIZE, FlashSector, FlashBank,
};
use crate::peripherals::FLASH;
use crate::Peripheral;

pub struct Flash<'d> {
    pub(crate) inner: PeripheralRef<'d, FLASH>,
}

pub(crate) static REGION_ACCESS: Mutex<CriticalSectionRawMutex, ()> = Mutex::new(());

impl<'d> Flash<'d> {
    pub fn new(p: impl Peripheral<P = FLASH> + 'd, irq: impl Peripheral<P = crate::interrupt::FLASH> + 'd) -> Self {
        into_ref!(p, irq);

        irq.set_handler(family::on_interrupt);
        irq.unpend();
        irq.enable();

        Self { inner: p }
    }

    pub fn into_regions(self) -> FlashLayout<'d> {
        family::set_default_layout();
        FlashLayout::new(self.inner)
    }

    pub fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Error> {
        read_blocking(FLASH_BASE as u32, FLASH_SIZE as u32, offset, bytes)
    }

    pub fn write_blocking(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Error> {
        unsafe { write_chunked_blocking(FLASH_BASE as u32, FLASH_SIZE as u32, offset, bytes) }
    }

    pub fn erase_blocking(&mut self, from: u32, to: u32) -> Result<(), Error> {
        unsafe { erase_sectored_blocking(FLASH_BASE as u32, from, to) }
    }
}

pub(super) fn read_blocking(base: u32, size: u32, offset: u32, bytes: &mut [u8]) -> Result<(), Error> {
    if offset + bytes.len() as u32 > size {
        return Err(Error::Size);
    }

    let start_address = base + offset;
    let flash_data = unsafe { core::slice::from_raw_parts(start_address as *const u8, bytes.len()) };
    bytes.copy_from_slice(flash_data);
    Ok(())
}

pub(super) unsafe fn write_chunked_blocking(base: u32, size: u32, offset: u32, bytes: &[u8]) -> Result<(), Error> {
    if offset + bytes.len() as u32 > size {
        return Err(Error::Size);
    }
    if offset % WRITE_SIZE as u32 != 0 || bytes.len() % WRITE_SIZE != 0 {
        return Err(Error::Unaligned);
    }

    let mut address = base + offset;
    trace!("Writing {} bytes at 0x{:x}", bytes.len(), address);

    for chunk in bytes.chunks(WRITE_SIZE) {
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

        family::write_blocking(address, chunk.try_into().unwrap())?;
        address += WRITE_SIZE as u32;
    }
    Ok(())
}

pub(super) unsafe fn erase_sectored_blocking(base: u32, from: u32, to: u32) -> Result<(), Error> {
    let start_address = base + from;
    let end_address = base + to;
    let regions = family::get_flash_regions();

    ensure_sector_aligned(start_address, end_address, regions)?;

    trace!("Erasing from 0x{:x} to 0x{:x}", start_address, end_address);

    let mut address = start_address;
    while address < end_address {
        let sector = get_sector(address, regions);
        trace!("Erasing sector: {:?}", sector);

        family::clear_all_err();
        fence(Ordering::SeqCst);
        family::unlock();
        fence(Ordering::SeqCst);

        let _on_drop = OnDrop::new(|| family::lock());

        family::erase_sector_blocking(&sector)?;
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

pub(crate) fn ensure_sector_aligned(
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

#[cfg(feature = "nightly")]
impl embedded_storage_async::nor_flash::ReadNorFlash for Flash<'_> {
    const READ_SIZE: usize = READ_SIZE;

    async fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        self.read(offset, bytes)
    }

    fn capacity(&self) -> usize {
        FLASH_SIZE
    }
}

pub struct BlockingFlashRegion<'d, const WRITE_SIZE: u32, const ERASE_SIZE: u32>(
    &'static FlashRegion,
    PeripheralRef<'d, FLASH>,
);

impl<const WRITE_SIZE: u32, const ERASE_SIZE: u32> BlockingFlashRegion<'_, WRITE_SIZE, ERASE_SIZE> {
    pub fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Error> {
        read_blocking(self.0.base, self.0.size, offset, bytes)
    }

    pub fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Error> {
        let _guard = block_on(REGION_ACCESS.lock());
        unsafe { write_chunked_blocking(self.0.base, self.0.size, offset, bytes) }
    }

    pub fn erase(&mut self, from: u32, to: u32) -> Result<(), Error> {
        let _guard = block_on(REGION_ACCESS.lock());
        unsafe { erase_sectored_blocking(self.0.base, from, to) }
    }
}

impl<const WRITE_SIZE: u32, const ERASE_SIZE: u32> embedded_storage::nor_flash::ErrorType
    for BlockingFlashRegion<'_, WRITE_SIZE, ERASE_SIZE>
{
    type Error = Error;
}

impl<const WRITE_SIZE: u32, const ERASE_SIZE: u32> embedded_storage::nor_flash::ReadNorFlash
    for BlockingFlashRegion<'_, WRITE_SIZE, ERASE_SIZE>
{
    const READ_SIZE: usize = READ_SIZE;

    fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        self.read(offset, bytes)
    }

    fn capacity(&self) -> usize {
        self.0.size as usize
    }
}

impl<const WRITE_SIZE: u32, const ERASE_SIZE: u32> embedded_storage::nor_flash::NorFlash
    for BlockingFlashRegion<'_, WRITE_SIZE, ERASE_SIZE>
{
    const WRITE_SIZE: usize = WRITE_SIZE as usize;
    const ERASE_SIZE: usize = ERASE_SIZE as usize;

    fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        self.write(offset, bytes)
    }

    fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
        self.erase(from, to)
    }
}

foreach_flash_region! {
    ($type_name:ident, $write_size:literal, $erase_size:literal) => {
        paste::paste! {
            pub type [<Blocking $type_name>]<'d> = BlockingFlashRegion<'d, $write_size, $erase_size>;
        }

        impl<'d> crate::_generated::flash_regions::$type_name<'d> {
            /// Make this flash region work in a blocking context.
            ///
            /// SAFETY
            ///
            /// This function is unsafe as incorect usage of parallel blocking operations
            /// on multiple regions may cause a deadlock because each region requires mutual access to the flash.
            pub unsafe fn into_blocking(self) -> BlockingFlashRegion<'d, $write_size, $erase_size> {
                BlockingFlashRegion(self.0, self.1)
            }

            pub fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Error> {
                read_blocking(self.0.base, self.0.size, offset, bytes)
            }

            pub fn try_write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Error> {
                let _guard = REGION_ACCESS.try_lock().map_err(|_| Error::TryLockError)?;
                unsafe { write_chunked_blocking(self.0.base, self.0.size, offset, bytes) }
            }

            pub fn try_erase(&mut self, from: u32, to: u32) -> Result<(), Error> {
                let _guard = REGION_ACCESS.try_lock().map_err(|_| Error::TryLockError)?;
                unsafe { erase_sectored_blocking(self.0.base, from, to) }
            }
        }

        impl embedded_storage::nor_flash::ErrorType for crate::_generated::flash_regions::$type_name<'_> {
            type Error = Error;
        }

        impl embedded_storage::nor_flash::ReadNorFlash for crate::_generated::flash_regions::$type_name<'_> {
            const READ_SIZE: usize = READ_SIZE;

            fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
                self.read(offset, bytes)
            }

            fn capacity(&self) -> usize {
                self.0.size as usize
            }
        }
    };
}
