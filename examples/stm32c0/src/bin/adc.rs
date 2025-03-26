#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::adc::vals::Scandir;
use embassy_stm32::adc::{Adc, AdcChannel, AnyAdcChannel, Resolution, SampleTime};
use embassy_stm32::peripherals::ADC1;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let config = Default::default();
    let p = embassy_stm32::init(config);

    info!("ADC STM32C0 example.");

    // We need to set certain sample time to be able to read temp sensor.
    let mut adc = Adc::new(p.ADC1, SampleTime::CYCLES12_5, Resolution::BITS12);
    let mut temp = adc.enable_temperature().degrade_adc();
    let mut vref = adc.enable_vrefint().degrade_adc();
    let mut pin0 = p.PA0.degrade_adc();

    let mut dma = p.DMA1_CH1;
    let mut read_buffer: [u16; 3] = [0; 3];

    loop {
        info!("============================");
        let blocking_temp = adc.blocking_read(&mut temp);
        let blocking_vref = adc.blocking_read(&mut vref);
        let blocing_pin0 = adc.blocking_read(&mut pin0);
        info!(
            "Blocking ADC read: vref = {}, temp = {}, pin0 = {}.",
            blocking_vref, blocking_temp, blocing_pin0
        );

        let channels_seqence: [&mut AnyAdcChannel<ADC1>; 3] = [&mut vref, &mut temp, &mut pin0];
        adc.read(dma.reborrow(), channels_seqence.into_iter(), &mut read_buffer)
            .await;
        // Values are ordered according to hardware ADC channel number!
        info!(
            "DMA ADC read in set: vref = {}, temp = {}, pin0 = {}.",
            read_buffer[0], read_buffer[1], read_buffer[2]
        );

        let hw_channel_selection: u32 =
            (1 << temp.get_hw_channel()) + (1 << vref.get_hw_channel()) + (1 << pin0.get_hw_channel());
        adc.read_in_hw_order(dma.reborrow(), hw_channel_selection, Scandir::UP, &mut read_buffer)
            .await;
        info!(
            "DMA ADC read in hardware order: vref = {}, temp = {}, pin0 = {}.",
            read_buffer[2], read_buffer[1], read_buffer[0]
        );

        Timer::after_millis(2000).await;
    }
}
