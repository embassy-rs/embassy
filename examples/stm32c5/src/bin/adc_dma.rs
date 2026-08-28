#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::adc::{Adc, AdcChannel, AdcConfig, SampleTime};
use embassy_stm32::{Config, bind_interrupts, dma, peripherals};
use panic_probe as _;

bind_interrupts!(struct Irqs {
    LPDMA1_CH0 =>
        dma::InterruptHandler<peripherals::LPDMA1_CH0>;
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

    info!("ADC DMA example");

    let mut adc1 = Adc::new_with_config(p.ADC1, AdcConfig::default());

    let mut adc1_channel = p.PA0.degrade_adc();
    let mut readings = [0u16; 1];

    adc1.read(
        p.LPDMA1_CH0,
        Irqs,
        [(adc1_channel.reborrow_adc(), SampleTime::Cycles289)].into_iter(),
        None,
        &mut readings,
    )
    .await;

    info!("PA0: {}", readings[0]);
}
