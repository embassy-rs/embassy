#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;

use embassy::executor::Spawner;
use embassy::time::Delay;
use embassy_stm32::adc::Adc;
use embassy_stm32::Peripherals;
use embassy_traits::delay::Delay as _;
use example_common::*;

#[embassy::main]
async fn main(_spawner: Spawner, p: Peripherals) {
    info!("Hello World!");

    let mut adc = Adc::new(p.ADC1, &mut Delay);
    let mut pin = p.PB1;

    let mut vref = adc.enable_vref(&mut Delay);
    adc.calibrate(&mut vref);
    loop {
        let v = adc.read(&mut pin);
        info!("--> {} - {} mV", v, adc.to_millivolts(v));
        Delay.delay_ms(100).await;
    }
}
