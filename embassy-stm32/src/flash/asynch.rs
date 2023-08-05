use core::marker::PhantomData;
use core::sync::atomic::{fence, Ordering};

use embassy_hal_internal::drop::OnDrop;
use embassy_hal_internal::into_ref;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;

use super::{
    blocking_read, ensure_sector_aligned, family, get_sector, Async, Error, Flash, FlashLayout, FLASH_BASE, FLASH_SIZE,
    WRITE_SIZE,
};
use crate::interrupt::InterruptExt;
use crate::peripherals::FLASH;
use crate::{interrupt, Peripheral};

pub(super) static REGION_ACCESS: Mutex<CriticalSectionRawMutex, ()> = Mutex::new(());

impl<'d> Flash<'d, Async> {
    pub fn new(
        p: impl Peripheral<P = FLASH> + 'd,
        _irq: impl interrupt::typelevel::Binding<crate::interrupt::typelevel::FLASH, InterruptHandler> + 'd,
    ) -> Self {
        into_ref!(p);

        crate::interrupt::FLASH.unpend();
        unsafe { crate::interrupt::FLASH.enable() };

        Self {
            inner: p,
            _mode: PhantomData,
        }
    }

    pub fn into_regions(self) -> FlashLayout<'d, Async> {
        assert!(family::is_default_layout());
        FlashLayout::new(self.inner)
    }

    pub async fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Error> {
        unsafe { write_chunked(FLASH_BASE as u32, FLASH_SIZE as u32, offset, bytes).await }
    }

    pub async fn erase(&mut self, from: u32, to: u32) -> Result<(), Error> {
        unsafe { erase_sectored(FLASH_BASE as u32, from, to).await }
    }
}

/// Interrupt handler
pub struct InterruptHandler;

impl interrupt::typelevel::Handler<crate::interrupt::typelevel::FLASH> for InterruptHandler {
    unsafe fn on_interrupt() {
        family::on_interrupt();
    }
}

#[cfg(feature = "nightly")]
impl embedded_storage_async::nor_flash::ReadNorFlash for Flash<'_, Async> {
    const READ_SIZE: usize = super::READ_SIZE;

    async fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        self.read(offset, bytes)
    }

    fn capacity(&self) -> usize {
        FLASH_SIZE
    }
}

#[cfg(feature = "nightly")]
impl embedded_storage_async::nor_flash::NorFlash for Flash<'_, Async> {
    const WRITE_SIZE: usize = WRITE_SIZE;
    const ERASE_SIZE: usize = super::MAX_ERASE_SIZE;

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
        impl crate::_generated::flash_regions::$type_name<'_, Async> {
            pub async fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Error> {
                blocking_read(self.0.base, self.0.size, offset, bytes)
            }

            pub async fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Error> {
                let _guard = REGION_ACCESS.lock().await;
                unsafe { write_chunked(self.0.base, self.0.size, offset, bytes).await }
            }

            pub async fn erase(&mut self, from: u32, to: u32) -> Result<(), Error> {
                let _guard = REGION_ACCESS.lock().await;
                unsafe { erase_sectored(self.0.base, from, to).await }
            }
        }

        #[cfg(feature = "nightly")]
        impl embedded_storage_async::nor_flash::ReadNorFlash for crate::_generated::flash_regions::$type_name<'_, Async> {
            const READ_SIZE: usize = super::READ_SIZE;

            async fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
                self.read(offset, bytes).await
            }

            fn capacity(&self) -> usize {
                self.0.size as usize
            }
        }

        #[cfg(feature = "nightly")]
        impl embedded_storage_async::nor_flash::NorFlash for crate::_generated::flash_regions::$type_name<'_, Async> {
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
