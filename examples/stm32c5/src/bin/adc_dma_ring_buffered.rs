#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::adc::{Adc, AdcChannel, Config as AdcConfig, SampleTime};
use embassy_stm32::{Config, bind_interrupts, dma, peripherals};
use panic_probe as _;

const DMA_BUF_LEN: usize = 16;
const READ_BUF_LEN: usize = DMA_BUF_LEN / 2;

bind_interrupts!(struct Irqs {
    LPDMA1_CH0 => dma::InterruptHandler<peripherals::LPDMA1_CH0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.mux.adcdacsel = mux::Adcdacsel::Hclk1;
        config.rcc.adcdac_pre = embassy_stm32::pac::rcc::vals::Adcdacpre::Div2;
    }

    let p = embassy_stm32::init(config);

    let mut adc = Adc::new_blocking(p.ADC1, AdcConfig::default());
    let mut pa0 = p.PA0.degrade_adc();

    let mut dma_buf = [0u16; DMA_BUF_LEN];

    let mut ring_adc = adc.into_ring_buffered(
        p.LPDMA1_CH0,
        &mut dma_buf,
        Irqs,
        [(pa0.reborrow_adc(), SampleTime::Cycles289)].into_iter(),
        None,
    );

    ring_adc.start();

    let mut samples = [0u16; READ_BUF_LEN];

    loop {
        match ring_adc.read(&mut samples).await {
            Ok(remaining) => {
                let sum: u32 = samples.iter().map(|&value| value as u32).sum();
                let average = sum / samples.len() as u32;

                for (index, &sample) in samples.iter().enumerate() {
                    info!("samples[{}]={=u16:#06x}", index, sample);
                }

                info!("average={}, remaining={}", average, remaining);
            }
            Err(_) => {
                warn!("ADC DMA ring-buffer overrun; discarding old samples");
                ring_adc.clear();
            }
        }
    }
}