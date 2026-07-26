#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::exti::{self, ExtiInput};
use embassy_stm32::gpio::Pull;
use embassy_stm32::{bind_interrupts, interrupt};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(
    pub struct Irqs{
        EXTI4_15 => exti::InterruptHandler<interrupt::typelevel::EXTI4_15>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Initialize and create handle for devicer peripherals
    let p = embassy_stm32::init(Default::default());
    // Configure the button pin and obtain handler.
    // On the Nucleo F091RC there is a button connected to pin PC13.
    let mut button = ExtiInput::new(p.PC13, p.EXTI13, Pull::Down, Irqs);

    info!("Press the USER button...");
    loop {
        button.wait_for_falling_edge().await;
        info!("Pressed!");
        button.wait_for_rising_edge().await;
        info!("Released!");
    }
}
