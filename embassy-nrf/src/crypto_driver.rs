//! `embassy-crypto` drivers served by the hardware accelerators, one per
//! `embassy-crypto-*` feature.
//!
//! Every operation creates a temporary driver on a stolen peripheral token, which is fine: the
//! drivers hold no state between calls, and each hardware transaction is self-contained.
//! Nothing is registered for algorithms the accelerator does not implement (MD5 everywhere;
//! the SHA-512 family outside CRACEN; AES-256 and GCM outside CryptoCell 312 and CRACEN).

#![allow(dead_code, unused_imports, unused_macros)]

use embassy_crypto::Error;
use embassy_crypto::driver::InOutBuf;

use crate::aes::{self, Aes, AesCbc, AesCcm, AesCmac, AesCtr, AesEcb, Direction};
use crate::hash::{self, Hash, Sha1, Sha224, Sha256};
use crate::mode::Blocking;
use crate::peripherals;

fn aes() -> Aes<'static, Blocking> {
    Aes::new_blocking(unsafe { peripherals::AES::steal() })
}

fn hash() -> Hash<'static, Blocking> {
    Hash::new_blocking(unsafe { peripherals::HASH::steal() })
}

fn map_error(error: aes::Error) -> Error {
    match error {
        aes::Error::InvalidKeyLength => Error::InvalidKey,
        aes::Error::InvalidNonceLength
        | aes::Error::InvalidTagLength
        | aes::Error::InvalidLength
        | aes::Error::AadAfterPayload => Error::InvalidInput,
        aes::Error::Hardware => Error::HardwareError,
    }
}

/// Constant-time comparison.
fn ct_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0;
    for (x, y) in a.iter().zip(b) {
        diff |= x ^ y;
    }
    diff == 0
}

/// Feeds an in/out buffer as payload, in place if input and output alias.
fn payload<C: aes::Cipher>(
    aes: &mut Aes<'_, Blocking>,
    ctx: &mut aes::Context<C>,
    buf: InOutBuf<'_, '_, u8>,
    last: bool,
) -> Result<(), aes::Error> {
    let len = buf.len();
    let (input, output) = buf.into_raw();
    if input == output as *const u8 {
        let data = unsafe { core::slice::from_raw_parts_mut(output, len) };
        aes.blocking_payload_in_place(ctx, data, last)
    } else {
        let input = unsafe { core::slice::from_raw_parts(input, len) };
        let output = unsafe { core::slice::from_raw_parts_mut(output, len) };
        aes.blocking_payload(ctx, input, output, last)
    }
}

// ===========================================================================
// Hashes and HMAC
// ===========================================================================

macro_rules! impl_digest {
    ($(#[$meta:meta])* $driver:ident, $trait:ident, $algo:ty, $size:literal, $register:ident) => {
        $(#[$meta])*
        mod $driver {
            use super::*;

            struct Driver;

            impl embassy_crypto::driver::$trait for Driver {
                type Context = hash::Context<$algo>;

                fn init() -> Self::Context {
                    hash().start::<$algo>()
                }

                fn update(ctx: &mut Self::Context, data: &[u8]) {
                    hash().blocking_update(ctx, data)
                }

                fn finalize(ctx: Self::Context, out: &mut [u8; $size]) {
                    out.copy_from_slice(hash().blocking_finish(ctx).as_ref());
                }
            }

            embassy_crypto::$register!(Driver);
        }
    };
}

macro_rules! impl_hmac {
    ($(#[$meta:meta])* $driver:ident, $trait:ident, $algo:ty, $size:literal, $register:ident) => {
        $(#[$meta])*
        mod $driver {
            use super::*;

            struct Driver;

            impl embassy_crypto::driver::$trait for Driver {
                type Context = hash::HmacContext<$algo>;

                fn init(key: &[u8]) -> Self::Context {
                    hash().start_hmac::<$algo>(key)
                }

                fn update(ctx: &mut Self::Context, data: &[u8]) {
                    hash().blocking_update(ctx, data)
                }

                fn finalize(ctx: Self::Context, out: &mut [u8; $size]) {
                    out.copy_from_slice(hash().blocking_finish(ctx).as_ref());
                }
            }

            embassy_crypto::$register!(Driver);
        }
    };
}

impl_digest!(
    #[cfg(feature = "embassy-crypto-sha1")]
    sha1_driver,
    Sha1,
    Sha1,
    20,
    sha1_impl
);
impl_digest!(
    #[cfg(feature = "embassy-crypto-sha224")]
    sha224_driver,
    Sha224,
    Sha224,
    28,
    sha224_impl
);
impl_digest!(
    #[cfg(feature = "embassy-crypto-sha256")]
    sha256_driver,
    Sha256,
    Sha256,
    32,
    sha256_impl
);
impl_digest!(
    #[cfg(all(feature = "embassy-crypto-sha384", feature = "_cracen"))]
    sha384_driver,
    Sha384,
    hash::Sha384,
    48,
    sha384_impl
);
impl_digest!(
    #[cfg(all(feature = "embassy-crypto-sha512", feature = "_cracen"))]
    sha512_driver,
    Sha512,
    hash::Sha512,
    64,
    sha512_impl
);
impl_digest!(
    #[cfg(all(feature = "embassy-crypto-sha512-224", feature = "_cracen"))]
    sha512_224_driver,
    Sha512_224,
    hash::Sha512_224,
    28,
    sha512_224_impl
);
impl_digest!(
    #[cfg(all(feature = "embassy-crypto-sha512-256", feature = "_cracen"))]
    sha512_256_driver,
    Sha512_256,
    hash::Sha512_256,
    32,
    sha512_256_impl
);

impl_hmac!(
    #[cfg(feature = "embassy-crypto-hmac-sha1")]
    hmac_sha1_driver,
    HmacSha1,
    Sha1,
    20,
    hmac_sha1_impl
);
impl_hmac!(
    #[cfg(feature = "embassy-crypto-hmac-sha224")]
    hmac_sha224_driver,
    HmacSha224,
    Sha224,
    28,
    hmac_sha224_impl
);
impl_hmac!(
    #[cfg(feature = "embassy-crypto-hmac-sha256")]
    hmac_sha256_driver,
    HmacSha256,
    Sha256,
    32,
    hmac_sha256_impl
);
impl_hmac!(
    #[cfg(all(feature = "embassy-crypto-hmac-sha384", feature = "_cracen"))]
    hmac_sha384_driver,
    HmacSha384,
    hash::Sha384,
    48,
    hmac_sha384_impl
);
impl_hmac!(
    #[cfg(all(feature = "embassy-crypto-hmac-sha512", feature = "_cracen"))]
    hmac_sha512_driver,
    HmacSha512,
    hash::Sha512,
    64,
    hmac_sha512_impl
);
impl_hmac!(
    #[cfg(all(feature = "embassy-crypto-hmac-sha512-224", feature = "_cracen"))]
    hmac_sha512_224_driver,
    HmacSha512_224,
    hash::Sha512_224,
    28,
    hmac_sha512_224_impl
);
impl_hmac!(
    #[cfg(all(feature = "embassy-crypto-hmac-sha512-256", feature = "_cracen"))]
    hmac_sha512_256_driver,
    HmacSha512_256,
    hash::Sha512_256,
    32,
    hmac_sha512_256_impl
);

// ===========================================================================
// AES block modes
// ===========================================================================

macro_rules! impl_ecb {
    ($(#[$meta:meta])* $driver:ident, $trait:ident, $key_len:literal, $register:ident) => {
        $(#[$meta])*
        mod $driver {
            use super::*;

            struct Driver;

            impl embassy_crypto::driver::$trait for Driver {
                type Context = AesEcb;

                fn init(key: &[u8; $key_len]) -> Self::Context {
                    AesEcb::new(key).unwrap()
                }

                fn encrypt_blocks(ctx: &Self::Context, blocks: InOutBuf<'_, '_, u8>) {
                    let mut aes = aes();
                    let mut op = aes.start(*ctx, Direction::Encrypt);
                    payload(&mut aes, &mut op, blocks, true).unwrap();
                }

                fn decrypt_blocks(ctx: &Self::Context, blocks: InOutBuf<'_, '_, u8>) {
                    let mut aes = aes();
                    let mut op = aes.start(*ctx, Direction::Decrypt);
                    payload(&mut aes, &mut op, blocks, true).unwrap();
                }
            }

            embassy_crypto::$register!(Driver);
        }
    };
}

macro_rules! impl_cbc {
    ($(#[$meta:meta])* $driver:ident, $trait:ident, $key_len:literal, $register:ident) => {
        $(#[$meta])*
        mod $driver {
            use super::*;

            struct Driver;

            impl embassy_crypto::driver::$trait for Driver {
                type EncryptContext = aes::Context<AesCbc>;
                type DecryptContext = aes::Context<AesCbc>;

                fn encrypt_init(key: &[u8; $key_len], iv: &[u8; 16]) -> Self::EncryptContext {
                    aes().start(AesCbc::new(key, iv).unwrap(), Direction::Encrypt)
                }

                fn decrypt_init(key: &[u8; $key_len], iv: &[u8; 16]) -> Self::DecryptContext {
                    aes().start(AesCbc::new(key, iv).unwrap(), Direction::Decrypt)
                }

                fn encrypt_blocks(ctx: &mut Self::EncryptContext, blocks: InOutBuf<'_, '_, u8>) {
                    payload(&mut aes(), ctx, blocks, false).unwrap();
                }

                fn decrypt_blocks(ctx: &mut Self::DecryptContext, blocks: InOutBuf<'_, '_, u8>) {
                    payload(&mut aes(), ctx, blocks, false).unwrap();
                }
            }

            embassy_crypto::$register!(Driver);
        }
    };
}

macro_rules! impl_ctr {
    ($(#[$meta:meta])* $driver:ident, $trait:ident, $key_len:literal, $register:ident) => {
        $(#[$meta])*
        mod $driver {
            use super::*;

            struct Driver;

            impl embassy_crypto::driver::$trait for Driver {
                type Context = aes::Context<AesCtr>;

                fn init(key: &[u8; $key_len], iv: &[u8; 16]) -> Self::Context {
                    aes().start(AesCtr::new(key, iv).unwrap(), Direction::Encrypt)
                }

                fn apply_keystream(ctx: &mut Self::Context, buf: InOutBuf<'_, '_, u8>) {
                    payload(&mut aes(), ctx, buf, false).unwrap();
                }
            }

            embassy_crypto::$register!(Driver);
        }
    };
}

macro_rules! impl_cmac {
    ($(#[$meta:meta])* $driver:ident, $trait:ident, $key_len:literal, $register:ident) => {
        $(#[$meta])*
        mod $driver {
            use super::*;

            struct Driver;

            impl embassy_crypto::driver::$trait for Driver {
                type Context = aes::Context<AesCmac>;

                fn init(key: &[u8; $key_len]) -> Self::Context {
                    aes().start(AesCmac::new(key).unwrap(), Direction::Encrypt)
                }

                fn update(ctx: &mut Self::Context, data: &[u8]) {
                    aes().blocking_payload(ctx, data, &mut [], false).unwrap();
                }

                fn finalize(ctx: Self::Context, out: &mut [u8; 16]) {
                    *out = aes().blocking_finish(ctx).unwrap().unwrap();
                }

                fn reset(ctx: &mut Self::Context) {
                    let cipher = *ctx.cipher();
                    *ctx = aes().start(cipher, Direction::Encrypt);
                }
            }

            embassy_crypto::$register!(Driver);
        }
    };
}

macro_rules! impl_ccm {
    ($(#[$meta:meta])* $driver:ident, $trait:ident, $key_len:literal, $register:ident) => {
        $(#[$meta])*
        mod $driver {
            use super::*;

            struct Driver;

            impl embassy_crypto::driver::$trait for Driver {
                type Context = [u8; $key_len];

                fn init(key: &[u8; $key_len]) -> Self::Context {
                    *key
                }

                fn encrypt(
                    key: &Self::Context,
                    nonce: &[u8],
                    aad: &[u8],
                    buffer: InOutBuf<'_, '_, u8>,
                    tag: &mut [u8],
                ) -> Result<(), Error> {
                    let cipher = AesCcm::new(key, nonce, aad.len(), buffer.len(), tag.len()).map_err(map_error)?;
                    let mut aes = aes();
                    let mut op = aes.start(cipher, Direction::Encrypt);
                    aes.blocking_aad(&mut op, aad, true).map_err(map_error)?;
                    payload(&mut aes, &mut op, buffer, true).map_err(map_error)?;
                    let computed = aes.blocking_finish(op).map_err(map_error)?.unwrap();
                    tag.copy_from_slice(&computed[..tag.len()]);
                    Ok(())
                }

                fn decrypt(
                    key: &Self::Context,
                    nonce: &[u8],
                    aad: &[u8],
                    buffer: InOutBuf<'_, '_, u8>,
                    tag: &[u8],
                ) -> Result<(), Error> {
                    let cipher = AesCcm::new(key, nonce, aad.len(), buffer.len(), tag.len()).map_err(map_error)?;
                    let mut aes = aes();
                    let mut op = aes.start(cipher, Direction::Decrypt);
                    aes.blocking_aad(&mut op, aad, true).map_err(map_error)?;
                    payload(&mut aes, &mut op, buffer, true).map_err(map_error)?;
                    let computed = aes.blocking_finish(op).map_err(map_error)?.unwrap();
                    if ct_eq(&computed[..tag.len()], tag) {
                        Ok(())
                    } else {
                        Err(Error::InvalidSignature)
                    }
                }
            }

            embassy_crypto::$register!(Driver);
        }
    };
}

macro_rules! impl_gcm {
    ($(#[$meta:meta])* $driver:ident, $trait:ident, $key_len:literal, $register:ident) => {
        $(#[$meta])*
        mod $driver {
            use super::*;

            struct Driver;

            impl embassy_crypto::driver::$trait for Driver {
                type Context = [u8; $key_len];

                fn init(key: &[u8; $key_len]) -> Self::Context {
                    *key
                }

                fn encrypt(
                    key: &Self::Context,
                    nonce: &[u8],
                    aad: &[u8],
                    buffer: InOutBuf<'_, '_, u8>,
                    tag: &mut [u8; 16],
                ) -> Result<(), Error> {
                    let nonce: &[u8; 12] = nonce.try_into().map_err(|_| Error::InvalidInput)?;
                    let cipher = aes::AesGcm::new(key, nonce).map_err(map_error)?;
                    let mut aes = aes();
                    let mut op = aes.start(cipher, Direction::Encrypt);
                    aes.blocking_aad(&mut op, aad, true).map_err(map_error)?;
                    payload(&mut aes, &mut op, buffer, true).map_err(map_error)?;
                    *tag = aes.blocking_finish(op).map_err(map_error)?.unwrap();
                    Ok(())
                }

                fn decrypt(
                    key: &Self::Context,
                    nonce: &[u8],
                    aad: &[u8],
                    buffer: InOutBuf<'_, '_, u8>,
                    tag: &[u8; 16],
                ) -> Result<(), Error> {
                    let nonce: &[u8; 12] = nonce.try_into().map_err(|_| Error::InvalidInput)?;
                    let cipher = aes::AesGcm::new(key, nonce).map_err(map_error)?;
                    let mut aes = aes();
                    let mut op = aes.start(cipher, Direction::Decrypt);
                    aes.blocking_aad(&mut op, aad, true).map_err(map_error)?;
                    payload(&mut aes, &mut op, buffer, true).map_err(map_error)?;
                    let computed = aes.blocking_finish(op).map_err(map_error)?.unwrap();
                    if ct_eq(&computed, tag) {
                        Ok(())
                    } else {
                        Err(Error::InvalidSignature)
                    }
                }
            }

            embassy_crypto::$register!(Driver);
        }
    };
}

impl_ecb!(
    #[cfg(feature = "embassy-crypto-aes128-ecb")]
    aes128_ecb_driver,
    Aes128Ecb,
    16,
    aes128_ecb_impl
);
impl_cbc!(
    #[cfg(feature = "embassy-crypto-aes128-cbc")]
    aes128_cbc_driver,
    Aes128Cbc,
    16,
    aes128_cbc_impl
);
impl_ctr!(
    #[cfg(feature = "embassy-crypto-aes128-ctr")]
    aes128_ctr_driver,
    Aes128Ctr,
    16,
    aes128_ctr_impl
);
impl_cmac!(
    #[cfg(feature = "embassy-crypto-aes128-cmac")]
    aes128_cmac_driver,
    Aes128Cmac,
    16,
    aes128_cmac_impl
);
impl_ccm!(
    #[cfg(feature = "embassy-crypto-aes128-ccm")]
    aes128_ccm_driver,
    Aes128Ccm,
    16,
    aes128_ccm_impl
);
impl_gcm!(
    #[cfg(all(
        feature = "embassy-crypto-aes128-gcm",
        any(feature = "_cryptocell-312", feature = "_cracen")
    ))]
    aes128_gcm_driver,
    Aes128Gcm,
    16,
    aes128_gcm_impl
);

impl_ecb!(
    #[cfg(all(
        feature = "embassy-crypto-aes256-ecb",
        any(feature = "_cryptocell-312", feature = "_cracen")
    ))]
    aes256_ecb_driver,
    Aes256Ecb,
    32,
    aes256_ecb_impl
);
impl_cbc!(
    #[cfg(all(
        feature = "embassy-crypto-aes256-cbc",
        any(feature = "_cryptocell-312", feature = "_cracen")
    ))]
    aes256_cbc_driver,
    Aes256Cbc,
    32,
    aes256_cbc_impl
);
impl_ctr!(
    #[cfg(all(
        feature = "embassy-crypto-aes256-ctr",
        any(feature = "_cryptocell-312", feature = "_cracen")
    ))]
    aes256_ctr_driver,
    Aes256Ctr,
    32,
    aes256_ctr_impl
);
impl_cmac!(
    #[cfg(all(
        feature = "embassy-crypto-aes256-cmac",
        any(feature = "_cryptocell-312", feature = "_cracen")
    ))]
    aes256_cmac_driver,
    Aes256Cmac,
    32,
    aes256_cmac_impl
);
impl_ccm!(
    #[cfg(all(
        feature = "embassy-crypto-aes256-ccm",
        any(feature = "_cryptocell-312", feature = "_cracen")
    ))]
    aes256_ccm_driver,
    Aes256Ccm,
    32,
    aes256_ccm_impl
);
impl_gcm!(
    #[cfg(all(
        feature = "embassy-crypto-aes256-gcm",
        any(feature = "_cryptocell-312", feature = "_cracen")
    ))]
    aes256_gcm_driver,
    Aes256Gcm,
    32,
    aes256_gcm_impl
);
