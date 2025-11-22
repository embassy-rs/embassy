#![no_std]
#![no_main]

use core::mem::MaybeUninit;

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::exti::{self, ExtiInput};
use embassy_stm32::gpio::Pull;
use embassy_stm32::{SharedData, bind_interrupts, interrupt};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(
    pub struct Irqs{
        EXTI0 => exti::InterruptHandler<interrupt::typelevel::EXTI0>;
});

#[unsafe(link_section = ".shared_data")]
static SHARED_DATA: MaybeUninit<SharedData> = MaybeUninit::uninit();

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init_primary(Default::default(), &SHARED_DATA);
    info!("Hello World!");

    let mut button = ExtiInput::new(p.PA0, p.EXTI0, Pull::Up, Irqs);

    info!("Press the USER button...");

    loop {
        button.wait_for_falling_edge().await;
        info!("Pressed!");
        button.wait_for_rising_edge().await;
        info!("Released!");
    }
}
