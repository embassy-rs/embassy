#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::adc::{Adc, Config};
use embassy_rp::interrupt;
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let irq = interrupt::take!(ADC_IRQ_FIFO);
    let mut adc = Adc::new(p.ADC, irq, Config::default());

    let mut p26 = p.PIN_26;
    let mut p27 = p.PIN_27;
    let mut p28 = p.PIN_28;

    loop {
        let level = adc.read(&mut p26).await;
        info!("Pin 26 ADC: {}", level);
        let level = adc.read(&mut p27).await;
        info!("Pin 27 ADC: {}", level);
        let level = adc.read(&mut p28).await;
        info!("Pin 28 ADC: {}", level);
        let temp = adc.read_temperature().await;
        info!("Temp: {}", temp);
        Timer::after(Duration::from_secs(1)).await;
    }
}
