#![no_std]
#![no_main]

extern crate embassy_imxrt_examples;

use defmt::*;
use embassy_executor::Spawner;
use embassy_imxrt::rng::Rng;
use embassy_imxrt::{bind_interrupts, peripherals, rng};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    RNG => rng::InterruptHandler<peripherals::RNG>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_imxrt::init(Default::default());

    info!("Initializing RNG");
    let mut rng = Rng::new(p.RNG, Irqs);
    let mut buf = [0u8; 65];

    // Async interface
    unwrap!(rng.async_fill_bytes(&mut buf).await);
    info!("random bytes: {:02x}", buf);

    // RngCore interface
    let mut random_bytes = [0; 16];

    let random_u32 = rng.blocking_next_u32();
    let random_u64 = rng.blocking_next_u64();

    rng.blocking_fill_bytes(&mut random_bytes);

    info!("random_u32 {}", random_u32);
    info!("random_u64 {}", random_u64);
    info!("random_bytes {}", random_bytes);
}
