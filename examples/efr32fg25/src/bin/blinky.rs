#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_silabs::gpio::{Level, Output, Speed};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_silabs::init(embassy_silabs::Config::default());
    info!("Hello World!");

    let mut led = Output::new(p.PC06, Level::Low, Speed::Medium);

    loop {
        info!("blink");
        led.set_high();
        Timer::after_millis(100).await;

        led.set_low();
        Timer::after_millis(900).await;
    }
}
