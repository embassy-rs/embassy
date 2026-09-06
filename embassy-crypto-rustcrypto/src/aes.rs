//! AES drivers over `aes`, `cbc`, `ctr`, `aes-gcm` and `cmac`.
//!
//! The contexts hold the raw key (plus mode state) rather than an expanded key
//! schedule: the `aes` crate's software key schedule is large and platform
//! dependent, and the opaque context sizes are shared with hardware drivers
//! that only hold the key. The schedule is recomputed on every call, which
//! costs a few hundred cycles, small next to processing any real buffer.

use embassy_crypto::driver::InOutBuf;

/// Convert our in/out buffer to the RustCrypto one.
fn inout<'i, 'o>(buf: InOutBuf<'i, 'o, u8>) -> cipher::inout::InOutBuf<'i, 'o, u8> {
    let len = buf.len();
    let (in_ptr, out_ptr) = buf.into_raw();
    // SAFETY: `InOutBuf` upholds the same validity invariants.
    unsafe { cipher::inout::InOutBuf::from_raw(in_ptr, out_ptr, len) }
}

/// Split an in/out buffer of bytes into 16-byte blocks.
#[allow(dead_code)]
fn blocks<'i, 'o>(buf: InOutBuf<'i, 'o, u8>) -> cipher::inout::InOutBuf<'i, 'o, cipher::Block<aes::Aes128Enc>> {
    let (blocks, tail) = inout(buf).into_chunks();
    debug_assert!(tail.is_empty());
    blocks
}

macro_rules! aes_drivers {
    (
        $bits:literal, $key:literal,
        enc = $enc:ty, dec = $dec:ty, both = $both:ty,
        ecb = ($ecb_feature:literal, $ecb_trait:ident, $ecb_register:ident),
        cbc = ($cbc_feature:literal, $cbc_trait:ident, $cbc_register:ident),
        ctr = ($ctr_feature:literal, $ctr_trait:ident, $ctr_register:ident),
        gcm = ($gcm_feature:literal, $gcm_trait:ident, $gcm_register:ident),
        ccm = ($ccm_feature:literal, $ccm_trait:ident, $ccm_register:ident),
        cmac = ($cmac_feature:literal, $cmac_trait:ident, $cmac_register:ident),
    ) => {
        #[cfg(feature = $ecb_feature)]
        mod $ecb_register {
            use cipher::{BlockCipherDecrypt, BlockCipherEncrypt, KeyInit};
            use embassy_crypto::driver::InOutBuf;

            struct Driver;

            impl embassy_crypto::driver::$ecb_trait for Driver {
                type Context = [u8; $key];

                fn init(key: &[u8; $key]) -> Self::Context {
                    *key
                }

                fn encrypt_blocks(ctx: &Self::Context, blocks: InOutBuf<'_, '_, u8>) {
                    <$enc>::new(ctx.into()).encrypt_blocks_inout(super::blocks(blocks));
                }

                fn decrypt_blocks(ctx: &Self::Context, blocks: InOutBuf<'_, '_, u8>) {
                    <$dec>::new(ctx.into()).decrypt_blocks_inout(super::blocks(blocks));
                }
            }

            embassy_crypto::$ecb_register!(Driver);
        }

        #[cfg(feature = $cbc_feature)]
        mod $cbc_register {
            use cipher::{BlockModeDecrypt, BlockModeEncrypt, KeyIvInit};
            use embassy_crypto::driver::InOutBuf;

            struct Driver;

            /// Key and chaining value.
            #[derive(Clone)]
            pub struct Context {
                key: [u8; $key],
                iv: [u8; 16],
            }

            impl embassy_crypto::driver::$cbc_trait for Driver {
                type EncryptContext = Context;
                type DecryptContext = Context;

                fn encrypt_init(key: &[u8; $key], iv: &[u8; 16]) -> Context {
                    Context { key: *key, iv: *iv }
                }

                fn decrypt_init(key: &[u8; $key], iv: &[u8; 16]) -> Context {
                    Context { key: *key, iv: *iv }
                }

                fn encrypt_blocks(ctx: &mut Context, blocks: InOutBuf<'_, '_, u8>) {
                    if blocks.is_empty() {
                        return;
                    }
                    let mut blocks = super::blocks(blocks);
                    cbc::Encryptor::<$enc>::new((&ctx.key).into(), (&ctx.iv).into())
                        .encrypt_blocks_inout(blocks.reborrow());
                    // The next chaining value is the last ciphertext block.
                    let n = blocks.len();
                    ctx.iv = blocks.get_out()[n - 1].into();
                }

                fn decrypt_blocks(ctx: &mut Context, blocks: InOutBuf<'_, '_, u8>) {
                    if blocks.is_empty() {
                        return;
                    }
                    let mut blocks = super::blocks(blocks);
                    // The next chaining value is the last ciphertext block; read it
                    // before decrypting, in case the operation is in place.
                    let n = blocks.len();
                    let next_iv: [u8; 16] = blocks.get_in()[n - 1].into();
                    cbc::Decryptor::<$dec>::new((&ctx.key).into(), (&ctx.iv).into())
                        .decrypt_blocks_inout(blocks.reborrow());
                    ctx.iv = next_iv;
                }
            }

            embassy_crypto::$cbc_register!(Driver);
        }

        #[cfg(feature = $ctr_feature)]
        mod $ctr_register {
            use cipher::{KeyIvInit, StreamCipher, StreamCipherSeek};
            use embassy_crypto::driver::InOutBuf;

            struct Driver;

            /// Key, initial counter block and keystream position.
            #[derive(Clone)]
            pub struct Context {
                key: [u8; $key],
                iv: [u8; 16],
                pos: u64,
            }

            impl embassy_crypto::driver::$ctr_trait for Driver {
                type Context = Context;

                fn init(key: &[u8; $key], iv: &[u8; 16]) -> Context {
                    Context {
                        key: *key,
                        iv: *iv,
                        pos: 0,
                    }
                }

                fn apply_keystream(ctx: &mut Context, buf: InOutBuf<'_, '_, u8>) {
                    let len = buf.len();
                    let mut cipher = ctr::Ctr128BE::<$enc>::new((&ctx.key).into(), (&ctx.iv).into());
                    cipher.seek(ctx.pos);
                    cipher.apply_keystream_inout(super::inout(buf));
                    ctx.pos += len as u64;
                }
            }

            embassy_crypto::$ctr_register!(Driver);
        }

        #[cfg(feature = $gcm_feature)]
        mod $gcm_register {
            use aead::{AeadInOut, KeyInit};
            use embassy_crypto::Error;
            use embassy_crypto::driver::InOutBuf;

            struct Driver;

            type Gcm = aes_gcm::AesGcm<$both, aead::consts::U12>;

            impl embassy_crypto::driver::$gcm_trait for Driver {
                type Context = [u8; $key];

                fn init(key: &[u8; $key]) -> Self::Context {
                    *key
                }

                fn encrypt(
                    ctx: &Self::Context,
                    nonce: &[u8],
                    aad: &[u8],
                    buffer: InOutBuf<'_, '_, u8>,
                    tag: &mut [u8; 16],
                ) -> Result<(), Error> {
                    let nonce: &[u8; 12] = nonce.try_into().map_err(|_| Error::InvalidInput)?;
                    let t = Gcm::new(ctx.into())
                        .encrypt_inout_detached(nonce.into(), aad, super::inout(buffer))
                        .map_err(|_| Error::InvalidInput)?;
                    tag.copy_from_slice(&t);
                    Ok(())
                }

                fn decrypt(
                    ctx: &Self::Context,
                    nonce: &[u8],
                    aad: &[u8],
                    buffer: InOutBuf<'_, '_, u8>,
                    tag: &[u8; 16],
                ) -> Result<(), Error> {
                    let nonce: &[u8; 12] = nonce.try_into().map_err(|_| Error::InvalidInput)?;
                    Gcm::new(ctx.into())
                        .decrypt_inout_detached(nonce.into(), aad, super::inout(buffer), tag.into())
                        .map_err(|_| Error::InvalidSignature)
                }
            }

            embassy_crypto::$gcm_register!(Driver);
        }

        #[cfg(feature = $ccm_feature)]
        mod $ccm_register {
            use cipher::{BlockCipherEncrypt, KeyInit};
            use embassy_crypto::Error;
            use embassy_crypto::driver::InOutBuf;

            struct Driver;

            fn block_fn(key: &[u8; $key]) -> impl Fn(&mut [u8; 16]) {
                let aes = <$enc>::new(key.into());
                move |block: &mut [u8; 16]| aes.encrypt_block(block.into())
            }

            impl embassy_crypto::driver::$ccm_trait for Driver {
                type Context = [u8; $key];

                fn init(key: &[u8; $key]) -> Self::Context {
                    *key
                }

                fn encrypt(
                    ctx: &Self::Context,
                    nonce: &[u8],
                    aad: &[u8],
                    buffer: InOutBuf<'_, '_, u8>,
                    tag: &mut [u8],
                ) -> Result<(), Error> {
                    crate::ccm::encrypt(&block_fn(ctx), nonce, aad, buffer, tag)
                }

                fn decrypt(
                    ctx: &Self::Context,
                    nonce: &[u8],
                    aad: &[u8],
                    buffer: InOutBuf<'_, '_, u8>,
                    tag: &[u8],
                ) -> Result<(), Error> {
                    crate::ccm::decrypt(&block_fn(ctx), nonce, aad, buffer, tag)
                }
            }

            embassy_crypto::$ccm_register!(Driver);
        }

        #[cfg(feature = $cmac_feature)]
        mod $cmac_register {
            use digest::{KeyInit, Mac};

            struct Driver;

            impl embassy_crypto::driver::$cmac_trait for Driver {
                type Context = cmac::Cmac<$enc>;

                fn init(key: &[u8; $key]) -> Self::Context {
                    cmac::Cmac::<$enc>::new(key.into())
                }

                fn update(ctx: &mut Self::Context, data: &[u8]) {
                    ctx.update(data);
                }

                fn finalize(ctx: Self::Context, out: &mut [u8; 16]) {
                    out.copy_from_slice(&ctx.finalize().into_bytes());
                }

                fn reset(ctx: &mut Self::Context) {
                    ctx.reset();
                }
            }

            embassy_crypto::$cmac_register!(Driver);
        }
    };
}

aes_drivers!(
    128,
    16,
    enc = aes::Aes128Enc,
    dec = aes::Aes128Dec,
    both = aes::Aes128,
    ecb = ("embassy-crypto-aes128-ecb", Aes128Ecb, aes128_ecb_impl),
    cbc = ("embassy-crypto-aes128-cbc", Aes128Cbc, aes128_cbc_impl),
    ctr = ("embassy-crypto-aes128-ctr", Aes128Ctr, aes128_ctr_impl),
    gcm = ("embassy-crypto-aes128-gcm", Aes128Gcm, aes128_gcm_impl),
    ccm = ("embassy-crypto-aes128-ccm", Aes128Ccm, aes128_ccm_impl),
    cmac = ("embassy-crypto-aes128-cmac", Aes128Cmac, aes128_cmac_impl),
);

aes_drivers!(
    256,
    32,
    enc = aes::Aes256Enc,
    dec = aes::Aes256Dec,
    both = aes::Aes256,
    ecb = ("embassy-crypto-aes256-ecb", Aes256Ecb, aes256_ecb_impl),
    cbc = ("embassy-crypto-aes256-cbc", Aes256Cbc, aes256_cbc_impl),
    ctr = ("embassy-crypto-aes256-ctr", Aes256Ctr, aes256_ctr_impl),
    gcm = ("embassy-crypto-aes256-gcm", Aes256Gcm, aes256_gcm_impl),
    ccm = ("embassy-crypto-aes256-ccm", Aes256Ccm, aes256_ccm_impl),
    cmac = ("embassy-crypto-aes256-cmac", Aes256Cmac, aes256_cmac_impl),
);
