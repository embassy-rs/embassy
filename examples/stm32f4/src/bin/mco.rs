#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::rcc::{Mco, Mco1Source, Mco2Source, McoPrescaler};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let _mco1 = Mco::new(p.MCO1, p.PA8, Mco1Source::HSI, McoPrescaler::DIV1);
    let _mco2 = Mco::new(p.MCO2, p.PC9, Mco2Source::PLL, McoPrescaler::DIV4);
    let mut led = Output::new(p.PB7, Level::High, Speed::Low);

    loop {
        info!("high");
        led.set_high();
        Timer::after_millis(300).await;

        info!("low");
        led.set_low();
        Timer::after_millis(300).await;
    }
}
