//! This example has been made with the LPCXpresso55S69 board in mind, which has a built-in LED on PIO1_6.

#![no_std]
#![no_main]

use cortex_m::asm::nop;
use defmt::*;
use embassy_executor::Spawner;
use embassy_nxp::gpio::{Level, Output};
use {defmt_rtt as _, panic_halt as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nxp::init(Default::default());

    let mut led = Output::new(p.PIO1_6, Level::Low);

    loop {
        info!("led off!");
        led.set_high();

        for _ in 0..200_000 {
            nop();
        }

        info!("led on!");
        led.set_low();

        for _ in 0..200_000 {
            nop();
        }
    }
}
