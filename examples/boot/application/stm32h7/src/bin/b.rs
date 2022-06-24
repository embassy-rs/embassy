#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[cfg(feature = "defmt-rtt")]
use defmt_rtt::*;
use embassy::executor::Spawner;
use embassy::time::{Duration, Timer};
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::Peripherals;
use panic_reset as _;

#[embassy::main]
async fn main(_spawner: Spawner, p: Peripherals) {
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
