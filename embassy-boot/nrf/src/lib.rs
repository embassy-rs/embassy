#![no_std]
pub use embassy_boot::{Partition, BOOT_MAGIC};
use embassy_nrf::nvmc::{Error, Nvmc, FLASH_SIZE, PAGE_SIZE};

pub struct BootLoader {
    boot: embassy_boot::BootLoader<PAGE_SIZE>,
}

// IMPORTANT: Make sure this matches locations in your linker script
const BOOTLOADER: Partition = Partition::new(0x3000, 0x4000);
const PARTITION_SIZE: usize = ((FLASH_SIZE - BOOTLOADER.to) / 2) - PAGE_SIZE;
const ACTIVE: Partition = Partition::new(BOOTLOADER.to, BOOTLOADER.to + PARTITION_SIZE);
const DFU: Partition = Partition::new(ACTIVE.to, ACTIVE.to + PARTITION_SIZE + PAGE_SIZE);

impl BootLoader {
    pub fn new() -> Self {
        Self {
            boot: embassy_boot::BootLoader::new(ACTIVE, DFU, BOOTLOADER),
        }
    }

    /// Boots the application without softdevice mechanisms
    pub fn boot<'d>(&mut self, mut flash: Nvmc<'d>) -> ! {
        match self.boot.prepare_boot(&mut flash) {
            Ok(_) => unsafe {
                let mut p = cortex_m::Peripherals::steal();
                let start = self.boot.boot_address();
                p.SCB.invalidate_icache();
                p.SCB.vtor.write(start as u32);
                cortex_m::asm::bootload(start as *const u32)
            },
            Err(e) => panic!("boot error: {:?}", e),
        }
    }
}

pub struct FirmwareUpdater(embassy_boot::FirmwareUpdater);

impl FirmwareUpdater {
    pub fn new() -> Self {
        Self(embassy_boot::FirmwareUpdater::new(DFU, BOOTLOADER))
    }

    pub fn mark_update(&mut self, flash: &mut Nvmc) -> Result<(), Error> {
        self.0.mark_update(flash)
    }

    pub fn mark_booted(&mut self, flash: &mut Nvmc) -> Result<(), Error> {
        self.0.mark_booted(flash)
    }

    pub fn write_firmware(
        &mut self,
        offset: usize,
        data: &[u8],
        flash: &mut Nvmc,
    ) -> Result<(), Error> {
        self.0.write_firmware(offset, data, flash)
    }

    pub fn reset(&mut self) {
        embassy_nrf::pac::SCB::sys_reset();
    }
}
