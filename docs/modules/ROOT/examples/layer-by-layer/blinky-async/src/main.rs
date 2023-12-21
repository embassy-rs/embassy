#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{Input, Level, Output, Pull, Speed};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    let mut led = Output::new(p.PB14, Level::Low, Speed::VeryHigh);
    let mut button = ExtiInput::new(Input::new(p.PC13, Pull::Up), p.EXTI13);

    loop {
        button.wait_for_any_edge().await;
        if button.is_low() {
            led.set_high();
        } else {
            led.set_low();
        }
    }
}
