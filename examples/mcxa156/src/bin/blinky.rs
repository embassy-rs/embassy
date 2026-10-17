#![no_std]
#![no_main]

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_mcxa as hal;
use embassy_time::Timer;
use hal::gpio::{DriveStrength, Level, Output, SlewRate};
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = hal::init(hal::config::Config::default());
    let mut green = Output::new(p.P3_13, Level::High, DriveStrength::Normal, SlewRate::Fast);

    loop {
        green.set_low();
        Timer::after_millis(500).await;
        green.set_high();
        Timer::after_millis(500).await;
    }
}
