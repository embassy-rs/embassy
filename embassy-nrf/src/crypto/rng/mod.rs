//! True random number generator of the cryptographic accelerator.
//!
//! The generator is owned through the `CRYPTO_RNG` peripheral.
//!
//! - On the CryptoCell (nRF52840, nRF5340, nRF91) the driver has a blocking API and an
//!   interrupt-driven async API.
//! - On CRACEN (nRF54L) it is blocking only.
//!
//! Both implement the `rand_core` traits.

#[cfg_attr(feature = "_cryptocell", path = "cryptocell.rs")]
#[cfg_attr(feature = "_cracen", path = "cracen.rs")]
mod hw;

pub use hw::*;

#[cfg(feature = "embassy-crypto-rng")]
mod driver {
    use embassy_sync::blocking_mutex::Mutex;
    use embassy_sync::blocking_mutex::raw::PanicRawMutex;

    use super::Rng;
    use crate::mode::Blocking;

    static LOCK: Mutex<PanicRawMutex, ()> = Mutex::new(());

    struct Driver;

    impl embassy_crypto::driver::Rng for Driver {
        fn fill_bytes(buf: &mut [u8]) {
            LOCK.lock(|_| {
                let mut rng = Rng::<'static, Blocking>::new_inner();
                rng.blocking_fill_bytes(buf);
            })
        }
    }

    embassy_crypto::rng_impl!(Driver);
}
