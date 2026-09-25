//! HKDF (RFC 5869).

use crate::{Error, HmacSha256, ct};

/// HKDF-SHA-256 (RFC 5869).
///
/// Built on [`HmacSha256`], so it is served by whichever HMAC-SHA-256 driver is
/// registered; see [`driver`](crate::driver).
///
/// # Example
///
/// ```ignore
/// let hk = HkdfSha256::new(b"salt", b"input key material");
/// let mut okm = [0u8; 42];
/// hk.expand(b"context", &mut okm)?;
///
/// // Or keep the pseudorandom key to expand later:
/// let (prk, _) = HkdfSha256::extract(b"salt", b"input key material");
/// HkdfSha256::from_prk(&prk)?.expand(b"context", &mut okm)?;
/// ```
#[derive(Clone)]
pub struct HkdfSha256 {
    mac: HmacSha256,
}

impl HkdfSha256 {
    /// Size of the pseudorandom key, in bytes.
    pub const PRK_SIZE: usize = HmacSha256::OUTPUT_SIZE;

    /// Largest output [`expand`](Self::expand) can produce, in bytes.
    pub const MAX_OUTPUT_SIZE: usize = 255 * HmacSha256::OUTPUT_SIZE;

    /// Extract a pseudorandom key from `ikm` with `salt`.
    ///
    /// An empty `salt` is equivalent to the RFC's default of `PRK_SIZE` zeros.
    pub fn new(salt: &[u8], ikm: &[u8]) -> Self {
        let (mut prk, hk) = Self::extract(salt, ikm);
        ct::zeroize(&mut prk);
        hk
    }

    /// Like [`new`](Self::new), also returning the pseudorandom key.
    pub fn extract(salt: &[u8], ikm: &[u8]) -> ([u8; Self::PRK_SIZE], Self) {
        let prk = HmacSha256::mac(salt, ikm);
        let hk = Self {
            mac: HmacSha256::new(&prk),
        };
        (prk, hk)
    }

    /// Skip the extract step, using `prk` as the pseudorandom key.
    ///
    /// Returns [`Error::InvalidKey`] if `prk` is shorter than `PRK_SIZE`.
    pub fn from_prk(prk: &[u8]) -> Result<Self, Error> {
        if prk.len() < Self::PRK_SIZE {
            return Err(Error::InvalidKey);
        }
        Ok(Self {
            mac: HmacSha256::new(prk),
        })
    }

    /// Fill `okm` with output keying material bound to `info`.
    ///
    /// Returns [`Error::InvalidInput`] if `okm` is longer than `MAX_OUTPUT_SIZE`.
    pub fn expand(&self, info: &[u8], okm: &mut [u8]) -> Result<(), Error> {
        if okm.len() > Self::MAX_OUTPUT_SIZE {
            return Err(Error::InvalidInput);
        }
        let mut t = [0u8; Self::PRK_SIZE];
        for (i, chunk) in okm.chunks_mut(Self::PRK_SIZE).enumerate() {
            let mut mac = self.mac.clone();
            if i > 0 {
                mac.update(&t);
            }
            mac.update(info);
            mac.update(&[i as u8 + 1]);
            t = mac.finalize();
            chunk.copy_from_slice(&t[..chunk.len()]);
        }
        ct::zeroize(&mut t);
        Ok(())
    }
}

impl core::fmt::Debug for HkdfSha256 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("HkdfSha256").finish_non_exhaustive()
    }
}
