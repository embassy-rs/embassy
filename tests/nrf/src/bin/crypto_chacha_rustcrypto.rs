// required-features: embassy-crypto-rustcrypto
#![no_std]
#![no_main]

//! ChaCha through the `embassy-crypto-rustcrypto` software drivers, against the shared
//! known-answer suite.
//!
//! The vectors start at block counter 0 and wrap the 32-bit counter, which used to make the
//! driver loop forever on targets with a 32-bit `usize`. Host tests cannot catch that.

#[path = "../common.rs"]
mod common;
use defmt::{error, info};
use defmt_rtt as _;
use embassy_crypto_rustcrypto as _;
use embassy_executor::Spawner;
use panic_probe as _;

teleprobe_meta::timeout!(120);

/// Run every suite, logging each result, and fail at the end if any failed.
macro_rules! suites {
    ($($(#[$meta:meta])* $name:ident),* $(,)?) => {{
        let mut ok = true;
        $(
            $(#[$meta])*
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

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let _p = embassy_nrf::init(Default::default());

    suites!(chacha8, chacha12, chacha20);

    info!("Test OK");
    cortex_m::asm::bkpt();
}
