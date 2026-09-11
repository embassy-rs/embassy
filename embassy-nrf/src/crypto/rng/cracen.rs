//! CRACEN true random number generator.

use core::marker::PhantomData;

use crate::crypto::cracen::{ActivationHandle, activate, ensure_rng_running, read_rng_word};
use crate::mode::Mode;

/// Driver for the true random number generator.
///
/// The AES and public key engines also draw from the generator for their side-channel
/// countermeasures. It keeps running from its first use until the accelerator powers down.
pub struct Rng<'d, M: Mode> {
    _activation: ActivationHandle,
    _phantom: PhantomData<(&'d (), M)>,
}

#[cfg(not(feature = "embassy-crypto-rng"))]
impl<'d> Rng<'d, crate::mode::Blocking> {
    /// Creates a new blocking RNG driver.
    pub fn new_blocking(_peri: crate::Peri<'d, crate::peripherals::CRYPTO_RNG>) -> Self {
        Self::new_inner()
    }
}

impl<'d, M: Mode> Rng<'d, M> {
    // Used by the `embassy-crypto` driver, which has no peripheral token.
    pub(crate) fn new_inner() -> Self {
        Self {
            _activation: activate(),
            _phantom: PhantomData,
        }
    }

    /// Fills the buffer with random bytes.
    pub fn blocking_fill_bytes(&mut self, dest: &mut [u8]) {
        if dest.is_empty() {
            return;
        }

        ensure_rng_running();

        for chunk in dest.chunks_mut(4) {
            let word = read_rng_word().to_ne_bytes();
            let to_copy = word.len().min(chunk.len());
            chunk[..to_copy].copy_from_slice(&word[..to_copy]);
        }
    }

    /// Returns a random `u32`.
    pub fn blocking_next_u32(&mut self) -> u32 {
        let mut bytes = [0; 4];
        self.blocking_fill_bytes(&mut bytes);
        // We don't care about the endianness, so just use the native one.
        u32::from_ne_bytes(bytes)
    }

    /// Returns a random `u64`.
    pub fn blocking_next_u64(&mut self) -> u64 {
        let mut bytes = [0; 8];
        self.blocking_fill_bytes(&mut bytes);
        u64::from_ne_bytes(bytes)
    }
}

impl<'d, M: Mode> rand_core_06::RngCore for Rng<'d, M> {
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.blocking_fill_bytes(dest);
    }
    fn next_u32(&mut self) -> u32 {
        self.blocking_next_u32()
    }
    fn next_u64(&mut self) -> u64 {
        self.blocking_next_u64()
    }
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core_06::Error> {
        self.blocking_fill_bytes(dest);
        Ok(())
    }
}

impl<'d, M: Mode> rand_core_06::CryptoRng for Rng<'d, M> {}

impl<'d, M: Mode> rand_core_09::RngCore for Rng<'d, M> {
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.blocking_fill_bytes(dest);
    }
    fn next_u32(&mut self) -> u32 {
        self.blocking_next_u32()
    }
    fn next_u64(&mut self) -> u64 {
        self.blocking_next_u64()
    }
}

impl<'d, M: Mode> rand_core_09::CryptoRng for Rng<'d, M> {}

impl<'d, M: Mode> rand_core_10::TryRng for Rng<'d, M> {
    type Error = core::convert::Infallible;

    fn try_next_u32(&mut self) -> Result<u32, Self::Error> {
        Ok(self.blocking_next_u32())
    }

    fn try_next_u64(&mut self) -> Result<u64, Self::Error> {
        Ok(self.blocking_next_u64())
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Self::Error> {
        self.blocking_fill_bytes(dest);
        Ok(())
    }
}

impl<'d, M: Mode> rand_core_10::TryCryptoRng for Rng<'d, M> {}
