//! Blinky on the EFM32GG STK3700 with reset-default clocks (HFRCO 14 MHz).
//! LED0 is on PE2 and LED1 is on PE3. The time driver is the RTC on LFRCO.
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

    let mut led0 = Output::new(p.PE2, Level::Low, Speed::Medium);
    let mut led1 = Output::new(p.PE3, Level::High, Speed::Medium);

    loop {
        info!("blink");
        led0.toggle();
        led1.toggle();
        Timer::after_millis(500).await;
    }
}
