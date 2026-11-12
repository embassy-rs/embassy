#![no_std]
#![no_main]

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_mcxn as hal;
use hal::gpio::{DriveStrength, Level, Output, SlewRate};
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = hal::init(hal::config::Config::default());

    defmt::info!("Blink example");

    let mut red = Output::new(p.P0_10, Level::High, DriveStrength::Normal, SlewRate::Fast);
    let mut green = Output::new(p.P0_27, Level::High, DriveStrength::Normal, SlewRate::Fast);
    let mut blue = Output::new(p.P1_2, Level::High, DriveStrength::Normal, SlewRate::Fast);

    loop {
        defmt::info!("toggle red");
        red.toggle();
        cortex_m::asm::delay(24_000_000);

        defmt::info!("toggle green");
        green.toggle();
        cortex_m::asm::delay(24_000_000);

        defmt::info!("toggle blue");
        blue.toggle();
        cortex_m::asm::delay(24_000_000);
    }
}
