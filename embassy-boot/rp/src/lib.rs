#![no_std]
#![warn(missing_docs)]
#![doc = include_str!("../README.md")]
mod fmt;

pub use embassy_boot::{AlignedBuffer, BootFlash, FirmwareUpdater, FlashConfig, Partition, SingleFlashConfig, State};
use embassy_rp::flash::{Flash, ERASE_SIZE};
use embassy_rp::peripherals::{FLASH, WATCHDOG};
use embassy_rp::watchdog::Watchdog;
use embassy_time::Duration;
use embedded_storage::nor_flash::{ErrorType, NorFlash, ReadNorFlash};

/// A bootloader for RP2040 devices.
pub struct BootLoader<const BUFFER_SIZE: usize = ERASE_SIZE> {
    boot: embassy_boot::BootLoader,
    aligned_buf: AlignedBuffer<BUFFER_SIZE>,
}

impl<const BUFFER_SIZE: usize> BootLoader<BUFFER_SIZE> {
    /// Create a new bootloader instance using the supplied partitions for active, dfu and state.
    pub fn new(active: Partition, dfu: Partition, state: Partition) -> Self {
        Self {
            boot: embassy_boot::BootLoader::new(active, dfu, state),
            aligned_buf: AlignedBuffer([0; BUFFER_SIZE]),
        }
    }

    /// Inspect the bootloader state and perform actions required before booting, such as swapping
    /// firmware.
    pub fn prepare<F: FlashConfig>(&mut self, flash: &mut F) -> usize {
        match self.boot.prepare_boot(flash, self.aligned_buf.as_mut()) {
            Ok(_) => embassy_rp::flash::FLASH_BASE + self.boot.boot_address(),
            Err(_) => panic!("boot prepare error!"),
        }
    }

    /// Boots the application.
    ///
    /// # Safety
    ///
    /// This modifies the stack pointer and reset vector and will run code placed in the active partition.
    pub unsafe fn load(&mut self, start: usize) -> ! {
        trace!("Loading app at 0x{:x}", start);
        #[allow(unused_mut)]
        let mut p = cortex_m::Peripherals::steal();
        #[cfg(not(armv6m))]
        p.SCB.invalidate_icache();
        p.SCB.vtor.write(start as u32);

        cortex_m::asm::bootload(start as *const u32)
    }
}

#[cfg(target_os = "none")]
impl Default for BootLoader<ERASE_SIZE> {
    /// Create a new bootloader instance using parameters from linker script
    fn default() -> Self {
        extern "C" {
            static __bootloader_state_start: u32;
            static __bootloader_state_end: u32;
            static __bootloader_active_start: u32;
            static __bootloader_active_end: u32;
            static __bootloader_dfu_start: u32;
            static __bootloader_dfu_end: u32;
        }

        let active = unsafe {
            Partition::new(
                &__bootloader_active_start as *const u32 as u32,
                &__bootloader_active_end as *const u32 as u32,
            )
        };
        let dfu = unsafe {
            Partition::new(
                &__bootloader_dfu_start as *const u32 as u32,
                &__bootloader_dfu_end as *const u32 as u32,
            )
        };
        let state = unsafe {
            Partition::new(
                &__bootloader_state_start as *const u32 as u32,
                &__bootloader_state_end as *const u32 as u32,
            )
        };

        trace!("ACTIVE: 0x{:x} - 0x{:x}", active.from, active.to);
        trace!("DFU: 0x{:x} - 0x{:x}", dfu.from, dfu.to);
        trace!("STATE: 0x{:x} - 0x{:x}", state.from, state.to);

        Self::new(active, dfu, state)
    }
}

/// A flash implementation that will feed a watchdog when touching flash.
pub struct WatchdogFlash<'d, const SIZE: usize> {
    flash: Flash<'d, FLASH, SIZE>,
    watchdog: Watchdog,
}

impl<'d, const SIZE: usize> WatchdogFlash<'d, SIZE> {
    /// Start a new watchdog with a given flash and watchdog peripheral and a timeout
    pub fn start(flash: FLASH, watchdog: WATCHDOG, timeout: Duration) -> Self {
        let flash: Flash<'_, FLASH, SIZE> = Flash::new(flash);
        let mut watchdog = Watchdog::new(watchdog);
        watchdog.start(timeout);
        Self { flash, watchdog }
    }
}

impl<'d, const SIZE: usize> ErrorType for WatchdogFlash<'d, SIZE> {
    type Error = <Flash<'d, FLASH, SIZE> as ErrorType>::Error;
}

impl<'d, const SIZE: usize> NorFlash for WatchdogFlash<'d, SIZE> {
    const WRITE_SIZE: usize = <Flash<'d, FLASH, SIZE> as NorFlash>::WRITE_SIZE;
    const ERASE_SIZE: usize = <Flash<'d, FLASH, SIZE> as NorFlash>::ERASE_SIZE;

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
    const READ_SIZE: usize = <Flash<'d, FLASH, SIZE> as ReadNorFlash>::READ_SIZE;
    fn read(&mut self, offset: u32, data: &mut [u8]) -> Result<(), Self::Error> {
        self.watchdog.feed();
        self.flash.read(offset, data)
    }
    fn capacity(&self) -> usize {
        self.flash.capacity()
    }
}
