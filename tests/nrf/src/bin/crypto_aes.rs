// required-features: crypto
#![no_std]
#![no_main]

//! AES-128 through `embassy-crypto`, against the shared known-answer suites.

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

    suites!(
        aes128_ecb,
        aes128_cbc,
        aes128_ctr,
        aes128_cmac,
        aes128_ccm,
        #[cfg(feature = "aes256")]
        aes128_gcm,
    );

    info!("Test OK");
    cortex_m::asm::bkpt();
}
