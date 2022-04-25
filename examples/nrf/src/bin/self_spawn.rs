#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::{info, unwrap};
use embassy::executor::Spawner;
use embassy::time::{Duration, Timer};
use embassy_nrf::Peripherals;

use defmt_rtt as _; // global logger
use panic_probe as _;

#[embassy::task(pool_size = 2)]
async fn my_task(spawner: Spawner, n: u32) {
    Timer::after(Duration::from_secs(1)).await;
    info!("Spawning self! {}", n);
    unwrap!(spawner.spawn(my_task(spawner, n + 1)));
}

#[embassy::main]
async fn main(spawner: Spawner, _p: Peripherals) {
    info!("Hello World!");
    unwrap!(spawner.spawn(my_task(spawner, 0)));
}
