#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;
use defmt::assert;
use embassy::executor::Spawner;
use embassy::time::{Duration, Instant, Timer};
use embassy_stm32::Peripherals;
use example_common::*;

#[embassy::main]
async fn main(_spawner: Spawner, _p: Peripherals) {
    info!("Hello World!");

    let start = Instant::now();
    Timer::after(Duration::from_millis(100)).await;
    let end = Instant::now();
    let ms = (end - start).as_millis();
    info!("slept for {} ms", ms);
    assert!(ms >= 99);
    assert!(ms < 110);

    info!("Test OK");
    cortex_m::asm::bkpt();
}
