#![no_std]
#![no_main]

// required-features: dac

#[path = "../common.rs"]
mod common;

use common::*;
use embassy_executor::Spawner;
use embassy_stm32::adc::{Adc, SampleTime};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Initialize the board and obtain a Peripherals instance
    let p: embassy_stm32::Peripherals = init();

    let adc = peri!(p, ADC);
    let mut adc_pin = peri!(p, DAC_PIN);

    let mut adc = Adc::new_adc4(adc);

    // Now wait a little to obtain a stable value
    Timer::after_millis(30).await;
    let _ = adc.blocking_read(&mut adc_pin, SampleTime::from_bits(0));

    for _ in 0..=255 {
        // Now wait a little to obtain a stable value
        Timer::after_millis(30).await;

        // Need to steal the peripherals here because PA4 is obviously in use already
        let _ = adc.blocking_read(&mut adc_pin, SampleTime::from_bits(0));
    }

    info!("Test OK");
    cortex_m::asm::bkpt();
}
