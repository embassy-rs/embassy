// required-features: aes-ecb-cbc
#![no_std]
#![no_main]

//! The AES, CRYP or SAES peripheral through `embassy-crypto`, against the shared known-answer
//! suites. CTR and the authenticated modes only run where the peripheral has them (`aes-basic`
//! and `aes` features).

#[path = "../common.rs"]
mod common;
use common::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
#[cfg(feature = "aes-via-saes")]
use embassy_stm32::rng::Rng;
#[cfg(feature = "aes-via-saes")]
use embassy_stm32::{bind_interrupts, peripherals, rng};
use panic_probe as _;

// The known-answer blobs take a while to load.
teleprobe_meta::timeout!(60);

#[cfg(feature = "aes-via-saes")]
bind_interrupts!(struct Irqs {
    RNG => rng::InterruptHandler<peripherals::RNG>;
});

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
    #[allow(unused_variables)]
    let p: embassy_stm32::Peripherals = init();

    // The SAES draws random numbers from the RNG whenever it is reset.
    #[cfg(feature = "aes-via-saes")]
    let _rng = Rng::new(p.RNG, Irqs);

    suites!(aes128_ecb, aes128_cbc);
    #[cfg(feature = "aes-basic")]
    suites!(aes128_ctr);
    #[cfg(feature = "aes")]
    suites!(aes128_gcm, aes256_gcm, aes128_ccm);

    info!("Test OK");
    cortex_m::asm::bkpt();
}
