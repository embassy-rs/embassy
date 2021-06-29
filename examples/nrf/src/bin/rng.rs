#![no_std]
#![no_main]
#![feature(min_type_alias_impl_trait)]
#![feature(impl_trait_in_bindings)]
#![feature(type_alias_impl_trait)]
#![allow(incomplete_features)]

#[path = "../example_common.rs"]
mod example_common;

use defmt::panic;
use embassy::executor::Spawner;
use embassy_nrf::Peripherals;
use embassy_nrf::rng::Rng;
use embassy_nrf::interrupt;
use embassy::traits::rng::Rng as _;
use rand::Rng as _;

#[embassy::main]
async fn main(_spawner: Spawner, p: Peripherals) {
    let mut rng = Rng::new(p.RNG, interrupt::take!(RNG));

    // Async API
    let mut bytes = [0; 4];
    rng.fill_bytes(&mut bytes).await.unwrap(); // nRF RNG is infallible
    defmt::info!("Some random bytes: {:?}", bytes);

    // Sync API with `rand`
    defmt::info!("A random number from 1 to 10: {:?}", rng.gen_range(1..=10));
}
