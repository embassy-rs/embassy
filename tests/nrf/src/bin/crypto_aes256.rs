// required-features: aes256
#![no_std]
#![no_main]

//! AES-256 through `embassy-crypto`, against the shared known-answer suites.

#[path = "../common.rs"]
mod common;
use defmt::{error, info};
use defmt_rtt as _;
use embassy_executor::Spawner;
use panic_probe as _;

/// Run every suite, logging each result, and fail at the end if any failed.
macro_rules! suites {
    ($($(#[$meta:meta])* $name:ident),* $(,)?) => {{
        let mut ok = true;
        $(
            $(#[$meta])*
            match embassy_crypto_tests::$name() {
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

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let _p = embassy_nrf::init(Default::default());

    suites!(aes256_ecb, aes256_cbc, aes256_ctr, aes256_cmac, aes256_ccm, aes256_gcm);

    info!("Test OK");
    cortex_m::asm::bkpt();
}
