//! X25519 (Curve25519 ECDH, RFC 7748).
//!
//! Served by [`driver::X25519`].
//!
//! # Example
//!
//! ```ignore
//! use embassy_crypto::x25519::{SecretKey, PublicKey};
//!
//! let mine = SecretKey::generate()?;
//! let peer = PublicKey::from_bytes(&peer_bytes);
//! let shared = mine.diffie_hellman(&peer)?;
//! // feed shared.as_bytes() to a KDF
//! ```

use crate::driver::{self, RngImpl, X25519PublicKey, X25519SecretKey};
use crate::{Error, ct};

/// An X25519 private key: 32 bytes, little-endian.
///
/// Stored unclamped; clamping (RFC 7748) is applied by the driver as part of
/// the X25519 function. Zeroed on drop.
#[derive(Clone)]
pub struct SecretKey(X25519SecretKey);

impl SecretKey {
    /// Generate a random private key from [`driver::RngImpl`].
    pub fn generate() -> Result<Self, Error> {
        let mut k = X25519SecretKey([0u8; 32]);
        RngImpl::fill_bytes(&mut k.0);
        Ok(Self(k))
    }

    /// Load a private key.
    pub fn from_bytes(bytes: &[u8; 32]) -> Self {
        Self(X25519SecretKey(*bytes))
    }

    /// The private key bytes.
    pub fn to_bytes(&self) -> [u8; 32] {
        self.0.0
    }

    /// The corresponding public key.
    pub fn public_key(&self) -> Result<PublicKey, Error> {
        driver::X25519Impl::public_key(&self.0).map(PublicKey)
    }

    /// The shared secret with `peer`.
    ///
    /// Fails with [`Error::InvalidKey`] if `peer` is a low-order point, which
    /// makes the shared secret all zeros (RFC 7748 section 6.1, required by
    /// TLS 1.3).
    pub fn diffie_hellman(&self, peer: &PublicKey) -> Result<SharedSecret, Error> {
        let shared = driver::X25519Impl::shared_secret(&self.0, &peer.0)?;
        if ct::is_zero(&shared) {
            return Err(Error::InvalidKey);
        }
        Ok(SharedSecret(shared))
    }
}

impl Drop for SecretKey {
    fn drop(&mut self) {
        ct::zeroize(&mut self.0.0);
    }
}

impl core::fmt::Debug for SecretKey {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("SecretKey(..)")
    }
}

/// An X25519 public key: a 32-byte u-coordinate, little-endian.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PublicKey(X25519PublicKey);

impl PublicKey {
    /// Load a public key. Every 32-byte string is a valid X25519 public key.
    pub fn from_bytes(bytes: &[u8; 32]) -> Self {
        Self(X25519PublicKey(*bytes))
    }

    /// The public key bytes.
    pub fn to_bytes(&self) -> [u8; 32] {
        self.0.0
    }

    /// The public key bytes.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0.0
    }

    /// The driver representation.
    pub fn as_driver(&self) -> &X25519PublicKey {
        &self.0
    }
}

/// An X25519 shared secret.
///
/// Feed it to a KDF; do not use it as a key directly. Zeroed on drop.
pub struct SharedSecret([u8; 32]);

impl SharedSecret {
    /// The raw shared secret.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl Drop for SharedSecret {
    fn drop(&mut self) {
        ct::zeroize(&mut self.0);
    }
}

impl core::fmt::Debug for SharedSecret {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("SharedSecret(..)")
    }
}
