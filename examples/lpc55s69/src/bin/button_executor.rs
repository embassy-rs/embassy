//! This example has been made with the LPCXpresso55S69 board in mind, which has a built-in LED on
//! PIO1_6 and a button (labeled "user") on PIO1_9.

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_nxp::gpio::{Input, Level, Output, Pull};
use {defmt_rtt as _, panic_halt as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let p = embassy_nxp::init(Default::default());

    let mut led = Output::new(p.PIO1_6, Level::Low);
    let mut button = Input::new(p.PIO1_9, Pull::Up);

    info!("Entered main loop");
    loop {
        button.wait_for_rising_edge().await;
        info!("Button pressed");
        led.toggle();
    }
}
