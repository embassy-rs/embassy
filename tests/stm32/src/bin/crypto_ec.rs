// required-features: pka
#![no_std]
#![no_main]

//! The PKA through the `embassy-crypto` curve arithmetic drivers, against the shared suites.
//! ECDH and ECDSA have their own binaries: together the suites exceed the farm's time limit.

#[path = "../common.rs"]
mod common;
#[path = "../crypto_pka.rs"]
mod crypto_pka;
use common::*;
use embassy_executor::Spawner;

teleprobe_meta::timeout!(60);

#[cfg_attr(
    feature = "stop",
    embassy_executor::main(executor = "embassy_stm32::executor::Executor", entry = "cortex_m_rt::entry")
)]
#[cfg_attr(not(feature = "stop"), embassy_executor::main)]
async fn main(_spawner: Spawner) {
    let _rng = crypto_pka::init();

    crypto_pka::suites!(p256_arith, p384_arith);

    info!("Test OK");
    cortex_m::asm::bkpt();
}
