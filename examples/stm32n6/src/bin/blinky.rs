#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::exti::{self, ExtiInput};
use embassy_stm32::gpio::{Level, Output, Pull, Speed};
use embassy_stm32::{bind_interrupts, interrupt};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(
    pub struct Irqs{
        EXTI13 => exti::InterruptHandler<interrupt::typelevel::EXTI13>;
});

#[embassy_executor::task]
async fn button_task(mut button: ExtiInput<'static>) {
    loop {
        button.wait_for_any_edge().await;
        info!("button pressed!");
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let mut led = Output::new(p.PG10, Level::High, Speed::Low);
    // Note: On STM32N6570-DK, the USER button (BP1) connects to 3.3V when pressed
    // (active high), so we need Pull::Down
    let button = ExtiInput::new(p.PC13, p.EXTI13, Pull::Down, Irqs);

    spawner.spawn(button_task(button).unwrap());

    loop {
        led.set_high();
        Timer::after_millis(500).await;

        led.set_low();
        Timer::after_millis(500).await;
    }
}
