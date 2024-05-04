#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(impl_trait_in_assoc_type)]
use cortex_m::singleton;
use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::adc::{Adc, SampleTime, Sequence};
use embassy_time::Instant;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    const ADC_BUF_SIZE: usize = 512;
    let mut p = embassy_stm32::init(Default::default());

    let adc_data: &mut [u16; ADC_BUF_SIZE] = singleton!(ADCDAT : [u16; ADC_BUF_SIZE] = [0u16; ADC_BUF_SIZE]).unwrap();
    let adc2_data: &mut [u16; ADC_BUF_SIZE] = singleton!(ADCDAT : [u16; ADC_BUF_SIZE] = [0u16; ADC_BUF_SIZE]).unwrap();

    let mut adc = Adc::new(p.ADC1);
    let mut adc2 = Adc::new(p.ADC2);

    adc.set_sample_sequence(Sequence::One, &mut p.PA0, SampleTime::CYCLES3)
        .await;

    adc.set_sample_sequence(Sequence::Two, &mut p.PA1, SampleTime::CYCLES3)
        .await;

    adc2.set_sample_sequence(Sequence::One, &mut p.PA2, SampleTime::CYCLES3)
        .await;

    adc2.set_sample_sequence(Sequence::Two, &mut p.PA3, SampleTime::CYCLES3)
        .await;

    let mut adc_dma = adc.start_read_continuous(p.DMA2_CH0, adc_data);
    let mut adc_dma2 = adc2.start_read_continuous(p.DMA2_CH2, adc2_data);

    let mut tic = Instant::now();
    let mut buffer1 = [0u16; 256];
    let mut buffer2 = [0u16; 256];
    loop {
        match adc.get_dma_buf(&mut adc_dma, &mut buffer1).await {
            Ok(_data) => {} //info!("adc1 sa: {}", data),
            Err(e) => {
                warn!("Error: {:?}", e);
                continue;
            }
        }

        match adc2.get_dma_buf(&mut adc_dma2, &mut buffer2).await {
            Ok(_data2) => {} //info!("adc2 sa: {}", data2),
            Err(e) => {
                warn!("Error: {:?}", e);
                continue;
            }
        }
        let toc = Instant::now();
        info!(
            "\n adc1: {}, adc2: {}, dt = {}",
            buffer1[0..56],
            buffer2[0..56],
            (toc - tic).as_micros()
        );
        // info!("{}", (toc - tic).as_micros());
        tic = toc;
    }
}
