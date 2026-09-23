// required-features: embassy-crypto
#![no_std]
#![no_main]

//! The PKA through the `embassy-crypto` ECDH drivers, against the shared suites.

#[path = "../common.rs"]
mod common;
#[cfg(feature = "cracen")]
#[path = "../ba414ep_ucode.rs"]
mod ucode;

use defmt::{error, info};
use defmt_rtt as _;
use embassy_executor::Spawner;
use panic_probe as _;

// P-384 ECDH alone takes about 80 s on the nRF52840.
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

    suites!(p256_ecdh, p384_ecdh);

    info!("Test OK");
    cortex_m::asm::bkpt();
}
