#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;
use embassy_stm32::gpio::{Input, Pull};
use example_common::*;

#[cortex_m_rt::entry]
fn main() -> ! {
    info!("Hello World!");

    let p = embassy_stm32::init(Default::default());

    let button = Input::new(p.PC13, Pull::Up);

    loop {
        if button.is_high() {
            info!("high");
        } else {
            info!("low");
        }
    }
}
