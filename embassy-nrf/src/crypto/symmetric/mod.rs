//! Symmetric crypto engines: AES, hash and ChaCha.
//!
//! The three engines share one DMA, so only one of them runs at a time. They are owned
//! together through the `CRYPTO_SYMMETRIC` peripheral and the [`Symmetric`] driver.
//!
//! The state of every operation lives in a context object outside the driver. Any number of
//! operations of any kind can be in progress at once, and their calls can be interleaved.
//!
//! # AES
//!
//! | Mode | Authenticated | nRF52840, nRF91 | nRF5340 | nRF54L |
//! |------|---------------|-----------------|---------|--------|
//! | ECB  | No            | ✓               | ✓       | ✓      |
//! | CBC  | No            | ✓               | ✓       | ✓      |
//! | CTR  | No            | ✓               | ✓       | ✓      |
//! | CMAC | MAC only      | ✓               | ✓       | ✓      |
//! | CCM  | Yes           | ✓               | ✓       | ✓      |
//! | GCM  | Yes           | ✗               | ✓       | ✓      |
//!
//! 128-bit keys are supported on all chips. 192-bit and 256-bit keys are supported on
//! nRF5340 and nRF54L.
//!
//! To run a cipher operation:
//!
//! 1. Start it with [`Symmetric::aes_start`]. This returns an [`AesContext`].
//! 2. Feed additional authenticated data with [`Symmetric::aes_blocking_aad`]
//!    (authenticated modes only).
//! 3. Feed the payload with [`Symmetric::aes_blocking_payload`].
//! 4. Finish with [`Symmetric::aes_blocking_finish`]. For MAC and AEAD modes this returns
//!    the authentication tag.
//!
//! # Hash and HMAC
//!
//! | Algorithm   | nRF52840, nRF91, nRF5340 | nRF54L |
//! |-------------|--------------------------|--------|
//! | SHA-1       | ✓                        | ✓      |
//! | SHA-224     | ✓                        | ✓      |
//! | SHA-256     | ✓                        | ✓      |
//! | SHA-384     | ✗                        | ✓      |
//! | SHA-512     | ✗                        | ✓      |
//! | SHA-512/224 | ✗                        | ✓      |
//! | SHA-512/256 | ✗                        | ✓      |
//!
//! HMAC is available for every supported algorithm.
//!
//! To compute a digest:
//!
//! 1. Start with [`Symmetric::hash_start`] or [`Symmetric::hmac_start`].
//! 2. Feed data with [`Symmetric::hash_blocking_update`].
//! 3. Finish with [`Symmetric::hash_blocking_finish`].
//!
//! # ChaCha and ChaCha-Poly1305
//!
//! Both follow RFC 8439: a 256-bit key, a 96-bit nonce and a 32-bit block counter. The
//! [`ChaChaVariant`] selects the number of rounds: ChaCha20 everywhere, and also the
//! reduced-round ChaCha12 and ChaCha8 on the CryptoCell.
//!
//! - The plain keystream is started with [`Symmetric::chacha_start`] and applied with
//!   [`Symmetric::chacha_blocking_apply_keystream`].
//! - The AEAD is started with [`Symmetric::chachapoly_start`] and then driven like an AES
//!   AEAD, with the `chachapoly_blocking_*` methods.
//!
//! The CryptoCell has no Poly1305 engine. There, the authenticator runs in software.
//!
//! Input data may be anywhere in memory, including flash.

use core::marker::PhantomData;

use crate::crypto::ActivationHandle;
use crate::mode::Mode;

mod aes;
mod chacha;
#[cfg(feature = "_embassy-crypto-symmetric")]
mod driver;
mod hash;
#[cfg(feature = "_cryptocell")]
mod poly1305;

#[cfg(any(feature = "_cryptocell-312", feature = "_cracen"))]
pub use aes::AesGcm;
pub use aes::{AES_BLOCK_LEN, AesCbc, AesCcm, AesCmac, AesContext, AesCtr, AesEcb, AuthenticatedCipher, Cipher};
pub use chacha::{CHACHA_BLOCK_LEN, CHACHA_KEY_LEN, CHACHA_NONCE_LEN, ChaChaContext, ChaChaPolyContext, ChaChaVariant};
pub use hash::{Buffer, DigestContext, HashAlgorithm, HashContext, HmacContext, Kind, Sha1, Sha224, Sha256};
#[cfg(feature = "_cracen")]
pub use hash::{Sha384, Sha512, Sha512_224, Sha512_256};

// Largest amount of data processed per hardware transaction.
const MAX_CHUNK: usize = 1024;

/// Symmetric crypto error.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// The key length is not supported by the hardware.
    InvalidKeyLength,
    /// The nonce or IV length is not valid for the cipher mode.
    InvalidNonceLength,
    /// The tag length is not valid for the cipher mode.
    InvalidTagLength,
    /// The data length is not valid.
    ///
    /// Returned when:
    /// - the input and output lengths differ,
    /// - a chunk that is not the last one is not a multiple of the block length, for modes
    ///   that require it,
    /// - the amount of data does not match the lengths given to [`AesCcm::new`].
    InvalidLength,
    /// Additional authenticated data was supplied after payload data.
    AadAfterPayload,
    /// The hardware reported an error.
    Hardware,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let s = match self {
            Self::InvalidKeyLength => "invalid key length",
            Self::InvalidNonceLength => "invalid nonce length",
            Self::InvalidTagLength => "invalid tag length",
            Self::InvalidLength => "invalid data length",
            Self::AadAfterPayload => "additional authenticated data after payload",
            Self::Hardware => "hardware error",
        };
        f.write_str(s)
    }
}

impl core::error::Error for Error {}

/// Cipher direction.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Direction {
    /// Encryption.
    Encrypt,
    /// Decryption.
    Decrypt,
}

/// Driver for the symmetric crypto engines: AES, hash and ChaCha.
///
/// See the [module](self) documentation.
///
/// ```no_run
/// use embassy_nrf::crypto::symmetric::{Sha256, Symmetric};
///
/// # let p: embassy_nrf::Peripherals = todo!();
/// let mut crypto = Symmetric::new_blocking(p.CRYPTO_SYMMETRIC);
/// let mut ctx = crypto.hash_start::<Sha256>();
/// crypto.hash_blocking_update(&mut ctx, b"hello");
/// let digest = crypto.hash_blocking_finish(ctx);
/// ```
pub struct Symmetric<'d, M: Mode> {
    _activation: ActivationHandle,
    _phantom: PhantomData<(&'d (), M)>,
}

#[cfg(not(feature = "_embassy-crypto-symmetric"))]
impl<'d> Symmetric<'d, crate::mode::Blocking> {
    /// Creates a new blocking driver for the symmetric engines.
    pub fn new_blocking(_peri: crate::Peri<'d, crate::peripherals::CRYPTO_SYMMETRIC>) -> Self {
        Self::new_inner()
    }
}

impl<'d, M: Mode> Symmetric<'d, M> {
    // Used by the `embassy-crypto` drivers, which have no peripheral token.
    pub(crate) fn new_inner() -> Self {
        let activation = crate::crypto::activate();
        #[cfg(feature = "_cracen")]
        aes::load_countermeasure_mask();
        Self {
            _activation: activation,
            _phantom: PhantomData,
        }
    }
}
