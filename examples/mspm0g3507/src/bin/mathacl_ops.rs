//! Example of using mathematical calculations performed by the MSPM0G3507 chip.
//!
//! It prints the result of basics trigonometric calculation.

#![no_std]
#![no_main]

use core::f32::consts::PI;

use defmt::*;
use embassy_executor::Spawner;
use embassy_mspm0::mathacl::{Mathacl, Precision};
use embassy_time::Timer;
use {defmt_rtt as _, panic_halt as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    info!("Hello world!");

    let d = embassy_mspm0::init(Default::default());

    let mut macl = Mathacl::new(d.MATHACL);

    // value radians [-PI; PI]
    let rads = PI * 0.5;
    match macl.sin(rads, Precision::High) {
        Ok(res) => info!("sin({}) = {}", rads, res),
        Err(e) => error!("sin Error: {:?}", e),
    }

    match macl.cos(rads, Precision::Medium) {
        Ok(res) => info!("cos({}) = {}", rads, res),
        Err(e) => error!("cos Error: {:?}", e),
    }

    loop {
        Timer::after_millis(500).await;
    }
}
