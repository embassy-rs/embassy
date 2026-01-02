#![no_std]
#![no_main]

use defmt::*;
use embassy_stm32::adc;
use embassy_stm32::adc::{adc4, AdcChannel};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    let config = embassy_stm32::Config::default();

    let mut p = embassy_stm32::init(config);

    // **** ADC1 init ****
    let mut adc1 = adc::Adc::new(p.ADC1);
    let mut adc1_pin1 = p.PA3; // A0 on nucleo u5a5
    let mut adc1_pin2 = p.PA2; // A1
    adc1.set_resolution(adc::Resolution::BITS14);
    adc1.set_averaging(adc::Averaging::Samples1024);
    adc1.set_sample_time(adc::SampleTime::CYCLES160_5);
    let max1 = adc::resolution_to_max_count(adc::Resolution::BITS14);

    // **** ADC2 init ****
    let mut adc2 = adc::Adc::new(p.ADC2);
    let mut adc2_pin1 = p.PC3; // A2
    let mut adc2_pin2 = p.PB0; // A3
    adc2.set_resolution(adc::Resolution::BITS14);
    adc2.set_averaging(adc::Averaging::Samples1024);
    adc2.set_sample_time(adc::SampleTime::CYCLES160_5);
    let max2 = adc::resolution_to_max_count(adc::Resolution::BITS14);

    // **** ADC4 init ****
    let mut adc4 = adc4::Adc4::new(p.ADC4);
    let mut adc4_pin1 = p.PC1; // A4
    let mut adc4_pin2 = p.PC0; // A5
    adc4.set_resolution(adc4::Resolution::BITS12);
    adc4.set_averaging(adc4::Averaging::Samples256);
    adc4.set_sample_time(adc4::SampleTime::CYCLES1_5);
    let max4 = adc4::resolution_to_max_count(adc4::Resolution::BITS12);

    // **** ADC1 blocking read ****
    let raw: u16 = adc1.blocking_read(&mut adc1_pin1);
    let volt: f32 = 3.3 * raw as f32 / max1 as f32;
    info!("Read adc1 pin 1 {}", volt);

    let raw: u16 = adc1.blocking_read(&mut adc1_pin2);
    let volt: f32 = 3.3 * raw as f32 / max1 as f32;
    info!("Read adc1 pin 2 {}", volt);

    // **** ADC2 blocking read ****
    let raw: u16 = adc2.blocking_read(&mut adc2_pin1);
    let volt: f32 = 3.3 * raw as f32 / max2 as f32;
    info!("Read adc2 pin 1 {}", volt);

    let raw: u16 = adc2.blocking_read(&mut adc2_pin2);
    let volt: f32 = 3.3 * raw as f32 / max2 as f32;
    info!("Read adc2 pin 2 {}", volt);

    // **** ADC4 blocking read ****
    let raw: u16 = adc4.blocking_read(&mut adc4_pin1);
    let volt: f32 = 3.3 * raw as f32 / max4 as f32;
    info!("Read adc4 pin 1 {}", volt);

    let raw: u16 = adc4.blocking_read(&mut adc4_pin2);
    let volt: f32 = 3.3 * raw as f32 / max4 as f32;
    info!("Read adc4 pin 2 {}", volt);

    // **** ADC1 async read ****
    let mut degraded11 = adc1_pin1.degrade_adc();
    let mut degraded12 = adc1_pin2.degrade_adc();
    let mut measurements = [0u16; 2];

    adc1.read(
        p.GPDMA1_CH0.reborrow(),
        [
            (&mut degraded11, adc::SampleTime::CYCLES160_5),
            (&mut degraded12, adc::SampleTime::CYCLES160_5),
        ]
        .into_iter(),
        &mut measurements,
    )
    .await;
    let volt1: f32 = 3.3 * measurements[0] as f32 / max1 as f32;
    let volt2: f32 = 3.3 * measurements[1] as f32 / max1 as f32;

    info!("Async read 1 pin 1 {}", volt1);
    info!("Async read 1 pin 2 {}", volt2);

    // **** ADC2 does not support async read ****

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
    let volt1: f32 = 3.3 * measurements[1] as f32 / max4 as f32;
    info!("Async read 4 pin 1 {}", volt1);
    info!("Async read 4 pin 2 {}", volt2);
}
