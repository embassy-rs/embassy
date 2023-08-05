#![no_std]
#![warn(missing_docs)]
#![doc = include_str!("../README.md")]
mod fmt;

#[cfg(feature = "nightly")]
pub use embassy_boot::FirmwareUpdater;
pub use embassy_boot::{AlignedBuffer, BlockingFirmwareUpdater, BootLoaderConfig, FirmwareUpdaterConfig, State};
use embassy_rp::flash::{Blocking, Flash, ERASE_SIZE};
use embassy_rp::peripherals::{FLASH, WATCHDOG};
use embassy_rp::watchdog::Watchdog;
use embassy_time::Duration;
use embedded_storage::nor_flash::{ErrorType, NorFlash, ReadNorFlash};

/// A bootloader for RP2040 devices.
pub struct BootLoader<ACTIVE: NorFlash, DFU: NorFlash, STATE: NorFlash, const BUFFER_SIZE: usize = ERASE_SIZE> {
    boot: embassy_boot::BootLoader<ACTIVE, DFU, STATE>,
    aligned_buf: AlignedBuffer<BUFFER_SIZE>,
}

impl<ACTIVE: NorFlash, DFU: NorFlash, STATE: NorFlash, const BUFFER_SIZE: usize>
    BootLoader<ACTIVE, DFU, STATE, BUFFER_SIZE>
{
    /// Create a new bootloader instance using the supplied partitions for active, dfu and state.
    pub fn new(config: BootLoaderConfig<ACTIVE, DFU, STATE>) -> Self {
        Self {
            boot: embassy_boot::BootLoader::new(config),
            aligned_buf: AlignedBuffer([0; BUFFER_SIZE]),
        }
    }

    /// Inspect the bootloader state and perform actions required before booting, such as swapping
    /// firmware.
    pub fn prepare(&mut self) {
        self.boot
            .prepare_boot(self.aligned_buf.as_mut())
            .expect("Boot prepare error");
    }

    /// Boots the application.
    ///
    /// # Safety
    ///
    /// This modifies the stack pointer and reset vector and will run code placed in the active partition.
    pub unsafe fn load(self, start: u32) -> ! {
        core::mem::drop(self.boot);

        trace!("Loading app at 0x{:x}", start);
        #[allow(unused_mut)]
        let mut p = cortex_m::Peripherals::steal();
        #[cfg(not(armv6m))]
        p.SCB.invalidate_icache();
        p.SCB.vtor.write(start);

        cortex_m::asm::bootload(start as *const u32)
    }
}

/// A flash implementation that will feed a watchdog when touching flash.
pub struct WatchdogFlash<'d, const SIZE: usize> {
    flash: Flash<'d, FLASH, Blocking, SIZE>,
    watchdog: Watchdog,
}

impl<'d, const SIZE: usize> WatchdogFlash<'d, SIZE> {
    /// Start a new watchdog with a given flash and watchdog peripheral and a timeout
    pub fn start(flash: FLASH, watchdog: WATCHDOG, timeout: Duration) -> Self {
        let flash = Flash::<_, Blocking, SIZE>::new(flash);
        let mut watchdog = Watchdog::new(watchdog);
        watchdog.start(timeout);
        Self { flash, watchdog }
    }
}

impl<'d, const SIZE: usize> ErrorType for WatchdogFlash<'d, SIZE> {
    type Error = <Flash<'d, FLASH, Blocking, SIZE> as ErrorType>::Error;
}

impl<'d, const SIZE: usize> NorFlash for WatchdogFlash<'d, SIZE> {
    const WRITE_SIZE: usize = <Flash<'d, FLASH, Blocking, SIZE> as NorFlash>::WRITE_SIZE;
    const ERASE_SIZE: usize = <Flash<'d, FLASH, Blocking, SIZE> as NorFlash>::ERASE_SIZE;

    fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
        self.watchdog.feed();
        self.flash.erase(from, to)
    }
    fn write(&mut self, offset: u32, data: &[u8]) -> Result<(), Self::Error> {
        self.watchdog.feed();
        self.flash.write(offset, data)
    }
}

impl<'d, const SIZE: usize> ReadNorFlash for WatchdogFlash<'d, SIZE> {
    const READ_SIZE: usize = <Flash<'d, FLASH, Blocking, SIZE> as ReadNorFlash>::READ_SIZE;
    fn read(&mut self, offset: u32, data: &mut [u8]) -> Result<(), Self::Error> {
        self.watchdog.feed();
        self.flash.read(offset, data)
    }
    fn capacity(&self) -> usize {
        self.flash.capacity()
    }
}
