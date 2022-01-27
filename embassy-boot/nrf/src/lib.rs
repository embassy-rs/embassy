#![no_std]
pub use embassy_boot::{Partition, BOOT_MAGIC};
use embassy_nrf::nvmc::{Error, Nvmc, FLASH_SIZE, PAGE_SIZE};

pub struct BootLoader {
    boot: embassy_boot::BootLoader<PAGE_SIZE>,
}

// IMPORTANT: Make sure this matches locations in your linker script
#[cfg(not(feature = "softdevice"))]
mod partitions {
    use super::*;
    pub const BOOTLOADER: Partition = Partition::new(0x3000, 0x4000);
    pub const PARTITION_SIZE: usize = ((FLASH_SIZE - BOOTLOADER.to) / 2) - PAGE_SIZE;
    pub const ACTIVE: Partition = Partition::new(BOOTLOADER.to, BOOTLOADER.to + PARTITION_SIZE);
    pub const DFU: Partition = Partition::new(ACTIVE.to, ACTIVE.to + PARTITION_SIZE + PAGE_SIZE);
}

#[cfg(feature = "softdevice")]
mod partitions {
    use super::*;
    // TODO: Make it work with !s140 too
    pub const SOFTDEVICE: Partition = Partition::new(0x1000, 0x27000);

    pub const BOOTLOADER: Partition = Partition::new(FLASH_SIZE - 0x1000, FLASH_SIZE);
    pub const PARTITION_SIZE: usize = ((BOOTLOADER.from - SOFTDEVICE.to) / 2) - PAGE_SIZE;
    pub const ACTIVE: Partition = Partition::new(SOFTDEVICE.to, SOFTDEVICE.to + PARTITION_SIZE);
    pub const DFU: Partition = Partition::new(ACTIVE.to, ACTIVE.to + PARTITION_SIZE + PAGE_SIZE);
}
use partitions::*;

impl BootLoader {
    pub fn new() -> Self {
        Self {
            boot: embassy_boot::BootLoader::new(ACTIVE, DFU, BOOTLOADER),
        }
    }

    /// Boots the application without softdevice mechanisms
    pub fn boot<'d>(&mut self, mut flash: Nvmc<'d>) -> ! {
        match self.boot.prepare_boot(&mut flash) {
            Ok(_) => {
                let start = self.boot.boot_address();
                unsafe {
                    self.load(start);
                }
            }
            Err(e) => panic!("boot error: {:?}", e),
        }
    }

    #[cfg(not(feature = "softdevice"))]
    pub unsafe fn load(&mut self, start: usize) -> ! {
        let mut p = cortex_m::Peripherals::steal();
        p.SCB.invalidate_icache();
        p.SCB.vtor.write(start as u32);
        cortex_m::asm::bootload(start as *const u32)
    }

    #[cfg(feature = "softdevice")]
    pub unsafe fn load(&mut self, _start: usize) -> ! {
        #[used]
        #[no_mangle]
        #[link_section = ".uicr_bootloader_start_address"]
        pub static UICR_BOOTLOADER_START_ADDRESS: usize = BOOTLOADER.from;

        use nrf_softdevice_mbr as mbr;
        const NRF_SUCCESS: u32 = 0;

        // Address of softdevice which we'll forward interrupts to
        let addr = 0x1000;
        let mut cmd = mbr::sd_mbr_command_t {
            command: mbr::NRF_MBR_COMMANDS_SD_MBR_COMMAND_IRQ_FORWARD_ADDRESS_SET,
            params: mbr::sd_mbr_command_t__bindgen_ty_1 {
                irq_forward_address_set: mbr::sd_mbr_command_irq_forward_address_set_t {
                    address: addr,
                },
            },
        };
        let ret = mbr::sd_mbr_command(&mut cmd);
        assert_eq!(ret, NRF_SUCCESS);

        let msp = *(addr as *const u32);
        let rv = *((addr + 4) as *const u32);

        // info!("msp = {=u32:x}, rv = {=u32:x}", msp, rv);

        core::arch::asm!(
            "mrs {tmp}, CONTROL",
            "bics {tmp}, {spsel}",
            "msr CONTROL, {tmp}",
            "isb",
            "msr MSP, {msp}",
            "mov lr, {new_lr}",
            "bx {rv}",
            // `out(reg) _` is not permitted in a `noreturn` asm! call,
            // so instead use `in(reg) 0` and don't restore it afterwards.
            tmp = in(reg) 0,
            spsel = in(reg) 2,
            new_lr = in(reg) 0xFFFFFFFFu32,
            msp = in(reg) msp,
            rv = in(reg) rv,
            options(noreturn),
        );
    }
}

pub struct FirmwareUpdater(embassy_boot::FirmwareUpdater);

impl FirmwareUpdater {
    pub fn new() -> Self {
        Self(embassy_boot::FirmwareUpdater::new(DFU, BOOTLOADER))
    }

    pub async fn mark_update(&mut self, flash: &mut Nvmc) -> Result<(), Error> {
        self.0.mark_update(flash).await
    }

    pub async fn mark_booted(&mut self, flash: &mut Nvmc) -> Result<(), Error> {
        self.0.mark_booted(flash).await
    }

    pub async fn write_firmware(
        &mut self,
        offset: usize,
        data: &[u8],
        flash: &mut Nvmc,
    ) -> Result<(), Error> {
        self.0.write_firmware(offset, data, flash).await
    }

    pub fn reset(&mut self) {
        embassy_nrf::pac::SCB::sys_reset();
    }
}
