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
use embassy_time::Timer;
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

    let mut button = ExtiInput::new(p.PB2, p.EXTI2, Pull::Up);

    let mut led = Output::new(p.PB5, Level::Low, Speed::Low);

    led.set_high();

    let config = FirmwareUpdaterConfig::from_linkerfile(&flash, &flash);
    let mut magic = AlignedBuffer([0; WRITE_SIZE]);
    let mut updater = FirmwareUpdater::new(config, &mut magic.0);
    button.wait_for_falling_edge().await;
    let mut offset = 0;
    for chunk in APP_B.chunks(128) {
        let mut buf: [u8; 128] = [0; 128];
        buf[..chunk.len()].copy_from_slice(chunk);
        updater.write_firmware(offset, &buf).await.unwrap();
        offset += chunk.len();
    }

    updater.mark_updated().await.unwrap();
    led.set_low();
    Timer::after_secs(1).await;
    cortex_m::peripheral::SCB::sys_reset();
}
