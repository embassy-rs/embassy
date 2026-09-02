#![allow(missing_docs)]

//! Software fallback driver for `embassy-crypto-driver` using RustCrypto crates.
//!
//! Enable the `driver-rustcrypto` feature on `embassy-crypto` to use this driver.
//!
//! Each of this drivers will delegate to the crate type for lower layers rather than
//! the rustcrypto type. Therefore if the crate type is accelerated, the higher layer
//! benefits from the acceleration.
//!

use embassy_crypto_driver::CryptoError;

#[inline]
fn to_rustcrypto_inout<'inp, 'out>(
    buf: embassy_crypto_driver::InOutBuf<'inp, 'out, u8>,
) -> cipher::inout::InOutBuf<'inp, 'out, u8> {
    let len = buf.len();
    let (in_ptr, out_ptr) = buf.into_raw();
    unsafe { cipher::inout::InOutBuf::from_raw(in_ptr, out_ptr, len) }
}

// ===========================================================================
// Digests
// ===========================================================================

macro_rules! impl_digest {
    (
        $driver:ident,
        $trait:path,
        $ctx:ty,
        [$init:ident, $clone:ident, $update:ident, $finalize:ident],
        $impl_macro:path
    ) => {
        struct $driver;

        impl $trait for $driver {
            type Context = $ctx;

            fn $init() -> Self::Context {
                use digest::Digest;
                Self::Context::new()
            }

            fn $clone(ctx: &Self::Context) -> Self::Context {
                ctx.clone()
            }

            fn $update(ctx: &mut Self::Context, data: &[u8]) {
                use digest::Update;
                ctx.update(data);
            }

            fn $finalize(ctx: Self::Context, out: &mut [u8]) {
                use digest::FixedOutput;
                let result = ctx.finalize_fixed();
                out.copy_from_slice(result.as_slice());
            }
        }

        $impl_macro!($driver);
    };
}

#[cfg(feature = "driver-md5")]
impl_digest!(
    Md5Driver,
    embassy_crypto_driver::Md5,
    md5::Md5,
    [md5_init, md5_clone, md5_update, md5_finalize],
    embassy_crypto_driver::embassy_crypto_md5_impl
);

#[cfg(feature = "driver-sha1")]
impl_digest!(
    Sha1Driver,
    embassy_crypto_driver::Sha1,
    sha1::Sha1,
    [sha1_init, sha1_clone, sha1_update, sha1_finalize],
    embassy_crypto_driver::embassy_crypto_sha1_impl
);

#[cfg(feature = "driver-sha2")]
impl_digest!(
    Sha224Driver,
    embassy_crypto_driver::Sha224,
    sha2::Sha224,
    [sha224_init, sha224_clone, sha224_update, sha224_finalize],
    embassy_crypto_driver::embassy_crypto_sha224_impl
);

#[cfg(feature = "driver-sha2")]
impl_digest!(
    Sha256Driver,
    embassy_crypto_driver::Sha256,
    sha2::Sha256,
    [sha256_init, sha256_clone, sha256_update, sha256_finalize],
    embassy_crypto_driver::embassy_crypto_sha256_impl
);

#[cfg(feature = "driver-sha2")]
impl_digest!(
    Sha384Driver,
    embassy_crypto_driver::Sha384,
    sha2::Sha384,
    [sha384_init, sha384_clone, sha384_update, sha384_finalize],
    embassy_crypto_driver::embassy_crypto_sha384_impl
);

#[cfg(feature = "driver-sha2")]
impl_digest!(
    Sha512_224Driver,
    embassy_crypto_driver::Sha512_224,
    sha2::Sha512_224,
    [
        sha512_224_init,
        sha512_224_clone,
        sha512_224_update,
        sha512_224_finalize
    ],
    embassy_crypto_driver::embassy_crypto_sha512_224_impl
);

#[cfg(feature = "driver-sha2")]
impl_digest!(
    Sha512_256Driver,
    embassy_crypto_driver::Sha512_256,
    sha2::Sha512_256,
    [
        sha512_256_init,
        sha512_256_clone,
        sha512_256_update,
        sha512_256_finalize
    ],
    embassy_crypto_driver::embassy_crypto_sha512_256_impl
);

#[cfg(feature = "driver-sha2")]
impl_digest!(
    Sha512Driver,
    embassy_crypto_driver::Sha512,
    sha2::Sha512,
    [sha512_init, sha512_clone, sha512_update, sha512_finalize],
    embassy_crypto_driver::embassy_crypto_sha512_impl
);

// ===========================================================================
// HMACs
// ===========================================================================

// When the RustCrypto hash driver is also enabled, HMAC can use the efficient
// `hmac::Hmac<D>` type directly with the RustCrypto hash (bypassing the driver
// indirection). When the hash driver is *not* the RustCrypto one (e.g. hardware
// accelerated), HMAC falls back to `hmac::SimpleHmac<crate::Sha*>` so that hash
// operations are delegated through the embassy-crypto driver unitrait.

#[allow(unused_macros)]
macro_rules! impl_hmac_fast {
    (
        $driver:ident,
        $ctx:ident,
        $trait:path,
        $hash:ty,
        $key_cap:literal,
        [$init:ident, $clone:ident, $update:ident, $finalize:ident],
        $impl_macro:path
    ) => {
        #[derive(Clone)]
        struct $ctx {
            inner: hmac::Hmac<$hash>,
        }

        struct $driver;

        impl $trait for $driver {
            type Context = $ctx;

            fn $init(key: &[u8]) -> Self::Context {
                use hmac::KeyInit;
                let inner = hmac::Hmac::new_from_slice(key).expect("key length validated by caller");
                $ctx { inner }
            }

            fn $clone(ctx: &Self::Context) -> Self::Context {
                ctx.clone()
            }

            fn $update(ctx: &mut Self::Context, data: &[u8]) {
                use hmac::Mac;
                ctx.inner.update(data);
            }

            fn $finalize(ctx: Self::Context, out: &mut [u8]) {
                use hmac::Mac;
                let result = ctx.inner.finalize();
                out.copy_from_slice(result.into_bytes().as_slice());
            }
        }

        $impl_macro!($driver);
    };
}

#[allow(unused_macros)]
macro_rules! impl_hmac_delegated {
    (
        $driver:ident,
        $ctx:ident,
        $trait:path,
        $hash:ty,
        $key_cap:literal,
        [$init:ident, $clone:ident, $update:ident, $finalize:ident],
        $impl_macro:path
    ) => {
        #[derive(Clone)]
        struct $ctx {
            inner: hmac::SimpleHmac<$hash>,
        }

        struct $driver;

        impl $trait for $driver {
            type Context = $ctx;

            fn $init(key: &[u8]) -> Self::Context {
                use hmac::KeyInit;
                let inner = hmac::SimpleHmac::new_from_slice(key).expect("key length validated by caller");
                $ctx { inner }
            }

            fn $clone(ctx: &Self::Context) -> Self::Context {
                ctx.clone()
            }

            fn $update(ctx: &mut Self::Context, data: &[u8]) {
                use hmac::Mac;
                ctx.inner.update(data);
            }

            fn $finalize(ctx: Self::Context, out: &mut [u8]) {
                use hmac::Mac;
                let result = ctx.inner.finalize();
                out.copy_from_slice(result.into_bytes().as_slice());
            }
        }

        $impl_macro!($driver);
    };
}

// SHA-1 HMAC
#[cfg(all(feature = "driver-hmac-sha1", feature = "driver-sha1"))]
impl_hmac_fast!(
    HmacSha1Driver,
    HmacSha1Context,
    embassy_crypto_driver::HmacSha1,
    sha1::Sha1,
    64,
    [hmac_sha1_init, hmac_sha1_clone, hmac_sha1_update, hmac_sha1_finalize],
    embassy_crypto_driver::embassy_crypto_hmac_sha1_impl
);

#[cfg(all(feature = "driver-hmac-sha1", not(feature = "driver-sha1")))]
impl_hmac_delegated!(
    HmacSha1Driver,
    HmacSha1Context,
    embassy_crypto_driver::HmacSha1,
    crate::Sha1,
    64,
    [hmac_sha1_init, hmac_sha1_clone, hmac_sha1_update, hmac_sha1_finalize],
    embassy_crypto_driver::embassy_crypto_hmac_sha1_impl
);

// SHA-224 HMAC
#[cfg(all(feature = "driver-hmac-sha2", feature = "driver-sha2"))]
impl_hmac_fast!(
    HmacSha224Driver,
    HmacSha224Context,
    embassy_crypto_driver::HmacSha224,
    sha2::Sha224,
    64,
    [
        hmac_sha224_init,
        hmac_sha224_clone,
        hmac_sha224_update,
        hmac_sha224_finalize
    ],
    embassy_crypto_driver::embassy_crypto_hmac_sha224_impl
);

#[cfg(all(feature = "driver-hmac-sha2", not(feature = "driver-sha2")))]
impl_hmac_delegated!(
    HmacSha224Driver,
    HmacSha224Context,
    embassy_crypto_driver::HmacSha224,
    crate::Sha224,
    64,
    [
        hmac_sha224_init,
        hmac_sha224_clone,
        hmac_sha224_update,
        hmac_sha224_finalize
    ],
    embassy_crypto_driver::embassy_crypto_hmac_sha224_impl
);

// SHA-256 HMAC
#[cfg(all(feature = "driver-hmac-sha2", feature = "driver-sha2"))]
impl_hmac_fast!(
    HmacSha256Driver,
    HmacSha256Context,
    embassy_crypto_driver::HmacSha256,
    sha2::Sha256,
    64,
    [
        hmac_sha256_init,
        hmac_sha256_clone,
        hmac_sha256_update,
        hmac_sha256_finalize
    ],
    embassy_crypto_driver::embassy_crypto_hmac_sha256_impl
);

#[cfg(all(feature = "driver-hmac-sha2", not(feature = "driver-sha2")))]
impl_hmac_delegated!(
    HmacSha256Driver,
    HmacSha256Context,
    embassy_crypto_driver::HmacSha256,
    crate::Sha256,
    64,
    [
        hmac_sha256_init,
        hmac_sha256_clone,
        hmac_sha256_update,
        hmac_sha256_finalize
    ],
    embassy_crypto_driver::embassy_crypto_hmac_sha256_impl
);

// SHA-384 HMAC
#[cfg(all(feature = "driver-hmac-sha2", feature = "driver-sha2"))]
impl_hmac_fast!(
    HmacSha384Driver,
    HmacSha384Context,
    embassy_crypto_driver::HmacSha384,
    sha2::Sha384,
    128,
    [
        hmac_sha384_init,
        hmac_sha384_clone,
        hmac_sha384_update,
        hmac_sha384_finalize
    ],
    embassy_crypto_driver::embassy_crypto_hmac_sha384_impl
);

#[cfg(all(feature = "driver-hmac-sha2", not(feature = "driver-sha2")))]
impl_hmac_delegated!(
    HmacSha384Driver,
    HmacSha384Context,
    embassy_crypto_driver::HmacSha384,
    crate::Sha384,
    128,
    [
        hmac_sha384_init,
        hmac_sha384_clone,
        hmac_sha384_update,
        hmac_sha384_finalize
    ],
    embassy_crypto_driver::embassy_crypto_hmac_sha384_impl
);

// SHA-512/224 HMAC
#[cfg(all(feature = "driver-hmac-sha2", feature = "driver-sha2"))]
impl_hmac_fast!(
    HmacSha512_224Driver,
    HmacSha512_224Context,
    embassy_crypto_driver::HmacSha512_224,
    sha2::Sha512_224,
    128,
    [
        hmac_sha512_224_init,
        hmac_sha512_224_clone,
        hmac_sha512_224_update,
        hmac_sha512_224_finalize
    ],
    embassy_crypto_driver::embassy_crypto_hmac_sha512_224_impl
);

#[cfg(all(feature = "driver-hmac-sha2", not(feature = "driver-sha2")))]
impl_hmac_delegated!(
    HmacSha512_224Driver,
    HmacSha512_224Context,
    embassy_crypto_driver::HmacSha512_224,
    crate::Sha512_224,
    128,
    [
        hmac_sha512_224_init,
        hmac_sha512_224_clone,
        hmac_sha512_224_update,
        hmac_sha512_224_finalize
    ],
    embassy_crypto_driver::embassy_crypto_hmac_sha512_224_impl
);

// SHA-512/256 HMAC
#[cfg(all(feature = "driver-hmac-sha2", feature = "driver-sha2"))]
impl_hmac_fast!(
    HmacSha512_256Driver,
    HmacSha512_256Context,
    embassy_crypto_driver::HmacSha512_256,
    sha2::Sha512_256,
    128,
    [
        hmac_sha512_256_init,
        hmac_sha512_256_clone,
        hmac_sha512_256_update,
        hmac_sha512_256_finalize
    ],
    embassy_crypto_driver::embassy_crypto_hmac_sha512_256_impl
);

#[cfg(all(feature = "driver-hmac-sha2", not(feature = "driver-sha2")))]
impl_hmac_delegated!(
    HmacSha512_256Driver,
    HmacSha512_256Context,
    embassy_crypto_driver::HmacSha512_256,
    crate::Sha512_256,
    128,
    [
        hmac_sha512_256_init,
        hmac_sha512_256_clone,
        hmac_sha512_256_update,
        hmac_sha512_256_finalize
    ],
    embassy_crypto_driver::embassy_crypto_hmac_sha512_256_impl
);

// SHA-512 HMAC
#[cfg(all(feature = "driver-hmac-sha2", feature = "driver-sha2"))]
impl_hmac_fast!(
    HmacSha512Driver,
    HmacSha512Context,
    embassy_crypto_driver::HmacSha512,
    sha2::Sha512,
    128,
    [
        hmac_sha512_init,
        hmac_sha512_clone,
        hmac_sha512_update,
        hmac_sha512_finalize
    ],
    embassy_crypto_driver::embassy_crypto_hmac_sha512_impl
);

#[cfg(all(feature = "driver-hmac-sha2", not(feature = "driver-sha2")))]
impl_hmac_delegated!(
    HmacSha512Driver,
    HmacSha512Context,
    embassy_crypto_driver::HmacSha512,
    crate::Sha512,
    128,
    [
        hmac_sha512_init,
        hmac_sha512_clone,
        hmac_sha512_update,
        hmac_sha512_finalize
    ],
    embassy_crypto_driver::embassy_crypto_hmac_sha512_impl
);
// ===========================================================================
// AES ECB
// ===========================================================================

#[cfg(feature = "driver-aes128")]
struct Aes128EcbDriver;

#[cfg(feature = "driver-aes128")]
impl embassy_crypto_driver::Aes128Ecb for Aes128EcbDriver {
    type Context = aes::Aes128;

    fn aes128ecb_init(key: &[u8; 16]) -> Self::Context {
        use cipher::KeyInit;
        aes::Aes128::new_from_slice(key.as_slice()).unwrap()
    }

    fn aes128ecb_clone(ctx: &Self::Context) -> Self::Context {
        ctx.clone()
    }

    fn aes128ecb_encrypt_blocks(ctx: &Self::Context, blocks: embassy_crypto_driver::InOutBuf<'_, '_, u8>) {
        use cipher::BlockCipherEncrypt;
        let buf = to_rustcrypto_inout(blocks);
        let (chunks, _tail) = buf.into_chunks();
        ctx.encrypt_blocks_inout(chunks);
    }

    fn aes128ecb_decrypt_blocks(ctx: &Self::Context, blocks: embassy_crypto_driver::InOutBuf<'_, '_, u8>) {
        use cipher::BlockCipherDecrypt;
        let buf = to_rustcrypto_inout(blocks);
        let (chunks, _tail) = buf.into_chunks();
        ctx.decrypt_blocks_inout(chunks);
    }
}

#[cfg(feature = "driver-aes128")]
embassy_crypto_driver::embassy_crypto_aes128ecb_impl!(Aes128EcbDriver);

#[cfg(feature = "driver-aes256")]
struct Aes256EcbDriver;

#[cfg(feature = "driver-aes256")]
impl embassy_crypto_driver::Aes256Ecb for Aes256EcbDriver {
    type Context = aes::Aes256;

    fn aes256ecb_init(key: &[u8; 32]) -> Self::Context {
        use cipher::KeyInit;
        aes::Aes256::new_from_slice(key.as_slice()).unwrap()
    }

    fn aes256ecb_clone(ctx: &Self::Context) -> Self::Context {
        ctx.clone()
    }

    fn aes256ecb_encrypt_blocks(ctx: &Self::Context, blocks: embassy_crypto_driver::InOutBuf<'_, '_, u8>) {
        use cipher::BlockCipherEncrypt;
        let buf = to_rustcrypto_inout(blocks);
        let (chunks, _tail) = buf.into_chunks();
        ctx.encrypt_blocks_inout(chunks);
    }

    fn aes256ecb_decrypt_blocks(ctx: &Self::Context, blocks: embassy_crypto_driver::InOutBuf<'_, '_, u8>) {
        use cipher::BlockCipherDecrypt;
        let buf = to_rustcrypto_inout(blocks);
        let (chunks, _tail) = buf.into_chunks();
        ctx.decrypt_blocks_inout(chunks);
    }
}

#[cfg(feature = "driver-aes256")]
embassy_crypto_driver::embassy_crypto_aes256ecb_impl!(Aes256EcbDriver);

// ===========================================================================
// AES CBC
// ===========================================================================

#[cfg(feature = "driver-aes128cbc")]
struct Aes128CbcEncryptContext {
    inner: cbc::Encryptor<crate::Aes128>,
}

#[cfg(feature = "driver-aes128cbc")]
struct Aes128CbcDecryptContext {
    inner: cbc::Decryptor<crate::Aes128>,
}

#[cfg(feature = "driver-aes128cbc")]
struct Aes128CbcDriver;

#[cfg(feature = "driver-aes128cbc")]
impl embassy_crypto_driver::Aes128Cbc for Aes128CbcDriver {
    type EncryptContext = Aes128CbcEncryptContext;
    type DecryptContext = Aes128CbcDecryptContext;

    fn aes128cbc_encrypt_init(key: &[u8; 16], iv: &[u8; 16]) -> Self::EncryptContext {
        use cipher::KeyIvInit;
        Aes128CbcEncryptContext {
            inner: cbc::Encryptor::<crate::Aes128>::new_from_slices(key.as_slice(), iv.as_slice()).unwrap(),
        }
    }

    fn aes128cbc_decrypt_init(key: &[u8; 16], iv: &[u8; 16]) -> Self::DecryptContext {
        use cipher::KeyIvInit;
        Aes128CbcDecryptContext {
            inner: cbc::Decryptor::<crate::Aes128>::new_from_slices(key.as_slice(), iv.as_slice()).unwrap(),
        }
    }

    fn aes128cbc_encrypt_blocks(ctx: &mut Self::EncryptContext, buf: embassy_crypto_driver::InOutBuf<'_, '_, u8>) {
        use cipher::BlockModeEncrypt;
        let out = buf.into_out_with_copied_in();
        let nblocks = out.len() / 16;
        let blocks = unsafe {
            core::slice::from_raw_parts_mut(
                out.as_mut_ptr() as *mut cipher::Block<cbc::Encryptor<crate::Aes128>>,
                nblocks,
            )
        };
        ctx.inner.encrypt_blocks(blocks);
    }

    fn aes128cbc_decrypt_blocks(ctx: &mut Self::DecryptContext, buf: embassy_crypto_driver::InOutBuf<'_, '_, u8>) {
        use cipher::BlockModeDecrypt;
        let out = buf.into_out_with_copied_in();
        let nblocks = out.len() / 16;
        let blocks = unsafe {
            core::slice::from_raw_parts_mut(
                out.as_mut_ptr() as *mut cipher::Block<cbc::Decryptor<crate::Aes128>>,
                nblocks,
            )
        };
        ctx.inner.decrypt_blocks(blocks);
    }
}

#[cfg(feature = "driver-aes128cbc")]
embassy_crypto_driver::embassy_crypto_aes128cbc_impl!(Aes128CbcDriver);

#[cfg(feature = "driver-aes256cbc")]
struct Aes256CbcEncryptContext {
    inner: cbc::Encryptor<crate::Aes256>,
}

#[cfg(feature = "driver-aes256cbc")]
struct Aes256CbcDecryptContext {
    inner: cbc::Decryptor<crate::Aes256>,
}

#[cfg(feature = "driver-aes256cbc")]
struct Aes256CbcDriver;

#[cfg(feature = "driver-aes256cbc")]
impl embassy_crypto_driver::Aes256Cbc for Aes256CbcDriver {
    type EncryptContext = Aes256CbcEncryptContext;
    type DecryptContext = Aes256CbcDecryptContext;

    fn aes256cbc_encrypt_init(key: &[u8; 32], iv: &[u8; 16]) -> Self::EncryptContext {
        use cipher::KeyIvInit;
        Aes256CbcEncryptContext {
            inner: cbc::Encryptor::<crate::Aes256>::new_from_slices(key.as_slice(), iv.as_slice()).unwrap(),
        }
    }

    fn aes256cbc_decrypt_init(key: &[u8; 32], iv: &[u8; 16]) -> Self::DecryptContext {
        use cipher::KeyIvInit;
        Aes256CbcDecryptContext {
            inner: cbc::Decryptor::<crate::Aes256>::new_from_slices(key.as_slice(), iv.as_slice()).unwrap(),
        }
    }

    fn aes256cbc_encrypt_blocks(ctx: &mut Self::EncryptContext, buf: embassy_crypto_driver::InOutBuf<'_, '_, u8>) {
        use cipher::BlockModeEncrypt;
        let out = buf.into_out_with_copied_in();
        let nblocks = out.len() / 16;
        let blocks = unsafe {
            core::slice::from_raw_parts_mut(
                out.as_mut_ptr() as *mut cipher::Block<cbc::Encryptor<crate::Aes256>>,
                nblocks,
            )
        };
        ctx.inner.encrypt_blocks(blocks);
    }

    fn aes256cbc_decrypt_blocks(ctx: &mut Self::DecryptContext, buf: embassy_crypto_driver::InOutBuf<'_, '_, u8>) {
        use cipher::BlockModeDecrypt;
        let out = buf.into_out_with_copied_in();
        let nblocks = out.len() / 16;
        let blocks = unsafe {
            core::slice::from_raw_parts_mut(
                out.as_mut_ptr() as *mut cipher::Block<cbc::Decryptor<crate::Aes256>>,
                nblocks,
            )
        };
        ctx.inner.decrypt_blocks(blocks);
    }
}

#[cfg(feature = "driver-aes256cbc")]
embassy_crypto_driver::embassy_crypto_aes256cbc_impl!(Aes256CbcDriver);

// ===========================================================================
// AES GCM
// ===========================================================================

#[cfg(feature = "driver-aes128gcm")]
struct Aes128GcmDriver;

#[cfg(feature = "driver-aes128gcm")]
impl embassy_crypto_driver::Aes128Gcm for Aes128GcmDriver {
    type Context = aes_gcm::AesGcm<crate::Aes128, generic_array::typenum::U12>;

    fn aes128gcm_init(key: &[u8; 16]) -> Self::Context {
        use aead::KeyInit;
        aes_gcm::AesGcm::<crate::Aes128, generic_array::typenum::U12>::new_from_slice(key.as_slice()).unwrap()
    }

    fn aes128gcm_clone(ctx: &Self::Context) -> Self::Context {
        ctx.clone()
    }

    fn aes128gcm_encrypt(
        ctx: &Self::Context,
        nonce: &[u8],
        aad: &[u8],
        buffer: embassy_crypto_driver::InOutBuf<'_, '_, u8>,
        tag: &mut [u8; 16],
    ) -> Result<(), CryptoError> {
        use aead::AeadInOut;
        let nonce = aead::Nonce::<aes_gcm::Aes128Gcm>::try_from(nonce).unwrap();
        let computed_tag = ctx
            .encrypt_inout_detached(&nonce, aad, to_rustcrypto_inout(buffer))
            .map_err(|_| CryptoError::InvalidInput)?;
        tag.copy_from_slice(computed_tag.as_slice());
        Ok(())
    }

    fn aes128gcm_decrypt(
        ctx: &Self::Context,
        nonce: &[u8],
        aad: &[u8],
        buffer: embassy_crypto_driver::InOutBuf<'_, '_, u8>,
        tag: &[u8; 16],
    ) -> Result<(), CryptoError> {
        use aead::AeadInOut;
        let nonce = aead::Nonce::<aes_gcm::Aes128Gcm>::try_from(nonce).unwrap();
        let tag_ga = aead::Tag::<aes_gcm::Aes128Gcm>::try_from(tag.as_slice()).unwrap();
        ctx.decrypt_inout_detached(&nonce, aad, to_rustcrypto_inout(buffer), &tag_ga)
            .map_err(|_| CryptoError::InvalidSignature)
    }
}

#[cfg(feature = "driver-aes128gcm")]
embassy_crypto_driver::embassy_crypto_aes128gcm_impl!(Aes128GcmDriver);

#[cfg(feature = "driver-aes256gcm")]
struct Aes256GcmDriver;

#[cfg(feature = "driver-aes256gcm")]
impl embassy_crypto_driver::Aes256Gcm for Aes256GcmDriver {
    type Context = aes_gcm::AesGcm<crate::Aes256, generic_array::typenum::U12>;

    fn aes256gcm_init(key: &[u8; 32]) -> Self::Context {
        use aead::KeyInit;
        aes_gcm::AesGcm::<crate::Aes256, generic_array::typenum::U12>::new_from_slice(key.as_slice()).unwrap()
    }

    fn aes256gcm_clone(ctx: &Self::Context) -> Self::Context {
        ctx.clone()
    }

    fn aes256gcm_encrypt(
        ctx: &Self::Context,
        nonce: &[u8],
        aad: &[u8],
        buffer: embassy_crypto_driver::InOutBuf<'_, '_, u8>,
        tag: &mut [u8; 16],
    ) -> Result<(), CryptoError> {
        use aead::AeadInOut;
        let nonce = aead::Nonce::<aes_gcm::Aes256Gcm>::try_from(nonce).unwrap();
        let computed_tag = ctx
            .encrypt_inout_detached(&nonce, aad, to_rustcrypto_inout(buffer))
            .map_err(|_| CryptoError::InvalidInput)?;
        tag.copy_from_slice(computed_tag.as_slice());
        Ok(())
    }

    fn aes256gcm_decrypt(
        ctx: &Self::Context,
        nonce: &[u8],
        aad: &[u8],
        buffer: embassy_crypto_driver::InOutBuf<'_, '_, u8>,
        tag: &[u8; 16],
    ) -> Result<(), CryptoError> {
        use aead::AeadInOut;
        let nonce = aead::Nonce::<aes_gcm::Aes256Gcm>::try_from(nonce).unwrap();
        let tag_ga = aead::Tag::<aes_gcm::Aes256Gcm>::try_from(tag.as_slice()).unwrap();
        ctx.decrypt_inout_detached(&nonce, aad, to_rustcrypto_inout(buffer), &tag_ga)
            .map_err(|_| CryptoError::InvalidSignature)
    }
}

#[cfg(feature = "driver-aes256gcm")]
embassy_crypto_driver::embassy_crypto_aes256gcm_impl!(Aes256GcmDriver);

// ===========================================================================
// AES CCM
// ===========================================================================

#[cfg(any(feature = "driver-aes128ccm", feature = "driver-aes256ccm"))]
macro_rules! ccm_case {
    (encrypt, $cipher:ty, $cipher_inst:expr, $nonce:expr, $aad:expr, $buffer:expr, $tag:expr, $tag_len:ty, $nonce_len:ty) => {{
        type C = ccm::Ccm<$cipher, $tag_len, $nonce_len>;
        let ccm = C::from($cipher_inst.clone());
        let t = ccm
            .encrypt_inout_detached($nonce.try_into().unwrap(), $aad, InOutBuf::from($buffer))
            .map_err(|_| CryptoError::InvalidInput)?;
        $tag.copy_from_slice(t.as_slice());
        Ok(())
    }};
    (decrypt, $cipher:ty, $cipher_inst:expr, $nonce:expr, $aad:expr, $buffer:expr, $tag:expr, $tag_len:ty, $nonce_len:ty) => {{
        type C = ccm::Ccm<$cipher, $tag_len, $nonce_len>;
        let ccm = C::from($cipher_inst.clone());
        ccm.decrypt_inout_detached(
            $nonce.try_into().unwrap(),
            $aad,
            InOutBuf::from($buffer),
            $tag.try_into().unwrap(),
        )
        .map_err(|_| CryptoError::InvalidSignature)
    }};
}

#[cfg(any(feature = "driver-aes128ccm", feature = "driver-aes256ccm"))]
macro_rules! ccm_nonce_match {
    ($op:ident, $cipher:ty, $cipher_inst:expr, $nonce:expr, $aad:expr, $buffer:expr, $tag:expr, $tag_len:ty) => {
        match $nonce.len() {
            7 => ccm_case!(
                $op,
                $cipher,
                $cipher_inst,
                $nonce,
                $aad,
                $buffer,
                $tag,
                $tag_len,
                ccm::consts::U7
            ),
            8 => ccm_case!(
                $op,
                $cipher,
                $cipher_inst,
                $nonce,
                $aad,
                $buffer,
                $tag,
                $tag_len,
                ccm::consts::U8
            ),
            9 => ccm_case!(
                $op,
                $cipher,
                $cipher_inst,
                $nonce,
                $aad,
                $buffer,
                $tag,
                $tag_len,
                ccm::consts::U9
            ),
            10 => ccm_case!(
                $op,
                $cipher,
                $cipher_inst,
                $nonce,
                $aad,
                $buffer,
                $tag,
                $tag_len,
                ccm::consts::U10
            ),
            11 => ccm_case!(
                $op,
                $cipher,
                $cipher_inst,
                $nonce,
                $aad,
                $buffer,
                $tag,
                $tag_len,
                ccm::consts::U11
            ),
            12 => ccm_case!(
                $op,
                $cipher,
                $cipher_inst,
                $nonce,
                $aad,
                $buffer,
                $tag,
                $tag_len,
                ccm::consts::U12
            ),
            13 => ccm_case!(
                $op,
                $cipher,
                $cipher_inst,
                $nonce,
                $aad,
                $buffer,
                $tag,
                $tag_len,
                ccm::consts::U13
            ),
            _ => Err(CryptoError::Unsupported),
        }
    };
}

#[cfg(any(feature = "driver-aes128ccm", feature = "driver-aes256ccm"))]
macro_rules! ccm_dispatch {
    ($op:ident, $cipher:ty, $cipher_inst:expr, $nonce:expr, $aad:expr, $buffer:expr, $tag:expr) => {
        match $tag.len() {
            4 => ccm_nonce_match!($op, $cipher, $cipher_inst, $nonce, $aad, $buffer, $tag, ccm::consts::U4),
            6 => ccm_nonce_match!($op, $cipher, $cipher_inst, $nonce, $aad, $buffer, $tag, ccm::consts::U6),
            8 => ccm_nonce_match!($op, $cipher, $cipher_inst, $nonce, $aad, $buffer, $tag, ccm::consts::U8),
            10 => ccm_nonce_match!(
                $op,
                $cipher,
                $cipher_inst,
                $nonce,
                $aad,
                $buffer,
                $tag,
                ccm::consts::U10
            ),
            12 => ccm_nonce_match!(
                $op,
                $cipher,
                $cipher_inst,
                $nonce,
                $aad,
                $buffer,
                $tag,
                ccm::consts::U12
            ),
            14 => ccm_nonce_match!(
                $op,
                $cipher,
                $cipher_inst,
                $nonce,
                $aad,
                $buffer,
                $tag,
                ccm::consts::U14
            ),
            16 => ccm_nonce_match!(
                $op,
                $cipher,
                $cipher_inst,
                $nonce,
                $aad,
                $buffer,
                $tag,
                ccm::consts::U16
            ),
            _ => Err(CryptoError::Unsupported),
        }
    };
}

#[cfg(feature = "driver-aes128ccm")]
#[derive(Clone)]
struct Aes128CcmContext {
    cipher: crate::Aes128,
}

#[cfg(feature = "driver-aes128ccm")]
struct Aes128CcmDriver;

#[cfg(feature = "driver-aes128ccm")]
impl embassy_crypto_driver::Aes128Ccm for Aes128CcmDriver {
    type Context = Aes128CcmContext;

    fn aes128ccm_init(key: &[u8; 16]) -> Self::Context {
        use cipher::KeyInit;
        Aes128CcmContext {
            cipher: crate::Aes128::new_from_slice(key.as_slice()).unwrap(),
        }
    }

    fn aes128ccm_clone(ctx: &Self::Context) -> Self::Context {
        ctx.clone()
    }

    fn aes128ccm_encrypt(
        ctx: &Self::Context,
        nonce: &[u8],
        aad: &[u8],
        buffer: embassy_crypto_driver::InOutBuf<'_, '_, u8>,
        tag: &mut [u8],
    ) -> Result<(), CryptoError> {
        use aead::AeadInOut;
        use aead::inout::InOutBuf;
        ccm_dispatch!(
            encrypt,
            crate::Aes128,
            &ctx.cipher,
            nonce,
            aad,
            to_rustcrypto_inout(buffer),
            tag
        )
    }

    fn aes128ccm_decrypt(
        ctx: &Self::Context,
        nonce: &[u8],
        aad: &[u8],
        buffer: embassy_crypto_driver::InOutBuf<'_, '_, u8>,
        tag: &[u8],
    ) -> Result<(), CryptoError> {
        use aead::AeadInOut;
        use aead::inout::InOutBuf;
        ccm_dispatch!(
            decrypt,
            crate::Aes128,
            &ctx.cipher,
            nonce,
            aad,
            to_rustcrypto_inout(buffer),
            tag
        )
    }
}

#[cfg(feature = "driver-aes128ccm")]
embassy_crypto_driver::embassy_crypto_aes128ccm_impl!(Aes128CcmDriver);

#[cfg(feature = "driver-aes256ccm")]
#[derive(Clone)]
struct Aes256CcmContext {
    cipher: crate::Aes256,
}

#[cfg(feature = "driver-aes256ccm")]
struct Aes256CcmDriver;

#[cfg(feature = "driver-aes256ccm")]
impl embassy_crypto_driver::Aes256Ccm for Aes256CcmDriver {
    type Context = Aes256CcmContext;

    fn aes256ccm_init(key: &[u8; 32]) -> Self::Context {
        use cipher::KeyInit;
        Aes256CcmContext {
            cipher: crate::Aes256::new_from_slice(key.as_slice()).unwrap(),
        }
    }

    fn aes256ccm_clone(ctx: &Self::Context) -> Self::Context {
        ctx.clone()
    }

    fn aes256ccm_encrypt(
        ctx: &Self::Context,
        nonce: &[u8],
        aad: &[u8],
        buffer: embassy_crypto_driver::InOutBuf<'_, '_, u8>,
        tag: &mut [u8],
    ) -> Result<(), CryptoError> {
        use aead::AeadInOut;
        use aead::inout::InOutBuf;
        ccm_dispatch!(
            encrypt,
            crate::Aes256,
            &ctx.cipher,
            nonce,
            aad,
            to_rustcrypto_inout(buffer),
            tag
        )
    }

    fn aes256ccm_decrypt(
        ctx: &Self::Context,
        nonce: &[u8],
        aad: &[u8],
        buffer: embassy_crypto_driver::InOutBuf<'_, '_, u8>,
        tag: &[u8],
    ) -> Result<(), CryptoError> {
        use aead::AeadInOut;
        use aead::inout::InOutBuf;
        ccm_dispatch!(
            decrypt,
            crate::Aes256,
            &ctx.cipher,
            nonce,
            aad,
            to_rustcrypto_inout(buffer),
            tag
        )
    }
}

#[cfg(feature = "driver-aes256ccm")]
embassy_crypto_driver::embassy_crypto_aes256ccm_impl!(Aes256CcmDriver);

// ===========================================================================
// AES CTR (software fallback via RustCrypto `ctr` crate)
// ===========================================================================

// ---------------------------------------------------------------------------
// AES-128 CTR
// ---------------------------------------------------------------------------

#[cfg(feature = "driver-aes128ctr")]
struct Aes128CtrContext {
    inner: ctr::Ctr128BE<crate::Aes128>,
}

#[cfg(feature = "driver-aes128ctr")]
struct Aes128CtrDriver;

#[cfg(feature = "driver-aes128ctr")]
impl embassy_crypto_driver::Aes128Ctr for Aes128CtrDriver {
    type Context = Aes128CtrContext;

    fn aes128ctr_init(key: &[u8; 16], iv: &[u8; 16]) -> Self::Context {
        use cipher::{InnerIvInit, KeyInit};
        let cipher = crate::Aes128::new_from_slice(key.as_slice()).unwrap();
        let nonce = cipher::Block::<crate::Aes128>::from(*iv);
        let core = ctr::CtrCore::<crate::Aes128, ctr::flavors::Ctr128BE>::inner_iv_init(cipher, &nonce);
        Aes128CtrContext {
            inner: ctr::Ctr128BE::from_core(core),
        }
    }

    fn aes128ctr_apply_keystream(ctx: &mut Self::Context, buf: embassy_crypto_driver::InOutBuf<'_, '_, u8>) {
        use cipher::StreamCipher;
        ctx.inner.unchecked_apply_keystream_inout(to_rustcrypto_inout(buf));
    }
}

#[cfg(feature = "driver-aes128ctr")]
embassy_crypto_driver::embassy_crypto_aes128ctr_impl!(Aes128CtrDriver);

// ---------------------------------------------------------------------------
// AES-256 CTR
// ---------------------------------------------------------------------------

#[cfg(feature = "driver-aes256ctr")]
struct Aes256CtrContext {
    inner: ctr::Ctr128BE<crate::Aes256>,
}

#[cfg(feature = "driver-aes256ctr")]
struct Aes256CtrDriver;

#[cfg(feature = "driver-aes256ctr")]
impl embassy_crypto_driver::Aes256Ctr for Aes256CtrDriver {
    type Context = Aes256CtrContext;

    fn aes256ctr_init(key: &[u8; 32], iv: &[u8; 16]) -> Self::Context {
        use cipher::{InnerIvInit, KeyInit};
        let cipher = crate::Aes256::new_from_slice(key.as_slice()).unwrap();
        let nonce = cipher::Block::<crate::Aes256>::from(*iv);
        let core = ctr::CtrCore::<crate::Aes256, ctr::flavors::Ctr128BE>::inner_iv_init(cipher, &nonce);
        Aes256CtrContext {
            inner: ctr::Ctr128BE::from_core(core),
        }
    }

    fn aes256ctr_apply_keystream(ctx: &mut Self::Context, buf: embassy_crypto_driver::InOutBuf<'_, '_, u8>) {
        use cipher::StreamCipher;
        ctx.inner.unchecked_apply_keystream_inout(to_rustcrypto_inout(buf));
    }
}

#[cfg(feature = "driver-aes256ctr")]
embassy_crypto_driver::embassy_crypto_aes256ctr_impl!(Aes256CtrDriver);

// ===========================================================================
// AES CMAC
// ===========================================================================

#[cfg(feature = "driver-aes128cmac")]
struct Aes128CmacDriver;

#[cfg(feature = "driver-aes128cmac")]
impl embassy_crypto_driver::Aes128Cmac for Aes128CmacDriver {
    type Context = cmac::Cmac<crate::Aes128>;

    fn aes128cmac_init(key: &[u8; 16]) -> Self::Context {
        use cipher::KeyInit;
        cmac::Cmac::<crate::Aes128>::new_from_slice(key.as_slice()).unwrap()
    }

    fn aes128cmac_clone(ctx: &Self::Context) -> Self::Context {
        ctx.clone()
    }

    fn aes128cmac_update(ctx: &mut Self::Context, data: &[u8]) {
        use digest::Update;
        ctx.update(data);
    }

    fn aes128cmac_finalize(ctx: Self::Context, out: &mut [u8; 16]) {
        use digest::FixedOutput;
        let result = ctx.finalize_fixed();
        out.copy_from_slice(result.as_slice());
    }

    fn aes128cmac_reset(ctx: &mut Self::Context) {
        use digest::Reset;
        ctx.reset();
    }
}

#[cfg(feature = "driver-aes128cmac")]
embassy_crypto_driver::embassy_crypto_aes128cmac_impl!(Aes128CmacDriver);

#[cfg(feature = "driver-aes256cmac")]
struct Aes256CmacDriver;

#[cfg(feature = "driver-aes256cmac")]
impl embassy_crypto_driver::Aes256Cmac for Aes256CmacDriver {
    type Context = cmac::Cmac<crate::Aes256>;

    fn aes256cmac_init(key: &[u8; 32]) -> Self::Context {
        use cipher::KeyInit;
        cmac::Cmac::<crate::Aes256>::new_from_slice(key.as_slice()).unwrap()
    }

    fn aes256cmac_clone(ctx: &Self::Context) -> Self::Context {
        ctx.clone()
    }

    fn aes256cmac_update(ctx: &mut Self::Context, data: &[u8]) {
        use digest::Update;
        ctx.update(data);
    }

    fn aes256cmac_finalize(ctx: Self::Context, out: &mut [u8; 16]) {
        use digest::FixedOutput;
        let result = ctx.finalize_fixed();
        out.copy_from_slice(result.as_slice());
    }

    fn aes256cmac_reset(ctx: &mut Self::Context) {
        use digest::Reset;
        ctx.reset();
    }
}

#[cfg(feature = "driver-aes256cmac")]
embassy_crypto_driver::embassy_crypto_aes256cmac_impl!(Aes256CmacDriver);
