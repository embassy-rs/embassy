#![no_std]
#![no_main]

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_mcxa as hal;
use embassy_time::Timer;
use hal::gpio::{Input, Pull};
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = hal::init(hal::config::Config::default());

    defmt::info!("Button example");

    // This button is labeled "WAKEUP" on the FRDM-MCXA276
    // The board already has a 10K pullup
    let monitor = Input::new(p.P1_7, Pull::Disabled);

    loop {
        defmt::info!("Pin level is {:?}", monitor.get_level());
        Timer::after_millis(1000).await;
    }
}
