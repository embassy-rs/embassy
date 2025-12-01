#![no_std]
#![no_main]

use defmt::*;
use embassy_stm32::adc::{self, Adc, AdcChannel, AdcConfig, SampleTime, adc4};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    let config = embassy_stm32::Config::default();

    let mut p = embassy_stm32::init(config);

    // **** ADC1 init ****
    let mut config = AdcConfig::default();
    config.averaging = Some(adc::Averaging::Samples1024);
    config.resolution = Some(adc::Resolution::BITS14);
    let mut adc1 = Adc::new_with_config(p.ADC1, config);
    let mut adc1_pin1 = p.PA3; // A0 on nucleo u5a5
    let mut adc1_pin2 = p.PA2; // A1
    let max1 = adc::resolution_to_max_count(adc::Resolution::BITS14);

    // **** ADC2 init ****
    let mut config = AdcConfig::default();
    config.averaging = Some(adc::Averaging::Samples1024);
    config.resolution = Some(adc::Resolution::BITS14);
    let mut adc2 = Adc::new_with_config(p.ADC2, config);
    let mut adc2_pin1 = p.PC3; // A2
    let mut adc2_pin2 = p.PB0; // A3
    let max2 = adc::resolution_to_max_count(adc::Resolution::BITS14);

    // **** ADC4 init ****
    let mut adc4 = Adc::new_adc4(p.ADC4);
    let mut adc4_pin1 = p.PC1.degrade_adc(); // A4
    let mut adc4_pin2 = p.PC0; // A5
    adc4.set_resolution_adc4(adc4::Resolution::BITS12);
    adc4.set_averaging_adc4(adc4::Averaging::Samples256);
    let max4 = adc4::resolution_to_max_count(adc4::Resolution::BITS12);

    // **** ADC1 blocking read ****
    let raw: u16 = adc1.blocking_read(&mut adc1_pin1, SampleTime::CYCLES160_5);
    let volt: f32 = 3.3 * raw as f32 / max1 as f32;
    info!("Read adc1 pin 1 {}", volt);

    let raw: u16 = adc1.blocking_read(&mut adc1_pin2, SampleTime::CYCLES160_5);
    let volt: f32 = 3.3 * raw as f32 / max1 as f32;
    info!("Read adc1 pin 2 {}", volt);

    // **** ADC2 blocking read ****
    let raw: u16 = adc2.blocking_read(&mut adc2_pin1, SampleTime::CYCLES160_5);
    let volt: f32 = 3.3 * raw as f32 / max2 as f32;
    info!("Read adc2 pin 1 {}", volt);

    let raw: u16 = adc2.blocking_read(&mut adc2_pin2, SampleTime::CYCLES160_5);
    let volt: f32 = 3.3 * raw as f32 / max2 as f32;
    info!("Read adc2 pin 2 {}", volt);

    // **** ADC4 blocking read ****
    let raw: u16 = adc4.blocking_read(&mut adc4_pin1, adc4::SampleTime::CYCLES1_5);
    let volt: f32 = 3.3 * raw as f32 / max4 as f32;
    info!("Read adc4 pin 1 {}", volt);

    let raw: u16 = adc4.blocking_read(&mut adc4_pin2, adc4::SampleTime::CYCLES1_5);
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
        [
            (&mut degraded42, adc4::SampleTime::CYCLES1_5),
            (&mut degraded41, adc4::SampleTime::CYCLES1_5),
        ]
        .into_iter(),
        &mut measurements,
    )
    .await;
    let volt2: f32 = 3.3 * measurements[0] as f32 / max4 as f32;
    let volt1: f32 = 3.3 * measurements[1] as f32 / max4 as f32;
    info!("Async read 4 pin 1 {}", volt1);
    info!("Async read 4 pin 2 {}", volt2);
}
