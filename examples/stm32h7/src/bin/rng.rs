#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;
use embassy::executor::Spawner;
use embassy::time::{Duration, Timer};
use embassy::traits::rng::Random;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::rng::Rng;
use embassy_stm32::Peripherals;
use example_common::*;

#[embassy::main]
async fn main(_spawner: Spawner, p: Peripherals) {
    info!("Hello World!");

    let mut led = Output::new(p.PB14, Level::High, Speed::Low);

    let mut rng = Random::new(Rng::new(p.RNG));

    loop {
        info!("high {}", unwrap!(rng.next_u8(16).await));
        led.set_high();
        Timer::after(Duration::from_millis(500)).await;

        info!("low {}", unwrap!(rng.next_u8(16).await));
        led.set_low();
        Timer::after(Duration::from_millis(500)).await;
    }
}
