use core::future::poll_fn;
use core::task::Poll;

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_sync::waitqueue::AtomicWaker;

use super::common::{Op, OpGuard, check_write, ensure_sector_aligned, get_sector};
use super::{
    Config, Error, FLASH_BASE, FLASH_SIZE, Flash, FlashLayout, MAX_ERASE_SIZE, READ_SIZE, WRITE_SIZE, blocking_read,
    check_flash_address, family, get_flash_regions,
};
use crate::_generated::peripheral_interrupts::FLASH::GLOBAL as FlashInterrupt;
use crate::interrupt::typelevel::{Binding, Handler, Interrupt};
use crate::mode::Async;
use crate::{Peri, peripherals};

static WAKER: AtomicWaker = AtomicWaker::new();

/// Serializes operations done through [`FlashLayout`] regions, which can be used concurrently.
static REGION_ACCESS: Mutex<CriticalSectionRawMutex, ()> = Mutex::new(());

/// Flash interrupt handler.
pub struct InterruptHandler {
    _private: (),
}

impl Handler<FlashInterrupt> for InterruptHandler {
    unsafe fn on_interrupt() {
        // Mask the interrupt in the NVIC: the flags that raise it are only cleared when the
        // operation is finished, and the control register must not be written while the
        // controller is busy. The woken future re-enables it if the operation is still ongoing.
        FlashInterrupt::disable();
        WAKER.wake();
    }
}

impl<'d> Flash<'d, Async> {
    /// Create a new flash driver with async capabilities.
    pub fn new(
        p: Peri<'d, peripherals::FLASH>,
        _irq: impl Binding<FlashInterrupt, InterruptHandler> + 'd,
        config: Config,
    ) -> Self {
        let this = Self::new_inner(p, config);

        FlashInterrupt::unpend();
        unsafe { FlashInterrupt::enable() };

        this
    }

    /// Split this flash driver into one async driver per flash memory region.
    ///
    /// See module-level documentation for details on how memory regions work.
    pub fn into_regions(self) -> FlashLayout<'d, Async> {
        FlashLayout::new(self.inner, self.config)
    }

    /// Async write.
    ///
    /// NOTE: `offset` is an offset from the flash start, NOT an absolute address.
    /// For example, to write address `0x0800_1234` you have to use offset `0x1234`.
    pub async fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Error> {
        let (base, size, offset) = check_flash_address(FLASH_BASE as u32, offset)?;
        write(base, size, offset, bytes).await
    }

    /// Async erase.
    ///
    /// NOTE: `from` and `to` are offsets from the flash start, NOT an absolute address.
    /// For example, to erase address `0x0801_0000` you have to use offset `0x1_0000`.
    pub async fn erase(&mut self, from: u32, to: u32) -> Result<(), Error> {
        erase(FLASH_BASE as u32, from, to, &self.config).await
    }
}

/// Runs a program or erase operation, waiting for its completion with the interrupt.
async fn op(op: Op<'_>) -> Result<(), Error> {
    op.check()?;
    // Stop mode is not allowed while a flash operation is ongoing.
    let _wake = crate::rcc::WakeGuard::new(
        #[cfg(feature = "low-power")]
        crate::rcc::StopMode::Stop1,
    );
    // Drop a pending interrupt left over by the previous operation.
    FlashInterrupt::unpend();
    let _guard = OpGuard::start(op, true);
    poll_fn(|cx| {
        WAKER.register(cx.waker());
        if family::busy() {
            unsafe { FlashInterrupt::enable() };
            // Recheck: the interrupt is level-triggered on the completion flags, but the
            // operation may have completed before it was re-enabled.
            if family::busy() {
                return Poll::Pending;
            }
        }
        Poll::Ready(())
    })
    .await;
    family::finish()
}

async fn write(base: u32, size: u32, offset: u32, bytes: &[u8]) -> Result<(), Error> {
    check_write(size, offset, bytes.len())?;

    let mut address = base + offset;
    trace!("Writing {} bytes at 0x{:x}", bytes.len(), address);

    for chunk in bytes.chunks(WRITE_SIZE) {
        op(Op::Write(address, unwrap!(chunk.try_into()))).await?;
        address += WRITE_SIZE as u32;
    }
    Ok(())
}

async fn erase(base: u32, from: u32, to: u32, config: &Config) -> Result<(), Error> {
    let start_address = base + from;
    let end_address = base + to;
    let regions = get_flash_regions();

    ensure_sector_aligned(start_address, end_address, regions)?;

    trace!("Erasing from 0x{:x} to 0x{:x}", start_address, end_address);

    let mut address = start_address;
    while address < end_address {
        let sector = get_sector(address, regions)?;
        trace!("Erasing sector: {:?}", sector);
        op(Op::Erase(&sector, config)).await?;
        address += sector.size;
    }
    Ok(())
}

impl embedded_storage_async::nor_flash::ReadNorFlash for Flash<'_, Async> {
    const READ_SIZE: usize = READ_SIZE;

    async fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        self.blocking_read(offset, bytes)
    }

    fn capacity(&self) -> usize {
        FLASH_SIZE
    }
}

impl embedded_storage_async::nor_flash::NorFlash for Flash<'_, Async> {
    const WRITE_SIZE: usize = WRITE_SIZE;
    const ERASE_SIZE: usize = MAX_ERASE_SIZE;

    async fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        self.write(offset, bytes).await
    }

    async fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
        self.erase(from, to).await
    }
}

foreach_flash_region! {
    ($type_name:ident, $write_size:literal, $erase_size:literal) => {
        impl crate::_generated::flash_regions::$type_name<'_, Async> {
            /// Async read.
            ///
            /// Note: reading from flash can't actually block, so this is the same as `blocking_read`.
            pub async fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Error> {
                blocking_read(self.0.base(), self.0.size, offset, bytes)
            }

            /// Async write.
            ///
            /// NOTE: `offset` is an offset from the region start, NOT an absolute address.
            pub async fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Error> {
                let _guard = REGION_ACCESS.lock().await;
                write(self.0.base(), self.0.size, offset, bytes).await
            }

            /// Async erase.
            ///
            /// NOTE: `from` and `to` are offsets from the region start, NOT absolute addresses.
            pub async fn erase(&mut self, from: u32, to: u32) -> Result<(), Error> {
                if to > self.0.size {
                    return Err(Error::OutOfBounds);
                }
                let _guard = REGION_ACCESS.lock().await;
                erase(self.0.base(), from, to, &self.3).await
            }
        }

        impl embedded_storage_async::nor_flash::ReadNorFlash for crate::_generated::flash_regions::$type_name<'_, Async> {
            const READ_SIZE: usize = READ_SIZE;

            async fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
                self.read(offset, bytes).await
            }

            fn capacity(&self) -> usize {
                self.0.size as usize
            }
        }

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
