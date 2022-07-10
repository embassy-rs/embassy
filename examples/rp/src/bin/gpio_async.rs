#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy::executor::Spawner;
use embassy::time::{Duration, Timer};
use embassy_rp::{gpio, Peripherals};
use gpio::{Input, Level, Output, Pull};
use {defmt_rtt as _, panic_probe as _};

/// This example shows how async gpio can be used with a RP2040.
///
/// It requires an external signal to be manually triggered on PIN 16. For
/// example, this could be accomplished using an external power source with a
/// button so that it is possible to toggle the signal from low to high.
///
/// This example will begin with turning on the LED on the board and wait for a
/// high signal on PIN 16. Once the high event/signal occurs the program will
/// continue and turn off the LED, and then wait for 2 seconds before completing
/// the loop and starting over again.
#[embassy::main]
async fn main(_spawner: Spawner, p: Peripherals) {
    let mut led = Output::new(p.PIN_25, Level::Low);
    let mut async_input = Input::new(p.PIN_16, Pull::None);

    loop {
        info!("wait_for_high. Turn on LED");
        led.set_high();

        async_input.wait_for_high().await;

        info!("done wait_for_high. Turn off LED");
        led.set_low();

        Timer::after(Duration::from_secs(2)).await;
    }
}
