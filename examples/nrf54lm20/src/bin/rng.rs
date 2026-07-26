#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_nrf::cracen::Cracen;
use rand::Rng as _;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    let mut rng = Cracen::new_blocking(p.CRACEN);

    // Async API
    let mut bytes = [0; 4];
    rng.blocking_fill_bytes(&mut bytes);
    defmt::info!("Some random bytes: {:?}", bytes);

    // Sync API with `rand`
    defmt::info!("A random number from 1 to 10: {:?}", rng.random_range(1..=10));

    let mut bytes = [0; 1024];
    rng.blocking_fill_bytes(&mut bytes);
    let zero_count: u32 = bytes.iter().fold(0, |acc, val| acc + val.count_zeros());
    let one_count: u32 = bytes.iter().fold(0, |acc, val| acc + val.count_ones());
    defmt::info!("Chance of zero: {}%", zero_count * 100 / (bytes.len() as u32 * 8));
    defmt::info!("Chance of one: {}%", one_count * 100 / (bytes.len() as u32 * 8));
}
