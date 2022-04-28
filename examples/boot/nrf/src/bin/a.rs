#![no_std]
#![no_main]
#![macro_use]
#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]

use embassy_boot_nrf::FirmwareUpdater;
use embassy_nrf::{
    gpio::{Input, Pull},
    gpio::{Level, Output, OutputDrive},
    nvmc::Nvmc,
    Peripherals,
};
use embassy_traits::adapter::BlockingAsync;
use panic_reset as _;

static APP_B: &[u8] = include_bytes!("../../b.bin");

#[embassy::main]
async fn main(_s: embassy::executor::Spawner, p: Peripherals) {
    let mut button = Input::new(p.P0_11, Pull::Up);
    let mut led = Output::new(p.P0_13, Level::Low, OutputDrive::Standard);
    //let mut led = Output::new(p.P1_10, Level::Low, OutputDrive::Standard);
    //let mut button = Input::new(p.P1_02, Pull::Up);

    let nvmc = Nvmc::new(p.NVMC);
    let mut nvmc = BlockingAsync::new(nvmc);

    let mut updater = FirmwareUpdater::default();
    loop {
        led.set_low();
        button.wait_for_any_edge().await;
        if button.is_low() {
            let mut offset = 0;
            for chunk in APP_B.chunks(4096) {
                let mut buf: [u8; 4096] = [0; 4096];
                buf[..chunk.len()].copy_from_slice(chunk);
                updater
                    .write_firmware(offset, &buf, &mut nvmc, 4096)
                    .await
                    .unwrap();
                offset += chunk.len();
            }
            updater.update(&mut nvmc).await.unwrap();
            led.set_high();
            cortex_m::peripheral::SCB::sys_reset();
        }
    }
}
