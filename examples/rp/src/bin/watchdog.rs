//! This example shows how to use Watchdog in the RP2040 chip.
//!
//! It does not work with the RP Pico W board. See wifi_blinky.rs or connect external LED and resistor.

#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_rp::gpio;
use embassy_rp::watchdog::*;
use embassy_time::{Duration, Timer};
use gpio::{Level, Output};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    info!("Hello world!");

    let mut watchdog = Watchdog::new(p.WATCHDOG);
    let mut led = Output::new(p.PIN_25, Level::Low);

    // Set the LED high for 2 seconds so we know when we're about to start the watchdog
    led.set_high();
    Timer::after_secs(2).await;

    // Set to watchdog to reset if it's not fed within 1.05 seconds, and start it
    watchdog.start(Duration::from_millis(1_050));
    info!("Started the watchdog timer");

    // Blink once a second for 5 seconds, feed the watchdog timer once a second to avoid a reset
    for _ in 1..=5 {
        led.set_low();
        Timer::after_millis(500).await;
        led.set_high();
        Timer::after_millis(500).await;
        info!("Feeding watchdog");
        watchdog.feed();
    }

    info!("Stopped feeding, device will reset in 1.05 seconds");
    // Blink 10 times per second, not feeding the watchdog.
    // The processor should reset in 1.05 seconds.
    loop {
        led.set_low();
        Timer::after_millis(100).await;
        led.set_high();
        Timer::after_millis(100).await;
    }
}
