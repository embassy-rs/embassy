#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    let mut led = Output::new(p.P0_00, Level::Low, OutputDrive::Standard);

    loop {
        led.set_high();
        defmt::info!("high");
        Timer::after_millis(500).await;
        led.set_low();
        defmt::info!("low");
        Timer::after_millis(1000).await;
    }
}
