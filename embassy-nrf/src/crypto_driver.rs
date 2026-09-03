//! `embassy-crypto-driver` implementation backed by the hardware accelerators.
//!
//! This lets `embassy-crypto` use the hardware through its RustCrypto-style types. Every
//! operation creates a temporary driver on a stolen peripheral token, which is fine: the
//! drivers hold no state between calls, and each hardware transaction is self-contained.

use embassy_crypto_driver::{CryptoError, InOutBuf};

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

fn map_error(error: aes::Error) -> CryptoError {
    match error {
        aes::Error::InvalidKeyLength => CryptoError::InvalidKey,
        aes::Error::InvalidNonceLength
        | aes::Error::InvalidTagLength
        | aes::Error::InvalidLength
        | aes::Error::AadAfterPayload => CryptoError::InvalidInput,
        aes::Error::Hardware => CryptoError::HardwareError,
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
    ($driver:ident, $trait:path, $algo:ty, $init:ident, $clone:ident, $update:ident, $finalize:ident, $impl_macro:path) => {
        struct $driver;

        impl $trait for $driver {
            type Context = hash::Context<$algo>;

            fn $init() -> Self::Context {
                hash().start::<$algo>()
            }

            fn $clone(ctx: &Self::Context) -> Self::Context {
                ctx.clone()
            }

            fn $update(ctx: &mut Self::Context, data: &[u8]) {
                hash().blocking_update(ctx, data)
            }

            fn $finalize(ctx: Self::Context, out: &mut [u8]) {
                let digest = hash().blocking_finish(ctx);
                let n = out.len().min(digest.len());
                out[..n].copy_from_slice(&digest[..n]);
            }
        }

        $impl_macro!($driver);
    };
}

macro_rules! impl_hmac {
    ($driver:ident, $trait:path, $algo:ty, $init:ident, $clone:ident, $update:ident, $finalize:ident, $reset:ident, $impl_macro:path) => {
        struct $driver;

        impl $trait for $driver {
            type Context = hash::HmacContext<$algo>;

            fn $init(key: &[u8]) -> Self::Context {
                hash().start_hmac::<$algo>(key)
            }

            fn $clone(ctx: &Self::Context) -> Self::Context {
                ctx.clone()
            }

            fn $update(ctx: &mut Self::Context, data: &[u8]) {
                hash().blocking_update(ctx, data)
            }

            fn $finalize(ctx: Self::Context, out: &mut [u8]) {
                let digest = hash().blocking_finish(ctx);
                let n = out.len().min(digest.len());
                out[..n].copy_from_slice(&digest[..n]);
            }

            fn $reset(ctx: &mut Self::Context) {
                ctx.reset()
            }
        }

        $impl_macro!($driver);
    };
}

impl_digest!(
    Sha1Driver,
    embassy_crypto_driver::Sha1,
    Sha1,
    sha1_init,
    sha1_clone,
    sha1_update,
    sha1_finalize,
    embassy_crypto_driver::embassy_crypto_sha1_impl
);
impl_digest!(
    Sha224Driver,
    embassy_crypto_driver::Sha224,
    Sha224,
    sha224_init,
    sha224_clone,
    sha224_update,
    sha224_finalize,
    embassy_crypto_driver::embassy_crypto_sha224_impl
);
impl_digest!(
    Sha256Driver,
    embassy_crypto_driver::Sha256,
    Sha256,
    sha256_init,
    sha256_clone,
    sha256_update,
    sha256_finalize,
    embassy_crypto_driver::embassy_crypto_sha256_impl
);
#[cfg(feature = "_cracen")]
impl_digest!(
    Sha384Driver,
    embassy_crypto_driver::Sha384,
    hash::Sha384,
    sha384_init,
    sha384_clone,
    sha384_update,
    sha384_finalize,
    embassy_crypto_driver::embassy_crypto_sha384_impl
);
#[cfg(feature = "_cracen")]
impl_digest!(
    Sha512Driver,
    embassy_crypto_driver::Sha512,
    hash::Sha512,
    sha512_init,
    sha512_clone,
    sha512_update,
    sha512_finalize,
    embassy_crypto_driver::embassy_crypto_sha512_impl
);
#[cfg(feature = "_cracen")]
impl_digest!(
    Sha512_224Driver,
    embassy_crypto_driver::Sha512_224,
    hash::Sha512_224,
    sha512_224_init,
    sha512_224_clone,
    sha512_224_update,
    sha512_224_finalize,
    embassy_crypto_driver::embassy_crypto_sha512_224_impl
);
#[cfg(feature = "_cracen")]
impl_digest!(
    Sha512_256Driver,
    embassy_crypto_driver::Sha512_256,
    hash::Sha512_256,
    sha512_256_init,
    sha512_256_clone,
    sha512_256_update,
    sha512_256_finalize,
    embassy_crypto_driver::embassy_crypto_sha512_256_impl
);

impl_hmac!(
    HmacSha1Driver,
    embassy_crypto_driver::HmacSha1,
    Sha1,
    hmac_sha1_init,
    hmac_sha1_clone,
    hmac_sha1_update,
    hmac_sha1_finalize,
    hmac_sha1_reset,
    embassy_crypto_driver::embassy_crypto_hmac_sha1_impl
);
impl_hmac!(
    HmacSha224Driver,
    embassy_crypto_driver::HmacSha224,
    Sha224,
    hmac_sha224_init,
    hmac_sha224_clone,
    hmac_sha224_update,
    hmac_sha224_finalize,
    hmac_sha224_reset,
    embassy_crypto_driver::embassy_crypto_hmac_sha224_impl
);
impl_hmac!(
    HmacSha256Driver,
    embassy_crypto_driver::HmacSha256,
    Sha256,
    hmac_sha256_init,
    hmac_sha256_clone,
    hmac_sha256_update,
    hmac_sha256_finalize,
    hmac_sha256_reset,
    embassy_crypto_driver::embassy_crypto_hmac_sha256_impl
);
#[cfg(feature = "_cracen")]
impl_hmac!(
    HmacSha384Driver,
    embassy_crypto_driver::HmacSha384,
    hash::Sha384,
    hmac_sha384_init,
    hmac_sha384_clone,
    hmac_sha384_update,
    hmac_sha384_finalize,
    hmac_sha384_reset,
    embassy_crypto_driver::embassy_crypto_hmac_sha384_impl
);
#[cfg(feature = "_cracen")]
impl_hmac!(
    HmacSha512Driver,
    embassy_crypto_driver::HmacSha512,
    hash::Sha512,
    hmac_sha512_init,
    hmac_sha512_clone,
    hmac_sha512_update,
    hmac_sha512_finalize,
    hmac_sha512_reset,
    embassy_crypto_driver::embassy_crypto_hmac_sha512_impl
);
#[cfg(feature = "_cracen")]
impl_hmac!(
    HmacSha512_224Driver,
    embassy_crypto_driver::HmacSha512_224,
    hash::Sha512_224,
    hmac_sha512_224_init,
    hmac_sha512_224_clone,
    hmac_sha512_224_update,
    hmac_sha512_224_finalize,
    hmac_sha512_224_reset,
    embassy_crypto_driver::embassy_crypto_hmac_sha512_224_impl
);
#[cfg(feature = "_cracen")]
impl_hmac!(
    HmacSha512_256Driver,
    embassy_crypto_driver::HmacSha512_256,
    hash::Sha512_256,
    hmac_sha512_256_init,
    hmac_sha512_256_clone,
    hmac_sha512_256_update,
    hmac_sha512_256_finalize,
    hmac_sha512_256_reset,
    embassy_crypto_driver::embassy_crypto_hmac_sha512_256_impl
);

// ===========================================================================
// AES block modes
// ===========================================================================

macro_rules! impl_ecb {
    ($driver:ident, $trait:path, $key_len:expr, $init:ident, $clone:ident, $encrypt:ident, $decrypt:ident, $impl_macro:path) => {
        struct $driver;

        impl $trait for $driver {
            type Context = AesEcb;

            fn $init(key: &[u8; $key_len]) -> Self::Context {
                AesEcb::new(key).unwrap()
            }

            fn $clone(ctx: &Self::Context) -> Self::Context {
                *ctx
            }

            fn $encrypt(ctx: &Self::Context, blocks: InOutBuf<'_, '_, u8>) {
                let mut aes = aes();
                let mut op = aes.start(*ctx, Direction::Encrypt);
                payload(&mut aes, &mut op, blocks, true).unwrap();
            }

            fn $decrypt(ctx: &Self::Context, blocks: InOutBuf<'_, '_, u8>) {
                let mut aes = aes();
                let mut op = aes.start(*ctx, Direction::Decrypt);
                payload(&mut aes, &mut op, blocks, true).unwrap();
            }
        }

        $impl_macro!($driver);
    };
}

macro_rules! impl_cbc {
    ($driver:ident, $trait:path, $key_len:expr, $enc_init:ident, $dec_init:ident, $encrypt:ident, $decrypt:ident, $impl_macro:path) => {
        struct $driver;

        impl $trait for $driver {
            type EncryptContext = aes::Context<AesCbc>;
            type DecryptContext = aes::Context<AesCbc>;

            fn $enc_init(key: &[u8; $key_len], iv: &[u8; 16]) -> Self::EncryptContext {
                aes().start(AesCbc::new(key, iv).unwrap(), Direction::Encrypt)
            }

            fn $dec_init(key: &[u8; $key_len], iv: &[u8; 16]) -> Self::DecryptContext {
                aes().start(AesCbc::new(key, iv).unwrap(), Direction::Decrypt)
            }

            fn $encrypt(ctx: &mut Self::EncryptContext, blocks: InOutBuf<'_, '_, u8>) {
                payload(&mut aes(), ctx, blocks, false).unwrap();
            }

            fn $decrypt(ctx: &mut Self::DecryptContext, blocks: InOutBuf<'_, '_, u8>) {
                payload(&mut aes(), ctx, blocks, false).unwrap();
            }
        }

        $impl_macro!($driver);
    };
}

macro_rules! impl_ctr {
    ($driver:ident, $trait:path, $key_len:expr, $init:ident, $apply:ident, $impl_macro:path) => {
        struct $driver;

        impl $trait for $driver {
            type Context = aes::Context<AesCtr>;

            fn $init(key: &[u8; $key_len], iv: &[u8; 16]) -> Self::Context {
                aes().start(AesCtr::new(key, iv).unwrap(), Direction::Encrypt)
            }

            fn $apply(ctx: &mut Self::Context, buf: InOutBuf<'_, '_, u8>) {
                payload(&mut aes(), ctx, buf, false).unwrap();
            }
        }

        $impl_macro!($driver);
    };
}

macro_rules! impl_cmac {
    ($driver:ident, $trait:path, $key_len:expr, $init:ident, $clone:ident, $update:ident, $finalize:ident, $reset:ident, $impl_macro:path) => {
        struct $driver;

        impl $trait for $driver {
            type Context = aes::Context<AesCmac>;

            fn $init(key: &[u8; $key_len]) -> Self::Context {
                aes().start(AesCmac::new(key).unwrap(), Direction::Encrypt)
            }

            fn $clone(ctx: &Self::Context) -> Self::Context {
                ctx.clone()
            }

            fn $update(ctx: &mut Self::Context, data: &[u8]) {
                aes().blocking_payload(ctx, data, &mut [], false).unwrap();
            }

            fn $finalize(ctx: Self::Context, out: &mut [u8; 16]) {
                *out = aes().blocking_finish(ctx).unwrap().unwrap();
            }

            fn $reset(ctx: &mut Self::Context) {
                let cipher = *ctx.cipher();
                *ctx = aes().start(cipher, Direction::Encrypt);
            }
        }

        $impl_macro!($driver);
    };
}

macro_rules! impl_ccm {
    ($driver:ident, $trait:path, $key_len:expr, $init:ident, $clone:ident, $encrypt:ident, $decrypt:ident, $impl_macro:path) => {
        struct $driver;

        impl $trait for $driver {
            type Context = [u8; $key_len];

            fn $init(key: &[u8; $key_len]) -> Self::Context {
                *key
            }

            fn $clone(ctx: &Self::Context) -> Self::Context {
                *ctx
            }

            fn $encrypt(
                key: &Self::Context,
                nonce: &[u8],
                aad: &[u8],
                buffer: InOutBuf<'_, '_, u8>,
                tag: &mut [u8],
            ) -> Result<(), CryptoError> {
                let cipher = AesCcm::new(key, nonce, aad.len(), buffer.len(), tag.len()).map_err(map_error)?;
                let mut aes = aes();
                let mut op = aes.start(cipher, Direction::Encrypt);
                aes.blocking_aad(&mut op, aad, true).map_err(map_error)?;
                payload(&mut aes, &mut op, buffer, true).map_err(map_error)?;
                let computed = aes.blocking_finish(op).map_err(map_error)?.unwrap();
                tag.copy_from_slice(&computed[..tag.len()]);
                Ok(())
            }

            fn $decrypt(
                key: &Self::Context,
                nonce: &[u8],
                aad: &[u8],
                buffer: InOutBuf<'_, '_, u8>,
                tag: &[u8],
            ) -> Result<(), CryptoError> {
                let cipher = AesCcm::new(key, nonce, aad.len(), buffer.len(), tag.len()).map_err(map_error)?;
                let mut aes = aes();
                let mut op = aes.start(cipher, Direction::Decrypt);
                aes.blocking_aad(&mut op, aad, true).map_err(map_error)?;
                payload(&mut aes, &mut op, buffer, true).map_err(map_error)?;
                let computed = aes.blocking_finish(op).map_err(map_error)?.unwrap();
                if ct_eq(&computed[..tag.len()], tag) {
                    Ok(())
                } else {
                    Err(CryptoError::InvalidSignature)
                }
            }
        }

        $impl_macro!($driver);
    };
}

#[cfg(any(feature = "_cryptocell-312", feature = "_cracen"))]
macro_rules! impl_gcm {
    ($driver:ident, $trait:path, $key_len:expr, $init:ident, $clone:ident, $encrypt:ident, $decrypt:ident, $impl_macro:path) => {
        struct $driver;

        impl $trait for $driver {
            type Context = [u8; $key_len];

            fn $init(key: &[u8; $key_len]) -> Self::Context {
                *key
            }

            fn $clone(ctx: &Self::Context) -> Self::Context {
                *ctx
            }

            fn $encrypt(
                key: &Self::Context,
                nonce: &[u8],
                aad: &[u8],
                buffer: InOutBuf<'_, '_, u8>,
                tag: &mut [u8; 16],
            ) -> Result<(), CryptoError> {
                let nonce: &[u8; 12] = nonce.try_into().map_err(|_| CryptoError::InvalidInput)?;
                let cipher = aes::AesGcm::new(key, nonce).map_err(map_error)?;
                let mut aes = aes();
                let mut op = aes.start(cipher, Direction::Encrypt);
                aes.blocking_aad(&mut op, aad, true).map_err(map_error)?;
                payload(&mut aes, &mut op, buffer, true).map_err(map_error)?;
                *tag = aes.blocking_finish(op).map_err(map_error)?.unwrap();
                Ok(())
            }

            fn $decrypt(
                key: &Self::Context,
                nonce: &[u8],
                aad: &[u8],
                buffer: InOutBuf<'_, '_, u8>,
                tag: &[u8; 16],
            ) -> Result<(), CryptoError> {
                let nonce: &[u8; 12] = nonce.try_into().map_err(|_| CryptoError::InvalidInput)?;
                let cipher = aes::AesGcm::new(key, nonce).map_err(map_error)?;
                let mut aes = aes();
                let mut op = aes.start(cipher, Direction::Decrypt);
                aes.blocking_aad(&mut op, aad, true).map_err(map_error)?;
                payload(&mut aes, &mut op, buffer, true).map_err(map_error)?;
                let computed = aes.blocking_finish(op).map_err(map_error)?.unwrap();
                if ct_eq(&computed, tag) {
                    Ok(())
                } else {
                    Err(CryptoError::InvalidSignature)
                }
            }
        }

        $impl_macro!($driver);
    };
}

impl_ecb!(
    Aes128EcbDriver,
    embassy_crypto_driver::Aes128Ecb,
    16,
    aes128ecb_init,
    aes128ecb_clone,
    aes128ecb_encrypt_blocks,
    aes128ecb_decrypt_blocks,
    embassy_crypto_driver::embassy_crypto_aes128ecb_impl
);
impl_cbc!(
    Aes128CbcDriver,
    embassy_crypto_driver::Aes128Cbc,
    16,
    aes128cbc_encrypt_init,
    aes128cbc_decrypt_init,
    aes128cbc_encrypt_blocks,
    aes128cbc_decrypt_blocks,
    embassy_crypto_driver::embassy_crypto_aes128cbc_impl
);
impl_ctr!(
    Aes128CtrDriver,
    embassy_crypto_driver::Aes128Ctr,
    16,
    aes128ctr_init,
    aes128ctr_apply_keystream,
    embassy_crypto_driver::embassy_crypto_aes128ctr_impl
);
impl_cmac!(
    Aes128CmacDriver,
    embassy_crypto_driver::Aes128Cmac,
    16,
    aes128cmac_init,
    aes128cmac_clone,
    aes128cmac_update,
    aes128cmac_finalize,
    aes128cmac_reset,
    embassy_crypto_driver::embassy_crypto_aes128cmac_impl
);
impl_ccm!(
    Aes128CcmDriver,
    embassy_crypto_driver::Aes128Ccm,
    16,
    aes128ccm_init,
    aes128ccm_clone,
    aes128ccm_encrypt,
    aes128ccm_decrypt,
    embassy_crypto_driver::embassy_crypto_aes128ccm_impl
);
#[cfg(any(feature = "_cryptocell-312", feature = "_cracen"))]
impl_gcm!(
    Aes128GcmDriver,
    embassy_crypto_driver::Aes128Gcm,
    16,
    aes128gcm_init,
    aes128gcm_clone,
    aes128gcm_encrypt,
    aes128gcm_decrypt,
    embassy_crypto_driver::embassy_crypto_aes128gcm_impl
);

#[cfg(any(feature = "_cryptocell-312", feature = "_cracen"))]
impl_ecb!(
    Aes256EcbDriver,
    embassy_crypto_driver::Aes256Ecb,
    32,
    aes256ecb_init,
    aes256ecb_clone,
    aes256ecb_encrypt_blocks,
    aes256ecb_decrypt_blocks,
    embassy_crypto_driver::embassy_crypto_aes256ecb_impl
);
#[cfg(any(feature = "_cryptocell-312", feature = "_cracen"))]
impl_cbc!(
    Aes256CbcDriver,
    embassy_crypto_driver::Aes256Cbc,
    32,
    aes256cbc_encrypt_init,
    aes256cbc_decrypt_init,
    aes256cbc_encrypt_blocks,
    aes256cbc_decrypt_blocks,
    embassy_crypto_driver::embassy_crypto_aes256cbc_impl
);
#[cfg(any(feature = "_cryptocell-312", feature = "_cracen"))]
impl_ctr!(
    Aes256CtrDriver,
    embassy_crypto_driver::Aes256Ctr,
    32,
    aes256ctr_init,
    aes256ctr_apply_keystream,
    embassy_crypto_driver::embassy_crypto_aes256ctr_impl
);
#[cfg(any(feature = "_cryptocell-312", feature = "_cracen"))]
impl_cmac!(
    Aes256CmacDriver,
    embassy_crypto_driver::Aes256Cmac,
    32,
    aes256cmac_init,
    aes256cmac_clone,
    aes256cmac_update,
    aes256cmac_finalize,
    aes256cmac_reset,
    embassy_crypto_driver::embassy_crypto_aes256cmac_impl
);
#[cfg(any(feature = "_cryptocell-312", feature = "_cracen"))]
impl_ccm!(
    Aes256CcmDriver,
    embassy_crypto_driver::Aes256Ccm,
    32,
    aes256ccm_init,
    aes256ccm_clone,
    aes256ccm_encrypt,
    aes256ccm_decrypt,
    embassy_crypto_driver::embassy_crypto_aes256ccm_impl
);
#[cfg(any(feature = "_cryptocell-312", feature = "_cracen"))]
impl_gcm!(
    Aes256GcmDriver,
    embassy_crypto_driver::Aes256Gcm,
    32,
    aes256gcm_init,
    aes256gcm_clone,
    aes256gcm_encrypt,
    aes256gcm_decrypt,
    embassy_crypto_driver::embassy_crypto_aes256gcm_impl
);
