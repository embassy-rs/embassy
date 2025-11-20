#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_stm32::bind_interrupts;
use embassy_stm32::exti::{self, ExtiInput};
use embassy_stm32::gpio::Pull;
use embassy_stm32::interrupt;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(
    pub struct Irqs{
        EXTI2_3 => exti::InterruptHandler<interrupt::typelevel::EXTI2_3>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let config = Config::default();
    let p = embassy_stm32::init(config);

    let mut button = ExtiInput::new(
        p.PB2,
        p.EXTI2,
        Pull::Up,
        Irqs::as_any::<interrupt::typelevel::EXTI2_3, exti::InterruptHandler<interrupt::typelevel::EXTI2_3>>(),
    );

    info!("Press the USER button...");

    loop {
        button.wait_for_falling_edge().await;
        info!("Pressed!");
        button.wait_for_rising_edge().await;
        info!("Released!");
    }
}
