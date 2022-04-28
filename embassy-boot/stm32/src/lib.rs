#![no_std]
#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

mod fmt;

pub use embassy_boot::{
    FirmwareUpdater, FlashConfig, FlashProvider, Partition, SingleFlashProvider, State,
};
use embedded_storage::nor_flash::NorFlash;

pub struct BootLoader<const PAGE_SIZE: usize> {
    boot: embassy_boot::BootLoader<PAGE_SIZE>,
}

impl<const PAGE_SIZE: usize> BootLoader<PAGE_SIZE> {
    /// Create a new bootloader instance using parameters from linker script
    pub fn default() -> Self {
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
                &__bootloader_active_start as *const u32 as usize,
                &__bootloader_active_end as *const u32 as usize,
            )
        };
        let dfu = unsafe {
            Partition::new(
                &__bootloader_dfu_start as *const u32 as usize,
                &__bootloader_dfu_end as *const u32 as usize,
            )
        };
        let state = unsafe {
            Partition::new(
                &__bootloader_state_start as *const u32 as usize,
                &__bootloader_state_end as *const u32 as usize,
            )
        };

        trace!("ACTIVE: 0x{:x} - 0x{:x}", active.from, active.to);
        trace!("DFU: 0x{:x} - 0x{:x}", dfu.from, dfu.to);
        trace!("STATE: 0x{:x} - 0x{:x}", state.from, state.to);

        Self::new(active, dfu, state)
    }

    /// Create a new bootloader instance using the supplied partitions for active, dfu and state.
    pub fn new(active: Partition, dfu: Partition, state: Partition) -> Self {
        Self {
            boot: embassy_boot::BootLoader::new(active, dfu, state),
        }
    }

    /// Boots the application
    pub fn prepare<F: FlashProvider>(&mut self, flash: &mut F) -> usize
    where
        [(); <<F as FlashProvider>::STATE as FlashConfig>::FLASH::WRITE_SIZE]:,
        [(); <<F as FlashProvider>::ACTIVE as FlashConfig>::FLASH::ERASE_SIZE]:,
    {
        match self.boot.prepare_boot(flash) {
            Ok(_) => self.boot.boot_address(),
            Err(_) => panic!("boot prepare error!"),
        }
    }

    pub unsafe fn load(&mut self, start: usize) -> ! {
        trace!("Loading app at 0x{:x}", start);
        #[allow(unused_mut)]
        let mut p = cortex_m::Peripherals::steal();
        #[cfg(not(feature = "thumbv6"))]
        p.SCB.invalidate_icache();
        p.SCB.vtor.write(start as u32);

        cortex_m::asm::bootload(start as *const u32)
    }
}
