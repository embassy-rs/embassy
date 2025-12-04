//! Example of using mathematical calculations performed by the MSPM0G3507 chip.
//!
//! It prints the result of basics mathematical calculation.

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
    let mut op1 = PI * 0.5;
    match macl.sin(op1, Precision::High) {
        Ok(res) => info!("sin({}) = {}", op1, res),
        Err(e) => error!("sin Error: {:?}", e),
    }

    match macl.cos(op1, Precision::Medium) {
        Ok(res) => info!("cos({}) = {}", op1, res),
        Err(e) => error!("cos Error: {:?}", e),
    }

    op1 = 1.0;
    let op2 = 3.0;
    match macl.div(op1, op2) {
        Ok(res) => info!("{}/{} = {}", op1, op2, res),
        Err(e) => error!("div Error: {:?}", e),
    }

    loop {
        Timer::after_millis(500).await;
    }
}
