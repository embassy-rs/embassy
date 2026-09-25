// required-features: pka
#![no_std]
#![no_main]

//! The PKA through the `embassy-crypto` ECDH drivers, against the shared suites.

#[path = "../common.rs"]
mod common;
#[path = "../crypto_pka.rs"]
mod crypto_pka;
use common::*;
use embassy_executor::Spawner;

teleprobe_meta::timeout!(120);

#[cfg_attr(
    feature = "stop",
    embassy_executor::main(executor = "embassy_stm32::executor::Executor", entry = "cortex_m_rt::entry")
)]
#[cfg_attr(not(feature = "stop"), embassy_executor::main)]
async fn main(_spawner: Spawner) {
    let _rng = crypto_pka::init();

    // The P-384 suite runs some 1500 scalar multiplications, at 85 ms each on the STM32WBA52 at
    // its top clock, and slower still on the `pka_v1c` of the STM32WB55 and WL55: more than the
    // farm's time limit.
    crypto_pka::suites!(
        p256_ecdh,
        #[cfg(not(any(feature = "stm32wba52cg", feature = "stm32wb55rg", feature = "stm32wl55jc")))]
        p384_ecdh,
    );

    info!("Test OK");
    cortex_m::asm::bkpt();
}
