#![no_std]
#![no_main]
#![feature(min_type_alias_impl_trait)]
#![feature(impl_trait_in_bindings)]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;
use example_common::*;

use defmt::panic;
use embassy;
use embassy::executor::Spawner;
use embassy::task;
use embassy::time::{Duration, Timer};
use embassy_stm32f4;
use embassy_stm32f4::hal;

#[task]
async fn run1() {
    loop {
        info!("BIG INFREQUENT TICK");
        Timer::after(Duration::from_ticks(32768 * 2 as u64)).await;
    }
}

#[task]
async fn run2() {
    loop {
        info!("tick");
        Timer::after(Duration::from_ticks(13000 as u64)).await;
    }
}

#[embassy::main(use_hse = 16)]
async fn main(spawner: Spawner) {
    let (dp, clocks) = embassy_stm32::Peripherals::take_with_clocks().unwrap();

    spawner.spawn(run1()).unwrap();
}
