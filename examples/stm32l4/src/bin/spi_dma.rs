#![no_std]
#![no_main]
#![feature(trait_alias)]
#![feature(min_type_alias_impl_trait)]
#![feature(impl_trait_in_bindings)]
#![feature(type_alias_impl_trait)]
#![allow(incomplete_features)]

#[path = "../example_common.rs"]
mod example_common;

use defmt::panic;
use embassy::executor::Spawner;
use embassy_stm32::gpio::{Input, Level, Output, Pull, Speed};
use embassy_stm32::spi::{Config, Spi};
use embassy_stm32::time::Hertz;
use embassy_stm32::{pac, Peripherals};
use embassy_traits::spi::FullDuplex;
use embedded_hal::digital::v2::{InputPin, OutputPin};
use example_common::*;

#[embassy::main]
async fn main(_spawner: Spawner, p: Peripherals) {
    info!("Hello World!");

    unsafe {
        pac::DBGMCU.cr().modify(|w| {
            w.set_dbg_sleep(true);
            w.set_dbg_standby(true);
            w.set_dbg_stop(true);
        });
    }

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
    reset.set_high().unwrap();
    cortex_m::asm::delay(100_000);

    while ready.is_low().unwrap() {
        info!("waiting for ready");
    }

    let write = [0x0A; 10];
    let mut read = [0; 10];
    unwrap!(cs.set_low());
    spi.read_write(&mut read, &write).await.ok();
    unwrap!(cs.set_high());
    info!("xfer {=[u8]:x}", read);
}
