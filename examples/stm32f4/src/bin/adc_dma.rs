#![no_std]
#![no_main]
use cortex_m::singleton;
use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::adc::RingBufferedAdc;
use embassy_stm32::adc::{Adc, SampleTime, Sequence};
use embassy_time::{Instant, Timer};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    const ADC_BUF_SIZE: usize = 1024;
    let mut p = embassy_stm32::init(Default::default());

    let adc_data: &mut [u16; ADC_BUF_SIZE] = singleton!(ADCDAT : [u16; ADC_BUF_SIZE] = [0u16; ADC_BUF_SIZE]).unwrap();
    
    let adc = Adc::new(p.ADC1);

    let mut adc: RingBufferedAdc<embassy_stm32::peripherals::ADC1> = adc.into_ring_buffered(p.DMA2_CH0, adc_data);

    adc.set_sample_sequence(Sequence::One, &mut p.PA0, SampleTime::CYCLES112);
    adc.set_sample_sequence(Sequence::Two, &mut p.PA2, SampleTime::CYCLES112);
    adc.set_sample_sequence(Sequence::Three, &mut p.PA1, SampleTime::CYCLES112);
    adc.set_sample_sequence(Sequence::Four, &mut p.PA3, SampleTime::CYCLES112);

    // Note that overrun is a big consideration in this implementation. Whatever task is running the adc.read() calls absolutely must circle back around
    // to the adc.read() call before the DMA buffer is wrapped around > 1 time. At this point, the overrun is so significant that the context of
    // what channel is at what index is lost. The buffer must be cleared and reset. This *is* handled here, but allowing this to happen will cause
    // a reduction of performance as each time the buffer is reset, the adc & dma buffer must be restarted.

    // An interrupt executor with a higher priority than other tasks may be a good approach here, allowing this task to wake and read the buffer most
    // frequently.
    let mut tic = Instant::now();
    let mut buffer1: [u16; 256] = [0u16; 256];
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
                buffer1 = [0u16; 256];
                let _ = adc.start();
                continue;
            }
        }

        Timer::after_micros(300).await;
    }
}
