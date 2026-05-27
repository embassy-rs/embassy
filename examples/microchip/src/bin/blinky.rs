#![no_main]
#![no_std]

use defmt::info;
use embassy_executor::Spawner;
use embassy_microchip::gpio::{Level, Output};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_microchip::init(Default::default());

    info!("Hello, world!");

    let mut led1 = Output::new(p.GPIO157, Level::Low);
    let mut led2 = Output::new(p.GPIO153, Level::High);

    loop {
        info!("toggle leds");
        led1.toggle();
        led2.toggle();
        Timer::after_secs(1).await;
    }
}
