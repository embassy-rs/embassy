#![no_std]
#![no_main]

use embassy_ambiq::gpio::{Input, Output, Pull};
use embassy_executor::Spawner;
use embassy_time::Timer;
use panic_halt as _;

// SparkFun RedBoard Artemis
// STAT LED is on Pad5
// A0 is on Pad29
#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_ambiq::init();

    let mut led = Output::new(p.P5);
    let mut button = Input::new(p.P29, Pull::Up);

    let mut state = false;

    loop {
        // Asynchronously wait for the falling edge (button pressed)
        button.wait_for_falling_edge().await;

        // Toggle the LED
        state = !state;
        if state {
            led.set_high();
        } else {
            led.set_low();
        }

        // 50ms debounce so it doesn't flicker multiple times on a single press
        Timer::after_millis(50).await;
    }
}
