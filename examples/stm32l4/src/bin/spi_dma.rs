#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;

use embassy::executor::Spawner;
use embassy_stm32::gpio::{Input, Level, Output, Pull, Speed};
use embassy_stm32::spi::{Config, Spi};
use embassy_stm32::time::Hertz;
use embassy_stm32::Peripherals;
use example_common::*;

#[embassy::main]
async fn main(_spawner: Spawner, p: Peripherals) {
    info!("Hello World!");

    let mut spi = Spi::new(
        p.SPI3,
        p.PC10,
        p.PC12,
        p.PC11,
        p.DMA1_CH0,
        p.DMA1_CH1,
        Hertz(1_000_000),
        Config::default(),
    );

    // These are the pins for the Inventek eS-Wifi SPI Wifi Adapter.

    let _boot = Output::new(p.PB12, Level::Low, Speed::VeryHigh);
    let _wake = Output::new(p.PB13, Level::Low, Speed::VeryHigh);
    let mut reset = Output::new(p.PE8, Level::Low, Speed::VeryHigh);
    let mut cs = Output::new(p.PE0, Level::High, Speed::VeryHigh);
    let ready = Input::new(p.PE1, Pull::Up);

    cortex_m::asm::delay(100_000);
    reset.set_high();
    cortex_m::asm::delay(100_000);

    while ready.is_low() {
        info!("waiting for ready");
    }

    let write = [0x0A; 10];
    let mut read = [0; 10];
    cs.set_low();
    spi.transfer(&mut read, &write).await.ok();
    cs.set_high();
    info!("xfer {=[u8]:x}", read);
}
