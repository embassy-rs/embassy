#![no_std]
#![no_main]

use defmt::*;
use embassy_stm32::adc::{AdcChannel, adc4};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    let config = embassy_stm32::Config::default();

    let mut p = embassy_stm32::init(config);

    // **** ADC4 init ****
    let mut adc4 = adc4::Adc4::new(p.ADC4);
    let mut adc4_pin1 = p.PA0; // A4
    let mut adc4_pin2 = p.PA1; // A5
    adc4.set_resolution(adc4::Resolution::BITS12);
    adc4.set_averaging(adc4::Averaging::Samples256);

    let max4 = adc4::resolution_to_max_count(adc4::Resolution::BITS12);

    // **** ADC4 blocking read ****
    let raw: u16 = adc4.blocking_read(&mut adc4_pin1, adc4::SampleTime::CYCLES1_5);
    let volt: f32 = 3.0 * raw as f32 / max4 as f32;
    info!("Read adc4 pin 1 {}", volt);

    let raw: u16 = adc4.blocking_read(&mut adc4_pin2, adc4::SampleTime::CYCLES1_5);
    let volt: f32 = 3.3 * raw as f32 / max4 as f32;
    info!("Read adc4 pin 2 {}", volt);

    // **** ADC4 async read ****
    let mut degraded41 = adc4_pin1.degrade_adc();
    let mut degraded42 = adc4_pin2.degrade_adc();
    let mut measurements = [0u16; 2];

    // The channels must be in ascending order and can't repeat for ADC4
    adc4.read(
        p.GPDMA1_CH1.reborrow(),
        [&mut degraded42, &mut degraded41].into_iter(),
        &mut measurements,
    )
    .await
    .unwrap();
    let volt2: f32 = 3.3 * measurements[0] as f32 / max4 as f32;
    let volt1: f32 = 3.0 * measurements[1] as f32 / max4 as f32;
    info!("Async read 4 pin 1 {}", volt1);
    info!("Async read 4 pin 2 {}", volt2);
}
