#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[cfg(feature = "defmt-rtt")]
use defmt_rtt::*;
use embassy_executor::executor::Spawner;
use embassy_executor::time::{Duration, Timer};
use embassy_stm32::gpio::{Level, Output, Speed};
use panic_reset as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    Timer::after(Duration::from_millis(300)).await;
    let mut led = Output::new(p.PB14, Level::High, Speed::Low);
    led.set_high();

    loop {
        led.set_high();
        Timer::after(Duration::from_millis(500)).await;

        led.set_low();
        Timer::after(Duration::from_millis(500)).await;
    }
}
