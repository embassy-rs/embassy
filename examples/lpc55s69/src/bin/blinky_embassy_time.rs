#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_nxp::gpio::{Level, Output};
use embassy_time::Timer;
use {defmt_rtt as _, panic_halt as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nxp::init(Default::default());
    info!("Initialization complete");
    let mut led = Output::new(p.PIO1_6, Level::Low);

    info!("Entering main loop");
    loop {
        info!("led off!");
        led.set_high();
        Timer::after_millis(500).await;

        info!("led on!");
        led.set_low();
        Timer::after_millis(500).await;
    }
}
