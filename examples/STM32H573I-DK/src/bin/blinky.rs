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
    info!("Hello STM32H573I-DK!");

    // User LD1 (green, PI9) — active-low.
    let mut led = Output::new(p.PI9, Level::High, Speed::Low);

    loop {
        info!("on");
        led.set_low();
        Timer::after_millis(500).await;

        info!("off");
        led.set_high();
        Timer::after_millis(500).await;
    }
}
