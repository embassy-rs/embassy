#![no_std]
#![no_main]

#[path = "../common.rs"]
mod common;

use defmt::{assert, info};
use embassy_executor::Spawner;
use embassy_nrf::gpio::{Input, Level, Output, OutputDrive, Pull};
use embassy_time::Timer;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());

    let input = Input::new(peri!(p, PIN_A), Pull::Up);
    let mut output = Output::new(peri!(p, PIN_B), Level::Low, OutputDrive::Standard);

    output.set_low();
    Timer::after_millis(10).await;
    assert!(input.is_low());

    output.set_high();
    Timer::after_millis(10).await;
    assert!(input.is_high());

    info!("Test OK");
    cortex_m::asm::bkpt();
}
