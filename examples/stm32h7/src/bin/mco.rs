#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::rcc::{Mco, Mco1Source};
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let mut led = Output::new(p.PB14, Level::High, Speed::Low);

    let _mco = Mco::new(p.MCO1, p.PA8, Mco1Source::HSI, 8);

    loop {
        info!("high");
        led.set_high();
        Timer::after(Duration::from_millis(500)).await;

        info!("low");
        led.set_low();
        Timer::after(Duration::from_millis(500)).await;
    }
}
