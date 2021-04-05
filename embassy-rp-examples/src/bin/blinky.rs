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
use embassy_rp::{gpio, Peripherals};
use embedded_hal::digital::v2::OutputPin;
use gpio::{Level, Output};

#[embassy::main]
async fn main(_spawner: Spawner) {
    let p = unwrap!(Peripherals::take());

    let mut led = Output::new(p.PIN_25, Level::Low);

    loop {
        info!("led on!");
        led.set_high().unwrap();
        cortex_m::asm::delay(1_000_000);

        info!("led off!");
        led.set_low().unwrap();
        cortex_m::asm::delay(1_000_000);
    }
}
