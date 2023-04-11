#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::rng::Rng;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let mut rng = Rng::new(p.RNG);

    let mut buf = [0u8; 16];
    unwrap!(rng.async_fill_bytes(&mut buf).await);
    info!("random bytes: {:02x}", buf);
}
