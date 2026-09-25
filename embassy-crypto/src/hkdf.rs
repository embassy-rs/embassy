//! HKDF (RFC 5869).

use crate::{Error, HmacSha256, HmacSha384, ct};

macro_rules! impl_hkdf {
    ($(#[$meta:meta])* $name:ident, $hmac:ident) => {
        $(#[$meta])*
        ///
        #[doc = concat!("Built on [`", stringify!($hmac), "`], so it is served by whichever driver is")]
        /// registered for it; see [`driver`](crate::driver).
        ///
        /// # Example
        ///
        /// ```ignore
        #[doc = concat!("let hk = ", stringify!($name), "::new(b\"salt\", b\"input key material\");")]
        /// let mut okm = [0u8; 42];
        /// hk.expand(b"context", &mut okm)?;
        ///
        /// // Or keep the pseudorandom key to expand later:
        #[doc = concat!("let (prk, _) = ", stringify!($name), "::extract(b\"salt\", b\"input key material\");")]
        #[doc = concat!(stringify!($name), "::from_prk(&prk)?.expand(b\"context\", &mut okm)?;")]
        /// ```
        #[derive(Clone)]
        pub struct $name {
            mac: $hmac,
        }

        impl $name {
            /// Size of the pseudorandom key, in bytes.
            pub const PRK_SIZE: usize = $hmac::OUTPUT_SIZE;

            /// Largest output [`expand`](Self::expand) can produce, in bytes.
            pub const MAX_OUTPUT_SIZE: usize = 255 * $hmac::OUTPUT_SIZE;

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
                let prk = $hmac::mac(salt, ikm);
                let hk = Self { mac: $hmac::new(&prk) };
                (prk, hk)
            }

            /// Skip the extract step, using `prk` as the pseudorandom key.
            ///
            /// Returns [`Error::InvalidKey`] if `prk` is shorter than `PRK_SIZE`.
            pub fn from_prk(prk: &[u8]) -> Result<Self, Error> {
                if prk.len() < Self::PRK_SIZE {
                    return Err(Error::InvalidKey);
                }
                Ok(Self { mac: $hmac::new(prk) })
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

        impl core::fmt::Debug for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.debug_struct(stringify!($name)).finish_non_exhaustive()
            }
        }
    };
}

impl_hkdf!(
    /// HKDF-SHA-256 (RFC 5869).
    HkdfSha256, HmacSha256
);
impl_hkdf!(
    /// HKDF-SHA-384 (RFC 5869).
    HkdfSha384, HmacSha384
);
