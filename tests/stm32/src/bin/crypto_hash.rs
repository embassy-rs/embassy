// required-features: hash
#![no_std]
#![no_main]

//! The HASH peripheral through `embassy-crypto`, against the shared known-answer suites.

#[path = "../common.rs"]
mod common;
use common::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use panic_probe as _;

/// Run every suite, logging each result, and fail at the end if any failed.
macro_rules! suites {
    ($($name:ident),* $(,)?) => {{
        let mut ok = true;
        $(
            match embassy_crypto_test::$name() {
                Ok(stats) => info!("{}: {:?}", stringify!($name), stats),
                Err(e) => {
                    error!("{}: {:?}", stringify!($name), e);
                    ok = false;
                }
            }
        )*
        defmt::assert!(ok, "some suites failed");
    }};
}

#[cfg_attr(
    feature = "stop",
    embassy_executor::main(executor = "embassy_stm32::executor::Executor", entry = "cortex_m_rt::entry")
)]
#[cfg_attr(not(feature = "stop"), embassy_executor::main)]
async fn main(_spawner: Spawner) {
    let _p: embassy_stm32::Peripherals = init();

    suites!(sha1, sha256, hmac_sha256);

    info!("Test OK");
    cortex_m::asm::bkpt();
}
