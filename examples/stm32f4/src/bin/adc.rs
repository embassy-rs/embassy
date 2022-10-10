#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use cortex_m::prelude::_embedded_hal_blocking_delay_DelayUs;
use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::adc::{Adc, Temperature, VrefInt};
use embassy_time::{Delay, Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let mut delay = Delay;
    let mut adc = Adc::new(p.ADC1, &mut delay);
    let mut pin = p.PC1;

    let mut vrefint = adc.enable_vrefint();
    let mut temp = adc.enable_temperature();

    // Startup delay can be combined to the maximum of either
    delay.delay_us(Temperature::start_time_us().max(VrefInt::start_time_us()));

    loop {
        // Read pin
        let v = adc.read(&mut pin);
        info!("PC1: {} ({} mV)", v, adc.to_millivolts(v));

        // Read internal temperature
        let v = adc.read_internal(&mut temp);
        let celcius = Temperature::to_celcius(adc.to_millivolts(v));
        info!("Internal temp: {} ({} C)", v, celcius);

        // Read internal voltage reference
        let v = adc.read_internal(&mut vrefint);
        info!("VrefInt: {} ({} mV)", v, adc.to_millivolts(v));

        Timer::after(Duration::from_millis(100)).await;
    }
}
