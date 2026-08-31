//! ADC with DMA ring buffer.
//!
//! The ADC scans a sequence of channels continuously and DMA writes the
//! results into a circular buffer with no CPU involvement -- the CPU
//! only drains the buffer.

#![no_std]
#![no_main]

use cortex_m::singleton;
use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::adc::{Adc, AdcChannel, RingBufferedAdc, SampleTime};
use embassy_stm32::{bind_interrupts, dma, peripherals};
use embassy_time::Instant;
use panic_probe as _;

bind_interrupts!(struct Irqs {
    DMA1_CHANNEL1 => dma::InterruptHandler<peripherals::DMA1_CH1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut p = embassy_stm32::init(Default::default());
    info!("ADC ring buffered on STM32F1");

    const ADC_BUF_SIZE: usize = 1024;
    let adc_data: &mut [u16; ADC_BUF_SIZE] = singleton!(ADCDAT : [u16; ADC_BUF_SIZE] = [0u16; ADC_BUF_SIZE]).unwrap();

    let adc = Adc::new(p.ADC1);

    let mut adc: RingBufferedAdc<_> = adc.into_ring_buffered(
        p.DMA1_CH1,
        adc_data,
        Irqs,
        [
            (p.PA0.reborrow_adc(), SampleTime::Cycles135),
            (p.PA1.reborrow_adc(), SampleTime::Cycles135),
        ]
        .into_iter(),
        None,
    );

    // Note that overrun is a big consideration in this implementation. Whatever task is running the adc.read() calls absolutely must circle back around
    // to the adc.read() call before the DMA buffer is wrapped around > 1 time. At this point, the overrun is so significant that the context of
    // what channel is at what index is lost. The buffer must be cleared and reset. This *is* handled here, but allowing this to happen will cause
    // a reduction of performance as each time the buffer is reset, the adc & dma buffer must be restarted.

    // An interrupt executor with a higher priority than other tasks may be a good approach here, allowing this task to wake and read the buffer most
    // frequently.
    let mut tic = Instant::now();
    let mut buffer = [0u16; 512];
    adc.start();

    loop {
        match adc.read(&mut buffer).await {
            Ok(_data) => {
                let toc = Instant::now();
                info!(
                    "\n adc: {} dt = {}, n = {}",
                    buffer[0..16],
                    (toc - tic).as_micros(),
                    _data
                );
                tic = toc;
            }
            Err(e) => {
                warn!("Error: {:?}", e);
                buffer = [0u16; 512];
                adc.start();
            }
        }
    }
}
