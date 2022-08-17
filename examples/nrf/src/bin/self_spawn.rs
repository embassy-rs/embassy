#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::{info, unwrap};
use embassy_executor::executor::Spawner;
use embassy_executor::time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::task(pool_size = 2)]
async fn my_task(spawner: Spawner, n: u32) {
    Timer::after(Duration::from_secs(1)).await;
    info!("Spawning self! {}", n);
    unwrap!(spawner.spawn(my_task(spawner, n + 1)));
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let _p = embassy_nrf::init(Default::default());
    info!("Hello World!");
    unwrap!(spawner.spawn(my_task(spawner, 0)));
}
