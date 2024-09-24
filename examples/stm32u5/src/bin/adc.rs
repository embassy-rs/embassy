#![no_std]
#![no_main]


use defmt::{*};
use defmt_rtt as _;

use embassy_stm32::adc;
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

    let p = embassy_stm32::init(config);
    info!("Hello World!");

    let mut adc = adc::Adc::new(p.ADC1);
    let mut adc_pin = p.PA3;
    adc.set_resolution(adc::Resolution::BITS14);
    adc.set_averaging(adc::Averaging::Samples1024);
    adc.set_sample_time(adc::SampleTime::CYCLES1_5);

    let mut adc2 = adc::Adc::new(p.ADC2);
    let mut adc_pin2 = p.PA5;
    adc2.set_resolution(adc::Resolution::BITS14);
    adc2.set_averaging(adc::Averaging::Samples1024);
    adc2.set_sample_time(adc::SampleTime::CYCLES1_5);

    let mut adc4 = adc4::Adc4::new(p.ADC4);
    let mut adc_pin4 = p.PD11;
    adc4.set_resolution(adc4::Resolution::BITS12);
    adc4.set_averaging(adc4::Averaging::Samples256);
    adc4.set_sample_time(adc4::SampleTime::CYCLES1_5);

    loop {
        embassy_time::Timer::after_millis(100).await;
        let raw :u16 = adc.blocking_read(&mut adc_pin);
        let max = adc::resolution_to_max_count(adc::Resolution::BITS14);
        let volt: f32 = 3.3 * raw as f32 / max as f32;
        info!("Read ADC1 {}", volt);

        let raw4 :u16 = adc4.blocking_read(&mut adc_pin4);
        let max4 = adc4::resolution_to_max_count(adc4::Resolution::BITS12);
        let volt4: f32 = 3.3 * raw4 as f32 / max4 as f32;
        info!("Read ADC4 {}", volt4);
    }
}