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
use embassy_nxp::gpio::{Input, Level, Output, Pull};
// Must include `embassy_imxrt1011_examples` to ensure the FCB gets linked.
use {defmt_rtt as _, embassy_imxrt1011_examples as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let p = embassy_nxp::init(Default::default());
    info!("Hello world!");

    /* Pick the pins to use depending on your board. */

    // IMXRT1010-EVK
    //
    // LED (D25) and user button (SW4)
    let (led, button) = (p.GPIO_11, p.GPIO_SD_05);

    // Adafruit Metro M7 (both microSD and AirLift variants)
    //
    // The LED is connected to D13 on the board.
    //
    // In particular the Metro M7 has no board user buttons, so you will need to connect a button.
    // Any other GPIO pin can be used. GPIO_04 is used for example since it is on pin D12.
    // let (led, button) = (p.GPIO_03, p.GPIO_04);

    // Makerdiary iMX RT1011 Nano Kit
    //
    // LED0 and user button.
    // let (led, button) = (p.GPIO_SD_04, p.GPIO_SD_03);

    let mut button = Input::new(button, Pull::Up100K);
    let mut led = Output::new(led, Level::Low);
    led.set_high();

    loop {
        button.wait_for_falling_edge().await;

        info!("Toggled");
        led.toggle();

        // The RT1010EVK has a 100 nF debouncing capacitor which results in false positive events
        // when listening for a falling edge in a loop, wait for the rising edge and then wait for
        // stabilization.
        button.wait_for_rising_edge().await;

        // Stabilization.
        for _ in 0..100_000 {
            cortex_m::asm::nop();
        }
    }
}
