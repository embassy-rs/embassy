#![no_std]
#![no_main]
#![feature(min_type_alias_impl_trait)]
#![feature(impl_trait_in_bindings)]
#![feature(type_alias_impl_trait)]
#![allow(incomplete_features)]

#[path = "../example_common.rs"]
mod example_common;

use core::pin::Pin;
use defmt::panic;
use embassy::executor::Spawner;
use embassy::traits::gpio::{WaitForHigh, WaitForLow};
use embassy_nrf::gpio::{AnyPin, Input, Pin as _, Pull};
use embassy_nrf::gpiote::{self, PortInput};
use embassy_nrf::interrupt;
use embassy_nrf::Peripherals;
use example_common::*;

async fn button(n: usize, mut pin: PortInput<'static, AnyPin>) {
    loop {
        Pin::new(&mut pin).wait_for_low().await;
        info!("Button {:?} pressed!", n);
        Pin::new(&mut pin).wait_for_high().await;
        info!("Button {:?} released!", n);
    }
}

#[embassy::main]
async fn main(spawner: Spawner) {
    let p = Peripherals::take().unwrap();

    let g = gpiote::initialize(p.GPIOTE, interrupt::take!(GPIOTE));

    let button1 = button(
        1,
        PortInput::new(g, Input::new(p.P0_11.degrade(), Pull::Up)),
    );
    let button2 = button(
        2,
        PortInput::new(g, Input::new(p.P0_12.degrade(), Pull::Up)),
    );
    let button3 = button(
        3,
        PortInput::new(g, Input::new(p.P0_24.degrade(), Pull::Up)),
    );
    let button4 = button(
        4,
        PortInput::new(g, Input::new(p.P0_25.degrade(), Pull::Up)),
    );
    futures::join!(button1, button2, button3, button4);
}
