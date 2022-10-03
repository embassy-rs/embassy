#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::adc::{Adc, SampleTime};
use embassy_time::{Delay, Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let mut adc = Adc::new(p.ADC, &mut Delay);
    adc.set_sample_time(SampleTime::Cycles71_5);
    let mut pin = p.PA1;
    let mut vref = adc.enable_temperature(&mut Delay);

    loop {
        let v = adc.read(&mut pin);
        let r = adc.read_internal(&mut vref);
        info!("--> {} - {} mV / vref = {} - {} mV", v, adc.to_millivolts(v), r, adc.to_millivolts(r));
        Timer::after(Duration::from_millis(100)).await;
    }
}
