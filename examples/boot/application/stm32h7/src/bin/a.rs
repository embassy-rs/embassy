#![no_std]
#![no_main]

use core::cell::RefCell;

#[cfg(feature = "defmt")]
use defmt_rtt::*;
use embassy_boot_stm32::{AlignedBuffer, BlockingFirmwareUpdater, FirmwareUpdaterConfig};
use embassy_executor::Spawner;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::flash::{Flash, WRITE_SIZE};
use embassy_stm32::gpio::{Level, Output, Pull, Speed};
use embassy_sync::blocking_mutex::Mutex;
use embedded_storage::nor_flash::NorFlash;
use panic_reset as _;

#[cfg(feature = "skip-include")]
static APP_B: &[u8] = &[0, 1, 2, 3];
#[cfg(not(feature = "skip-include"))]
static APP_B: &[u8] = include_bytes!("../../b.bin");

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    let flash = Flash::new_blocking(p.FLASH);
    let flash = Mutex::new(RefCell::new(flash));

    let mut button = ExtiInput::new(p.PC13, p.EXTI13, Pull::Down);

    let mut led = Output::new(p.PB14, Level::Low, Speed::Low);
    led.set_high();

    let config = FirmwareUpdaterConfig::from_linkerfile_blocking(&flash, &flash);
    let mut magic = AlignedBuffer([0; WRITE_SIZE]);
    let mut updater = BlockingFirmwareUpdater::new(config, &mut magic.0);
    let writer = updater.prepare_update().unwrap();
    button.wait_for_rising_edge().await;
    let mut offset = 0;
    let mut buf = AlignedBuffer([0; 4096]);
    for chunk in APP_B.chunks(4096) {
        buf.as_mut()[..chunk.len()].copy_from_slice(chunk);
        writer.write(offset, buf.as_ref()).unwrap();
        offset += chunk.len() as u32;
    }
    updater.mark_updated().unwrap();
    led.set_low();
    cortex_m::peripheral::SCB::sys_reset();
}
