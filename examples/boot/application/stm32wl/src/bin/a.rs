#![no_std]
#![no_main]

#[cfg(feature = "defmt-rtt")]
use defmt_rtt::*;
use embassy_boot_stm32::{AlignedBuffer, FirmwareUpdater, FirmwareUpdaterConfig};
use embassy_embedded_hal::adapter::BlockingAsync;
use embassy_executor::Spawner;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::flash::{Flash, WRITE_SIZE};
use embassy_stm32::gpio::{Level, Output, Pull, Speed};
use embassy_sync::mutex::Mutex;
use panic_reset as _;

#[cfg(feature = "skip-include")]
static APP_B: &[u8] = &[0, 1, 2, 3];
#[cfg(not(feature = "skip-include"))]
static APP_B: &[u8] = include_bytes!("../../b.bin");

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    let flash = Flash::new_blocking(p.FLASH);
    let flash = Mutex::new(BlockingAsync::new(flash));

    let mut button = ExtiInput::new(p.PA0, p.EXTI0, Pull::Up);

    let mut led = Output::new(p.PB9, Level::Low, Speed::Low);
    led.set_high();

    let config = FirmwareUpdaterConfig::from_linkerfile(&flash, &flash);
    let mut magic = AlignedBuffer([0; WRITE_SIZE]);
    let mut updater = FirmwareUpdater::new(config, &mut magic.0);
    button.wait_for_falling_edge().await;
    //defmt::info!("Starting update");
    let mut offset = 0;
    for chunk in APP_B.chunks(2048) {
        let mut buf: [u8; 2048] = [0; 2048];
        buf[..chunk.len()].copy_from_slice(chunk);
        //        defmt::info!("Writing chunk at 0x{:x}", offset);
        updater.write_firmware(offset, &buf).await.unwrap();
        offset += chunk.len();
    }
    updater.mark_updated().await.unwrap();
    //defmt::info!("Marked as updated");
    led.set_low();
    cortex_m::peripheral::SCB::sys_reset();
}
