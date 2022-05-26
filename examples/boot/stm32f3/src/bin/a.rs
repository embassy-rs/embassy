#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy_boot_stm32::FirmwareUpdater;
use embassy_embedded_hal::adapter::BlockingAsync;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::flash::Flash;
use embassy_stm32::gpio::{Input, Level, Output, Pull, Speed};
use embassy_stm32::Peripherals;
use panic_reset as _;

#[cfg(feature = "defmt-rtt")]
use defmt_rtt::*;

static APP_B: &[u8] = include_bytes!("../../b.bin");

#[embassy::main]
async fn main(_s: embassy::executor::Spawner, p: Peripherals) {
    let flash = Flash::unlock(p.FLASH);
    let mut flash = BlockingAsync::new(flash);

    let button = Input::new(p.PC13, Pull::Up);
    let mut button = ExtiInput::new(button, p.EXTI13);

    let mut led = Output::new(p.PA5, Level::Low, Speed::Low);
    led.set_high();

    let mut updater = FirmwareUpdater::default();
    button.wait_for_falling_edge().await;
    let mut offset = 0;
    for chunk in APP_B.chunks(2048) {
        let mut buf: [u8; 2048] = [0; 2048];
        buf[..chunk.len()].copy_from_slice(chunk);
        updater
            .write_firmware(offset, &buf, &mut flash, 2048)
            .await
            .unwrap();
        offset += chunk.len();
    }
    updater.update(&mut flash).await.unwrap();
    led.set_low();
    cortex_m::peripheral::SCB::sys_reset();
}
