#![allow(missing_docs)]

//! Software fallback driver for `embassy-crypto-driver` using RustCrypto crates.
//!
//! Enable the `driver-rustcrypto` feature on `embassy-crypto` to use this driver.

use embassy_crypto_driver::CryptoError;

// ===========================================================================
// Digests
// ===========================================================================

struct Md5Driver;

impl embassy_crypto_driver::Md5 for Md5Driver {
    type Context = md5::Md5;

    fn md5_init() -> Self::Context {
        use digest::Digest;
        md5::Md5::new()
    }

    fn md5_clone(ctx: &Self::Context) -> Self::Context {
        ctx.clone()
    }

    fn md5_update(ctx: &mut Self::Context, data: &[u8]) {
        use digest::Update;
        ctx.update(data);
    }

    fn md5_finalize(ctx: Self::Context, out: &mut [u8]) {
        use digest::FixedOutput;
        let result = ctx.finalize_fixed();
        out.copy_from_slice(result.as_slice());
    }
}

embassy_crypto_driver::embassy_crypto_md5_impl!(Md5Driver);

struct Sha1Driver;

impl embassy_crypto_driver::Sha1 for Sha1Driver {
    type Context = sha1::Sha1;

    fn sha1_init() -> Self::Context {
        use digest::Digest;
        sha1::Sha1::new()
    }

    fn sha1_clone(ctx: &Self::Context) -> Self::Context {
        ctx.clone()
    }

    fn sha1_update(ctx: &mut Self::Context, data: &[u8]) {
        use digest::Update;
        ctx.update(data);
    }

    fn sha1_finalize(ctx: Self::Context, out: &mut [u8]) {
        use digest::FixedOutput;
        let result = ctx.finalize_fixed();
        out.copy_from_slice(result.as_slice());
    }
}

embassy_crypto_driver::embassy_crypto_sha1_impl!(Sha1Driver);

struct Sha224Driver;

impl embassy_crypto_driver::Sha224 for Sha224Driver {
    type Context = sha2::Sha224;

    fn sha224_init() -> Self::Context {
        use digest::Digest;
        sha2::Sha224::new()
    }

    fn sha224_clone(ctx: &Self::Context) -> Self::Context {
        ctx.clone()
    }

    fn sha224_update(ctx: &mut Self::Context, data: &[u8]) {
        use digest::Update;
        ctx.update(data);
    }

    fn sha224_finalize(ctx: Self::Context, out: &mut [u8]) {
        use digest::FixedOutput;
        let result = ctx.finalize_fixed();
        out.copy_from_slice(result.as_slice());
    }
}

embassy_crypto_driver::embassy_crypto_sha224_impl!(Sha224Driver);

struct Sha256Driver;

impl embassy_crypto_driver::Sha256 for Sha256Driver {
    type Context = sha2::Sha256;

    fn sha256_init() -> Self::Context {
        use digest::Digest;
        sha2::Sha256::new()
    }

    fn sha256_clone(ctx: &Self::Context) -> Self::Context {
        ctx.clone()
    }

    fn sha256_update(ctx: &mut Self::Context, data: &[u8]) {
        use digest::Update;
        ctx.update(data);
    }

    fn sha256_finalize(ctx: Self::Context, out: &mut [u8]) {
        use digest::FixedOutput;
        let result = ctx.finalize_fixed();
        out.copy_from_slice(result.as_slice());
    }
}

embassy_crypto_driver::embassy_crypto_sha256_impl!(Sha256Driver);

struct Sha384Driver;

impl embassy_crypto_driver::Sha384 for Sha384Driver {
    type Context = sha2::Sha384;

    fn sha384_init() -> Self::Context {
        use digest::Digest;
        sha2::Sha384::new()
    }

    fn sha384_clone(ctx: &Self::Context) -> Self::Context {
        ctx.clone()
    }

    fn sha384_update(ctx: &mut Self::Context, data: &[u8]) {
        use digest::Update;
        ctx.update(data);
    }

    fn sha384_finalize(ctx: Self::Context, out: &mut [u8]) {
        use digest::FixedOutput;
        let result = ctx.finalize_fixed();
        out.copy_from_slice(result.as_slice());
    }
}

embassy_crypto_driver::embassy_crypto_sha384_impl!(Sha384Driver);

struct Sha512_224Driver;

impl embassy_crypto_driver::Sha512_224 for Sha512_224Driver {
    type Context = sha2::Sha512_224;

    fn sha512_224_init() -> Self::Context {
        use digest::Digest;
        sha2::Sha512_224::new()
    }

    fn sha512_224_clone(ctx: &Self::Context) -> Self::Context {
        ctx.clone()
    }

    fn sha512_224_update(ctx: &mut Self::Context, data: &[u8]) {
        use digest::Update;
        ctx.update(data);
    }

    fn sha512_224_finalize(ctx: Self::Context, out: &mut [u8]) {
        use digest::FixedOutput;
        let result = ctx.finalize_fixed();
        out.copy_from_slice(result.as_slice());
    }
}

embassy_crypto_driver::embassy_crypto_sha512_224_impl!(Sha512_224Driver);

struct Sha512_256Driver;

impl embassy_crypto_driver::Sha512_256 for Sha512_256Driver {
    type Context = sha2::Sha512_256;

    fn sha512_256_init() -> Self::Context {
        use digest::Digest;
        sha2::Sha512_256::new()
    }

    fn sha512_256_clone(ctx: &Self::Context) -> Self::Context {
        ctx.clone()
    }

    fn sha512_256_update(ctx: &mut Self::Context, data: &[u8]) {
        use digest::Update;
        ctx.update(data);
    }

    fn sha512_256_finalize(ctx: Self::Context, out: &mut [u8]) {
        use digest::FixedOutput;
        let result = ctx.finalize_fixed();
        out.copy_from_slice(result.as_slice());
    }
}

embassy_crypto_driver::embassy_crypto_sha512_256_impl!(Sha512_256Driver);

struct Sha512Driver;

impl embassy_crypto_driver::Sha512 for Sha512Driver {
    type Context = sha2::Sha512;

    fn sha512_init() -> Self::Context {
        use digest::Digest;
        sha2::Sha512::new()
    }

    fn sha512_clone(ctx: &Self::Context) -> Self::Context {
        ctx.clone()
    }

    fn sha512_update(ctx: &mut Self::Context, data: &[u8]) {
        use digest::Update;
        ctx.update(data);
    }

    fn sha512_finalize(ctx: Self::Context, out: &mut [u8]) {
        use digest::FixedOutput;
        let result = ctx.finalize_fixed();
        out.copy_from_slice(result.as_slice());
    }
}

embassy_crypto_driver::embassy_crypto_sha512_impl!(Sha512Driver);

// ===========================================================================
// HMACs
// ===========================================================================

// HMAC reset is not available in hmac 0.13, so we store the key and recreate.

#[derive(Clone)]
struct HmacSha1Context {
    key: heapless::Vec<u8, 64>,
    inner: hmac::Hmac<sha1::Sha1>,
}

struct HmacSha1Driver;

impl embassy_crypto_driver::HmacSha1 for HmacSha1Driver {
    type Context = HmacSha1Context;

    fn hmac_sha1_init(key: &[u8]) -> Self::Context {
        use hmac::KeyInit;
        let mut key_storage = heapless::Vec::new();
        key_storage.extend_from_slice(key).unwrap();
        let inner = hmac::Hmac::new_from_slice(key).expect("key length validated by caller");
        HmacSha1Context {
            key: key_storage,
            inner,
        }
    }

    fn hmac_sha1_clone(ctx: &Self::Context) -> Self::Context {
        ctx.clone()
    }

    fn hmac_sha1_update(ctx: &mut Self::Context, data: &[u8]) {
        use hmac::Mac;
        ctx.inner.update(data);
    }

    fn hmac_sha1_finalize(ctx: Self::Context, out: &mut [u8]) {
        use hmac::Mac;
        let result = ctx.inner.finalize();
        out.copy_from_slice(result.into_bytes().as_slice());
    }

    fn hmac_sha1_reset(ctx: &mut Self::Context) {
        *ctx = Self::hmac_sha1_init(&ctx.key);
    }
}

embassy_crypto_driver::embassy_crypto_hmac_sha1_impl!(HmacSha1Driver);

#[derive(Clone)]
struct HmacSha224Context {
    key: heapless::Vec<u8, 64>,
    inner: hmac::Hmac<sha2::Sha224>,
}

struct HmacSha224Driver;

impl embassy_crypto_driver::HmacSha224 for HmacSha224Driver {
    type Context = HmacSha224Context;

    fn hmac_sha224_init(key: &[u8]) -> Self::Context {
        use hmac::KeyInit;
        let mut key_storage = heapless::Vec::new();
        key_storage.extend_from_slice(key).unwrap();
        let inner = hmac::Hmac::new_from_slice(key).expect("key length validated by caller");
        HmacSha224Context {
            key: key_storage,
            inner,
        }
    }

    fn hmac_sha224_clone(ctx: &Self::Context) -> Self::Context {
        ctx.clone()
    }

    fn hmac_sha224_update(ctx: &mut Self::Context, data: &[u8]) {
        use hmac::Mac;
        ctx.inner.update(data);
    }

    fn hmac_sha224_finalize(ctx: Self::Context, out: &mut [u8]) {
        use hmac::Mac;
        let result = ctx.inner.finalize();
        out.copy_from_slice(result.into_bytes().as_slice());
    }

    fn hmac_sha224_reset(ctx: &mut Self::Context) {
        *ctx = Self::hmac_sha224_init(&ctx.key);
    }
}

embassy_crypto_driver::embassy_crypto_hmac_sha224_impl!(HmacSha224Driver);

#[derive(Clone)]
struct HmacSha256Context {
    key: heapless::Vec<u8, 64>,
    inner: hmac::Hmac<sha2::Sha256>,
}

struct HmacSha256Driver;

impl embassy_crypto_driver::HmacSha256 for HmacSha256Driver {
    type Context = HmacSha256Context;

    fn hmac_sha256_init(key: &[u8]) -> Self::Context {
        use hmac::KeyInit;
        let mut key_storage = heapless::Vec::new();
        key_storage.extend_from_slice(key).unwrap();
        let inner = hmac::Hmac::new_from_slice(key).expect("key length validated by caller");
        HmacSha256Context {
            key: key_storage,
            inner,
        }
    }

    fn hmac_sha256_clone(ctx: &Self::Context) -> Self::Context {
        ctx.clone()
    }

    fn hmac_sha256_update(ctx: &mut Self::Context, data: &[u8]) {
        use hmac::Mac;
        ctx.inner.update(data);
    }

    fn hmac_sha256_finalize(ctx: Self::Context, out: &mut [u8]) {
        use hmac::Mac;
        let result = ctx.inner.finalize();
        out.copy_from_slice(result.into_bytes().as_slice());
    }

    fn hmac_sha256_reset(ctx: &mut Self::Context) {
        *ctx = Self::hmac_sha256_init(&ctx.key);
    }
}

embassy_crypto_driver::embassy_crypto_hmac_sha256_impl!(HmacSha256Driver);

#[derive(Clone)]
struct HmacSha384Context {
    key: heapless::Vec<u8, 128>,
    inner: hmac::Hmac<sha2::Sha384>,
}

struct HmacSha384Driver;

impl embassy_crypto_driver::HmacSha384 for HmacSha384Driver {
    type Context = HmacSha384Context;

    fn hmac_sha384_init(key: &[u8]) -> Self::Context {
        use hmac::KeyInit;
        let mut key_storage = heapless::Vec::new();
        key_storage.extend_from_slice(key).unwrap();
        let inner = hmac::Hmac::new_from_slice(key).expect("key length validated by caller");
        HmacSha384Context {
            key: key_storage,
            inner,
        }
    }

    fn hmac_sha384_clone(ctx: &Self::Context) -> Self::Context {
        ctx.clone()
    }

    fn hmac_sha384_update(ctx: &mut Self::Context, data: &[u8]) {
        use hmac::Mac;
        ctx.inner.update(data);
    }

    fn hmac_sha384_finalize(ctx: Self::Context, out: &mut [u8]) {
        use hmac::Mac;
        let result = ctx.inner.finalize();
        out.copy_from_slice(result.into_bytes().as_slice());
    }

    fn hmac_sha384_reset(ctx: &mut Self::Context) {
        *ctx = Self::hmac_sha384_init(&ctx.key);
    }
}

embassy_crypto_driver::embassy_crypto_hmac_sha384_impl!(HmacSha384Driver);

#[derive(Clone)]
struct HmacSha512_224Context {
    key: heapless::Vec<u8, 128>,
    inner: hmac::Hmac<sha2::Sha512_224>,
}

struct HmacSha512_224Driver;

impl embassy_crypto_driver::HmacSha512_224 for HmacSha512_224Driver {
    type Context = HmacSha512_224Context;

    fn hmac_sha512_224_init(key: &[u8]) -> Self::Context {
        use hmac::KeyInit;
        let mut key_storage = heapless::Vec::new();
        key_storage.extend_from_slice(key).unwrap();
        let inner = hmac::Hmac::new_from_slice(key).expect("key length validated by caller");
        HmacSha512_224Context {
            key: key_storage,
            inner,
        }
    }

    fn hmac_sha512_224_clone(ctx: &Self::Context) -> Self::Context {
        ctx.clone()
    }

    fn hmac_sha512_224_update(ctx: &mut Self::Context, data: &[u8]) {
        use hmac::Mac;
        ctx.inner.update(data);
    }

    fn hmac_sha512_224_finalize(ctx: Self::Context, out: &mut [u8]) {
        use hmac::Mac;
        let result = ctx.inner.finalize();
        out.copy_from_slice(result.into_bytes().as_slice());
    }

    fn hmac_sha512_224_reset(ctx: &mut Self::Context) {
        *ctx = Self::hmac_sha512_224_init(&ctx.key);
    }
}

embassy_crypto_driver::embassy_crypto_hmac_sha512_224_impl!(HmacSha512_224Driver);

#[derive(Clone)]
struct HmacSha512_256Context {
    key: heapless::Vec<u8, 128>,
    inner: hmac::Hmac<sha2::Sha512_256>,
}

struct HmacSha512_256Driver;

impl embassy_crypto_driver::HmacSha512_256 for HmacSha512_256Driver {
    type Context = HmacSha512_256Context;

    fn hmac_sha512_256_init(key: &[u8]) -> Self::Context {
        use hmac::KeyInit;
        let mut key_storage = heapless::Vec::new();
        key_storage.extend_from_slice(key).unwrap();
        let inner = hmac::Hmac::new_from_slice(key).expect("key length validated by caller");
        HmacSha512_256Context {
            key: key_storage,
            inner,
        }
    }

    fn hmac_sha512_256_clone(ctx: &Self::Context) -> Self::Context {
        ctx.clone()
    }

    fn hmac_sha512_256_update(ctx: &mut Self::Context, data: &[u8]) {
        use hmac::Mac;
        ctx.inner.update(data);
    }

    fn hmac_sha512_256_finalize(ctx: Self::Context, out: &mut [u8]) {
        use hmac::Mac;
        let result = ctx.inner.finalize();
        out.copy_from_slice(result.into_bytes().as_slice());
    }

    fn hmac_sha512_256_reset(ctx: &mut Self::Context) {
        *ctx = Self::hmac_sha512_256_init(&ctx.key);
    }
}

embassy_crypto_driver::embassy_crypto_hmac_sha512_256_impl!(HmacSha512_256Driver);

#[derive(Clone)]
struct HmacSha512Context {
    key: heapless::Vec<u8, 128>,
    inner: hmac::Hmac<sha2::Sha512>,
}

struct HmacSha512Driver;

impl embassy_crypto_driver::HmacSha512 for HmacSha512Driver {
    type Context = HmacSha512Context;

    fn hmac_sha512_init(key: &[u8]) -> Self::Context {
        use hmac::KeyInit;
        let mut key_storage = heapless::Vec::new();
        key_storage.extend_from_slice(key).unwrap();
        let inner = hmac::Hmac::new_from_slice(key).expect("key length validated by caller");
        HmacSha512Context {
            key: key_storage,
            inner,
        }
    }

    fn hmac_sha512_clone(ctx: &Self::Context) -> Self::Context {
        ctx.clone()
    }

    fn hmac_sha512_update(ctx: &mut Self::Context, data: &[u8]) {
        use hmac::Mac;
        ctx.inner.update(data);
    }

    fn hmac_sha512_finalize(ctx: Self::Context, out: &mut [u8]) {
        use hmac::Mac;
        let result = ctx.inner.finalize();
        out.copy_from_slice(result.into_bytes().as_slice());
    }

    fn hmac_sha512_reset(ctx: &mut Self::Context) {
        *ctx = Self::hmac_sha512_init(&ctx.key);
    }
}

embassy_crypto_driver::embassy_crypto_hmac_sha512_impl!(HmacSha512Driver);

// ===========================================================================
// AES ECB
// ===========================================================================

struct Aes128EcbDriver;

impl embassy_crypto_driver::Aes128Ecb for Aes128EcbDriver {
    type Context = aes::Aes128;

    fn aes128ecb_init(key: &[u8; 16]) -> Self::Context {
        use cipher::KeyInit;
        aes::Aes128::new_from_slice(key.as_slice()).unwrap()
    }

    fn aes128ecb_clone(ctx: &Self::Context) -> Self::Context {
        ctx.clone()
    }

    fn aes128ecb_encrypt_block(ctx: &Self::Context, blocks: &mut [[u8; 16]]) {
        use cipher::BlockCipherEncrypt;
        for block in blocks {
            let ga = unsafe { &mut *(block as *mut [u8; 16] as *mut cipher::Block<aes::Aes128>) };
            ctx.encrypt_block(ga);
        }
    }

    fn aes128ecb_decrypt_block(ctx: &Self::Context, blocks: &mut [[u8; 16]]) {
        use cipher::BlockCipherDecrypt;
        for block in blocks {
            let ga = unsafe { &mut *(block as *mut [u8; 16] as *mut cipher::Block<aes::Aes128>) };
            ctx.decrypt_block(ga);
        }
    }
}

embassy_crypto_driver::embassy_crypto_aes128ecb_impl!(Aes128EcbDriver);

struct Aes256EcbDriver;

impl embassy_crypto_driver::Aes256Ecb for Aes256EcbDriver {
    type Context = aes::Aes256;

    fn aes256ecb_init(key: &[u8; 32]) -> Self::Context {
        use cipher::KeyInit;
        aes::Aes256::new_from_slice(key.as_slice()).unwrap()
    }

    fn aes256ecb_clone(ctx: &Self::Context) -> Self::Context {
        ctx.clone()
    }

    fn aes256ecb_encrypt_block(ctx: &Self::Context, blocks: &mut [[u8; 16]]) {
        use cipher::BlockCipherEncrypt;
        for block in blocks {
            let ga = unsafe { &mut *(block as *mut [u8; 16] as *mut cipher::Block<aes::Aes256>) };
            ctx.encrypt_block(ga);
        }
    }

    fn aes256ecb_decrypt_block(ctx: &Self::Context, blocks: &mut [[u8; 16]]) {
        use cipher::BlockCipherDecrypt;
        for block in blocks {
            let ga = unsafe { &mut *(block as *mut [u8; 16] as *mut cipher::Block<aes::Aes256>) };
            ctx.decrypt_block(ga);
        }
    }
}

embassy_crypto_driver::embassy_crypto_aes256ecb_impl!(Aes256EcbDriver);

// ===========================================================================
// AES CBC
// ===========================================================================

#[derive(Clone)]
struct Aes128CbcContext {
    cipher: aes::Aes128,
    iv: [u8; 16],
}

struct Aes128CbcDriver;

impl embassy_crypto_driver::Aes128Cbc for Aes128CbcDriver {
    type Context = Aes128CbcContext;

    fn aes128cbc_init(key: &[u8; 16], iv: &[u8; 16]) -> Self::Context {
        use cipher::KeyInit;
        Aes128CbcContext {
            cipher: aes::Aes128::new_from_slice(key.as_slice()).unwrap(),
            iv: *iv,
        }
    }

    fn aes128cbc_clone(ctx: &Self::Context) -> Self::Context {
        ctx.clone()
    }

    fn aes128cbc_encrypt_block(ctx: &mut Self::Context, blocks: &mut [[u8; 16]]) {
        use cipher::BlockCipherEncrypt;
        for block in blocks {
            for i in 0..16 {
                block[i] ^= ctx.iv[i];
            }
            let ga = unsafe { &mut *(block as *mut [u8; 16] as *mut cipher::Block<aes::Aes128>) };
            ctx.cipher.encrypt_block(ga);
            ctx.iv.copy_from_slice(block);
        }
    }

    fn aes128cbc_decrypt_block(ctx: &mut Self::Context, blocks: &mut [[u8; 16]]) {
        use cipher::BlockCipherDecrypt;
        for block in blocks {
            let saved = *block;
            let ga = unsafe { &mut *(block as *mut [u8; 16] as *mut cipher::Block<aes::Aes128>) };
            ctx.cipher.decrypt_block(ga);
            for i in 0..16 {
                block[i] ^= ctx.iv[i];
            }
            ctx.iv = saved;
        }
    }
}

embassy_crypto_driver::embassy_crypto_aes128cbc_impl!(Aes128CbcDriver);

#[derive(Clone)]
struct Aes256CbcContext {
    cipher: aes::Aes256,
    iv: [u8; 16],
}

struct Aes256CbcDriver;

impl embassy_crypto_driver::Aes256Cbc for Aes256CbcDriver {
    type Context = Aes256CbcContext;

    fn aes256cbc_init(key: &[u8; 32], iv: &[u8; 16]) -> Self::Context {
        use cipher::KeyInit;
        Aes256CbcContext {
            cipher: aes::Aes256::new_from_slice(key.as_slice()).unwrap(),
            iv: *iv,
        }
    }

    fn aes256cbc_clone(ctx: &Self::Context) -> Self::Context {
        ctx.clone()
    }

    fn aes256cbc_encrypt_block(ctx: &mut Self::Context, blocks: &mut [[u8; 16]]) {
        use cipher::BlockCipherEncrypt;
        for block in blocks {
            for i in 0..16 {
                block[i] ^= ctx.iv[i];
            }
            let ga = unsafe { &mut *(block as *mut [u8; 16] as *mut cipher::Block<aes::Aes256>) };
            ctx.cipher.encrypt_block(ga);
            ctx.iv.copy_from_slice(block);
        }
    }

    fn aes256cbc_decrypt_block(ctx: &mut Self::Context, blocks: &mut [[u8; 16]]) {
        use cipher::BlockCipherDecrypt;
        for block in blocks {
            let saved = *block;
            let ga = unsafe { &mut *(block as *mut [u8; 16] as *mut cipher::Block<aes::Aes256>) };
            ctx.cipher.decrypt_block(ga);
            for i in 0..16 {
                block[i] ^= ctx.iv[i];
            }
            ctx.iv = saved;
        }
    }
}

embassy_crypto_driver::embassy_crypto_aes256cbc_impl!(Aes256CbcDriver);

// ===========================================================================
// AES GCM
// ===========================================================================

struct Aes128GcmDriver;

impl embassy_crypto_driver::Aes128Gcm for Aes128GcmDriver {
    type Context = aes_gcm::Aes128Gcm;

    fn aes128gcm_init(key: &[u8; 16]) -> Self::Context {
        use aead::KeyInit;
        aes_gcm::Aes128Gcm::new_from_slice(key.as_slice()).unwrap()
    }

    fn aes128gcm_clone(ctx: &Self::Context) -> Self::Context {
        ctx.clone()
    }

    fn aes128gcm_encrypt(
        ctx: &Self::Context,
        nonce: &[u8],
        aad: &[u8],
        buffer: &mut [u8],
        tag: &mut [u8; 16],
    ) -> Result<(), CryptoError> {
        use aead::AeadInOut;
        let nonce = aead::Nonce::<aes_gcm::Aes128Gcm>::try_from(nonce).unwrap();
        let computed_tag = ctx
            .encrypt_inout_detached(&nonce, aad, buffer.into())
            .map_err(|_| CryptoError::InvalidInput)?;
        tag.copy_from_slice(computed_tag.as_slice());
        Ok(())
    }

    fn aes128gcm_decrypt(
        ctx: &Self::Context,
        nonce: &[u8],
        aad: &[u8],
        buffer: &mut [u8],
        tag: &[u8; 16],
    ) -> Result<(), CryptoError> {
        use aead::AeadInOut;
        let nonce = aead::Nonce::<aes_gcm::Aes128Gcm>::try_from(nonce).unwrap();
        let tag_ga = aead::Tag::<aes_gcm::Aes128Gcm>::try_from(tag.as_slice()).unwrap();
        ctx.decrypt_inout_detached(&nonce, aad, buffer.into(), &tag_ga)
            .map_err(|_| CryptoError::InvalidSignature)
    }
}

embassy_crypto_driver::embassy_crypto_aes128gcm_impl!(Aes128GcmDriver);

struct Aes256GcmDriver;

impl embassy_crypto_driver::Aes256Gcm for Aes256GcmDriver {
    type Context = aes_gcm::Aes256Gcm;

    fn aes256gcm_init(key: &[u8; 32]) -> Self::Context {
        use aead::KeyInit;
        aes_gcm::Aes256Gcm::new_from_slice(key.as_slice()).unwrap()
    }

    fn aes256gcm_clone(ctx: &Self::Context) -> Self::Context {
        ctx.clone()
    }

    fn aes256gcm_encrypt(
        ctx: &Self::Context,
        nonce: &[u8],
        aad: &[u8],
        buffer: &mut [u8],
        tag: &mut [u8; 16],
    ) -> Result<(), CryptoError> {
        use aead::AeadInOut;
        let nonce = aead::Nonce::<aes_gcm::Aes256Gcm>::try_from(nonce).unwrap();
        let computed_tag = ctx
            .encrypt_inout_detached(&nonce, aad, buffer.into())
            .map_err(|_| CryptoError::InvalidInput)?;
        tag.copy_from_slice(computed_tag.as_slice());
        Ok(())
    }

    fn aes256gcm_decrypt(
        ctx: &Self::Context,
        nonce: &[u8],
        aad: &[u8],
        buffer: &mut [u8],
        tag: &[u8; 16],
    ) -> Result<(), CryptoError> {
        use aead::AeadInOut;
        let nonce = aead::Nonce::<aes_gcm::Aes256Gcm>::try_from(nonce).unwrap();
        let tag_ga = aead::Tag::<aes_gcm::Aes256Gcm>::try_from(tag.as_slice()).unwrap();
        ctx.decrypt_inout_detached(&nonce, aad, buffer.into(), &tag_ga)
            .map_err(|_| CryptoError::InvalidSignature)
    }
}

embassy_crypto_driver::embassy_crypto_aes256gcm_impl!(Aes256GcmDriver);

// ===========================================================================
// AES CCM
// ===========================================================================

macro_rules! ccm_case {
    (encrypt, $cipher:ty, $key:expr, $nonce:expr, $aad:expr, $buffer:expr, $tag:expr, $tag_len:ty, $nonce_len:ty) => {{
        type C = ccm::Ccm<$cipher, $tag_len, $nonce_len>;
        let ccm = C::new_from_slice($key).unwrap();
        let t = ccm
            .encrypt_inout_detached($nonce.try_into().unwrap(), $aad, InOutBuf::from($buffer))
            .map_err(|_| CryptoError::InvalidInput)?;
        $tag.copy_from_slice(t.as_slice());
        Ok(())
    }};
    (decrypt, $cipher:ty, $key:expr, $nonce:expr, $aad:expr, $buffer:expr, $tag:expr, $tag_len:ty, $nonce_len:ty) => {{
        type C = ccm::Ccm<$cipher, $tag_len, $nonce_len>;
        let ccm = C::new_from_slice($key).unwrap();
        ccm.decrypt_inout_detached(
            $nonce.try_into().unwrap(),
            $aad,
            InOutBuf::from($buffer),
            $tag.try_into().unwrap(),
        )
        .map_err(|_| CryptoError::InvalidSignature)
    }};
}

macro_rules! ccm_nonce_match {
    ($op:ident, $cipher:ty, $key:expr, $nonce:expr, $aad:expr, $buffer:expr, $tag:expr, $tag_len:ty) => {
        match $nonce.len() {
            7 => ccm_case!(
                $op,
                $cipher,
                $key,
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
                $key,
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
                $key,
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
                $key,
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
                $key,
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
                $key,
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
                $key,
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

macro_rules! ccm_dispatch {
    ($op:ident, $cipher:ty, $key:expr, $nonce:expr, $aad:expr, $buffer:expr, $tag:expr) => {
        match $tag.len() {
            4 => ccm_nonce_match!($op, $cipher, $key, $nonce, $aad, $buffer, $tag, ccm::consts::U4),
            6 => ccm_nonce_match!($op, $cipher, $key, $nonce, $aad, $buffer, $tag, ccm::consts::U6),
            8 => ccm_nonce_match!($op, $cipher, $key, $nonce, $aad, $buffer, $tag, ccm::consts::U8),
            10 => ccm_nonce_match!($op, $cipher, $key, $nonce, $aad, $buffer, $tag, ccm::consts::U10),
            12 => ccm_nonce_match!($op, $cipher, $key, $nonce, $aad, $buffer, $tag, ccm::consts::U12),
            14 => ccm_nonce_match!($op, $cipher, $key, $nonce, $aad, $buffer, $tag, ccm::consts::U14),
            16 => ccm_nonce_match!($op, $cipher, $key, $nonce, $aad, $buffer, $tag, ccm::consts::U16),
            _ => Err(CryptoError::Unsupported),
        }
    };
}

#[derive(Clone)]
struct Aes128CcmContext {
    key: [u8; 16],
}

struct Aes128CcmDriver;

impl embassy_crypto_driver::Aes128Ccm for Aes128CcmDriver {
    type Context = Aes128CcmContext;

    fn aes128ccm_init(key: &[u8; 16]) -> Self::Context {
        Aes128CcmContext { key: *key }
    }

    fn aes128ccm_clone(ctx: &Self::Context) -> Self::Context {
        ctx.clone()
    }

    fn aes128ccm_encrypt(
        ctx: &Self::Context,
        nonce: &[u8],
        aad: &[u8],
        buffer: &mut [u8],
        tag: &mut [u8],
    ) -> Result<(), CryptoError> {
        use aead::inout::InOutBuf;
        use aead::{AeadInOut, KeyInit};
        ccm_dispatch!(encrypt, aes::Aes128, &ctx.key, nonce, aad, buffer, tag)
    }

    fn aes128ccm_decrypt(
        ctx: &Self::Context,
        nonce: &[u8],
        aad: &[u8],
        buffer: &mut [u8],
        tag: &[u8],
    ) -> Result<(), CryptoError> {
        use aead::inout::InOutBuf;
        use aead::{AeadInOut, KeyInit};
        ccm_dispatch!(decrypt, aes::Aes128, &ctx.key, nonce, aad, buffer, tag)
    }
}

embassy_crypto_driver::embassy_crypto_aes128ccm_impl!(Aes128CcmDriver);

#[derive(Clone)]
struct Aes256CcmContext {
    key: [u8; 32],
}

struct Aes256CcmDriver;

impl embassy_crypto_driver::Aes256Ccm for Aes256CcmDriver {
    type Context = Aes256CcmContext;

    fn aes256ccm_init(key: &[u8; 32]) -> Self::Context {
        Aes256CcmContext { key: *key }
    }

    fn aes256ccm_clone(ctx: &Self::Context) -> Self::Context {
        ctx.clone()
    }

    fn aes256ccm_encrypt(
        ctx: &Self::Context,
        nonce: &[u8],
        aad: &[u8],
        buffer: &mut [u8],
        tag: &mut [u8],
    ) -> Result<(), CryptoError> {
        use aead::inout::InOutBuf;
        use aead::{AeadInOut, KeyInit};
        ccm_dispatch!(encrypt, aes::Aes256, &ctx.key, nonce, aad, buffer, tag)
    }

    fn aes256ccm_decrypt(
        ctx: &Self::Context,
        nonce: &[u8],
        aad: &[u8],
        buffer: &mut [u8],
        tag: &[u8],
    ) -> Result<(), CryptoError> {
        use aead::inout::InOutBuf;
        use aead::{AeadInOut, KeyInit};
        ccm_dispatch!(decrypt, aes::Aes256, &ctx.key, nonce, aad, buffer, tag)
    }
}

embassy_crypto_driver::embassy_crypto_aes256ccm_impl!(Aes256CcmDriver);
