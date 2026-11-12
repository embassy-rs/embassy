//! Blink a GPIO at 1 Hz.
//!
//! Wire an LED (with a series resistor) to the chosen pin, or change the pin
//! below to match your board. This example also exercises the LPTIM0 time
//! driver.

#![no_std]
#![no_main]

use embassy_asr::gpio::{Level, Output};
use embassy_executor::Spawner;
use embassy_time::Timer;
use panic_halt as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_asr::init(Default::default());
    let mut led = Output::new(p.PA0, Level::Low);

    loop {
        led.toggle();
        Timer::after_millis(500).await;
    }
}
