#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::executor::Spawner;
use embassy_executor::time::{Delay, Duration, Timer};
use embassy_stm32::adc::{Adc,SamplerState};
use embassy_stm32::Peripherals;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner, p: Peripherals) {
    info!("Hello World!");

    let mut adc = Adc::new(p.ADC1, &mut Delay);
    let mut pin = p.PB1;

    let mut vref = adc.enable_vref(&mut Delay);
    adc.calibrate(&mut vref);
    adc.set_sample_time(embassy_stm32::adc::SampleTime::Cycles239_5);
    let mut data =  [[0;500]; 2];
    
    adc.read_continuous(&mut pin, p.DMA1_CH1,&mut data,
         &mut |buf| {
        info!("--> 0 {} ", buf[0]);
        info!("--> 99 {} ", buf[99]);
        info!("--> 199 {} ", buf[199]);
        info!("--> 299 {} ", buf[299]);
        info!("--> 399 {} ", buf[399]);
        info!("--> 499 {} ", buf[499]);
        SamplerState::Sampled
    }).await;

}
