#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::rcc::{Mco, Mco1Source, McoConfig, McoPrescaler};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let mut led = Output::new(p.PB14, Level::High, Speed::Low);

    let config = {
        let mut config = McoConfig::default();
        config.prescaler = McoPrescaler::DIV8;
        config
    };

    let _mco = Mco::new(p.MCO1, p.PA8, Mco1Source::HSI, config);

    loop {
        info!("high");
        led.set_high();
        Timer::after_millis(500).await;

        info!("low");
        led.set_low();
        Timer::after_millis(500).await;
    }
}
