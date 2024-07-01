#![no_std]
#![no_main]
#[path = "../common.rs"]
mod common;

use common::*;
use defmt::assert;
use embassy_executor::Spawner;
use embassy_time::{Instant, Timer};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let _p = embassy_stm32::init(config());
    info!("Hello World!");

    let start = Instant::now();
    Timer::after_millis(100).await;
    let end = Instant::now();
    let ms = (end - start).as_millis();
    info!("slept for {} ms", ms);
    assert!(ms >= 99);
    assert!(ms < 110);

    info!("Test OK");
    cortex_m::asm::bkpt();
}
