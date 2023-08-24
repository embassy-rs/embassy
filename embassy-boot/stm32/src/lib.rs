#![no_std]
#![warn(missing_docs)]
#![doc = include_str!("../README.md")]
mod fmt;

pub use embassy_boot::{
    AlignedBuffer, BlockingFirmwareState, BlockingFirmwareUpdater, BootLoaderConfig, FirmwareUpdaterConfig, State,
};
#[cfg(feature = "nightly")]
pub use embassy_boot::{FirmwareState, FirmwareUpdater};
use embedded_storage::nor_flash::NorFlash;

/// A bootloader for STM32 devices.
pub struct BootLoader;

impl BootLoader {
    /// Inspect the bootloader state and perform actions required before booting, such as swapping firmware
    pub fn prepare<ACTIVE: NorFlash, DFU: NorFlash, STATE: NorFlash, const BUFFER_SIZE: usize>(
        config: BootLoaderConfig<ACTIVE, DFU, STATE>,
    ) -> Self {
        let mut aligned_buf = AlignedBuffer([0; BUFFER_SIZE]);
        let mut boot = embassy_boot::BootLoader::new(config);
        boot.prepare_boot(aligned_buf.as_mut()).expect("Boot prepare error");
        Self
    }

    /// Boots the application.
    ///
    /// # Safety
    ///
    /// This modifies the stack pointer and reset vector and will run code placed in the active partition.
    pub unsafe fn load(self, start: u32) -> ! {
        trace!("Loading app at 0x{:x}", start);
        #[allow(unused_mut)]
        let mut p = cortex_m::Peripherals::steal();
        #[cfg(not(armv6m))]
        p.SCB.invalidate_icache();
        p.SCB.vtor.write(start);

        cortex_m::asm::bootload(start as *const u32)
    }
}
