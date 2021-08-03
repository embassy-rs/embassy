#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![allow(incomplete_features)]

#[path = "../example_common.rs"]
mod example_common;

use embassy::executor::Spawner;
use embassy::time::{Duration, Timer};
use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_nrf::Peripherals;
use embedded_hal::digital::v2::OutputPin;

#[embassy::main]
async fn main(_spawner: Spawner, p: Peripherals) {
    let mut led = Output::new(p.P0_13, Level::Low, OutputDrive::Standard);

    loop {
        led.set_high().unwrap();
        Timer::after(Duration::from_millis(300)).await;
        led.set_low().unwrap();
        Timer::after(Duration::from_millis(300)).await;
    }
}
