#![no_std]
#![no_main]

use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::task]
async fn run1() {
    loop {
        info!("BIG INFREQUENT TICK");
        Timer::after_ticks(64000).await;
    }
}

#[embassy_executor::task]
async fn run2() {
    loop {
        info!("tick");
        Timer::after_ticks(13000).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let _p = embassy_nrf::init(Default::default());
    unwrap!(spawner.spawn(run1()));
    unwrap!(spawner.spawn(run2()));
}
