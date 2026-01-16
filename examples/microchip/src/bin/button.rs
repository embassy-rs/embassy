#![no_main]
#![no_std]

use defmt::info;
use embassy_executor::Spawner;
use embassy_microchip::gpio::{Input, Pull};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_microchip::init(Default::default());

    info!("Hello, world!");
    let btn = Input::new(p.GPIO141, Pull::None);

    loop {
        info!("Button State: {}", btn.get_level());
        Timer::after_secs(1).await;
    }
}
