#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    let mut led = Output::new(p.P1_22, Level::Low, OutputDrive::HighDrive);

    loop {
        info!("high!");
        led.set_high();
        Timer::after_millis(300).await;
        info!("low!");
        led.set_low();
        Timer::after_millis(300).await;
    }
}
