#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_mspm0::gpio::{Input, Level, Output, Pull};
use embassy_mspm0::Config;
use {defmt_rtt as _, panic_halt as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    info!("Hello world!");

    let p = embassy_mspm0::init(Config::default());

    let led1 = p.PA22;
    let s2 = p.PA16;

    let mut led1 = Output::new(led1, Level::Low);

    let mut s2 = Input::new(s2, Pull::Up);

    // led1 is active low
    led1.set_high();

    loop {
        s2.wait_for_falling_edge().await;

        info!("Switch 2 was pressed");

        led1.toggle();
    }
}
