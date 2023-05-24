use atomic_polyfill::{fence, Ordering};
use embassy_hal_common::drop::OnDrop;

use super::{
    ensure_sector_aligned, family, get_sector, Error, Flash, FLASH_BASE, FLASH_SIZE, MAX_ERASE_SIZE, READ_SIZE,
    REGION_ACCESS, WRITE_SIZE,
};

impl<'d> Flash<'d> {
    pub async fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Error> {
        unsafe { write_chunked(FLASH_BASE as u32, FLASH_SIZE as u32, offset, bytes).await }
    }

    pub async fn erase(&mut self, from: u32, to: u32) -> Result<(), Error> {
        unsafe { erase_sectored(FLASH_BASE as u32, from, to).await }
    }
}

impl embedded_storage_async::nor_flash::NorFlash for Flash<'_> {
    const WRITE_SIZE: usize = WRITE_SIZE;
    const ERASE_SIZE: usize = MAX_ERASE_SIZE;

    async fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        self.write(offset, bytes).await
    }

    async fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
        self.erase(from, to).await
    }
}

pub(super) async unsafe fn write_chunked(base: u32, size: u32, offset: u32, bytes: &[u8]) -> Result<(), Error> {
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
        family::enable_write();
        fence(Ordering::SeqCst);

        let _on_drop = OnDrop::new(|| {
            family::disable_write();
            fence(Ordering::SeqCst);
            family::lock();
        });

        family::write(address, chunk.try_into().unwrap()).await?;
        address += WRITE_SIZE as u32;
    }
    Ok(())
}

pub(super) async unsafe fn erase_sectored(base: u32, from: u32, to: u32) -> Result<(), Error> {
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

        family::erase_sector(&sector).await?;
        address += sector.size;
    }
    Ok(())
}

foreach_flash_region! {
    ($type_name:ident, $write_size:literal, $erase_size:literal) => {
        impl crate::_generated::flash_regions::$type_name<'_> {
            pub async fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Error> {
                let _guard = REGION_ACCESS.lock().await;
                unsafe { write_chunked(self.0.base, self.0.size, offset, bytes).await }
            }

            pub async fn erase(&mut self, from: u32, to: u32) -> Result<(), Error> {
                let _guard = REGION_ACCESS.lock().await;
                unsafe { erase_sectored(self.0.base, from, to).await }
            }
        }

        impl embedded_storage_async::nor_flash::ReadNorFlash for crate::_generated::flash_regions::$type_name<'_> {
            const READ_SIZE: usize = READ_SIZE;

            async fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
                self.read(offset, bytes)
            }

            fn capacity(&self) -> usize {
                self.0.size as usize
            }
        }

        impl embedded_storage_async::nor_flash::NorFlash for crate::_generated::flash_regions::$type_name<'_> {
            const WRITE_SIZE: usize = $write_size;
            const ERASE_SIZE: usize = $erase_size;

            async fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
                self.write(offset, bytes).await
            }

            async fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
                self.erase(from, to).await
            }
        }
    };
}
