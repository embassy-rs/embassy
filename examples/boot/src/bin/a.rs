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
use embedded_hal::digital::v2::InputPin;
use panic_reset as _;

static APP_B: &[u8] = include_bytes!("../../b.bin");

#[embassy::main]
async fn main(_s: embassy::executor::Spawner, p: Peripherals) {
    let mut button = Input::new(p.P0_13, Pull::Up);
    let mut nvmc = Nvmc::new(p.NVMC);
    let mut led = Output::new(p.P0_17, Level::Low, OutputDrive::Standard);
    led.set_low();

    loop {
        button.wait_for_any_edge().await;
        if button.is_low().unwrap() {
            let mut updater = FirmwareUpdater::new();
            let mut offset = 0;
            for chunk in APP_B.chunks(4096) {
                let mut buf: [u8; 4096] = [0; 4096];
                buf[..chunk.len()].copy_from_slice(chunk);
                updater.write_firmware(offset, &buf, &mut nvmc).unwrap();
                offset += chunk.len();
            }
            updater.mark_update(&mut nvmc).unwrap();
            led.set_high();
            updater.reset();
        }
    }
}
