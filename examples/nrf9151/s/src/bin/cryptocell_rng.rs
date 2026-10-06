#![no_std]
#![no_main]

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_nrf::bind_interrupts;
use embassy_nrf::crypto::rng::{self, Rng};
use panic_probe as _;
use rand::Rng as _;

bind_interrupts!(struct Irqs {
    CRYPTOCELL => rng::InterruptHandler;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());

    let mut rng = Rng::new(p.CRYPTO_RNG, Irqs);

    // Async API
    let mut bytes = [0; 4];
    rng.fill_bytes(&mut bytes).await;
    defmt::info!("Some random bytes: {:?}", bytes);

    // Sync API with `rand`
    defmt::info!("A random number from 1 to 10: {:?}", rng.random_range(1..=10));

    let mut bytes = [0; 1024];
    rng.fill_bytes(&mut bytes).await;
    let zero_count: u32 = bytes.iter().fold(0, |acc, val| acc + val.count_zeros());
    let one_count: u32 = bytes.iter().fold(0, |acc, val| acc + val.count_ones());
    defmt::info!("Chance of zero: {}%", zero_count * 100 / (bytes.len() as u32 * 8));
    defmt::info!("Chance of one: {}%", one_count * 100 / (bytes.len() as u32 * 8));
}
