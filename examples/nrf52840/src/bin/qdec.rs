#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_nrf::qdec::{self, Qdec};
use embassy_nrf::{bind_interrupts, peripherals};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    QDEC => qdec::InterruptHandler<peripherals::QDEC>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    let config = qdec::Config::default();
    let mut rotary_enc = Qdec::new(p.QDEC, Irqs, p.P0_31, p.P0_30, config);

    info!("Turn rotary encoder!");
    let mut value = 0;
    loop {
        value += rotary_enc.read().await;
        info!("Value: {}", value);
    }
}
