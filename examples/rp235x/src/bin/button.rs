//! This example uses the RP Pico on board LED to test input pin 28. This is not the button on the board.
//!
//! It does not work with the RP Pico W board. Use wifi_blinky.rs and add input pin.

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let mut led = Output::new(p.PIN_25, Level::Low);

    // Use PIN_28, Pin34 on J0 for RP Pico, as a input.
    // You need to add your own button.
    let button = Input::new(p.PIN_28, Pull::Up);

    loop {
        if button.is_high() {
            led.set_high();
        } else {
            led.set_low();
        }
    }
}
