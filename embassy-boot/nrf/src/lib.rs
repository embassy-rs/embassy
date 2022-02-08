#![no_std]
#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]

mod fmt;

pub use embassy_boot::{Partition, State, BOOT_MAGIC};
use embassy_nrf::gpio::*;
use embassy_nrf::nvmc::{Nvmc, FLASH_SIZE, PAGE_SIZE};

pub struct BootLoader {
    boot: embassy_boot::BootLoader<PAGE_SIZE>,
    ledbefore: Output<'static, AnyPin>,
    led: Output<'static, AnyPin>,
}

// IMPORTANT: Make sure this matches locations in your linker script
#[cfg(not(feature = "softdevice"))]
mod partitions {
    use super::*;
    pub const BOOTLOADER: Partition = Partition::new(0x0000, 0x3000);
    pub const BOOTLOADER_STATE: Partition = Partition::new(0x3000, 0x4000);
    pub const PARTITION_SIZE: usize = ((FLASH_SIZE - BOOTLOADER_STATE.to) / 2) - PAGE_SIZE;
    pub const ACTIVE: Partition =
        Partition::new(BOOTLOADER_STATE.to, BOOTLOADER_STATE.to + PARTITION_SIZE);
    pub const DFU: Partition = Partition::new(ACTIVE.to, ACTIVE.to + PARTITION_SIZE + PAGE_SIZE);
}

#[cfg(feature = "softdevice")]
mod partitions {
    use super::*;
    // TODO: Make it work with !s140 too
    pub const SOFTDEVICE: Partition = Partition::new(0x1000, 0x27000);

    pub const BOOTLOADER: Partition = Partition::new(FLASH_SIZE - 0x7000, FLASH_SIZE - 0x2000);
    pub const MBR_PARAMS_PAGE: Partition = Partition::new(BOOTLOADER.to, FLASH_SIZE - 0x1000);
    pub const BOOTLOADER_STATE: Partition = Partition::new(FLASH_SIZE - 0x1000, FLASH_SIZE);
    pub const PARTITION_SIZE: usize = ((BOOTLOADER.from - SOFTDEVICE.to) / 2) - PAGE_SIZE;
    pub const ACTIVE: Partition = Partition::new(SOFTDEVICE.to, SOFTDEVICE.to + PARTITION_SIZE);
    pub const DFU: Partition = Partition::new(ACTIVE.to, ACTIVE.to + PARTITION_SIZE + PAGE_SIZE);
}
pub use partitions::*;

impl BootLoader {
    pub fn new(ledbefore: Output<'static, AnyPin>, led: Output<'static, AnyPin>) -> Self {
        Self {
            boot: embassy_boot::BootLoader::new(ACTIVE, DFU, BOOTLOADER_STATE),
            ledbefore,
            led,
        }
    }

    /// Boots the application without softdevice mechanisms
    pub fn boot<'d>(&mut self, mut flash: Nvmc<'d>) -> ! {
        info!("Booting!");
        self.ledbefore.set_low();
        match self.boot.prepare_boot(&mut flash) {
            Ok(p) => {
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

        self.led.set_low();

        info!("msp = {=u32:x}, rv = {=u32:x}", msp, rv);

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

pub mod updater {
    use super::partitions::*;
    use super::*;
    pub fn new() -> embassy_boot::FirmwareUpdater {
        info!("Flash size: {}", FLASH_SIZE);
        info!("Bootloader from: {:x}", BOOTLOADER.from);
        info!("Softdevice end: {:x}", SOFTDEVICE.to);
        info!("Page size: {}", PAGE_SIZE);
        info!("Partition size is {}", PARTITION_SIZE);
        info!("Active partition starts at {:x}", ACTIVE.from);
        info!("DFU partition starts at {:x}", DFU.from);
        embassy_boot::FirmwareUpdater::new(DFU, BOOTLOADER_STATE)
    }
}

/*
    struct FlashWrapper<'a, F: NorFlash + ReadNorFlash>(&'a mut F);
    impl<'a, F: NorFlash + ReadNorFlash> embedded_storage_async::nor_flash::AsyncReadNorFlash
        for FlashWrapper<'a, F>
    {
        type Error = F::Error;
        const READ_SIZE: usize = F::READ_SIZE;

        type ReadFuture<'m>
        where
            Self: 'm,
        = impl Future<Output = Result<(), Self::Error>> + 'm;
        fn read<'m>(&'m mut self, address: usize, data: &'m mut [u8]) -> Self::ReadFuture<'m> {
            async move { self.0.read(address as u32, data) }
        }

        fn capacity(&self) -> usize {
            self.0.capacity()
        }
    }

    impl<'a, F: NorFlash + ReadNorFlash> embedded_storage_async::nor_flash::AsyncNorFlash
        for FlashWrapper<'a, F>
    {
        const WRITE_SIZE: usize = F::WRITE_SIZE;
        const ERASE_SIZE: usize = F::ERASE_SIZE;

        type WriteFuture<'m>
        where
            Self: 'm,
        = impl Future<Output = Result<(), Self::Error>> + 'm;
        fn write<'m>(&'m mut self, offset: u32, data: &'m [u8]) -> Self::WriteFuture<'m> {
            async move { self.0.write(offset, data) }
        }

        type EraseFuture<'m>
        where
            Self: 'm,
        = impl Future<Output = Result<(), Self::Error>> + 'm;
        fn erase<'m>(&'m mut self, from: u32, to: u32) -> Self::EraseFuture<'m> {
            async move { self.0.erase(from, to) }
        }
    }
*/
