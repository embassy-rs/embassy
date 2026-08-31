#![allow(missing_docs)]

//! Software fallback driver for `embassy-crypto-driver` using RustCrypto crates.
//!
//! Enable the `driver-rustcrypto` feature on `embassy-crypto` to use this driver.

use embassy_crypto_driver::CryptoError;

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

impl_digest!(
    Md5Driver,
    embassy_crypto_driver::Md5,
    md5::Md5,
    [md5_init, md5_clone, md5_update, md5_finalize],
    embassy_crypto_driver::embassy_crypto_md5_impl
);

impl_digest!(
    Sha1Driver,
    embassy_crypto_driver::Sha1,
    sha1::Sha1,
    [sha1_init, sha1_clone, sha1_update, sha1_finalize],
    embassy_crypto_driver::embassy_crypto_sha1_impl
);

impl_digest!(
    Sha224Driver,
    embassy_crypto_driver::Sha224,
    sha2::Sha224,
    [sha224_init, sha224_clone, sha224_update, sha224_finalize],
    embassy_crypto_driver::embassy_crypto_sha224_impl
);

impl_digest!(
    Sha256Driver,
    embassy_crypto_driver::Sha256,
    sha2::Sha256,
    [sha256_init, sha256_clone, sha256_update, sha256_finalize],
    embassy_crypto_driver::embassy_crypto_sha256_impl
);

impl_digest!(
    Sha384Driver,
    embassy_crypto_driver::Sha384,
    sha2::Sha384,
    [sha384_init, sha384_clone, sha384_update, sha384_finalize],
    embassy_crypto_driver::embassy_crypto_sha384_impl
);

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

// HMAC reset is not available in hmac 0.13, so we store the key and recreate.

macro_rules! impl_hmac {
    (
        $driver:ident,
        $ctx:ident,
        $trait:path,
        $hash:ty,
        $key_cap:literal,
        [$init:ident, $clone:ident, $update:ident, $finalize:ident, $reset:ident],
        $impl_macro:path
    ) => {
        #[derive(Clone)]
        struct $ctx {
            key: heapless::Vec<u8, $key_cap>,
            inner: hmac::Hmac<$hash>,
        }

        struct $driver;

        impl $trait for $driver {
            type Context = $ctx;

            fn $init(key: &[u8]) -> Self::Context {
                use hmac::KeyInit;
                let mut key_storage = heapless::Vec::new();
                key_storage.extend_from_slice(key).unwrap();
                let inner = hmac::Hmac::new_from_slice(key).expect("key length validated by caller");
                $ctx {
                    key: key_storage,
                    inner,
                }
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

            fn $reset(ctx: &mut Self::Context) {
                *ctx = Self::$init(&ctx.key);
            }
        }

        $impl_macro!($driver);
    };
}

impl_hmac!(
    HmacSha1Driver,
    HmacSha1Context,
    embassy_crypto_driver::HmacSha1,
    sha1::Sha1,
    64,
    [
        hmac_sha1_init,
        hmac_sha1_clone,
        hmac_sha1_update,
        hmac_sha1_finalize,
        hmac_sha1_reset
    ],
    embassy_crypto_driver::embassy_crypto_hmac_sha1_impl
);

impl_hmac!(
    HmacSha224Driver,
    HmacSha224Context,
    embassy_crypto_driver::HmacSha224,
    sha2::Sha224,
    64,
    [
        hmac_sha224_init,
        hmac_sha224_clone,
        hmac_sha224_update,
        hmac_sha224_finalize,
        hmac_sha224_reset
    ],
    embassy_crypto_driver::embassy_crypto_hmac_sha224_impl
);

impl_hmac!(
    HmacSha256Driver,
    HmacSha256Context,
    embassy_crypto_driver::HmacSha256,
    sha2::Sha256,
    64,
    [
        hmac_sha256_init,
        hmac_sha256_clone,
        hmac_sha256_update,
        hmac_sha256_finalize,
        hmac_sha256_reset
    ],
    embassy_crypto_driver::embassy_crypto_hmac_sha256_impl
);

impl_hmac!(
    HmacSha384Driver,
    HmacSha384Context,
    embassy_crypto_driver::HmacSha384,
    sha2::Sha384,
    128,
    [
        hmac_sha384_init,
        hmac_sha384_clone,
        hmac_sha384_update,
        hmac_sha384_finalize,
        hmac_sha384_reset
    ],
    embassy_crypto_driver::embassy_crypto_hmac_sha384_impl
);

impl_hmac!(
    HmacSha512_224Driver,
    HmacSha512_224Context,
    embassy_crypto_driver::HmacSha512_224,
    sha2::Sha512_224,
    128,
    [
        hmac_sha512_224_init,
        hmac_sha512_224_clone,
        hmac_sha512_224_update,
        hmac_sha512_224_finalize,
        hmac_sha512_224_reset
    ],
    embassy_crypto_driver::embassy_crypto_hmac_sha512_224_impl
);

impl_hmac!(
    HmacSha512_256Driver,
    HmacSha512_256Context,
    embassy_crypto_driver::HmacSha512_256,
    sha2::Sha512_256,
    128,
    [
        hmac_sha512_256_init,
        hmac_sha512_256_clone,
        hmac_sha512_256_update,
        hmac_sha512_256_finalize,
        hmac_sha512_256_reset
    ],
    embassy_crypto_driver::embassy_crypto_hmac_sha512_256_impl
);

impl_hmac!(
    HmacSha512Driver,
    HmacSha512Context,
    embassy_crypto_driver::HmacSha512,
    sha2::Sha512,
    128,
    [
        hmac_sha512_init,
        hmac_sha512_clone,
        hmac_sha512_update,
        hmac_sha512_finalize,
        hmac_sha512_reset
    ],
    embassy_crypto_driver::embassy_crypto_hmac_sha512_impl
);

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

    fn aes128ecb_encrypt_block(ctx: &Self::Context, blocks: embassy_crypto_driver::InOutBuf<'_, '_, u8>) {
        use cipher::BlockCipherEncrypt;
        let blocks = blocks.into_out_with_copied_in();
        if blocks.len() >= 32 {
            // Process pairs of blocks using encrypt_blocks for potential SIMD
            let mut chunks = blocks.chunks_exact_mut(32);
            for pair in &mut chunks {
                let ga =
                    unsafe { core::slice::from_raw_parts_mut(pair.as_mut_ptr() as *mut cipher::Block<aes::Aes128>, 2) };
                ctx.encrypt_blocks(ga);
            }
            // Remainder
            for chunk in chunks.into_remainder().chunks_exact_mut(16) {
                let ga = unsafe { &mut *(chunk.as_mut_ptr() as *mut cipher::Block<aes::Aes128>) };
                ctx.encrypt_block(ga);
            }
        } else {
            for chunk in blocks.chunks_exact_mut(16) {
                let ga = unsafe { &mut *(chunk.as_mut_ptr() as *mut cipher::Block<aes::Aes128>) };
                ctx.encrypt_block(ga);
            }
        }
    }

    fn aes128ecb_decrypt_block(ctx: &Self::Context, blocks: embassy_crypto_driver::InOutBuf<'_, '_, u8>) {
        use cipher::BlockCipherDecrypt;
        let blocks = blocks.into_out_with_copied_in();
        if blocks.len() >= 32 {
            let mut chunks = blocks.chunks_exact_mut(32);
            for pair in &mut chunks {
                let ga =
                    unsafe { core::slice::from_raw_parts_mut(pair.as_mut_ptr() as *mut cipher::Block<aes::Aes128>, 2) };
                ctx.decrypt_blocks(ga);
            }
            for chunk in chunks.into_remainder().chunks_exact_mut(16) {
                let ga = unsafe { &mut *(chunk.as_mut_ptr() as *mut cipher::Block<aes::Aes128>) };
                ctx.decrypt_block(ga);
            }
        } else {
            for chunk in blocks.chunks_exact_mut(16) {
                let ga = unsafe { &mut *(chunk.as_mut_ptr() as *mut cipher::Block<aes::Aes128>) };
                ctx.decrypt_block(ga);
            }
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

    fn aes256ecb_encrypt_block(ctx: &Self::Context, blocks: embassy_crypto_driver::InOutBuf<'_, '_, u8>) {
        use cipher::BlockCipherEncrypt;
        let blocks = blocks.into_out_with_copied_in();
        if blocks.len() >= 32 {
            let mut chunks = blocks.chunks_exact_mut(32);
            for pair in &mut chunks {
                let ga =
                    unsafe { core::slice::from_raw_parts_mut(pair.as_mut_ptr() as *mut cipher::Block<aes::Aes256>, 2) };
                ctx.encrypt_blocks(ga);
            }
            for chunk in chunks.into_remainder().chunks_exact_mut(16) {
                let ga = unsafe { &mut *(chunk.as_mut_ptr() as *mut cipher::Block<aes::Aes256>) };
                ctx.encrypt_block(ga);
            }
        } else {
            for chunk in blocks.chunks_exact_mut(16) {
                let ga = unsafe { &mut *(chunk.as_mut_ptr() as *mut cipher::Block<aes::Aes256>) };
                ctx.encrypt_block(ga);
            }
        }
    }

    fn aes256ecb_decrypt_block(ctx: &Self::Context, blocks: embassy_crypto_driver::InOutBuf<'_, '_, u8>) {
        use cipher::BlockCipherDecrypt;
        let blocks = blocks.into_out_with_copied_in();
        if blocks.len() >= 32 {
            let mut chunks = blocks.chunks_exact_mut(32);
            for pair in &mut chunks {
                let ga =
                    unsafe { core::slice::from_raw_parts_mut(pair.as_mut_ptr() as *mut cipher::Block<aes::Aes256>, 2) };
                ctx.decrypt_blocks(ga);
            }
            for chunk in chunks.into_remainder().chunks_exact_mut(16) {
                let ga = unsafe { &mut *(chunk.as_mut_ptr() as *mut cipher::Block<aes::Aes256>) };
                ctx.decrypt_block(ga);
            }
        } else {
            for chunk in blocks.chunks_exact_mut(16) {
                let ga = unsafe { &mut *(chunk.as_mut_ptr() as *mut cipher::Block<aes::Aes256>) };
                ctx.decrypt_block(ga);
            }
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

    fn aes128cbc_encrypt_block(ctx: &mut Self::Context, blocks: embassy_crypto_driver::InOutBuf<'_, '_, u8>) {
        use cipher::BlockCipherEncrypt;
        let blocks = blocks.into_out_with_copied_in();
        for chunk in blocks.chunks_exact_mut(16) {
            for i in 0..16 {
                chunk[i] ^= ctx.iv[i];
            }
            let ga = unsafe { &mut *(chunk.as_mut_ptr() as *mut cipher::Block<aes::Aes128>) };
            ctx.cipher.encrypt_block(ga);
            ctx.iv.copy_from_slice(chunk);
        }
    }

    fn aes128cbc_decrypt_block(ctx: &mut Self::Context, blocks: embassy_crypto_driver::InOutBuf<'_, '_, u8>) {
        use cipher::BlockCipherDecrypt;
        let blocks = blocks.into_out_with_copied_in();
        for chunk in blocks.chunks_exact_mut(16) {
            let saved: [u8; 16] = chunk.try_into().unwrap();
            let ga = unsafe { &mut *(chunk.as_mut_ptr() as *mut cipher::Block<aes::Aes128>) };
            ctx.cipher.decrypt_block(ga);
            for i in 0..16 {
                chunk[i] ^= ctx.iv[i];
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

    fn aes256cbc_encrypt_block(ctx: &mut Self::Context, blocks: embassy_crypto_driver::InOutBuf<'_, '_, u8>) {
        use cipher::BlockCipherEncrypt;
        let blocks = blocks.into_out_with_copied_in();
        for chunk in blocks.chunks_exact_mut(16) {
            for i in 0..16 {
                chunk[i] ^= ctx.iv[i];
            }
            let ga = unsafe { &mut *(chunk.as_mut_ptr() as *mut cipher::Block<aes::Aes256>) };
            ctx.cipher.encrypt_block(ga);
            ctx.iv.copy_from_slice(chunk);
        }
    }

    fn aes256cbc_decrypt_block(ctx: &mut Self::Context, blocks: embassy_crypto_driver::InOutBuf<'_, '_, u8>) {
        use cipher::BlockCipherDecrypt;
        let blocks = blocks.into_out_with_copied_in();
        for chunk in blocks.chunks_exact_mut(16) {
            let saved: [u8; 16] = chunk.try_into().unwrap();
            let ga = unsafe { &mut *(chunk.as_mut_ptr() as *mut cipher::Block<aes::Aes256>) };
            ctx.cipher.decrypt_block(ga);
            for i in 0..16 {
                chunk[i] ^= ctx.iv[i];
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
        buffer: embassy_crypto_driver::InOutBuf<'_, '_, u8>,
        tag: &mut [u8; 16],
    ) -> Result<(), CryptoError> {
        use aead::AeadInOut;
        let nonce = aead::Nonce::<aes_gcm::Aes128Gcm>::try_from(nonce).unwrap();
        let len = buffer.len();
        let (in_ptr, out_ptr) = buffer.into_raw();
        let aead_buf = unsafe { aead::inout::InOutBuf::from_raw(in_ptr, out_ptr, len) };
        let computed_tag = ctx
            .encrypt_inout_detached(&nonce, aad, aead_buf)
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
        let len = buffer.len();
        let (in_ptr, out_ptr) = buffer.into_raw();
        let aead_buf = unsafe { aead::inout::InOutBuf::from_raw(in_ptr, out_ptr, len) };
        ctx.decrypt_inout_detached(&nonce, aad, aead_buf, &tag_ga)
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
        buffer: embassy_crypto_driver::InOutBuf<'_, '_, u8>,
        tag: &mut [u8; 16],
    ) -> Result<(), CryptoError> {
        use aead::AeadInOut;
        let nonce = aead::Nonce::<aes_gcm::Aes256Gcm>::try_from(nonce).unwrap();
        let len = buffer.len();
        let (in_ptr, out_ptr) = buffer.into_raw();
        let aead_buf = unsafe { aead::inout::InOutBuf::from_raw(in_ptr, out_ptr, len) };
        let computed_tag = ctx
            .encrypt_inout_detached(&nonce, aad, aead_buf)
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
        let len = buffer.len();
        let (in_ptr, out_ptr) = buffer.into_raw();
        let aead_buf = unsafe { aead::inout::InOutBuf::from_raw(in_ptr, out_ptr, len) };
        ctx.decrypt_inout_detached(&nonce, aad, aead_buf, &tag_ga)
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
        buffer: embassy_crypto_driver::InOutBuf<'_, '_, u8>,
        tag: &mut [u8],
    ) -> Result<(), CryptoError> {
        use aead::inout::InOutBuf;
        use aead::{AeadInOut, KeyInit};
        let len = buffer.len();
        let (in_ptr, out_ptr) = buffer.into_raw();
        let aead_buf = unsafe { InOutBuf::from_raw(in_ptr, out_ptr, len) };
        ccm_dispatch!(encrypt, aes::Aes128, &ctx.key, nonce, aad, aead_buf, tag)
    }

    fn aes128ccm_decrypt(
        ctx: &Self::Context,
        nonce: &[u8],
        aad: &[u8],
        buffer: embassy_crypto_driver::InOutBuf<'_, '_, u8>,
        tag: &[u8],
    ) -> Result<(), CryptoError> {
        use aead::inout::InOutBuf;
        use aead::{AeadInOut, KeyInit};
        let len = buffer.len();
        let (in_ptr, out_ptr) = buffer.into_raw();
        let aead_buf = unsafe { InOutBuf::from_raw(in_ptr, out_ptr, len) };
        ccm_dispatch!(decrypt, aes::Aes128, &ctx.key, nonce, aad, aead_buf, tag)
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
        buffer: embassy_crypto_driver::InOutBuf<'_, '_, u8>,
        tag: &mut [u8],
    ) -> Result<(), CryptoError> {
        use aead::inout::InOutBuf;
        use aead::{AeadInOut, KeyInit};
        let len = buffer.len();
        let (in_ptr, out_ptr) = buffer.into_raw();
        let aead_buf = unsafe { InOutBuf::from_raw(in_ptr, out_ptr, len) };
        ccm_dispatch!(encrypt, aes::Aes256, &ctx.key, nonce, aad, aead_buf, tag)
    }

    fn aes256ccm_decrypt(
        ctx: &Self::Context,
        nonce: &[u8],
        aad: &[u8],
        buffer: embassy_crypto_driver::InOutBuf<'_, '_, u8>,
        tag: &[u8],
    ) -> Result<(), CryptoError> {
        use aead::inout::InOutBuf;
        use aead::{AeadInOut, KeyInit};
        let len = buffer.len();
        let (in_ptr, out_ptr) = buffer.into_raw();
        let aead_buf = unsafe { InOutBuf::from_raw(in_ptr, out_ptr, len) };
        ccm_dispatch!(decrypt, aes::Aes256, &ctx.key, nonce, aad, aead_buf, tag)
    }
}

embassy_crypto_driver::embassy_crypto_aes256ccm_impl!(Aes256CcmDriver);

// ===========================================================================
// AES CTR (software fallback via RustCrypto `ctr` crate)
// ===========================================================================

// ---------------------------------------------------------------------------
// AES-128 CTR
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct Aes128CtrContext {
    cipher: aes::Aes128,
    iv: [u8; 16],
}

struct Aes128CtrDriver;

impl embassy_crypto_driver::Aes128Ctr for Aes128CtrDriver {
    type Context = Aes128CtrContext;

    fn aes128ctr_init(key: &[u8; 16], iv: &[u8; 16]) -> Self::Context {
        use cipher::KeyInit;
        Aes128CtrContext {
            cipher: aes::Aes128::new_from_slice(key.as_slice()).unwrap(),
            iv: *iv,
        }
    }

    fn aes128ctr_clone(ctx: &Self::Context) -> Self::Context {
        ctx.clone()
    }

    fn aes128ctr_apply_keystream(ctx: &mut Self::Context, buf: embassy_crypto_driver::InOutBuf<'_, '_, u8>) {
        use cipher::BlockCipherEncrypt;

        let buf = buf.into_out_with_copied_in();
        let mut counter = ctx.iv;
        let mut offset = 0usize;

        while offset + 16 <= buf.len() {
            let mut block = counter;
            let block_ptr = &mut block as *mut [u8; 16] as *mut cipher::Block<aes::Aes128>;
            let block_ref = unsafe { &mut *block_ptr };
            ctx.cipher.encrypt_block(block_ref);

            for i in 0..16 {
                buf[offset + i] ^= block[i];
            }

            counter = u128::from_be_bytes(counter).wrapping_add(1).to_be_bytes();
            offset += 16;
        }

        if offset < buf.len() {
            let mut block = counter;
            let block_ptr = &mut block as *mut [u8; 16] as *mut cipher::Block<aes::Aes128>;
            let block_ref = unsafe { &mut *block_ptr };
            ctx.cipher.encrypt_block(block_ref);

            for i in 0..buf.len() - offset {
                buf[offset + i] ^= block[i];
            }

            counter = u128::from_be_bytes(counter).wrapping_add(1).to_be_bytes();
        }

        ctx.iv = counter;
    }
}

embassy_crypto_driver::embassy_crypto_aes128ctr_impl!(Aes128CtrDriver);

// ---------------------------------------------------------------------------
// AES-256 CTR
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct Aes256CtrContext {
    cipher: aes::Aes256,
    iv: [u8; 16],
}

struct Aes256CtrDriver;

impl embassy_crypto_driver::Aes256Ctr for Aes256CtrDriver {
    type Context = Aes256CtrContext;

    fn aes256ctr_init(key: &[u8; 32], iv: &[u8; 16]) -> Self::Context {
        use cipher::KeyInit;
        Aes256CtrContext {
            cipher: aes::Aes256::new_from_slice(key.as_slice()).unwrap(),
            iv: *iv,
        }
    }

    fn aes256ctr_clone(ctx: &Self::Context) -> Self::Context {
        ctx.clone()
    }

    fn aes256ctr_apply_keystream(ctx: &mut Self::Context, buf: embassy_crypto_driver::InOutBuf<'_, '_, u8>) {
        use cipher::BlockCipherEncrypt;

        let buf = buf.into_out_with_copied_in();
        let mut counter = ctx.iv;
        let mut offset = 0usize;

        while offset + 16 <= buf.len() {
            let mut block = counter;
            let block_ptr = &mut block as *mut [u8; 16] as *mut cipher::Block<aes::Aes256>;
            let block_ref = unsafe { &mut *block_ptr };
            ctx.cipher.encrypt_block(block_ref);

            for i in 0..16 {
                buf[offset + i] ^= block[i];
            }

            counter = u128::from_be_bytes(counter).wrapping_add(1).to_be_bytes();
            offset += 16;
        }

        if offset < buf.len() {
            let mut block = counter;
            let block_ptr = &mut block as *mut [u8; 16] as *mut cipher::Block<aes::Aes256>;
            let block_ref = unsafe { &mut *block_ptr };
            ctx.cipher.encrypt_block(block_ref);

            for i in 0..buf.len() - offset {
                buf[offset + i] ^= block[i];
            }

            counter = u128::from_be_bytes(counter).wrapping_add(1).to_be_bytes();
        }

        ctx.iv = counter;
    }
}

embassy_crypto_driver::embassy_crypto_aes256ctr_impl!(Aes256CtrDriver);
