#![no_std]
#![no_main]

#[cfg(feature = "defmt-rtt")]
use defmt_rtt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_time::Timer;
use panic_reset as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    Timer::after_millis(300).await;
    let mut led = Output::new(p.PB7, Level::High, Speed::Low);
    led.set_high();

    loop {
        led.set_high();
        Timer::after_millis(500).await;

        led.set_low();
        Timer::after_millis(500).await;
    }
}
