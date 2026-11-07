#![no_std]
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

pub mod driver;

mod aes;
mod chacha;
mod ct;
mod ec;
mod hash;
mod hkdf;

pub mod ed25519;
pub mod p256;
pub mod p384;
pub mod x25519;

pub use aes::*;
pub use chacha::*;
pub use driver::Error;
pub use hash::*;
pub use hkdf::*;

/// Fill `buf` with cryptographically secure random bytes.
pub fn rng_fill_bytes(buf: &mut [u8]) {
    driver::RngImpl::fill_bytes(buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    // A constant that disagrees with the array its constructor takes fails to compile.
    macro_rules! curve {
        ($curve:ident, field: $field:literal) => {
            assert_eq!($curve::FIELD_SIZE, $field);
            assert_eq!($curve::SEC1_SIZE, 2 * $field + 1);
            assert_eq!($curve::COMPRESSED_SEC1_SIZE, $field + 1);
            assert_eq!($curve::Signature::SIGNATURE_SIZE, 2 * $field);
            let _: fn(&[u8; $curve::SEC1_SIZE]) -> Result<$curve::VerifyingKey, Error> =
                $curve::VerifyingKey::from_sec1;
            let _: fn(&[u8; $curve::COMPRESSED_SEC1_SIZE]) -> Result<$curve::VerifyingKey, Error> =
                $curve::VerifyingKey::from_sec1_compressed;
            let _: fn(&[u8; $curve::Signature::SIGNATURE_SIZE]) -> Result<$curve::Signature, Error> =
                $curve::Signature::from_bytes;
        };
    }

    #[test]
    fn p256_sizes() {
        curve!(p256, field: 32);
    }

    #[test]
    fn p384_sizes() {
        curve!(p384, field: 48);
    }

    #[test]
    fn ed25519_sizes() {
        assert_eq!(ed25519::SigningKey::KEY_SIZE, 32);
        assert_eq!(ed25519::VerifyingKey::KEY_SIZE, 32);
        assert_eq!(ed25519::Signature::SIGNATURE_SIZE, 64);
        let _: fn(&[u8; ed25519::SigningKey::KEY_SIZE]) -> ed25519::SigningKey = ed25519::SigningKey::from_bytes;
        let _: fn(&[u8; ed25519::VerifyingKey::KEY_SIZE]) -> ed25519::VerifyingKey = ed25519::VerifyingKey::from_bytes;
        let _: fn(&[u8; ed25519::Signature::SIGNATURE_SIZE]) -> ed25519::Signature = ed25519::Signature::from_bytes;
    }

    #[test]
    fn x25519_sizes() {
        assert_eq!(x25519::SecretKey::KEY_SIZE, 32);
        assert_eq!(x25519::PublicKey::KEY_SIZE, 32);
        assert_eq!(x25519::SharedSecret::SECRET_SIZE, 32);
        let _: fn(&[u8; x25519::SecretKey::KEY_SIZE]) -> x25519::SecretKey = x25519::SecretKey::from_bytes;
        let _: fn(&[u8; x25519::PublicKey::KEY_SIZE]) -> x25519::PublicKey = x25519::PublicKey::from_bytes;
        let _: fn(&x25519::SharedSecret) -> &[u8; x25519::SharedSecret::SECRET_SIZE] = x25519::SharedSecret::as_bytes;
    }

    #[test]
    fn aes_iv_sizes() {
        assert_eq!(Aes128CbcEncrypt::IV_SIZE, 16);
        assert_eq!(Aes256CbcDecrypt::IV_SIZE, 16);
        assert_eq!(Aes128Ctr::IV_SIZE, 16);
        let _: fn(&[u8; 16], &[u8; Aes128CbcEncrypt::IV_SIZE]) -> Aes128CbcEncrypt = Aes128CbcEncrypt::new;
        let _: fn(&[u8; 32], &[u8; Aes256CbcDecrypt::IV_SIZE]) -> Aes256CbcDecrypt = Aes256CbcDecrypt::new;
        let _: fn(&[u8; 32], &[u8; Aes256Ctr::IV_SIZE]) -> Aes256Ctr = Aes256Ctr::new;
    }
}
