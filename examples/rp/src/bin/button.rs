#![no_std]
#![no_main]
#![feature(asm)]
#![feature(min_type_alias_impl_trait)]
#![feature(impl_trait_in_bindings)]
#![feature(type_alias_impl_trait)]
#![allow(incomplete_features)]

#[path = "../example_common.rs"]
mod example_common;

use embassy::executor::Spawner;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::Peripherals;
use embedded_hal::digital::v2::{InputPin, OutputPin};

#[embassy::main]
async fn main(_spawner: Spawner, p: Peripherals) {
    let button = Input::new(p.PIN_28, Pull::Up);
    let mut led = Output::new(p.PIN_25, Level::Low);

    loop {
        if button.is_high() {
            led.set_high();
        } else {
            led.set_low();
        }
    }
}
