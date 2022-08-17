#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy_executor::executor::Spawner;
use embassy_executor::time::{Duration, Timer};
use embassy_nrf::gpio::{Level, Output, OutputDrive};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    let mut led = Output::new(p.P0_13, Level::Low, OutputDrive::Standard);

    loop {
        led.set_high();
        Timer::after(Duration::from_millis(300)).await;
        led.set_low();
        Timer::after(Duration::from_millis(300)).await;
    }
}
