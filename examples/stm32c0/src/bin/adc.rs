#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::adc::{Adc, AdcChannel, AnyAdcChannel, Resolution, SampleTime};
use embassy_stm32::peripherals::{ADC1, DMA1_CH1};
use embassy_stm32::{bind_interrupts, dma};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    DMA1_CHANNEL1 => dma::InterruptHandler<DMA1_CH1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let config = Default::default();
    let p = embassy_stm32::init(config);

    info!("ADC STM32C0 example.");

    // We need to set certain sample time to be able to read temp sensor.
    let mut adc = Adc::new(p.ADC1, Resolution::BITS12);
    let mut temp = adc.enable_temperature().degrade_adc();
    let mut vref = adc.enable_vrefint().degrade_adc();
    let mut pin0 = p.PA0.degrade_adc();

    let mut dma = p.DMA1_CH1;
    let mut read_buffer: [u16; 3] = [0; 3];

    loop {
        info!("============================");
        let blocking_temp = adc.blocking_read(&mut temp, SampleTime::CYCLES12_5);
        let blocking_vref = adc.blocking_read(&mut vref, SampleTime::CYCLES12_5);
        let blocing_pin0 = adc.blocking_read(&mut pin0, SampleTime::CYCLES12_5);
        info!(
            "Blocking ADC read: vref = {}, temp = {}, pin0 = {}.",
            blocking_vref, blocking_temp, blocing_pin0
        );

        let channels_sequence: [(&mut AnyAdcChannel<ADC1>, SampleTime); 3] = [
            (&mut vref, SampleTime::CYCLES12_5),
            (&mut temp, SampleTime::CYCLES12_5),
            (&mut pin0, SampleTime::CYCLES12_5),
        ];
        adc.read(dma.reborrow(), Irqs, channels_sequence.into_iter(), &mut read_buffer)
            .await;
        // Values are ordered according to hardware ADC channel number!
        info!(
            "DMA ADC read in set: vref = {}, temp = {}, pin0 = {}.",
            read_buffer[0], read_buffer[1], read_buffer[2]
        );

        Timer::after_millis(2000).await;
    }
}
