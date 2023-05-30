#![no_std]
#![warn(missing_docs)]
#![doc = include_str!("../README.md")]
mod fmt;

#[cfg(feature = "nightly")]
pub use embassy_boot::FirmwareUpdater;
pub use embassy_boot::{AlignedBuffer, BlockingFirmwareUpdater, BootLoaderConfig, FirmwareUpdaterConfig, State};
use embedded_storage::nor_flash::NorFlash;

/// A bootloader for STM32 devices.
pub struct BootLoader<ACTIVE: NorFlash, DFU: NorFlash, STATE: NorFlash, const BUFFER_SIZE: usize> {
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
    pub unsafe fn load(&mut self, start: u32) -> ! {
        trace!("Loading app at 0x{:x}", start);
        #[allow(unused_mut)]
        let mut p = cortex_m::Peripherals::steal();
        #[cfg(not(armv6m))]
        p.SCB.invalidate_icache();
        p.SCB.vtor.write(start);

        cortex_m::asm::bootload(start as *const u32)
    }
}
