#![no_std]
#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]

mod fmt;

pub use embassy_boot::{FirmwareUpdater, Partition, State, BOOT_MAGIC};
use embassy_nrf::{
    nvmc::{Nvmc, PAGE_SIZE},
    peripherals::WDT,
    wdt,
};
use embedded_storage::nor_flash::{ErrorType, NorFlash, ReadNorFlash};

pub struct BootLoader {
    boot: embassy_boot::BootLoader<PAGE_SIZE>,
}

impl BootLoader {
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

    /// Boots the application without softdevice mechanisms
    pub fn prepare<F: NorFlash + ReadNorFlash>(&mut self, flash: &mut F) -> usize {
        match self.boot.prepare_boot(flash) {
            Ok(_) => self.boot.boot_address(),
            Err(_) => panic!("boot prepare error!"),
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
    pub unsafe fn load(&mut self, _app: usize) -> ! {
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

        trace!("msp = {=u32:x}, rv = {=u32:x}", msp, rv);

        // These instructions perform the following operations:
        //
        // * Modify control register to use MSP as stack pointer (clear spsel bit)
        // * Synchronize instruction barrier
        // * Initialize stack pointer (0x1000)
        // * Set link register to not return (0xFF)
        // * Jump to softdevice reset vector
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

/// A flash implementation that wraps NVMC and will pet a watchdog when touching flash.
pub struct WatchdogFlash<'d> {
    flash: Nvmc<'d>,
    wdt: wdt::WatchdogHandle,
}

impl<'d> WatchdogFlash<'d> {
    /// Start a new watchdog with a given flash and WDT peripheral and a timeout
    pub fn start(flash: Nvmc<'d>, wdt: WDT, timeout: u32) -> Self {
        let mut config = wdt::Config::default();
        config.timeout_ticks = 32768 * timeout; // timeout seconds
        config.run_during_sleep = true;
        config.run_during_debug_halt = false;
        let (_wdt, [wdt]) = match wdt::Watchdog::try_new(wdt, config) {
            Ok(x) => x,
            Err(_) => {
                // In case the watchdog is already running, just spin and let it expire, since
                // we can't configure it anyway. This usually happens when we first program
                // the device and the watchdog was previously active
                info!("Watchdog already active with wrong config, waiting for it to timeout...");
                loop {}
            }
        };
        Self { flash, wdt }
    }
}

impl<'d> ErrorType for WatchdogFlash<'d> {
    type Error = <Nvmc<'d> as ErrorType>::Error;
}

impl<'d> NorFlash for WatchdogFlash<'d> {
    const WRITE_SIZE: usize = <Nvmc<'d> as NorFlash>::WRITE_SIZE;
    const ERASE_SIZE: usize = <Nvmc<'d> as NorFlash>::ERASE_SIZE;

    fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
        self.wdt.pet();
        self.flash.erase(from, to)
    }
    fn write(&mut self, offset: u32, data: &[u8]) -> Result<(), Self::Error> {
        self.wdt.pet();
        self.flash.write(offset, data)
    }
}

impl<'d> ReadNorFlash for WatchdogFlash<'d> {
    const READ_SIZE: usize = <Nvmc<'d> as ReadNorFlash>::READ_SIZE;
    fn read(&mut self, offset: u32, data: &mut [u8]) -> Result<(), Self::Error> {
        self.wdt.pet();
        self.flash.read(offset, data)
    }
    fn capacity(&self) -> usize {
        self.flash.capacity()
    }
}

pub mod updater {
    use super::*;
    pub fn new() -> embassy_boot::FirmwareUpdater {
        extern "C" {
            static __bootloader_state_start: u32;
            static __bootloader_state_end: u32;
            static __bootloader_dfu_start: u32;
            static __bootloader_dfu_end: u32;
        }

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
        embassy_boot::FirmwareUpdater::new(dfu, state)
    }
}
