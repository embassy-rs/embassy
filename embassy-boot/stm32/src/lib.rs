#![no_std]
#![feature(type_alias_impl_trait)]
#![warn(missing_docs)]
#![doc = include_str!("../README.md")]
mod fmt;

pub use embassy_boot::{AlignedBuffer, BootFlash, FirmwareUpdater, FlashConfig, Partition, SingleFlashConfig, State};

/// A bootloader for STM32 devices.
pub struct BootLoader<const PAGE_SIZE: usize, const WRITE_SIZE: usize> {
    boot: embassy_boot::BootLoader,
    magic: AlignedBuffer<WRITE_SIZE>,
    page: AlignedBuffer<PAGE_SIZE>,
}

impl<const PAGE_SIZE: usize, const WRITE_SIZE: usize> BootLoader<PAGE_SIZE, WRITE_SIZE> {
    /// Create a new bootloader instance using the supplied partitions for active, dfu and state.
    pub fn new(active: Partition, dfu: Partition, state: Partition) -> Self {
        Self {
            boot: embassy_boot::BootLoader::new(active, dfu, state),
            magic: AlignedBuffer([0; WRITE_SIZE]),
            page: AlignedBuffer([0; PAGE_SIZE]),
        }
    }

    /// Inspect the bootloader state and perform actions required before booting, such as swapping
    /// firmware.
    pub fn prepare<F: FlashConfig>(&mut self, flash: &mut F) -> usize {
        match self.boot.prepare_boot(flash, self.magic.as_mut(), self.page.as_mut()) {
            Ok(_) => embassy_stm32::flash::FLASH_BASE + self.boot.boot_address(),
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

impl<const PAGE_SIZE: usize, const WRITE_SIZE: usize> Default for BootLoader<PAGE_SIZE, WRITE_SIZE> {
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
}
