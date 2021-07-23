#![no_std]
#![no_main]
#![feature(trait_alias)]
#![feature(min_type_alias_impl_trait)]
#![feature(impl_trait_in_bindings)]
#![feature(type_alias_impl_trait)]
#![allow(incomplete_features)]

#[path = "../example_common.rs"]
mod example_common;
use cortex_m_rt::entry;
use embassy_stm32::dbgmcu::Dbgmcu;
use embassy_stm32::gpio::{Input, Level, Output, Pull, Speed};
use embedded_hal::digital::v2::{InputPin, OutputPin};
use example_common::*;

#[entry]
fn main() -> ! {
    info!("Hello World!");

    unsafe {
        Dbgmcu::enable_all();
    }

    let p = embassy_stm32::init(Default::default());

    let button = Input::new(p.PC13, Pull::Down);
    let mut led1 = Output::new(p.PB0, Level::High, Speed::Low);
    let _led2 = Output::new(p.PB7, Level::High, Speed::Low);
    let mut led3 = Output::new(p.PB14, Level::High, Speed::Low);

    loop {
        if button.is_high().unwrap() {
            info!("high");
            led1.set_high().unwrap();
            led3.set_low().unwrap();
        } else {
            info!("low");
            led1.set_low().unwrap();
            led3.set_high().unwrap();
        }
    }
}
