#![no_std]
#![no_main]
#![feature(trait_alias)]
#![feature(min_type_alias_impl_trait)]
#![feature(impl_trait_in_bindings)]
#![feature(type_alias_impl_trait)]
#![allow(incomplete_features)]

#[path = "../example_common.rs"]
mod example_common;
use defmt::panic;
use embassy::executor::Spawner;
use embassy_stm32::gpio::{Input, Pull};
use embassy_stm32::{pac, Peripherals};
use embedded_hal::digital::v2::InputPin;
use example_common::*;

#[embassy::main]
async fn main(_spawner: Spawner, p: Peripherals) {
    info!("Hello World!");

    unsafe {
        pac::DBGMCU.cr().modify(|w| {
            w.set_dbg_sleep(true);
            w.set_dbg_standby(true);
            w.set_dbg_stop(true);
        });
    }

    let button = Input::new(p.PC13, Pull::Up);

    loop {
        if button.is_high().unwrap() {
            info!("high");
        } else {
            info!("low");
        }
    }
}
