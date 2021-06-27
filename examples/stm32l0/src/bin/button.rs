#![no_std]
#![no_main]
#![feature(trait_alias)]
#![feature(min_type_alias_impl_trait)]
#![feature(impl_trait_in_bindings)]
#![feature(type_alias_impl_trait)]
#![allow(incomplete_features)]

#[path = "../example_common.rs"]
mod example_common;
use embassy_stm32::{
    gpio::{Input, Level, Output, Pull, Speed},
    rcc::*,
};
use embedded_hal::digital::v2::{InputPin, OutputPin};
use example_common::*;

use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    info!("Hello World!");

    let mut p = embassy_stm32::init(Default::default());
    Rcc::new(p.RCC).enable_debug_wfe(&mut p.DBGMCU, true);

    let button = Input::new(p.PB2, Pull::Up);
    let mut led1 = Output::new(p.PA5, Level::High, Speed::Low);
    let mut led2 = Output::new(p.PB5, Level::High, Speed::Low);

    loop {
        if button.is_high().unwrap() {
            info!("high");
            led1.set_high().unwrap();
            led2.set_low().unwrap();
        } else {
            info!("low");
            led1.set_low().unwrap();
            led2.set_high().unwrap();
        }
    }
}
