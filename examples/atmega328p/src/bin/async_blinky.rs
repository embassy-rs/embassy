#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

use embassy_atmega328p::gpio::{Level, Output};
use embassy_executor::Spawner;
use embassy_time::Timer;
use panic_halt as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_atmega328p::init();
    let mut led = Output::new(p.PB5, Level::Low);

    loop {
        led.toggle();
        Timer::after_millis(500).await;
    }
}
