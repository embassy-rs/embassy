#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::executor::Spawner;
use embassy_executor::time::{Delay, Duration, Timer};
use embassy_stm32::adc::{Adc,SamplerState, Instance, RxDma, AdcPin};
use embassy_stm32::{Peripherals, Peripheral, PeripheralRef};
use {defmt_rtt as _, panic_probe as _};

async fn single_read<T : Instance + Peripheral<P = T>>(adc : &mut Adc<'_, T>, pin: &mut impl AdcPin<T>){
    info!("Single read");

    for i in (0..20) {
        let v = adc.read(pin);
        info!("--> {} - {} mV", v, adc.to_millivolts(v));
        Timer::after(Duration::from_millis(100)).await;
    }
}

async fn continous_read<T : Instance + Peripheral<P = T>, U :  RxDma<T> + Peripheral<P = U>>(adc : &mut Adc<'_, T>, pin: &mut impl AdcPin<T>, mut dma_chan: PeripheralRef<'_, U>){
    
    info!("Continuous read");

    let mut data =  [[0;500]; 2];
    let mut cnt : u8 = 0;
    adc.read_continuous( pin, dma_chan,&mut data,
         &mut |buf| {
            let state;
            if cnt <= 20{
                info!("--> 0 {} ", buf[0]);
                info!("--> 99 {} ", buf[99]);
                info!("--> 199 {} ", buf[199]);
                info!("--> 299 {} ", buf[299]);
                info!("--> 399 {} ", buf[399]);
                info!("--> 499 {} ", buf[499]);
                state = SamplerState::Sampled
            }
            else{
                state = SamplerState::Stopped
            }
            cnt = cnt +1;
            state

    }).await;
}

#[embassy_executor::main]
async fn main(_spawner: Spawner, p: Peripherals) {
    info!("Hello World!");

    let mut adc = Adc::new(p.ADC1, &mut Delay);
    let mut pin = p.PB1;
    let mut dma_chan = p.DMA1_CH1.into_ref();

    let mut vref = adc.enable_vref(&mut Delay);
    adc.calibrate(&mut vref);
    adc.set_sample_time(embassy_stm32::adc::SampleTime::Cycles239_5);

    // Switching alternatively between ADC continuous mode with DMA and ADC single reading

    continous_read(&mut adc, &mut pin, dma_chan.reborrow()).await;
    
    single_read(&mut adc, &mut pin).await;

    continous_read(&mut adc, &mut pin, dma_chan).await;
    
}
