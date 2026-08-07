//! This example works on the following boards:
//! - IMXRT1010-EVK
//! - Adafruit Metro M7 (with microSD or with AirLift), requires an external button
//! - Makerdiary iMX RT1011 Nano Kit (TODO: currently untested, please change this)
//!
//! Although beware you will need to change the GPIO pins being used (scroll down).

#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_nxp::gpio::{Level, Output};
use embassy_time::Timer;
// Must include `embassy_imxrt1011_examples` to ensure the FCB gets linked.
use {defmt_rtt as _, embassy_imxrt1011_examples as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let p = embassy_nxp::init(Default::default());
    info!("Hello world!");

    /* Pick the pins to use depending on your board. */

    // IMXRT1010-EVK
    //
    // LED (D25)
    let led = p.GPIO_11;

    // Adafruit Metro M7 (both microSD and AirLift variants)
    //
    // The LED is connected to D13 on the board.
    // let led = p.GPIO_03;

    // Makerdiary iMX RT1011 Nano Kit
    //
    // LED0
    // let led = p.GPIO_SD_04;

    let mut led = Output::new(led, Level::Low);

    loop {
        Timer::after_millis(500).await;

        info!("Toggle");
        led.toggle();
    }
}
