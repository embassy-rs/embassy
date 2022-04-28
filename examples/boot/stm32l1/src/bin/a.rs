#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy::time::{Duration, Timer};
use embassy_boot_stm32::FirmwareUpdater;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::flash::Flash;
use embassy_stm32::gpio::{Input, Level, Output, Pull, Speed};
use embassy_stm32::Peripherals;
use embassy_traits::adapter::BlockingAsync;
use panic_reset as _;

#[cfg(feature = "defmt-rtt")]
use defmt_rtt::*;

static APP_B: &[u8] = include_bytes!("../../b.bin");

#[embassy::main]
async fn main(_s: embassy::executor::Spawner, p: Peripherals) {
    let flash = Flash::unlock(p.FLASH);
    let mut flash = BlockingAsync::new(flash);

    let button = Input::new(p.PB2, Pull::Up);
    let mut button = ExtiInput::new(button, p.EXTI2);

    let mut led = Output::new(p.PB5, Level::Low, Speed::Low);

    led.set_high();

    let mut updater = FirmwareUpdater::default();
    button.wait_for_falling_edge().await;
    let mut offset = 0;
    for chunk in APP_B.chunks(128) {
        let mut buf: [u8; 128] = [0; 128];
        buf[..chunk.len()].copy_from_slice(chunk);
        updater
            .write_firmware(offset, &buf, &mut flash, 128)
            .await
            .unwrap();
        offset += chunk.len();
    }

    updater.update(&mut flash).await.unwrap();
    led.set_low();
    Timer::after(Duration::from_secs(1)).await;
    cortex_m::peripheral::SCB::sys_reset();
}
