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

    // Ideally I would like that array to go directly into .bss
    let mut data =  [[0;1000]; 2];

    adc.read_continuous(&mut pin, p.DMA1_CH1,&mut data,
         &mut |buf| {
        info!("--> 0 {} ", buf[0]);
        info!("--> 1 {} ", buf[1]);
        info!("--> 2 {} ", buf[2]);
        SamplerState::Sampled
    }).await;

    /*
    loop {
        let v = adc.read(&mut pin);
        info!("--> {} - {} mV", v, adc.to_millivolts(v));
        Timer::after(Duration::from_millis(100)).await;
    }
    */
}
