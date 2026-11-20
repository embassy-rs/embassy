// required-features: hash
#![no_std]
#![no_main]

//! The HASH peripheral through `embassy-crypto`, against the shared known-answer suites.

#[path = "../common.rs"]
mod common;
use common::*;
use embassy_executor::Spawner;

// The known-answer blobs take a while to load.
teleprobe_meta::timeout!(60);

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

#[embassy_executor::main(executor = "common::Executor", entry = "cortex_m_rt::entry")]
async fn main(_spawner: Spawner) {
    let _p: embassy_stm32::Peripherals = init();

    suites!(sha1, sha256, hmac_sha256);

    info!("Test OK");
    cortex_m::asm::bkpt();
}
