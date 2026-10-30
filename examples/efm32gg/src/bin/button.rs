//! Toggles an LED on each button press, on the Bosch XDK110 (EFM32GG390F1024).
//!
//! Connect a button from PE2 (extension-bus pin A13) to GND. The input has a pull-up. Connect an
//! LED with a resistor from PA1 (A12) to GND. No public document gives the on-board button pins.
#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_silabs::gpio::{Input, Level, Output, Pull, Speed};
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_silabs::init(embassy_silabs::Config::default());
    info!("Hello World!");

    let mut button = Input::new(p.PE2, Pull::Up);
    let mut led = Output::new(p.PA1, Level::Low, Speed::Medium);

    loop {
        button.wait_for_falling_edge().await;
        info!("pressed");
        led.toggle();
        button.wait_for_high().await;
    }
}
