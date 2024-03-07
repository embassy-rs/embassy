#![no_std]
#![no_main]
teleprobe_meta::target!(b"nrf51-dk");

use defmt::{assert, info};
use embassy_executor::Spawner;
use embassy_nrf::gpio::{Input, Level, Output, OutputDrive, Pull};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());

    let input = Input::new(p.P0_13, Pull::Up);
    let mut output = Output::new(p.P0_14, Level::Low, OutputDrive::Standard);

    output.set_low();
    Timer::after_millis(10).await;
    assert!(input.is_low());

    output.set_high();
    Timer::after_millis(10).await;
    assert!(input.is_high());

    info!("Test OK");
    cortex_m::asm::bkpt();
}
