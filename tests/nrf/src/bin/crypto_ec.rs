// required-features: embassy-crypto
#![no_std]
#![no_main]

//! The PKA through the `embassy-crypto` curve arithmetic drivers, against the shared suites.
//! ECDH and ECDSA have their own binaries: together the suites exceed the farm's time limit.

#[path = "../common.rs"]
mod common;
#[cfg(feature = "cracen")]
#[path = "../ba414ep_ucode.rs"]
mod ucode;

use defmt::{error, info};
use defmt_rtt as _;
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

    // On CRACEN the drivers load the engine's microcode themselves, from what is registered.
    #[cfg(feature = "cracen")]
    embassy_nrf::crypto::pka::set_microcode(&ucode::BA414EP_UCODE);

    suites!(p256_arith, p384_arith);

    info!("Test OK");
    cortex_m::asm::bkpt();
}
