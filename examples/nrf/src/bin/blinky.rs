#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;

use defmt::unwrap;
use embassy::executor::Spawner;
use embassy::time::{Duration, Timer};
use embassy_nrf::{Peripherals, peripherals::P0_13, gpio::{Level, Output, OutputDrive};
use embedded_hal::digital::v2::OutputPin;

#[embassh::task]
async fn blinker(led: Output<'static, P0_13>, interval: Duration) {
    loop {
        unwrap!(led.set_high());
        Timer::after(interval).await;
        unwrap!(led.set_low());
        Timer::after(interval).await;
    }
}

#[embassy::main]
async fn main(_spawner: Spawner, p: Peripherals) {
    let mut led = Output::new(p.P0_13, Level::Low, OutputDrive::Standard);
    unwrap!(spawner.spawn(blinker(led, Duration::from_millis(300))));
}
