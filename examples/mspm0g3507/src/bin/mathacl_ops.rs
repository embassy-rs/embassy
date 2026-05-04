//! Example of using mathematical calculations performed by the MSPM0G3507 chip.
//!
//! It prints the result of basics mathematical calculation.

#![no_std]
#![no_main]

use core::f32::consts::PI;

use defmt::*;
use embassy_executor::Spawner;
use embassy_mspm0::mathacl::{IQType, Mathacl, Precision};
use embassy_time::Timer;
use {defmt_rtt as _, panic_halt as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    info!("Hello world!");

    let d = embassy_mspm0::init(Default::default());

    let mut macl = Mathacl::new(d.MATHACL);

    // value radians [-PI; PI]
    let op1 = PI * 0.5;
    match macl.sin(op1, Precision::High) {
        Ok(res) => info!("sin({}) = {}", op1, res),
        Err(e) => error!("sin Error: {:?}", e),
    }

    match macl.cos(op1, Precision::Medium) {
        Ok(res) => info!("cos({}) = {}", op1, res),
        Err(e) => error!("cos Error: {:?}", e),
    }

    // 1 bit for the sign, 15 bits integer & 16 bits fractional part
    let div_iq_s1 = IQType::from_f32(-1.0, 15, true).unwrap();
    let div_iq_s2 = IQType::from_f32(3.0, 15, true).unwrap();
    match macl.div_iq(div_iq_s1, div_iq_s2) {
        Ok(res) => info!("div IQ signed {}/{} = {}", div_iq_s1.to_f32(), div_iq_s2.to_f32(), res),
        Err(e) => error!("div IQ signed Error: {:?}", e),
    }

    // 16 bits integer & 16 bits fractional part
    let div_iq_u1 = IQType::from_f32(1.0, 16, false).unwrap();
    let div_iq_u2 = IQType::from_f32(3.0, 16, false).unwrap();
    match macl.div_iq(div_iq_u1, div_iq_u2) {
        Ok(res) => info!(
            "div IQ unsigned {}/{} = {}",
            div_iq_u1.to_f32(),
            div_iq_u2.to_f32(),
            res
        ),
        Err(e) => error!("div IQ unsigned Error: {:?}", e),
    }

    let div1_i32 = -5;
    let div2_i32 = -2;
    match macl.div_i32(div1_i32, div2_i32) {
        Ok(res) => info!("div i32 {}/{} = {}", div1_i32, div2_i32, res),
        Err(e) => error!("div i32 Error: {:?}", e),
    }

    let div1_u32 = 5;
    let div2_u32 = 2;
    match macl.div_u32(div1_u32, div2_u32) {
        Ok(res) => info!("div u32 {}/{} = {}", div1_u32, div2_u32, res),
        Err(e) => error!("div u32 Error: {:?}", e),
    }

    loop {
        Timer::after_millis(500).await;
    }
}
