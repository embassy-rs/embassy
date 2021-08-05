#![no_std]
#![no_main]
#![feature(trait_alias)]
#![feature(type_alias_impl_trait)]
#![allow(incomplete_features)]

#[path = "../example_common.rs"]
mod example_common;

use embassy::executor::Spawner;
use embassy::time::{Duration, Timer};
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::rcc::Rcc;
use embassy_stm32::Peripherals;
use embedded_hal::digital::v2::OutputPin;
use example_common::*;

#[embassy::main]
async fn main(_spawner: Spawner, mut p: Peripherals) {
    info!("Hello World!");

    Rcc::new(p.RCC).enable_debug_wfe(&mut p.DBGMCU, true);

    let mut led = Output::new(p.PB5, Level::High, Speed::Low);

    loop {
        info!("high");
        unwrap!(led.set_high());
        Timer::after(Duration::from_millis(300)).await;

        info!("low");
        unwrap!(led.set_low());
        Timer::after(Duration::from_millis(300)).await;
    }
}
