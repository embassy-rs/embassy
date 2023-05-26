use core::marker::PhantomData;
use core::task::Poll;

use embassy_cortex_m::interrupt::{Interrupt, InterruptExt};
use embassy_hal_common::into_ref;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_sync::waitqueue::AtomicWaker;
use family::FAMILY;
use futures::future::poll_fn;

use super::{
    blocking_read, ensure_sector_aligned, family, get_sector, Async, Error, Flash, FlashLayout, FLASH_BASE, FLASH_SIZE,
    WRITE_SIZE,
};
use crate::flash::FlashCtrl;
use crate::peripherals::FLASH;
use crate::{interrupt, Peripheral};

pub(super) static REGION_ACCESS: Mutex<CriticalSectionRawMutex, ()> = Mutex::new(());
pub(crate) static WAKER: AtomicWaker = AtomicWaker::new();

impl<'d> Flash<'d, Async> {
    /// Create a new flash driver that supports blocking and async operations
    pub fn new(
        p: impl Peripheral<P = FLASH> + 'd,
        _irq: impl interrupt::Binding<crate::interrupt::FLASH, crate::flash::InterruptHandler> + 'd,
    ) -> Self {
        into_ref!(p);

        let flash_irq = unsafe { crate::interrupt::FLASH::steal() };
        flash_irq.unpend();
        flash_irq.enable();

        Self {
            inner: p,
            _mode: PhantomData,
        }
    }

    /// Split the flash into its the distinct regions
    /// When the flash is split, then it is no longer possible to do blocking operations
    pub fn into_regions(self) -> FlashLayout<'d, Async> {
        assert!(FAMILY::default_layout_configured());
        FlashLayout::new(self.inner)
    }

    pub async fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Error> {
        unsafe { write(FLASH_BASE as u32, FLASH_SIZE as u32, offset, bytes).await }
    }

    pub async fn erase(&mut self, from: u32, to: u32) -> Result<(), Error> {
        unsafe { erase(FLASH_BASE as u32, from, to).await }
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

pub(super) async unsafe fn write(base: u32, size: u32, offset: u32, bytes: &[u8]) -> Result<(), Error> {
    use super::UnlockedWrite;

    if offset + bytes.len() as u32 > size {
        return Err(Error::Size);
    }
    if offset % WRITE_SIZE as u32 != 0 || bytes.len() % WRITE_SIZE != 0 {
        return Err(Error::Unaligned);
    }

    let start_address = base + offset;
    trace!("Writing {} bytes at 0x{:x}", bytes.len(), start_address);

    let regions = FAMILY::get_flash_regions();
    let mut sector = get_sector(base, regions);
    let mut offset = start_address - sector.start;
    let mut writer = FAMILY::unlocked_writer(&sector);

    for chunk in bytes.chunks(WRITE_SIZE) {
        if offset == sector.size {
            sector = get_sector(sector.start + sector.size, regions);
            writer = FAMILY::unlocked_writer(&sector);
            offset = 0;
        }

        writer.initiate_word_write(&sector, offset, chunk.try_into().unwrap());
        poll_fn(|cx| {
            WAKER.register(cx.waker());

            if !FAMILY::is_busy() {
                Poll::Ready(writer.read_result())
            } else {
                return Poll::Pending;
            }
        })
        .await?;

        offset += WRITE_SIZE as u32;
    }

    Ok(())
}

pub(super) async unsafe fn erase(base: u32, from: u32, to: u32) -> Result<(), Error> {
    use super::UnlockedErase;

    let start_address = base + from;
    let end_address = base + to;
    let regions = family::get_flash_regions();

    ensure_sector_aligned(start_address, end_address, regions)?;

    trace!("Erasing from 0x{:x} to 0x{:x}", start_address, end_address);

    let mut eraser = FAMILY::unlocked_eraser();
    let mut address = start_address;
    while address < end_address {
        let sector = get_sector(address, regions);
        trace!("Erasing sector: {:?}", sector);

        eraser.initiate_sector_erase(&sector);
        poll_fn(|cx| {
            WAKER.register(cx.waker());

            if !FAMILY::is_busy() {
                Poll::Ready(eraser.read_result())
            } else {
                return Poll::Pending;
            }
        })
        .await?;

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
                unsafe { write(self.0.base, self.0.size, offset, bytes).await }
            }

            pub async fn erase(&mut self, from: u32, to: u32) -> Result<(), Error> {
                let _guard = REGION_ACCESS.lock().await;
                unsafe { erase(self.0.base, from, to).await }
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
