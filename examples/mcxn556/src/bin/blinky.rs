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

    defmt::info!("MCXN556 CPU0 FRO12 blinky");

    let mut red = Output::new(p.P0_10, Level::High, DriveStrength::Normal, SlewRate::Fast);
    let mut green = Output::new(p.P0_27, Level::High, DriveStrength::Normal, SlewRate::Fast);
    let mut blue = Output::new(p.P1_2, Level::High, DriveStrength::Normal, SlewRate::Fast);

    loop {
        defmt::info!("Toggle LEDs");
        red.toggle();
        Timer::after_millis(500).await;
        red.set_high();
        green.set_low();
        Timer::after_millis(500).await;
        green.set_high();
        blue.set_low();
        Timer::after_millis(500).await;
        blue.set_high();
        Timer::after_millis(500).await;
    }
}
