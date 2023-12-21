#![no_std]
#![no_main]

use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

mod config {
    pub const MY_TASK_POOL_SIZE: usize = 2;
}

#[embassy_executor::task(pool_size = config::MY_TASK_POOL_SIZE)]
async fn my_task(spawner: Spawner, n: u32) {
    Timer::after_secs(1).await;
    info!("Spawning self! {}", n);
    unwrap!(spawner.spawn(my_task(spawner, n + 1)));
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let _p = embassy_nrf::init(Default::default());
    info!("Hello World!");
    unwrap!(spawner.spawn(my_task(spawner, 0)));
}
