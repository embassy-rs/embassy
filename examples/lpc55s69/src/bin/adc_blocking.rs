//! This example has been made with the LPCXpresso55S69 board in mind, which has PIO0_16 labled as A0.

#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_nxp::adc::{Adc, Config, Resolution, resolution_to_max_count};
use embassy_time::Timer;
use panic_halt as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nxp::init(Default::default());

    // The default configuration corresponds to Config::new(Resolution::Bits16, Averaging::None);
    let config = Config::default();
    let mut adc = Adc::new(p.ADC0, config);

    // PIO0_16 corresponds A0 on the dev board
    let mut adc_pin = p.PIO0_16;

    let max = resolution_to_max_count(Resolution::Bits16);

    loop {
        let reading = adc.blocking_read(&mut adc_pin);
        info!("Raw ADC reading: {}", reading);
        info!("Scaled: {}%", reading as f32 / max as f32 * 100f32);
        Timer::after_millis(500).await;
    }
}
