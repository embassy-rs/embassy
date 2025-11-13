#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{Level, Output, Pull, Speed};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::task]
async fn button_task(mut p: ExtiInput<'static>) {
    loop {
        p.wait_for_any_edge().await;
        info!("button pressed!");
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let mut led = Output::new(p.PG10, Level::High, Speed::Low);
    let button = ExtiInput::new(p.PC13, p.EXTI13, Pull::Up);

    spawner.spawn(button_task(button).unwrap());

    loop {
        led.set_high();
        Timer::after_millis(500).await;

        led.set_low();
        Timer::after_millis(500).await;
    }
}
