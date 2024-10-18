#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let mut led_green = Output::new(p.PG13, Level::High, Speed::Low);
    let mut led_red = Output::new(p.PG14, Level::High, Speed::Low);

    loop {
        info!("high");
        led_green.set_high();
        led_red.set_high();
        Timer::after_millis(300).await;

        info!("low");
        led_green.set_low();
        led_red.set_low();
        Timer::after_millis(300).await;
    }
}
