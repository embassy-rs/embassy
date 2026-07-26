#![no_main]
#![no_std]

use defmt::info;
use embassy_executor::Spawner;
use embassy_microchip::gpio::{Input, Pull};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Hello, world!");
    let p = embassy_microchip::init(Default::default());
    let mut btn = Input::new(p.GPIO141, Pull::None);

    loop {
        if btn.is_high() {
            info!("Press button...");
        } else {
            info!("Release button...");
        }
        btn.wait_for_any_edge().await;
    }
}
