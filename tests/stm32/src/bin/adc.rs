#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use defmt::assert;
use embassy_executor::executor::Spawner;
use embassy_executor::time::{Delay, Duration, Timer};
use embassy_stm32::{adc::Adc, gpio::{Output, Level, Speed}} ;
use embassy_stm32::Peripherals;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner, p: Peripherals) {

    // Arduino pins D0 and D1
    // They're connected together with a 1K resistor.
    #[cfg(feature = "stm32f103c8")]
    let (mut gpio_out, mut pin) = (p.PA6, p.PA7);
    
    let mut gpio_out = Output::new(&mut gpio_out, Level::High, Speed::Low);

    let mut adc = Adc::new(p.ADC1, &mut Delay);
    let mut vref = adc.enable_vref(&mut Delay);

    adc.calibrate(&mut vref);

    Timer::after(Duration::from_millis(100)).await;
    let v = adc.read(&mut pin);
    let mv = adc.to_millivolts(v);
    info!("--> {} mv", mv);
    assert!(mv > 3200); //100 mv tolerance is large enough

    gpio_out.set_low();
    Timer::after(Duration::from_millis(100)).await;
    let v = adc.read(&mut pin);
    let mv = adc.to_millivolts(v);
    info!("--> {} mv", mv);
    assert!(mv < 100); //100 mv tolerance is large enough

    info!("Test OK");
    cortex_m::asm::bkpt();
}
