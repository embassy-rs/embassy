//! Blinky on the Bosch XDK110 (EFM32GG390F1024) with reset-default clocks.
//!
//! HFRCO runs at 14 MHz, and the RTC time driver runs on LFRCO.
//!
//! Connect an LED with a resistor from PA1 (extension-bus pin A12) to GND. No public document gives
//! the on-board LED pins (see `README.md`).
#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_silabs::gpio::{Level, Output, Speed};
use embassy_time::Timer;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_silabs::init(embassy_silabs::Config::default());
    info!("Hello World!");

    let mut led = Output::new(p.PA1, Level::Low, Speed::Medium);

    loop {
        info!("blink");
        led.toggle();
        Timer::after_millis(500).await;
    }
}
