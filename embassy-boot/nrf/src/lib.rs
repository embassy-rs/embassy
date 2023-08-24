#![no_std]
#![warn(missing_docs)]
#![doc = include_str!("../README.md")]
mod fmt;

pub use embassy_boot::{
    AlignedBuffer, BlockingFirmwareState, BlockingFirmwareUpdater, BootLoaderConfig, FirmwareUpdaterConfig,
};
#[cfg(feature = "nightly")]
pub use embassy_boot::{FirmwareState, FirmwareUpdater};
use embassy_nrf::nvmc::PAGE_SIZE;
use embassy_nrf::peripherals::WDT;
use embassy_nrf::wdt;
use embedded_storage::nor_flash::{ErrorType, NorFlash, ReadNorFlash};

/// A bootloader for nRF devices.
pub struct BootLoader<const BUFFER_SIZE: usize = PAGE_SIZE>;

impl<const BUFFER_SIZE: usize> BootLoader<BUFFER_SIZE> {
    /// Inspect the bootloader state and perform actions required before booting, such as swapping firmware.
    pub fn prepare<ACTIVE: NorFlash, DFU: NorFlash, STATE: NorFlash>(
        config: BootLoaderConfig<ACTIVE, DFU, STATE>,
    ) -> Self {
        let mut aligned_buf = AlignedBuffer([0; BUFFER_SIZE]);
        let mut boot = embassy_boot::BootLoader::new(config);
        boot.prepare_boot(&mut aligned_buf.0).expect("Boot prepare error");
        Self
    }

    /// Boots the application without softdevice mechanisms.
    ///
    /// # Safety
    ///
    /// This modifies the stack pointer and reset vector and will run code placed in the active partition.
    #[cfg(not(feature = "softdevice"))]
    pub unsafe fn load(self, start: u32) -> ! {
        let mut p = cortex_m::Peripherals::steal();
        p.SCB.invalidate_icache();
        p.SCB.vtor.write(start);
        cortex_m::asm::bootload(start as *const u32)
    }

    /// Boots the application assuming softdevice is present.
    ///
    /// # Safety
    ///
    /// This modifies the stack pointer and reset vector and will run code placed in the active partition.
    #[cfg(feature = "softdevice")]
    pub unsafe fn load(self, _app: u32) -> ! {
        use nrf_softdevice_mbr as mbr;
        const NRF_SUCCESS: u32 = 0;

        // Address of softdevice which we'll forward interrupts to
        let addr = 0x1000;
        let mut cmd = mbr::sd_mbr_command_t {
            command: mbr::NRF_MBR_COMMANDS_SD_MBR_COMMAND_IRQ_FORWARD_ADDRESS_SET,
            params: mbr::sd_mbr_command_t__bindgen_ty_1 {
                irq_forward_address_set: mbr::sd_mbr_command_irq_forward_address_set_t { address: addr },
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

/// A flash implementation that wraps any flash and will pet a watchdog when touching flash.
pub struct WatchdogFlash<FLASH> {
    flash: FLASH,
    wdt: wdt::WatchdogHandle,
}

impl<FLASH> WatchdogFlash<FLASH> {
    /// Start a new watchdog with a given flash and WDT peripheral and a timeout
    pub fn start(flash: FLASH, wdt: WDT, config: wdt::Config) -> Self {
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

impl<FLASH: ErrorType> ErrorType for WatchdogFlash<FLASH> {
    type Error = FLASH::Error;
}

impl<FLASH: NorFlash> NorFlash for WatchdogFlash<FLASH> {
    const WRITE_SIZE: usize = FLASH::WRITE_SIZE;
    const ERASE_SIZE: usize = FLASH::ERASE_SIZE;

    fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
        self.wdt.pet();
        self.flash.erase(from, to)
    }
    fn write(&mut self, offset: u32, data: &[u8]) -> Result<(), Self::Error> {
        self.wdt.pet();
        self.flash.write(offset, data)
    }
}

impl<FLASH: ReadNorFlash> ReadNorFlash for WatchdogFlash<FLASH> {
    const READ_SIZE: usize = FLASH::READ_SIZE;
    fn read(&mut self, offset: u32, data: &mut [u8]) -> Result<(), Self::Error> {
        self.wdt.pet();
        self.flash.read(offset, data)
    }
    fn capacity(&self) -> usize {
        self.flash.capacity()
    }
}
