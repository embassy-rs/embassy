#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;

use embassy::executor::Spawner;
use embassy::time::{Duration, Timer};
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::Peripherals;
use example_common::*;

#[embassy::main]
async fn main(_spawner: Spawner, p: Peripherals) {
    info!("Hello World!");

    let mut led = Output::new(p.PA12, Level::High, Speed::Low);

    loop {
        info!("high");
        led.set_high();
        Timer::after(Duration::from_millis(1000)).await;

        info!("low");
        led.set_low();
        Timer::after(Duration::from_millis(1000)).await;
    }
}
