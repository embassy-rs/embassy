#![no_std]
#![no_main]

use embassy_executor::Spawner;
use hal::config::Config;
use hal::reset_reason::reset_reason;
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let config = Config::default();
    let _p = hal::init(config);

    defmt::info!("Reset Reason: '{}'", reset_reason());
}
