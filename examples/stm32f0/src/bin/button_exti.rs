#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{Input, Pull};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Initialize and create handle for devicer peripherals
    let p = embassy_stm32::init(Default::default());
    // Configure the button pin and obtain handler.
    // On the Nucleo F091RC there is a button connected to pin PC13.
    let button = Input::new(p.PC13, Pull::Down);
    let mut button = ExtiInput::new(button, p.EXTI13);

    info!("Press the USER button...");
    loop {
        button.wait_for_falling_edge().await;
        info!("Pressed!");
        button.wait_for_rising_edge().await;
        info!("Released!");
    }
}
