#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use defmt_rtt as _;
use embassy::executor::Spawner;
use embassy::time::{Duration, Timer};
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::Peripherals;
// global logger
use panic_probe as _;

#[embassy::main]
async fn main(_spawner: Spawner, p: Peripherals) -> ! {
    info!("Hello World!");

    let mut led = Output::new(p.PH7, Level::Low, Speed::Medium);

    loop {
        defmt::info!("on!");
        led.set_low();
        Timer::after(Duration::from_millis(200)).await;

        defmt::info!("off!");
        led.set_high();
        Timer::after(Duration::from_millis(200)).await;
    }
}
