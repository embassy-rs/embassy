#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy::executor::Spawner;
use embassy_nrf::interrupt;
use embassy_nrf::rng::Rng;
use embassy_nrf::Peripherals;
use rand::Rng as _;

use defmt_rtt as _; // global logger
use panic_probe as _;

#[embassy::main]
async fn main(_spawner: Spawner, p: Peripherals) {
    let mut rng = Rng::new(p.RNG, interrupt::take!(RNG));

    // Async API
    let mut bytes = [0; 4];
    rng.fill_bytes(&mut bytes).await;
    defmt::info!("Some random bytes: {:?}", bytes);

    // Sync API with `rand`
    defmt::info!("A random number from 1 to 10: {:?}", rng.gen_range(1..=10));

    let mut bytes = [0; 1024];
    rng.fill_bytes(&mut bytes).await;
    let zero_count: u32 = bytes.iter().fold(0, |acc, val| acc + val.count_zeros());
    let one_count: u32 = bytes.iter().fold(0, |acc, val| acc + val.count_ones());
    defmt::info!(
        "Chance of zero: {}%",
        zero_count * 100 / (bytes.len() as u32 * 8)
    );
    defmt::info!(
        "Chance of one: {}%",
        one_count * 100 / (bytes.len() as u32 * 8)
    );
}
