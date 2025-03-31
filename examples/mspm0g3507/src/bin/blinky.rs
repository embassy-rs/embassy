#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_mspm0::gpio::{Level, Output};
use embassy_mspm0::Config;
use embassy_time::Timer;
use {defmt_rtt as _, panic_halt as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    info!("Hello world!");
    let p = embassy_mspm0::init(Config::default());

    let mut led1 = Output::new(p.PA0, Level::Low);
    led1.set_inversion(true);

    loop {
        Timer::after_millis(400).await;

        info!("Toggle");
        led1.toggle();
    }
}
