#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![allow(incomplete_features)]

#[path = "../example_common.rs"]
mod example_common;

use defmt::panic;
use embassy::executor::Spawner;
use embassy::traits::gpio::{WaitForHigh, WaitForLow};
use embassy_nrf::gpio::{AnyPin, Input, Pin as _, Pull};
use embassy_nrf::gpiote::PortInput;
use embassy_nrf::interrupt;
use embassy_nrf::Peripherals;
use example_common::*;

#[embassy::task(pool_size = 4)]
async fn button_task(n: usize, mut pin: PortInput<'static, AnyPin>) {
    loop {
        pin.wait_for_low().await;
        info!("Button {:?} pressed!", n);
        pin.wait_for_high().await;
        info!("Button {:?} released!", n);
    }
}

#[embassy::main]
async fn main(spawner: Spawner, p: Peripherals) {
    info!("Starting!");

    let btn1 = PortInput::new(Input::new(p.P0_11.degrade(), Pull::Up));
    let btn2 = PortInput::new(Input::new(p.P0_12.degrade(), Pull::Up));
    let btn3 = PortInput::new(Input::new(p.P0_24.degrade(), Pull::Up));
    let btn4 = PortInput::new(Input::new(p.P0_25.degrade(), Pull::Up));

    spawner.spawn(button_task(1, btn1)).unwrap();
    spawner.spawn(button_task(2, btn2)).unwrap();
    spawner.spawn(button_task(3, btn3)).unwrap();
    spawner.spawn(button_task(4, btn4)).unwrap();
}
