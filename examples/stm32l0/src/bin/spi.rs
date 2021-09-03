#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;

use embassy::executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embedded_hal::digital::v2::OutputPin;
use example_common::*;

use embassy_stm32::dma::NoDma;
use embassy_stm32::spi::{Config, Spi};
use embassy_stm32::time::Hertz;
use embassy_stm32::Peripherals;
use embedded_hal::blocking::spi::Transfer;

#[embassy::main]
async fn main(_spawner: Spawner, p: Peripherals) {
    info!("Hello World, folks!");

    let mut spi = Spi::new(
        p.SPI1,
        p.PB3,
        p.PA7,
        p.PA6,
        NoDma,
        NoDma,
        Hertz(1_000_000),
        Config::default(),
    );

    let mut cs = Output::new(p.PA15, Level::High, Speed::VeryHigh);

    loop {
        let mut buf = [0x0Au8; 4];
        unwrap!(cs.set_low());
        unwrap!(spi.transfer(&mut buf));
        unwrap!(cs.set_high());
        info!("xfer {=[u8]:x}", buf);
    }
}
