#![no_std]
#![allow(unsafe_op_in_unsafe_fn)]
#![warn(missing_docs)]
#![doc = include_str!("../README.md")]
mod fmt;

pub use embassy_boot::{
    AlignedBuffer, BlockingFirmwareState, BlockingFirmwareUpdater, BootError, BootLoaderConfig, FirmwareState,
    FirmwareUpdater, FirmwareUpdaterConfig, State,
};
use embedded_storage::nor_flash::NorFlash;

/// A bootloader for STM32 devices.
pub struct BootLoader {
    /// The reported state of the bootloader after preparing for boot
    pub state: State,
}

impl BootLoader {
    /// Inspect the bootloader state and perform actions required before booting, such as swapping firmware
    pub fn prepare<ACTIVE: NorFlash, DFU: NorFlash, STATE: NorFlash, const BUFFER_SIZE: usize>(
        config: BootLoaderConfig<ACTIVE, DFU, STATE>,
    ) -> Self {
        if let Ok(loader) = Self::try_prepare::<ACTIVE, DFU, STATE, BUFFER_SIZE>(config) {
            loader
        } else {
            // Use explicit panic instead of .expect() to ensure this gets routed via defmt/etc.
            // properly
            panic!("Boot prepare error")
        }
    }

    /// Inspect the bootloader state and perform actions required before booting, such as swapping firmware
    pub fn try_prepare<ACTIVE: NorFlash, DFU: NorFlash, STATE: NorFlash, const BUFFER_SIZE: usize>(
        config: BootLoaderConfig<ACTIVE, DFU, STATE>,
    ) -> Result<Self, BootError> {
        let mut aligned_buf = AlignedBuffer([0; BUFFER_SIZE]);
        let mut boot = embassy_boot::BootLoader::new(config);
        let state = boot.prepare_boot(aligned_buf.as_mut())?;
        Ok(Self { state })
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
