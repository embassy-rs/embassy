#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_stm32::adc::{Adc, AdcChannel, SampleTime};
use {defmt_rtt as _, panic_probe as _};

const DMA_BUF_LEN: usize = 512;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Hello World!");

    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.mux.adcsel = mux::Adcsel::SYS;
    }
    let p = embassy_stm32::init(config);

    let mut adc = Adc::new(p.ADC1);
    let mut adc_pin0 = p.PA0.degrade_adc();
    let mut adc_pin1 = p.PA1.degrade_adc();
    let mut adc_dma_buf = [0u16; DMA_BUF_LEN];
    let mut measurements = [0u16; DMA_BUF_LEN / 2];
    let mut ring_buffered_adc = adc.into_ring_buffered(
        p.DMA1_CH1,
        &mut adc_dma_buf,
        [
            (&mut adc_pin0, SampleTime::CYCLES640_5),
            (&mut adc_pin1, SampleTime::CYCLES640_5),
        ]
        .into_iter(),
    );

    info!("starting measurement loop");
    loop {
        match ring_buffered_adc.read(&mut measurements).await {
            Ok(_) => {
                //note: originally there was a print here showing all the samples,
                //but even that takes too much time and would cause adc overruns
                info!("adc1 first 10 samples: {}", measurements[0..10]);
            }
            Err(e) => {
                warn!("Error: {:?}", e);
            }
        }
    }
}
