#![no_std]
#![no_main]

use cortex_m::prelude::_embedded_hal_blocking_delay_DelayUs;
use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::adc::vals::Exten;
use embassy_stm32::adc::{Adc, AdcChannel, ConversionTrigger, RegularConversionMode, SampleTime, Temperature, VrefInt};
use embassy_time::{Delay, Timer};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let mut adc_dma_buf: [u16; 2] = [0; 2];

    let mut adc_ring_buffered = Adc::new(p.ADC2).into_ring_buffered(
        p.DMA2_CH2,
        &mut adc_dma_buf,
        [(p.PA0.degrade_adc(), SampleTime::CYCLES112)].into_iter(),
        RegularConversionMode::Triggered(ConversionTrigger {
            channel: 0,
            edge: Exten::RISING_EDGE,
        }),
    );
    adc_ring_buffered.start();

    let mut delay = Delay;
    let mut adc = Adc::new_with_config(p.ADC1, Default::default());
    let mut pin = p.PC1;

    let mut vrefint = adc.enable_vrefint();
    let mut temp = adc.enable_temperature();

    // Startup delay can be combined to the maximum of either
    delay.delay_us(Temperature::start_time_us().max(VrefInt::start_time_us()));

    let vrefint_sample = adc.blocking_read(&mut vrefint, SampleTime::CYCLES112);

    let convert_to_millivolts = |sample| {
        // From http://www.st.com/resource/en/datasheet/DM00071990.pdf
        // 6.3.24 Reference voltage
        const VREFINT_MV: u32 = 1210; // mV

        (u32::from(sample) * VREFINT_MV / u32::from(vrefint_sample)) as u16
    };

    let convert_to_celcius = |sample| {
        // From http://www.st.com/resource/en/datasheet/DM00071990.pdf
        // 6.3.22 Temperature sensor characteristics
        const V25: i32 = 760; // mV
        const AVG_SLOPE: f32 = 2.5; // mV/C

        let sample_mv = convert_to_millivolts(sample) as i32;

        (sample_mv - V25) as f32 / AVG_SLOPE + 25.0
    };

    info!("VrefInt: {}", vrefint_sample);
    const MAX_ADC_SAMPLE: u16 = (1 << 12) - 1;
    info!("VCCA: {} mV", convert_to_millivolts(MAX_ADC_SAMPLE));

    loop {
        // Read pin
        let v = adc.blocking_read(&mut pin, SampleTime::CYCLES112);
        info!("PC1: {} ({} mV)", v, convert_to_millivolts(v));

        // Read internal temperature
        let v = adc.blocking_read(&mut temp, SampleTime::CYCLES112);
        let celcius = convert_to_celcius(v);
        info!("Internal temp: {} ({} C)", v, celcius);

        // Read internal voltage reference
        let v = adc.blocking_read(&mut vrefint, SampleTime::CYCLES112);
        info!("VrefInt: {}", v);

        Timer::after_millis(100).await;
    }
}
