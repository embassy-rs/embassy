// required-features: pka
#![no_std]
#![no_main]

//! The PKA through the `embassy-crypto` ECDSA drivers, against the shared suites.

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

    // The `pka_v1c` of the STM32WB55 and WL55 takes half the time limit for P-256 alone.
    crypto_pka::suites!(
        p256_ecdsa,
        #[cfg(not(any(feature = "stm32wb55rg", feature = "stm32wl55jc")))]
        p384_ecdsa,
    );

    info!("Test OK");
    cortex_m::asm::bkpt();
}
