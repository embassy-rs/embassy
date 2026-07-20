#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let config = Config::default();
    // Fine-tune PLL1 dividers/multipliers
    // voltage scale for max performance
    // route PLL1_P into the USB‐OTG‐HS block
    let p = embassy_stm32::init(config);
    info!("Hello World!");

    // LD2 - PC4
    let mut led = Output::new(p.PC4, Level::High, Speed::Low);

    loop {
        info!("led on!");
        led.set_high();
        Timer::after_millis(500).await;

        info!("led off!");
        led.set_low();
        Timer::after_millis(500).await;
    }
}
