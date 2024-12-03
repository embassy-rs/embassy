#![no_std]
#![no_main]
#![macro_use]

use embassy_executor::Spawner;
use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_time::Timer;
use panic_reset as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    let mut led = Output::new(p.P0_13, Level::Low, OutputDrive::Standard);
    // let mut led = Output::new(p.P1_10, Level::Low, OutputDrive::Standard);

    // nRF91 DK
    // let mut led = Output::new(p.P0_02, Level::Low, OutputDrive::Standard);

    loop {
        led.set_high();
        Timer::after_millis(300).await;
        led.set_low();
        Timer::after_millis(300).await;
    }
}
