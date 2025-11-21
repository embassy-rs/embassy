#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::Timer;
use hal::gpio::{DriveStrength, Input, Pull, SlewRate};
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = hal::init(hal::config::Config::default());

    defmt::info!("Button example");

    let monitor = Input::new(p.P1_7, Pull::Disabled, DriveStrength::Normal, SlewRate::Slow);

    loop {
        defmt::info!("Pin level is {:?}", monitor.get_level());
        Timer::after_millis(1000).await;
    }
}
