#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_time::Timer;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello from STM32WLE5CC!");

    // Common user LED pin on STM32WLE5 boards (e.g., Seeed LoRa-E5 mini is PB5)
    let mut led = Output::new(p.PB5, Level::High, Speed::Low);

    loop {
        info!("LED on");
        led.set_low();
        Timer::after_millis(300).await;

        info!("LED off");
        led.set_high();
        Timer::after_millis(300).await;
    }
}
