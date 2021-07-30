#![no_std]
#![no_main]
#![feature(trait_alias)]
#![feature(min_type_alias_impl_trait)]
#![feature(impl_trait_in_bindings)]
#![feature(type_alias_impl_trait)]
#![allow(incomplete_features)]

#[path = "../example_common.rs"]
mod example_common;

use defmt::panic;
use embassy::executor::Spawner;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{Input, Pull};
use embassy_stm32::{rcc, Peripherals};
use embassy_traits::gpio::{WaitForFallingEdge, WaitForRisingEdge};
use example_common::*;

#[embassy::main]
async fn main(_spawner: Spawner, mut p: Peripherals) {
    let mut rcc = rcc::Rcc::new(p.RCC);
    rcc.enable_debug_wfe(&mut p.DBGMCU, true);
    // Enables SYSCFG
    let _ = rcc.enable_hsi48(&mut p.SYSCFG, p.CRS);

    let button = Input::new(p.PB2, Pull::Up);
    let mut button = ExtiInput::new(button, p.EXTI2);

    info!("Press the USER button...");

    loop {
        button.wait_for_falling_edge().await;
        info!("Pressed!");
        button.wait_for_rising_edge().await;
        info!("Released!");
    }
}
