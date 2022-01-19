#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;
use embassy::executor::Spawner;
use embassy_stm32::gpio::{Input, Level, Output, Pull, Speed};
use embassy_stm32::Peripherals;
use example_common::*;

#[embassy::main]
async fn main(_spawner: Spawner, p: Peripherals) {
    info!("Hello World!");

    let button = Input::new(p.PB2, Pull::Up);
    let mut led1 = Output::new(p.PA5, Level::High, Speed::Low);
    let mut led2 = Output::new(p.PB5, Level::High, Speed::Low);

    loop {
        if button.is_high() {
            info!("high");
            led1.set_high();
            led2.set_low();
        } else {
            info!("low");
            led1.set_low();
            led2.set_high();
        }
    }
}
