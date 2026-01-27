#![no_std]
#![no_main]
use cortex_m::singleton;
use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::adc::{Adc, AdcChannel, RegularConversionMode, RingBufferedAdc, SampleTime};
use embassy_stm32::{Peripherals, bind_interrupts, dma, peripherals};
use embassy_time::Instant;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    DMA2_STREAM0 => dma::InterruptHandler<peripherals::DMA2_CH0>;
    DMA2_STREAM2 => dma::InterruptHandler<peripherals::DMA2_CH2>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    spawner.spawn(adc_task(p).unwrap());
}

#[embassy_executor::task]
async fn adc_task(p: Peripherals) {
    const ADC_BUF_SIZE: usize = 1024;
    let adc_data: &mut [u16; ADC_BUF_SIZE] = singleton!(ADCDAT : [u16; ADC_BUF_SIZE] = [0u16; ADC_BUF_SIZE]).unwrap();
    let adc_data2: &mut [u16; ADC_BUF_SIZE] = singleton!(ADCDAT2 : [u16; ADC_BUF_SIZE] = [0u16; ADC_BUF_SIZE]).unwrap();

    let adc = Adc::new_with_config(p.ADC1, Default::default());
    let adc2 = Adc::new_with_config(p.ADC2, Default::default());

    let mut adc: RingBufferedAdc<embassy_stm32::peripherals::ADC1> = adc.into_ring_buffered(
        p.DMA2_CH0,
        adc_data,
        Irqs,
        [
            (p.PA0.degrade_adc(), SampleTime::CYCLES112),
            (p.PA2.degrade_adc(), SampleTime::CYCLES112),
        ]
        .into_iter(),
        RegularConversionMode::Continuous,
    );
    let mut adc2: RingBufferedAdc<embassy_stm32::peripherals::ADC2> = adc2.into_ring_buffered(
        p.DMA2_CH2,
        adc_data2,
        Irqs,
        [
            (p.PA1.degrade_adc(), SampleTime::CYCLES112),
            (p.PA3.degrade_adc(), SampleTime::CYCLES112),
        ]
        .into_iter(),
        RegularConversionMode::Continuous,
    );

    // Note that overrun is a big consideration in this implementation. Whatever task is running the adc.read() calls absolutely must circle back around
    // to the adc.read() call before the DMA buffer is wrapped around > 1 time. At this point, the overrun is so significant that the context of
    // what channel is at what index is lost. The buffer must be cleared and reset. This *is* handled here, but allowing this to happen will cause
    // a reduction of performance as each time the buffer is reset, the adc & dma buffer must be restarted.

    // An interrupt executor with a higher priority than other tasks may be a good approach here, allowing this task to wake and read the buffer most
    // frequently.
    let mut tic = Instant::now();
    let mut buffer1 = [0u16; 512];
    let mut buffer2 = [0u16; 512];
    let _ = adc.start();
    let _ = adc2.start();
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
            }
        }

        match adc2.read(&mut buffer2).await {
            Ok(_data) => {
                let toc = Instant::now();
                info!(
                    "\n adc2: {} dt = {}, n = {}",
                    buffer2[0..16],
                    (toc - tic).as_micros(),
                    _data
                );
                tic = toc;
            }
            Err(e) => {
                warn!("Error: {:?}", e);
                buffer2 = [0u16; 512];
                let _ = adc2.start();
            }
        }
    }
}
