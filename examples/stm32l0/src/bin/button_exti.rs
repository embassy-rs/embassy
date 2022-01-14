#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;

use embassy::executor::Spawner;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{Input, Pull};
use embassy_stm32::Peripherals;
use example_common::*;

fn config() -> embassy_stm32::Config {
    let mut config = embassy_stm32::Config::default();
    config.rcc.enable_hsi48 = true;
    config
}

#[embassy::main(config = "config()")]
async fn main(_spawner: Spawner, p: Peripherals) {
    let button = Input::new(p.PB2, Pull::Up);
    let mut button = ExtiInput::new(button, p.EXTI2);

    info!("Press the USER button...");

    loop {
        button.wait_for_falling_edge().await;
        info!("Pressed!");
        button.wait_for_rising_edge().await;
        info!("Released!");
    }
}
