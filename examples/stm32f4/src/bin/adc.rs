#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt_rtt as _; // global logger
use panic_probe as _;

use defmt::*;
use embassy::executor::Spawner;
use embassy::time::{Delay, Duration, Timer};
use embassy_stm32::adc::Adc;
use embassy_stm32::Peripherals;

#[embassy::main]
async fn main(_spawner: Spawner, p: Peripherals) {
    info!("Hello World!");

    let mut adc = Adc::new(p.ADC1, &mut Delay);
    let mut pin = p.PC1;

    loop {
        let v = adc.read(&mut pin);
        info!("--> {} - {} mV", v, adc.to_millivolts(v));
        Timer::after(Duration::from_millis(100)).await;
    }
}
