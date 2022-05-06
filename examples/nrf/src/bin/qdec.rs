#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::info;
use embassy::executor::Spawner;
use embassy_nrf::{
    interrupt,
    qdec::{self, Qdec},
    Peripherals,
};

use defmt_rtt as _; // global logger
use panic_probe as _;

#[embassy::main]
async fn main(_spawner: Spawner, p: Peripherals) {
    let irq = interrupt::take!(QDEC);
    let config = qdec::Config::default();
    let mut rotary = Qdec::new(p.QDEC, irq, p.P1_13, p.P0_12, config);
    // let mut rotary = Qdec::new(p.QDEC, irq, p.P0_31, p.P0_30, config);

    info!("Turn rotary encoder!");
    let mut value = 0;
    loop {
        value += rotary.read().await;
        info!("Value: {}", value);
    }
}
