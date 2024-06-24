#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(impl_trait_in_assoc_type)]
use cortex_m::singleton;
use cortex_m_rt::entry;
use defmt::*;
use embassy_executor::Spawner;
use embassy_futures::block_on;
use embassy_stm32::adc::ringbuffered_v2::RingBufferedAdc;
use embassy_stm32::adc::{Adc, SampleTime, Sequence};
use embassy_time::{Instant, Timer};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    const ADC_BUF_SIZE: usize = 1024;
    let mut p = embassy_stm32::init(Default::default());

    let adc_data: &mut [u16; ADC_BUF_SIZE] = singleton!(ADCDAT : [u16; ADC_BUF_SIZE] = [0u16; ADC_BUF_SIZE]).unwrap();
    let mut adc = Adc::new(p.ADC1);

    adc.set_sample_sequence(Sequence::One, &mut p.PA0, SampleTime::CYCLES112)
        .await;
    adc.set_sample_sequence(Sequence::Two, &mut p.PA1, SampleTime::CYCLES112)
        .await;

    // adc.set_sample_sequence(Sequence::Three, &mut p.PA2, SampleTime::CYCLES112)
    //     .await;
    // //     .await;
    // adc.set_sample_sequence(Sequence::Four, &mut p.PA3, SampleTime::CYCLES112)
    //     .await;

    let mut adc: RingBufferedAdc<embassy_stm32::peripherals::ADC1> = adc.into_ring_buffered(p.DMA2_CH0, adc_data);

    let mut tic = Instant::now();
    let mut buffer1 = [0u16; 512];
    let _ = adc.start();
    loop {
        match adc.read(&mut buffer1).await {
            Ok(_data) => {
                let toc = Instant::now();
                info!(
                    "\n adc1: {} dt = {}, n = {}",
                    buffer1[0..16],
                    (toc - tic).as_micros(),
                    _data
                );
                tic = toc;
            }
            Err(e) => {
                warn!("Error: {:?}", e);
                buffer1 = [0u16; 512];
                let _ = adc.start();
                continue;
            }
        }

        Timer::after_micros(800).await;
    }
}
