#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_nrf::gpio::{Input, Pull};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    let btn = Input::new(p.P1_26, Pull::Up);

    loop {
        if btn.is_high() {
            info!("high");
        } else {
            info!("low");
        }
        Timer::after_millis(100).await;
    }
}
