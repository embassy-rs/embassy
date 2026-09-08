#![no_std]
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

use embassy_crypto::Error;
use embassy_crypto::driver::Rng;
use rand::TryRngCore;
use rand::rngs::OsRng;

/// The operating system's random number generator, registered as the global
/// [`embassy_crypto::driver::Rng`].
struct Driver;

impl Rng for Driver {
    fn fill_bytes(buf: &mut [u8]) -> Result<(), Error> {
        OsRng.try_fill_bytes(buf).map_err(|_| Error::HardwareError)
    }
}

embassy_crypto::rng_impl!(Driver);

#[cfg(test)]
mod tests {
    #[test]
    fn fills_random_bytes() {
        let mut a = [0u8; 32];
        let mut b = [0u8; 32];
        embassy_crypto::driver::RngImpl::fill_bytes(&mut a).unwrap();
        embassy_crypto::driver::RngImpl::fill_bytes(&mut b).unwrap();
        assert_ne!(a, [0u8; 32]);
        assert_ne!(a, b);
    }
}
