#![no_std]
#![feature(type_alias_impl_trait)]
#![warn(missing_docs)]
#![doc = include_str!("../README.md")]
mod fmt;

pub use embassy_boot::{AlignedBuffer, BootFlash, FirmwareUpdater, FlashConfig, Partition, SingleFlashConfig, State};
use embassy_rp::flash::{ERASE_SIZE, FLASH_BASE, WRITE_SIZE};

/// A bootloader for RP2040 devices.
pub struct BootLoader {
    boot: embassy_boot::BootLoader,
    magic: AlignedBuffer<WRITE_SIZE>,
    page: AlignedBuffer<ERASE_SIZE>,
}

impl Default for BootLoader {
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

        trace!(
            "ACTIVE: 0x{:x} - 0x{:x}. LEN: {:?}",
            active.from,
            active.to,
            active.to - active.from
        );
        trace!("DFU: 0x{:x} - 0x{:x}. LEN: {:?}", dfu.from, dfu.to, dfu.to - dfu.from);
        trace!(
            "STATE: 0x{:x} - 0x{:x}. LEN: {:?}",
            state.from,
            state.to,
            state.to - state.from
        );

        Self::new(active, dfu, state)
    }
}

impl BootLoader {
    /// Create a new bootloader instance using the supplied partitions for active, dfu and state.
    pub fn new(active: Partition, dfu: Partition, state: Partition) -> Self {
        Self {
            boot: embassy_boot::BootLoader::new(active, dfu, state),
            magic: AlignedBuffer([0; WRITE_SIZE]),
            page: AlignedBuffer([0; ERASE_SIZE]),
        }
    }

    /// Inspect the bootloader state and perform actions required before booting, such as swapping
    /// firmware.
    pub fn prepare<F: FlashConfig>(&mut self, flash: &mut F) -> usize {
        match self.boot.prepare_boot(flash, &mut self.magic.0, &mut self.page.0) {
            Ok(_) => FLASH_BASE as usize + self.boot.boot_address(),
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
        let p = cortex_m::Peripherals::steal();
        p.SCB.vtor.write(start as u32);

        cortex_m::asm::bootload(start as *const u32)
    }
}
