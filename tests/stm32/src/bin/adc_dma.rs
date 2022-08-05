#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::executor::Spawner;
use embassy_executor::time::{Delay, Duration, Timer};
use embassy_stm32::{adc::Adc, gpio::{Output, Level, Speed}} ;
use embassy_stm32::Peripherals;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner, p: Peripherals) {

    #[cfg(feature = "stm32f103c8")]

    // Arduino pins D0 and D1
    // They're connected together with a 1K resistor.
    let (mut gpio_out, mut pin) = (p.PA6, p.PA7);
    
    let mut gpio_out = Output::new(&mut gpio_out, Level::High, Speed::Low);

    let mut adc = Adc::new(p.ADC1, &mut Delay);
    let mut vref = adc.enable_vref(&mut Delay);

    adc.calibrate(&mut vref);

    Timer::after(Duration::from_millis(100)).await;
    let v = adc.read(&mut pin);
    info!("--> {} - {} mV", v, adc.to_millivolts(v));

    gpio_out.set_low();
    Timer::after(Duration::from_millis(100)).await;
    let v = adc.read(&mut pin);
    info!("--> {} - {} mV", v, adc.to_millivolts(v));


}
