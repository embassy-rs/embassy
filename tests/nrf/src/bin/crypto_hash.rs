// required-features: crypto
#![no_std]
#![no_main]

//! The hash accelerator through `embassy-crypto`, against the shared known-answer suites.

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
        sha1,
        sha224,
        sha256,
        hmac_sha1,
        hmac_sha224,
        hmac_sha256,
        #[cfg(feature = "sha512")]
        sha384,
        #[cfg(feature = "sha512")]
        sha512,
        #[cfg(feature = "sha512")]
        sha512_224,
        #[cfg(feature = "sha512")]
        sha512_256,
        #[cfg(feature = "sha512")]
        hmac_sha384,
        #[cfg(feature = "sha512")]
        hmac_sha512,
        #[cfg(feature = "sha512")]
        hmac_sha512_224,
        #[cfg(feature = "sha512")]
        hmac_sha512_256,
    );

    info!("Test OK");
    cortex_m::asm::bkpt();
}
