#![no_std]
#![no_main]


use defmt::{*};
use defmt_rtt as _;

use embassy_stm32::adc;
use embassy_stm32::adc::AdcChannel;
use embassy_stm32::adc::adc4;
use panic_probe as _;


#[embassy_executor::main]
async fn main(spawner: embassy_executor::Spawner) {
    let mut config = embassy_stm32::Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hsi = true;

        config.rcc.pll1 = Some(Pll {
            source: PllSource::HSI, // 16 MHz
            prediv: PllPreDiv::DIV1, // 16 MHz
            mul: PllMul::MUL10, // 160 MHz
            divp: Some(PllDiv::DIV1), // don't care
            divq: Some(PllDiv::DIV1), // don't care
            divr: Some(PllDiv::DIV1), // 160 MHz
        });

        config.rcc.sys = Sysclk::PLL1_R;
        config.rcc.voltage_range = VoltageScale::RANGE1;
        config.rcc.hsi48 = Some(Hsi48Config { sync_from_usb: true }); // needed for USB
        config.rcc.mux.iclksel = mux::Iclksel::HSI48; // USB uses ICLK

    }

    let mut p = embassy_stm32::init(config);

    let mut adc = adc::Adc::new(p.ADC1);
    let mut adc_pin1 = p.PA3; // A0 on nucleo u5a5
    let mut adc_pin2 = p.PA2; // A1 on nucleo u5a5
    adc.set_resolution(adc::Resolution::BITS14);
    adc.set_averaging(adc::Averaging::Samples1024);
    adc.set_sample_time(adc::SampleTime::CYCLES160_5);
    let max = adc::resolution_to_max_count(adc::Resolution::BITS14);

    let mut adc4 = adc4::Adc4::new(p.ADC4);
    let mut adc4_pin1 = p.PD11;
    let mut adc4_pin2 = p.PC0;
    adc4.set_resolution(adc4::Resolution::BITS12);
    adc4.set_averaging(adc4::Averaging::Samples256);
    adc4.set_sample_time(adc4::SampleTime::CYCLES1_5);
    let max4 = adc4::resolution_to_max_count(adc4::Resolution::BITS12);

    let raw: u16 = adc.blocking_read(&mut adc_pin1);
    let volt: f32 = 3.3 * raw as f32 / max as f32;
    info!("Read 1 pin 1 {}", volt);

    let raw: u16 = adc.blocking_read(&mut adc_pin2);
    let volt: f32 = 3.3 * raw as f32 / max as f32;
    info!("Read 1 pin 2 {}", volt);

    let raw4: u16 = adc4.blocking_read(&mut adc4_pin1);
    let volt4: f32 = 3.3 * raw4 as f32 / max4 as f32;
    info!("Read 4 pin 1 {}", volt4);

    let raw4: u16 = adc4.blocking_read(&mut adc4_pin2);
    let volt4: f32 = 3.3 * raw4 as f32 / max4 as f32;
    info!("Read 4 pin 2 {}", volt4);

    let mut degraded1 = adc_pin1.degrade_adc();
    let mut degraded2 = adc_pin2.degrade_adc();
    let mut measurements = [0u16; 2];

    adc.read(
        &mut p.GPDMA1_CH0,
        [
            (&mut degraded2, adc::SampleTime::CYCLES160_5),
            (&mut degraded1, adc::SampleTime::CYCLES160_5),
        ]
        .into_iter(),
        &mut measurements,
    ).await;
    let volt1: f32 = 3.3 * measurements[1] as f32 / max as f32;
    let volt2: f32 = 3.3 * measurements[0] as f32 / max as f32;

    info!("Async read 1 pin 1 {}", volt1);
    info!("Async read 1 pin 2 {}", volt2);

}