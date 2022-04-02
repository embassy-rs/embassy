#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt_rtt as _; // global logger
use panic_probe as _;

use defmt::*;
use embassy::executor::Spawner;
use embassy_stm32::dma::NoDma;
use embassy_stm32::gpio::{Input, Level, Output, Pull, Speed};
use embassy_stm32::spi::{Config, Spi};
use embassy_stm32::time::Hertz;
use embassy_stm32::Peripherals;
use embassy_traits::adapter::BlockingAsync;
use embedded_hal_async::spi::SpiBus;

#[embassy::main]
async fn main(_spawner: Spawner, p: Peripherals) {
    info!("Hello World!");

    let spi = Spi::new(
        p.SPI3,
        p.PC10,
        p.PC12,
        p.PC11,
        NoDma,
        NoDma,
        Hertz(1_000_000),
        Config::default(),
    );

    let mut spi = BlockingAsync::new(spi);

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

    let write: [u8; 10] = [0x0A; 10];
    let mut read: [u8; 10] = [0; 10];
    cs.set_low();
    spi.transfer(&mut read, &write).await.ok();
    cs.set_high();
    info!("xfer {=[u8]:x}", read);
}
