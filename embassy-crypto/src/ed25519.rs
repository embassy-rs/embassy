//! Ed25519 (EdDSA over edwards25519, RFC 8032).
//!
//! Pure Ed25519: messages are signed directly, without pre-hashing or a
//! context string. Served by [`driver::Ed25519`].
//!
//! # Example
//!
//! ```ignore
//! use embassy_crypto::ed25519::{SigningKey, VerifyingKey, Signature};
//!
//! let sk = SigningKey::generate()?;
//! let vk = sk.verifying_key()?;
//! let sig = sk.sign(b"hello")?;
//! vk.verify(b"hello", &sig)?;
//! ```

use crate::driver::{self, Ed25519PublicKey, Ed25519SecretKey, Ed25519Signature, RngImpl};
use crate::{Error, ct};

/// An Ed25519 signing key: the 32-byte seed of RFC 8032 section 5.1.5.
///
/// Zeroed on drop.
#[derive(Clone)]
pub struct SigningKey(Ed25519SecretKey);

impl SigningKey {
    /// Generate a random signing key from [`driver::RngImpl`].
    pub fn generate() -> Result<Self, Error> {
        let mut k = Ed25519SecretKey([0u8; 32]);
        RngImpl::fill_bytes(&mut k.0);
        Ok(Self(k))
    }

    /// Load a signing key. Every 32-byte string is a valid seed.
    pub fn from_bytes(bytes: &[u8; 32]) -> Self {
        Self(Ed25519SecretKey(*bytes))
    }

    /// The seed bytes.
    pub fn to_bytes(&self) -> [u8; 32] {
        self.0.0
    }

    /// The corresponding verifying key.
    pub fn verifying_key(&self) -> Result<VerifyingKey, Error> {
        driver::Ed25519Impl::public_key(&self.0).map(VerifyingKey)
    }

    /// Sign a message.
    ///
    /// Ed25519 signatures are deterministic: signing the same message twice
    /// gives the same signature.
    pub fn sign(&self, msg: &[u8]) -> Result<Signature, Error> {
        driver::Ed25519Impl::sign(&self.0, msg).map(Signature)
    }
}

impl Drop for SigningKey {
    fn drop(&mut self) {
        ct::zeroize(&mut self.0.0);
    }
}

impl core::fmt::Debug for SigningKey {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("SigningKey(..)")
    }
}

/// An Ed25519 verifying key: a compressed edwards25519 point, 32 bytes.
///
/// Parsing does not validate the point; the driver does, when the key is
/// used, and reports [`Error::InvalidKey`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct VerifyingKey(Ed25519PublicKey);

impl VerifyingKey {
    /// Load a verifying key.
    pub fn from_bytes(bytes: &[u8; 32]) -> Self {
        Self(Ed25519PublicKey(*bytes))
    }

    /// The verifying key bytes.
    pub fn to_bytes(&self) -> [u8; 32] {
        self.0.0
    }

    /// The verifying key bytes.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0.0
    }

    /// The driver representation.
    pub fn as_driver(&self) -> &Ed25519PublicKey {
        &self.0
    }

    /// Verify the signature of a message.
    ///
    /// Fails with [`Error::InvalidSignature`] if the signature does not
    /// verify, or with [`Error::InvalidKey`] if this key does not decode to
    /// a point on the curve.
    pub fn verify(&self, msg: &[u8], signature: &Signature) -> Result<(), Error> {
        driver::Ed25519Impl::verify(&self.0, msg, &signature.0)
    }
}

/// An Ed25519 signature: `R || S`, 64 bytes.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Signature(Ed25519Signature);

impl Signature {
    /// Load a signature.
    ///
    /// Not validated here: an `R` that is not a point on the curve or an `S`
    /// out of range are rejected by [`VerifyingKey::verify`].
    pub fn from_bytes(bytes: &[u8; 64]) -> Self {
        Self(Ed25519Signature(*bytes))
    }

    /// The signature bytes, `R || S`.
    pub fn to_bytes(&self) -> [u8; 64] {
        self.0.0
    }

    /// The signature bytes, `R || S`.
    pub fn as_bytes(&self) -> &[u8; 64] {
        &self.0.0
    }

    /// The driver representation.
    pub fn as_driver(&self) -> &Ed25519Signature {
        &self.0
    }
}
