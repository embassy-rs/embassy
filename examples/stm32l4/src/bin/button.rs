#![no_std]
#![no_main]
#![feature(trait_alias)]
#![feature(type_alias_impl_trait)]
#![allow(incomplete_features)]

#[path = "../example_common.rs"]
mod example_common;
use embassy_stm32::dbgmcu::Dbgmcu;
use embassy_stm32::gpio::{Input, Pull};
use embedded_hal::digital::v2::InputPin;
use example_common::*;

#[cortex_m_rt::entry]
fn main() -> ! {
    info!("Hello World!");

    unsafe {
        Dbgmcu::enable_all();
    }

    let p = embassy_stm32::init(Default::default());

    let button = Input::new(p.PC13, Pull::Up);

    loop {
        if unwrap!(button.is_high()) {
            info!("high");
        } else {
            info!("low");
        }
    }
}
