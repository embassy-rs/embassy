#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;
use example_common::*;

use defmt::panic;
use embassy::{
    executor::Spawner,
    time::{Duration, Timer},
};
use embassy_nrf::{interrupt, temp::Temp, Peripherals};

#[embassy::main]
async fn main(_spawner: Spawner, p: Peripherals) {
    let irq = interrupt::take!(TEMP);
    let mut temp = Temp::new(p.TEMP, irq);

    loop {
        let value = temp.read().await;
        info!("temperature: {}℃", value.to_num::<u16>());
        Timer::after(Duration::from_secs(1)).await;
    }
}
