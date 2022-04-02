#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use defmt_rtt as _; // global logger
use embassy::executor::Spawner;
use embassy::time::{Duration, Timer};
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::rcc::{Mco, Mco1Source, McoClock};
use embassy_stm32::Peripherals;
use panic_probe as _;

#[embassy::main]
async fn main(_spawner: Spawner, p: Peripherals) {
    info!("Hello World!");

    let mut led = Output::new(p.PB14, Level::High, Speed::Low);

    let _mco = Mco::new(p.MCO1, p.PA8, Mco1Source::Hsi, McoClock::Divided(8));

    loop {
        info!("high");
        led.set_high();
        Timer::after(Duration::from_millis(500)).await;

        info!("low");
        led.set_low();
        Timer::after(Duration::from_millis(500)).await;
    }
}
