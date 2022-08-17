#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::info;
use embassy_executor::Spawner;
use embassy_nrf::interrupt;
use embassy_nrf::qdec::{self, Qdec};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    let irq = interrupt::take!(QDEC);
    let config = qdec::Config::default();
    let mut rotary_enc = Qdec::new(p.QDEC, irq, p.P0_31, p.P0_30, config);

    info!("Turn rotary encoder!");
    let mut value = 0;
    loop {
        value += rotary_enc.read().await;
        info!("Value: {}", value);
    }
}
