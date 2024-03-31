#![no_std]
#![no_main]

use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::task(pool_size = 2)]
async fn my_task(n: u32) {
    Timer::after_secs(1).await;
    info!("Spawning self! {}", n);
    unwrap!(Spawner::for_current_executor().await.spawn(my_task(n + 1)));
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let _p = embassy_nrf::init(Default::default());
    info!("Hello World!");
    unwrap!(spawner.spawn(my_task(0)));
}
