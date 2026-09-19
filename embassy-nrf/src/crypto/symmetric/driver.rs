#![allow(dead_code, unused_imports, unused_macros)]

use embassy_crypto::Error;
use embassy_crypto::driver::InOutBuf;
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::PanicRawMutex;

use super::{
    self as symmetric, AesCbc, AesCcm, AesCmac, AesContext, AesCtr, AesEcb, ChaChaContext, ChaChaVariant, Direction,
    HashContext, HmacContext, Sha1, Sha224, Sha256, Symmetric,
};
#[cfg(feature = "_cracen")]
use super::{Sha384, Sha512, Sha512_224, Sha512_256};
use crate::mode::Blocking;

type Engine = Symmetric<'static, Blocking>;

static LOCK: Mutex<PanicRawMutex, ()> = Mutex::new(());

/// Runs `f` with the symmetric engines locked and powered.
fn with_engine<R>(f: impl FnOnce(&mut Engine) -> R) -> R {
    LOCK.lock(|_| {
        let mut engine = Engine::new_inner();
        f(&mut engine)
    })
}

fn map_error(error: symmetric::Error) -> Error {
    match error {
        symmetric::Error::InvalidKeyLength => Error::InvalidKey,
        symmetric::Error::InvalidNonceLength
        | symmetric::Error::InvalidTagLength
        | symmetric::Error::InvalidLength
        | symmetric::Error::AadAfterPayload => Error::InvalidInput,
        symmetric::Error::Hardware => Error::HardwareError,
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
fn payload<C: symmetric::Cipher>(
    aes: &mut Engine,
    ctx: &mut AesContext<C>,
    buf: InOutBuf<'_, '_, u8>,
    last: bool,
) -> Result<(), symmetric::Error> {
    let len = buf.len();
    let (input, output) = buf.into_raw();
    if input == output as *const u8 {
        let data = unsafe { core::slice::from_raw_parts_mut(output, len) };
        aes.aes_blocking_payload_in_place(ctx, data, last)
    } else {
        let input = unsafe { core::slice::from_raw_parts(input, len) };
        let output = unsafe { core::slice::from_raw_parts_mut(output, len) };
        aes.aes_blocking_payload(ctx, input, output, last)
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
                type Context = HashContext<$algo>;

                fn init() -> Self::Context {
                    with_engine(|e| e.hash_start::<$algo>())
                }

                fn update(ctx: &mut Self::Context, data: &[u8]) {
                    with_engine(|e| e.hash_blocking_update(ctx, data))
                }

                fn finalize(ctx: Self::Context, out: &mut [u8; $size]) {
                    with_engine(|e| out.copy_from_slice(e.hash_blocking_finish(ctx).as_ref()));
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
                type Context = HmacContext<$algo>;

                fn init(key: &[u8]) -> Self::Context {
                    with_engine(|e| e.hmac_start::<$algo>(key))
                }

                fn update(ctx: &mut Self::Context, data: &[u8]) {
                    with_engine(|e| e.hash_blocking_update(ctx, data))
                }

                fn finalize(ctx: Self::Context, out: &mut [u8; $size]) {
                    with_engine(|e| out.copy_from_slice(e.hash_blocking_finish(ctx).as_ref()));
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
    Sha384,
    48,
    sha384_impl
);
impl_digest!(
    #[cfg(all(feature = "embassy-crypto-sha512", feature = "_cracen"))]
    sha512_driver,
    Sha512,
    Sha512,
    64,
    sha512_impl
);
impl_digest!(
    #[cfg(all(feature = "embassy-crypto-sha512-224", feature = "_cracen"))]
    sha512_224_driver,
    Sha512_224,
    Sha512_224,
    28,
    sha512_224_impl
);
impl_digest!(
    #[cfg(all(feature = "embassy-crypto-sha512-256", feature = "_cracen"))]
    sha512_256_driver,
    Sha512_256,
    Sha512_256,
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
    Sha384,
    48,
    hmac_sha384_impl
);
impl_hmac!(
    #[cfg(all(feature = "embassy-crypto-hmac-sha512", feature = "_cracen"))]
    hmac_sha512_driver,
    HmacSha512,
    Sha512,
    64,
    hmac_sha512_impl
);
impl_hmac!(
    #[cfg(all(feature = "embassy-crypto-hmac-sha512-224", feature = "_cracen"))]
    hmac_sha512_224_driver,
    HmacSha512_224,
    Sha512_224,
    28,
    hmac_sha512_224_impl
);
impl_hmac!(
    #[cfg(all(feature = "embassy-crypto-hmac-sha512-256", feature = "_cracen"))]
    hmac_sha512_256_driver,
    HmacSha512_256,
    Sha512_256,
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
                    with_engine(|e| {
                        let mut op = e.aes_start(*ctx, Direction::Encrypt);
                        payload(e, &mut op, blocks, true).unwrap();
                    })
                }

                fn decrypt_blocks(ctx: &Self::Context, blocks: InOutBuf<'_, '_, u8>) {
                    with_engine(|e| {
                        let mut op = e.aes_start(*ctx, Direction::Decrypt);
                        payload(e, &mut op, blocks, true).unwrap();
                    })
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
                type EncryptContext = AesContext<AesCbc>;
                type DecryptContext = AesContext<AesCbc>;

                fn encrypt_init(key: &[u8; $key_len], iv: &[u8; 16]) -> Self::EncryptContext {
                    with_engine(|e| e.aes_start(AesCbc::new(key, iv).unwrap(), Direction::Encrypt))
                }

                fn decrypt_init(key: &[u8; $key_len], iv: &[u8; 16]) -> Self::DecryptContext {
                    with_engine(|e| e.aes_start(AesCbc::new(key, iv).unwrap(), Direction::Decrypt))
                }

                fn encrypt_blocks(ctx: &mut Self::EncryptContext, blocks: InOutBuf<'_, '_, u8>) {
                    with_engine(|e| payload(e, ctx, blocks, false).unwrap());
                }

                fn decrypt_blocks(ctx: &mut Self::DecryptContext, blocks: InOutBuf<'_, '_, u8>) {
                    with_engine(|e| payload(e, ctx, blocks, false).unwrap());
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
                type Context = AesContext<AesCtr>;

                fn init(key: &[u8; $key_len], iv: &[u8; 16]) -> Self::Context {
                    with_engine(|e| e.aes_start(AesCtr::new(key, iv).unwrap(), Direction::Encrypt))
                }

                fn apply_keystream(ctx: &mut Self::Context, buf: InOutBuf<'_, '_, u8>) {
                    with_engine(|e| payload(e, ctx, buf, false).unwrap());
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
                type Context = AesContext<AesCmac>;

                fn init(key: &[u8; $key_len]) -> Self::Context {
                    with_engine(|e| e.aes_start(AesCmac::new(key).unwrap(), Direction::Encrypt))
                }

                fn update(ctx: &mut Self::Context, data: &[u8]) {
                    with_engine(|e| e.aes_blocking_payload(ctx, data, &mut [], false).unwrap());
                }

                fn finalize(ctx: Self::Context, out: &mut [u8; 16]) {
                    with_engine(|e| *out = e.aes_blocking_finish(ctx).unwrap().unwrap());
                }

                fn reset(ctx: &mut Self::Context) {
                    let cipher = *ctx.cipher();
                    *ctx = with_engine(|e| e.aes_start(cipher, Direction::Encrypt));
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
                    with_engine(|e| {
                        let mut op = e.aes_start(cipher, Direction::Encrypt);
                        e.aes_blocking_aad(&mut op, aad, true).map_err(map_error)?;
                        payload(e, &mut op, buffer, true).map_err(map_error)?;
                        let computed = e.aes_blocking_finish(op).map_err(map_error)?.unwrap();
                        tag.copy_from_slice(&computed[..tag.len()]);
                        Ok(())
                    })
                }

                fn decrypt(
                    key: &Self::Context,
                    nonce: &[u8],
                    aad: &[u8],
                    buffer: InOutBuf<'_, '_, u8>,
                    tag: &[u8],
                ) -> Result<(), Error> {
                    let cipher = AesCcm::new(key, nonce, aad.len(), buffer.len(), tag.len()).map_err(map_error)?;
                    let computed = with_engine(|e| {
                        let mut op = e.aes_start(cipher, Direction::Decrypt);
                        e.aes_blocking_aad(&mut op, aad, true).map_err(map_error)?;
                        payload(e, &mut op, buffer, true).map_err(map_error)?;
                        Ok::<_, Error>(e.aes_blocking_finish(op).map_err(map_error)?.unwrap())
                    })?;
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
                    nonce: &[u8; 12],
                    aad: &[u8],
                    buffer: InOutBuf<'_, '_, u8>,
                    tag: &mut [u8; 16],
                ) -> Result<(), Error> {
                    let cipher = symmetric::AesGcm::new(key, nonce).map_err(map_error)?;
                    with_engine(|e| {
                        let mut op = e.aes_start(cipher, Direction::Encrypt);
                        e.aes_blocking_aad(&mut op, aad, true).map_err(map_error)?;
                        payload(e, &mut op, buffer, true).map_err(map_error)?;
                        *tag = e.aes_blocking_finish(op).map_err(map_error)?.unwrap();
                        Ok(())
                    })
                }

                fn decrypt(
                    key: &Self::Context,
                    nonce: &[u8; 12],
                    aad: &[u8],
                    buffer: InOutBuf<'_, '_, u8>,
                    tag: &[u8; 16],
                ) -> Result<(), Error> {
                    let cipher = symmetric::AesGcm::new(key, nonce).map_err(map_error)?;
                    let computed = with_engine(|e| {
                        let mut op = e.aes_start(cipher, Direction::Decrypt);
                        e.aes_blocking_aad(&mut op, aad, true).map_err(map_error)?;
                        payload(e, &mut op, buffer, true).map_err(map_error)?;
                        Ok::<_, Error>(e.aes_blocking_finish(op).map_err(map_error)?.unwrap())
                    })?;
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

// ===========================================================================
// ChaCha and ChaCha-Poly1305
// ===========================================================================

macro_rules! impl_chacha {
    ($(#[$meta:meta])* $driver:ident, $trait:ident, $variant:ident, $register:ident) => {
        $(#[$meta])*
        mod $driver {
            use super::*;

            struct Driver;

            impl embassy_crypto::driver::$trait for Driver {
                type Context = ChaChaContext;

                fn init(key: &[u8; 32], nonce: &[u8; 12], counter: u32) -> Self::Context {
                    with_engine(|e| e.chacha_start(ChaChaVariant::$variant, key, nonce, counter))
                }

                fn apply_keystream(ctx: &mut Self::Context, buf: InOutBuf<'_, '_, u8>) {
                    let len = buf.len();
                    let (input, output) = buf.into_raw();
                    with_engine(|e| {
                        if input == output as *const u8 {
                            let data = unsafe { core::slice::from_raw_parts_mut(output, len) };
                            e.chacha_blocking_apply_keystream_in_place(ctx, data);
                        } else {
                            let input = unsafe { core::slice::from_raw_parts(input, len) };
                            let output = unsafe { core::slice::from_raw_parts_mut(output, len) };
                            e.chacha_blocking_apply_keystream(ctx, input, output).unwrap();
                        }
                    })
                }
            }

            embassy_crypto::$register!(Driver);
        }
    };
}

macro_rules! impl_chachapoly {
    ($(#[$meta:meta])* $driver:ident, $trait:ident, $variant:ident, $register:ident) => {
        $(#[$meta])*
        mod $driver {
            use super::*;

            struct Driver;

            fn run(
                e: &mut Engine,
                key: &[u8; 32],
                nonce: &[u8; 12],
                aad: &[u8],
                buffer: InOutBuf<'_, '_, u8>,
                dir: Direction,
            ) -> Result<[u8; 16], Error> {
                let mut op = e.chachapoly_start(ChaChaVariant::$variant, key, nonce, dir);
                e.chachapoly_blocking_aad(&mut op, aad, true).map_err(map_error)?;
                let len = buffer.len();
                let (input, output) = buffer.into_raw();
                if input == output as *const u8 {
                    let data = unsafe { core::slice::from_raw_parts_mut(output, len) };
                    e.chachapoly_blocking_payload_in_place(&mut op, data, true)
                        .map_err(map_error)?;
                } else {
                    let input = unsafe { core::slice::from_raw_parts(input, len) };
                    let output = unsafe { core::slice::from_raw_parts_mut(output, len) };
                    e.chachapoly_blocking_payload(&mut op, input, output, true)
                        .map_err(map_error)?;
                }
                e.chachapoly_blocking_finish(op).map_err(map_error)
            }

            impl embassy_crypto::driver::$trait for Driver {
                type Context = [u8; 32];

                fn init(key: &[u8; 32]) -> Self::Context {
                    *key
                }

                fn encrypt(
                    key: &Self::Context,
                    nonce: &[u8; 12],
                    aad: &[u8],
                    buffer: InOutBuf<'_, '_, u8>,
                    tag: &mut [u8; 16],
                ) -> Result<(), Error> {
                    *tag = with_engine(|e| run(e, key, nonce, aad, buffer, Direction::Encrypt))?;
                    Ok(())
                }

                fn decrypt(
                    key: &Self::Context,
                    nonce: &[u8; 12],
                    aad: &[u8],
                    buffer: InOutBuf<'_, '_, u8>,
                    tag: &[u8; 16],
                ) -> Result<(), Error> {
                    let computed = with_engine(|e| run(e, key, nonce, aad, buffer, Direction::Decrypt))?;
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

impl_chacha!(
    #[cfg(all(feature = "embassy-crypto-chacha8", feature = "_cryptocell"))]
    chacha8_driver,
    ChaCha8,
    ChaCha8,
    chacha8_impl
);
impl_chacha!(
    #[cfg(all(feature = "embassy-crypto-chacha12", feature = "_cryptocell"))]
    chacha12_driver,
    ChaCha12,
    ChaCha12,
    chacha12_impl
);
impl_chacha!(
    #[cfg(feature = "embassy-crypto-chacha20")]
    chacha20_driver,
    ChaCha20,
    ChaCha20,
    chacha20_impl
);
impl_chachapoly!(
    #[cfg(all(feature = "embassy-crypto-chacha8-poly1305", feature = "_cryptocell"))]
    chacha8_poly1305_driver,
    ChaCha8Poly1305,
    ChaCha8,
    chacha8_poly1305_impl
);
impl_chachapoly!(
    #[cfg(all(feature = "embassy-crypto-chacha12-poly1305", feature = "_cryptocell"))]
    chacha12_poly1305_driver,
    ChaCha12Poly1305,
    ChaCha12,
    chacha12_poly1305_impl
);
impl_chachapoly!(
    #[cfg(feature = "embassy-crypto-chacha20-poly1305")]
    chacha20_poly1305_driver,
    ChaCha20Poly1305,
    ChaCha20,
    chacha20_poly1305_impl
);

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
