#![no_std]
#![no_main]
#![feature(asm)]
#![feature(min_type_alias_impl_trait)]
#![feature(impl_trait_in_bindings)]
#![feature(type_alias_impl_trait)]
#![allow(incomplete_features)]

#[path = "../example_common.rs"]
mod example_common;

use defmt::*;
use embassy::executor::Spawner;
use embassy_rp::gpio::{Input, Pull};
use embassy_rp::Peripherals;
use embedded_hal::digital::v2::InputPin;

#[embassy::main]
async fn main(_spawner: Spawner) {
    let p = unwrap!(Peripherals::take());

    let button = Input::new(p.PIN_28, Pull::Up);

    loop {
        info!("high? {=bool}", button.is_high().unwrap());
        cortex_m::asm::delay(1_000_000);
    }
}
