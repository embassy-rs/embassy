#![no_std]
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

#[cfg(any(
    feature = "driver-md5",
    feature = "driver-sha1",
    feature = "driver-sha2",
    feature = "driver-hmac-sha1",
    feature = "driver-hmac-sha2",
    feature = "driver-aes128",
    feature = "driver-aes256",
    feature = "driver-aes128cbc",
    feature = "driver-aes256cbc",
    feature = "driver-aes128ctr",
    feature = "driver-aes256ctr",
    feature = "driver-aes128gcm",
    feature = "driver-aes256gcm",
    feature = "driver-aes128ccm",
    feature = "driver-aes256ccm",
    feature = "driver-aes128cmac",
    feature = "driver-aes256cmac",
    feature = "driver-p256",
    feature = "driver-p256-scalar-mul",
))]
mod driver_rustcrypto;

pub mod p256;

use aead::common::array::ArraySize;
use aead::inout::InOutBuf;
use aead::{AeadCore, AeadInOut, TagPosition};
use cipher::{
    BlockCipherDecBackend, BlockCipherDecClosure, BlockCipherDecrypt, BlockCipherEncBackend, BlockCipherEncClosure,
    BlockCipherEncrypt, BlockModeDecBackend, BlockModeDecClosure, BlockModeDecrypt, BlockModeEncBackend,
    BlockModeEncClosure, BlockModeEncrypt, BlockSizeUser, InOut, IvSizeUser, KeyIvInit, ParBlocksSizeUser,
    StreamCipher, StreamCipherError,
};
use crypto_common::{AlgorithmName, KeySizeUser};
pub use digest;
use digest::{
    FixedOutput, FixedOutputReset, HashMarker, InvalidLength, Key, KeyInit, MacMarker, Output, OutputSizeUser, Reset,
    Update,
};
use generic_array::typenum::{U1, U12, U16, U20, U28, U32, U48, U64, U128};

#[inline]
fn unwrap_inout<'inp, 'out>(buf: InOutBuf<'inp, 'out, u8>) -> embassy_crypto_driver::InOutBuf<'inp, 'out, u8> {
    let len = buf.len();
    let (in_ptr, out_ptr) = buf.into_raw();
    unsafe { embassy_crypto_driver::InOutBuf::from_raw(in_ptr, out_ptr, len) }
}

// ===========================================================================
// Digest macro
// ===========================================================================

macro_rules! impl_digest {
    (
        $name:ident,
        $drv:path,
        $trait:path,
        $size:ty,
        $block_size:ty,
        $alg_name:expr
    ) => {
        /// RustCrypto `Digest` implementation backed by the embassy-crypto-driver unitrait.
        #[derive(Clone)]
        pub struct $name {
            ctx: <$drv as $trait>::Context,
        }

        impl Default for $name {
            #[inline]
            fn default() -> Self {
                Self { ctx: <$drv>::init() }
            }
        }

        impl core::fmt::Debug for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.debug_struct(stringify!($name)).finish_non_exhaustive()
            }
        }

        impl OutputSizeUser for $name {
            type OutputSize = $size;
        }

        impl Update for $name {
            #[inline]
            fn update(&mut self, data: &[u8]) {
                <$drv>::update(&mut self.ctx, data);
            }
        }

        impl FixedOutput for $name {
            #[inline]
            fn finalize_into(self, out: &mut Output<Self>) {
                <$drv>::finalize(self.ctx, out.as_mut_slice());
            }
        }

        impl Reset for $name {
            #[inline]
            fn reset(&mut self) {
                *self = Self::default();
            }
        }

        impl FixedOutputReset for $name {
            #[inline]
            fn finalize_into_reset(&mut self, out: &mut Output<Self>) {
                self.clone().finalize_into(out);
                self.reset();
            }
        }

        impl HashMarker for $name {}

        impl BlockSizeUser for $name {
            type BlockSize = $block_size;
        }

        impl AlgorithmName for $name {
            fn write_alg_name(f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str($alg_name)
            }
        }
    };
}

// ===========================================================================
// HMAC macro
// ===========================================================================

macro_rules! impl_hmac {
    (
        $name:ident,
        $drv:path,
        $trait:path,
        $key_size:ty,
        $out_size:ty
    ) => {
        /// RustCrypto `Mac` implementation backed by the embassy-crypto-driver HMAC unitrait.
        #[derive(Clone)]
        pub struct $name {
            ctx: <$drv as $trait>::Context,
        }

        impl core::fmt::Debug for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.debug_struct(stringify!($name)).finish_non_exhaustive()
            }
        }

        impl OutputSizeUser for $name {
            type OutputSize = $out_size;
        }

        impl KeySizeUser for $name {
            type KeySize = $key_size;
        }

        impl KeyInit for $name {
            #[inline]
            fn new(key: &Key<Self>) -> Self {
                Self {
                    ctx: <$drv>::init(key.as_slice()),
                }
            }

            #[inline]
            fn new_from_slice(key: &[u8]) -> Result<Self, InvalidLength> {
                Ok(Self {
                    ctx: <$drv>::init(key),
                })
            }
        }

        impl Update for $name {
            #[inline]
            fn update(&mut self, data: &[u8]) {
                <$drv>::update(&mut self.ctx, data);
            }
        }

        impl FixedOutput for $name {
            #[inline]
            fn finalize_into(self, out: &mut Output<Self>) {
                <$drv>::finalize(self.ctx, out.as_mut_slice());
            }
        }

        impl MacMarker for $name {}
    };
}

// ===========================================================================
// Digests
// ===========================================================================

impl_digest!(
    Md5,
    embassy_crypto_driver::Md5Impl,
    embassy_crypto_driver::Md5,
    U16,
    U64,
    "MD5"
);

impl_digest!(
    Sha1,
    embassy_crypto_driver::Sha1Impl,
    embassy_crypto_driver::Sha1,
    U20,
    U64,
    "SHA-1"
);

impl_digest!(
    Sha224,
    embassy_crypto_driver::Sha224Impl,
    embassy_crypto_driver::Sha224,
    U28,
    U64,
    "SHA-224"
);

impl_digest!(
    Sha256,
    embassy_crypto_driver::Sha256Impl,
    embassy_crypto_driver::Sha256,
    U32,
    U64,
    "SHA-256"
);

impl_digest!(
    Sha384,
    embassy_crypto_driver::Sha384Impl,
    embassy_crypto_driver::Sha384,
    U48,
    U128,
    "SHA-384"
);

impl_digest!(
    Sha512_224,
    embassy_crypto_driver::Sha512_224Impl,
    embassy_crypto_driver::Sha512_224,
    U28,
    U128,
    "SHA-512/224"
);

impl_digest!(
    Sha512_256,
    embassy_crypto_driver::Sha512_256Impl,
    embassy_crypto_driver::Sha512_256,
    U32,
    U128,
    "SHA-512/256"
);

impl_digest!(
    Sha512,
    embassy_crypto_driver::Sha512Impl,
    embassy_crypto_driver::Sha512,
    U64,
    U128,
    "SHA-512"
);

// ===========================================================================
// HMACs
// ===========================================================================

impl_hmac!(
    HmacSha1,
    embassy_crypto_driver::HmacSha1Impl,
    embassy_crypto_driver::HmacSha1,
    U64,
    U20
);

impl_hmac!(
    HmacSha224,
    embassy_crypto_driver::HmacSha224Impl,
    embassy_crypto_driver::HmacSha224,
    U64,
    U28
);

impl_hmac!(
    HmacSha256,
    embassy_crypto_driver::HmacSha256Impl,
    embassy_crypto_driver::HmacSha256,
    U64,
    U32
);

impl_hmac!(
    HmacSha384,
    embassy_crypto_driver::HmacSha384Impl,
    embassy_crypto_driver::HmacSha384,
    U128,
    U48
);

impl_hmac!(
    HmacSha512_224,
    embassy_crypto_driver::HmacSha512_224Impl,
    embassy_crypto_driver::HmacSha512_224,
    U128,
    U28
);

impl_hmac!(
    HmacSha512_256,
    embassy_crypto_driver::HmacSha512_256Impl,
    embassy_crypto_driver::HmacSha512_256,
    U128,
    U32
);

impl_hmac!(
    HmacSha512,
    embassy_crypto_driver::HmacSha512Impl,
    embassy_crypto_driver::HmacSha512,
    U128,
    U64
);

// ===========================================================================
// ECB block-cipher macro
// ===========================================================================

macro_rules! impl_ecb {
    (
        $name:ident,
        $drv:path,
        $trait:path,
        $key_size:ty
    ) => {
        /// RustCrypto `BlockCipherEncrypt`/`BlockCipherDecrypt` implementation backed by the embassy-crypto-driver unitrait.
        #[derive(Clone)]
        pub struct $name {
            ctx: <$drv as $trait>::Context,
        }

        impl core::fmt::Debug for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.debug_struct(stringify!($name)).finish_non_exhaustive()
            }
        }

        impl BlockSizeUser for $name {
            type BlockSize = U16;
        }

        impl ParBlocksSizeUser for $name {
            type ParBlocksSize = U1;
        }

        impl KeySizeUser for $name {
            type KeySize = $key_size;
        }

        impl KeyInit for $name {
            #[inline]
            fn new(key: &Key<Self>) -> Self {
                Self {
                    ctx: <$drv>::init(key.as_slice().try_into().unwrap()),
                }
            }
        }

        impl BlockCipherEncBackend for $name {
            #[inline]
            fn encrypt_block(&self, block: InOut<'_, '_, cipher::Block<Self>>) {
                let (in_ptr, out_ptr) = block.into_raw();
                let buf =
                    unsafe { embassy_crypto_driver::InOutBuf::from_raw(in_ptr as *const u8, out_ptr as *mut u8, 16) };
                <$drv>::encrypt_blocks(&self.ctx, buf);
            }
        }

        impl BlockCipherEncrypt for $name {
            #[inline]
            fn encrypt_with_backend(&self, f: impl BlockCipherEncClosure<BlockSize = Self::BlockSize>) {
                f.call(self);
            }

            #[inline]
            fn encrypt_blocks(&self, blocks: &mut [cipher::Block<Self>]) {
                if blocks.is_empty() {
                    return;
                }
                let in_ptr = blocks.as_ptr() as *const u8;
                let out_ptr = blocks.as_mut_ptr() as *mut u8;
                let buf = unsafe { embassy_crypto_driver::InOutBuf::from_raw(in_ptr, out_ptr, blocks.len() * 16) };
                <$drv>::encrypt_blocks(&self.ctx, buf);
            }

            #[inline]
            fn encrypt_blocks_inout(&self, blocks: InOutBuf<'_, '_, cipher::Block<Self>>) {
                if blocks.is_empty() {
                    return;
                }
                let len = blocks.len() * 16;
                let (in_ptr, out_ptr) = blocks.into_raw();
                let buf =
                    unsafe { embassy_crypto_driver::InOutBuf::from_raw(in_ptr as *const u8, out_ptr as *mut u8, len) };
                <$drv>::encrypt_blocks(&self.ctx, buf);
            }
        }

        impl BlockCipherDecBackend for $name {
            #[inline]
            fn decrypt_block(&self, block: InOut<'_, '_, cipher::Block<Self>>) {
                let (in_ptr, out_ptr) = block.into_raw();
                let buf =
                    unsafe { embassy_crypto_driver::InOutBuf::from_raw(in_ptr as *const u8, out_ptr as *mut u8, 16) };
                <$drv>::decrypt_blocks(&self.ctx, buf);
            }
        }

        impl BlockCipherDecrypt for $name {
            #[inline]
            fn decrypt_with_backend(&self, f: impl BlockCipherDecClosure<BlockSize = Self::BlockSize>) {
                f.call(self);
            }

            #[inline]
            fn decrypt_blocks(&self, blocks: &mut [cipher::Block<Self>]) {
                if blocks.is_empty() {
                    return;
                }
                let in_ptr = blocks.as_ptr() as *const u8;
                let out_ptr = blocks.as_mut_ptr() as *mut u8;
                let buf = unsafe { embassy_crypto_driver::InOutBuf::from_raw(in_ptr, out_ptr, blocks.len() * 16) };
                <$drv>::decrypt_blocks(&self.ctx, buf);
            }

            #[inline]
            fn decrypt_blocks_inout(&self, blocks: InOutBuf<'_, '_, cipher::Block<Self>>) {
                if blocks.is_empty() {
                    return;
                }
                let len = blocks.len() * 16;
                let (in_ptr, out_ptr) = blocks.into_raw();
                let buf =
                    unsafe { embassy_crypto_driver::InOutBuf::from_raw(in_ptr as *const u8, out_ptr as *mut u8, len) };
                <$drv>::decrypt_blocks(&self.ctx, buf);
            }
        }
    };
}

// ===========================================================================
// CBC block-cipher macro
// ===========================================================================

macro_rules! impl_cbc_enc {
    (
        $name:ident,
        $drv:path,
        $trait:path,
        $key_size:ty
    ) => {
        /// RustCrypto `BlockModeEncrypt` implementation backed by the embassy-crypto-driver unitrait.
        pub struct $name {
            ctx: <$drv as $trait>::EncryptContext,
        }

        impl core::fmt::Debug for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.debug_struct(stringify!($name)).finish_non_exhaustive()
            }
        }

        impl BlockSizeUser for $name {
            type BlockSize = U16;
        }

        impl ParBlocksSizeUser for $name {
            type ParBlocksSize = U1;
        }

        impl KeySizeUser for $name {
            type KeySize = $key_size;
        }

        impl IvSizeUser for $name {
            type IvSize = U16;
        }

        impl KeyIvInit for $name {
            #[inline]
            fn new(key: &Key<Self>, iv: &cipher::Iv<Self>) -> Self {
                Self {
                    ctx: <$drv>::encrypt_init(
                        key.as_slice().try_into().unwrap(),
                        iv.as_slice().try_into().unwrap(),
                    ),
                }
            }
        }

        impl BlockModeEncBackend for $name {
            #[inline]
            fn encrypt_block(&mut self, block: InOut<'_, '_, cipher::Block<Self>>) {
                let (in_ptr, out_ptr) = block.into_raw();
                let buf =
                    unsafe { embassy_crypto_driver::InOutBuf::from_raw(in_ptr as *const u8, out_ptr as *mut u8, 16) };
                <$drv>::encrypt_blocks(&mut self.ctx, buf);
            }
        }

        impl BlockModeEncrypt for $name {
            #[inline]
            fn encrypt_with_backend(&mut self, f: impl BlockModeEncClosure<BlockSize = Self::BlockSize>) {
                f.call(self);
            }

            #[inline]
            fn encrypt_blocks(&mut self, blocks: &mut [cipher::Block<Self>]) {
                if blocks.is_empty() {
                    return;
                }
                let in_ptr = blocks.as_ptr() as *const u8;
                let out_ptr = blocks.as_mut_ptr() as *mut u8;
                let buf = unsafe { embassy_crypto_driver::InOutBuf::from_raw(in_ptr, out_ptr, blocks.len() * 16) };
                <$drv>::encrypt_blocks(&mut self.ctx, buf);
            }
        }
    };
}

macro_rules! impl_cbc_dec {
    (
        $name:ident,
        $drv:path,
        $trait:path,
        $key_size:ty
    ) => {
        /// RustCrypto `BlockModeDecrypt` implementation backed by the embassy-crypto-driver unitrait.
        pub struct $name {
            ctx: <$drv as $trait>::DecryptContext,
        }

        impl core::fmt::Debug for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.debug_struct(stringify!($name)).finish_non_exhaustive()
            }
        }

        impl BlockSizeUser for $name {
            type BlockSize = U16;
        }

        impl ParBlocksSizeUser for $name {
            type ParBlocksSize = U1;
        }

        impl KeySizeUser for $name {
            type KeySize = $key_size;
        }

        impl IvSizeUser for $name {
            type IvSize = U16;
        }

        impl KeyIvInit for $name {
            #[inline]
            fn new(key: &Key<Self>, iv: &cipher::Iv<Self>) -> Self {
                Self {
                    ctx: <$drv>::decrypt_init(
                        key.as_slice().try_into().unwrap(),
                        iv.as_slice().try_into().unwrap(),
                    ),
                }
            }
        }

        impl BlockModeDecBackend for $name {
            #[inline]
            fn decrypt_block(&mut self, block: InOut<'_, '_, cipher::Block<Self>>) {
                let (in_ptr, out_ptr) = block.into_raw();
                let buf =
                    unsafe { embassy_crypto_driver::InOutBuf::from_raw(in_ptr as *const u8, out_ptr as *mut u8, 16) };
                <$drv>::decrypt_blocks(&mut self.ctx, buf);
            }
        }

        impl BlockModeDecrypt for $name {
            #[inline]
            fn decrypt_with_backend(&mut self, f: impl BlockModeDecClosure<BlockSize = Self::BlockSize>) {
                f.call(self);
            }

            #[inline]
            fn decrypt_blocks(&mut self, blocks: &mut [cipher::Block<Self>]) {
                if blocks.is_empty() {
                    return;
                }
                let in_ptr = blocks.as_ptr() as *const u8;
                let out_ptr = blocks.as_mut_ptr() as *mut u8;
                let buf = unsafe { embassy_crypto_driver::InOutBuf::from_raw(in_ptr, out_ptr, blocks.len() * 16) };
                <$drv>::decrypt_blocks(&mut self.ctx, buf);
            }
        }
    };
}

// ===========================================================================
// GCM AEAD macro
// ===========================================================================

macro_rules! impl_gcm {
    (
        $name:ident,
        $drv:path,
        $trait:path,
        $key_size:ty
    ) => {
        /// RustCrypto `AeadInPlace` implementation backed by the embassy-crypto-driver unitrait.
        pub struct $name {
            ctx: <$drv as $trait>::Context,
        }

        impl core::fmt::Debug for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.debug_struct(stringify!($name)).finish_non_exhaustive()
            }
        }

        impl KeySizeUser for $name {
            type KeySize = $key_size;
        }

        impl KeyInit for $name {
            #[inline]
            fn new(key: &Key<Self>) -> Self {
                Self {
                    ctx: <$drv>::init(key.as_slice().try_into().unwrap()),
                }
            }
        }

        impl AeadCore for $name {
            type NonceSize = U12;
            type TagSize = U16;
            const TAG_POSITION: TagPosition = TagPosition::Postfix;
        }

        impl AeadInOut for $name {
            fn encrypt_inout_detached(
                &self,
                nonce: &aead::Nonce<Self>,
                associated_data: &[u8],
                buffer: InOutBuf<'_, '_, u8>,
            ) -> Result<aead::Tag<Self>, aead::Error> {
                let mut tag = aead::Tag::<Self>::default();
                <$drv>::encrypt(
                    &self.ctx,
                    nonce.as_slice(),
                    associated_data,
                    unwrap_inout(buffer),
                    tag.as_mut_slice().try_into().unwrap(),
                )
                .map_err(|_| aead::Error)?;
                Ok(tag)
            }

            fn decrypt_inout_detached(
                &self,
                nonce: &aead::Nonce<Self>,
                associated_data: &[u8],
                buffer: InOutBuf<'_, '_, u8>,
                tag: &aead::Tag<Self>,
            ) -> Result<(), aead::Error> {
                <$drv>::decrypt(
                    &self.ctx,
                    nonce.as_slice(),
                    associated_data,
                    unwrap_inout(buffer),
                    tag.as_slice().try_into().unwrap(),
                )
                .map_err(|_| aead::Error)
            }
        }
    };
}

// ===========================================================================
// ECB implementations
// ===========================================================================

impl_ecb!(
    Aes128,
    embassy_crypto_driver::Aes128EcbImpl,
    embassy_crypto_driver::Aes128Ecb,
    U16
);

impl_ecb!(
    Aes256,
    embassy_crypto_driver::Aes256EcbImpl,
    embassy_crypto_driver::Aes256Ecb,
    U32
);

// ===========================================================================
// CBC implementations
// ===========================================================================

impl_cbc_enc!(
    Aes128CbcEncrypt,
    embassy_crypto_driver::Aes128CbcImpl,
    embassy_crypto_driver::Aes128Cbc,
    U16
);

impl_cbc_dec!(
    Aes128CbcDecrypt,
    embassy_crypto_driver::Aes128CbcImpl,
    embassy_crypto_driver::Aes128Cbc,
    U16
);

impl_cbc_enc!(
    Aes256CbcEncrypt,
    embassy_crypto_driver::Aes256CbcImpl,
    embassy_crypto_driver::Aes256Cbc,
    U32
);

impl_cbc_dec!(
    Aes256CbcDecrypt,
    embassy_crypto_driver::Aes256CbcImpl,
    embassy_crypto_driver::Aes256Cbc,
    U32
);

// ===========================================================================
// GCM implementations
// ===========================================================================

impl_gcm!(
    Aes128Gcm,
    embassy_crypto_driver::Aes128GcmImpl,
    embassy_crypto_driver::Aes128Gcm,
    U16
);

impl_gcm!(
    Aes256Gcm,
    embassy_crypto_driver::Aes256GcmImpl,
    embassy_crypto_driver::Aes256Gcm,
    U32
);

// ===========================================================================
// CCM implementations
// ===========================================================================

/// RustCrypto `AeadInPlace` implementation for AES-128 CCM.
///
/// Generic over `TagSize` (4, 8, or 16) and `NonceSize` (7–13).
pub struct Aes128Ccm<TagSize, NonceSize> {
    ctx: embassy_crypto_driver::Aes128CcmImplContext,
    _phantom: core::marker::PhantomData<(TagSize, NonceSize)>,
}

impl<TagSize, NonceSize> Clone for Aes128Ccm<TagSize, NonceSize> {
    fn clone(&self) -> Self {
        Self {
            ctx: self.ctx.clone(),
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<TagSize, NonceSize> core::fmt::Debug for Aes128Ccm<TagSize, NonceSize> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Aes128Ccm").finish_non_exhaustive()
    }
}

impl<TagSize, NonceSize> KeySizeUser for Aes128Ccm<TagSize, NonceSize> {
    type KeySize = U16;
}

impl<TagSize, NonceSize> KeyInit for Aes128Ccm<TagSize, NonceSize> {
    fn new(key: &Key<Self>) -> Self {
        Self {
            ctx: embassy_crypto_driver::Aes128CcmImpl::init(key.as_slice().try_into().unwrap()),
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<TagSize, NonceSize> AeadCore for Aes128Ccm<TagSize, NonceSize>
where
    TagSize: ArraySize,
    NonceSize: ArraySize,
{
    type NonceSize = NonceSize;
    type TagSize = TagSize;
    const TAG_POSITION: TagPosition = TagPosition::Postfix;
}

impl<TagSize, NonceSize> AeadInOut for Aes128Ccm<TagSize, NonceSize>
where
    TagSize: ArraySize,
    NonceSize: ArraySize,
{
    fn encrypt_inout_detached(
        &self,
        nonce: &aead::Nonce<Self>,
        associated_data: &[u8],
        buffer: InOutBuf<'_, '_, u8>,
    ) -> Result<aead::Tag<Self>, aead::Error> {
        let mut tag = aead::Tag::<Self>::default();
        embassy_crypto_driver::Aes128CcmImpl::encrypt(
            &self.ctx,
            nonce.as_slice(),
            associated_data,
            unwrap_inout(buffer),
            tag.as_mut_slice(),
        )
        .map_err(|_| aead::Error)?;
        Ok(tag)
    }

    fn decrypt_inout_detached(
        &self,
        nonce: &aead::Nonce<Self>,
        associated_data: &[u8],
        buffer: InOutBuf<'_, '_, u8>,
        tag: &aead::Tag<Self>,
    ) -> Result<(), aead::Error> {
        embassy_crypto_driver::Aes128CcmImpl::decrypt(
            &self.ctx,
            nonce.as_slice(),
            associated_data,
            unwrap_inout(buffer),
            tag.as_slice(),
        )
        .map_err(|_| aead::Error)
    }
}

/// RustCrypto `AeadInPlace` implementation for AES-256 CCM.
///
/// Generic over `TagSize` (4, 8, or 16) and `NonceSize` (7–13).
pub struct Aes256Ccm<TagSize, NonceSize> {
    ctx: embassy_crypto_driver::Aes256CcmImplContext,
    _phantom: core::marker::PhantomData<(TagSize, NonceSize)>,
}

impl<TagSize, NonceSize> Clone for Aes256Ccm<TagSize, NonceSize> {
    fn clone(&self) -> Self {
        Self {
            ctx: self.ctx.clone(),
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<TagSize, NonceSize> core::fmt::Debug for Aes256Ccm<TagSize, NonceSize> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Aes256Ccm").finish_non_exhaustive()
    }
}

impl<TagSize, NonceSize> KeySizeUser for Aes256Ccm<TagSize, NonceSize> {
    type KeySize = U32;
}

impl<TagSize, NonceSize> KeyInit for Aes256Ccm<TagSize, NonceSize> {
    fn new(key: &Key<Self>) -> Self {
        Self {
            ctx: embassy_crypto_driver::Aes256CcmImpl::init(key.as_slice().try_into().unwrap()),
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<TagSize, NonceSize> AeadCore for Aes256Ccm<TagSize, NonceSize>
where
    TagSize: ArraySize,
    NonceSize: ArraySize,
{
    type NonceSize = NonceSize;
    type TagSize = TagSize;
    const TAG_POSITION: TagPosition = TagPosition::Postfix;
}

impl<TagSize, NonceSize> AeadInOut for Aes256Ccm<TagSize, NonceSize>
where
    TagSize: ArraySize,
    NonceSize: ArraySize,
{
    fn encrypt_inout_detached(
        &self,
        nonce: &aead::Nonce<Self>,
        associated_data: &[u8],
        buffer: InOutBuf<'_, '_, u8>,
    ) -> Result<aead::Tag<Self>, aead::Error> {
        let mut tag = aead::Tag::<Self>::default();
        embassy_crypto_driver::Aes256CcmImpl::encrypt(
            &self.ctx,
            nonce.as_slice(),
            associated_data,
            unwrap_inout(buffer),
            tag.as_mut_slice(),
        )
        .map_err(|_| aead::Error)?;
        Ok(tag)
    }

    fn decrypt_inout_detached(
        &self,
        nonce: &aead::Nonce<Self>,
        associated_data: &[u8],
        buffer: InOutBuf<'_, '_, u8>,
        tag: &aead::Tag<Self>,
    ) -> Result<(), aead::Error> {
        embassy_crypto_driver::Aes256CcmImpl::decrypt(
            &self.ctx,
            nonce.as_slice(),
            associated_data,
            unwrap_inout(buffer),
            tag.as_slice(),
        )
        .map_err(|_| aead::Error)
    }
}

// ===========================================================================
// CTR stream-cipher wrapper types
// ===========================================================================

macro_rules! impl_ctr {
    (
        $name:ident,
        $drv:path,
        $trait:path,
        $key_size:ty
    ) => {
        /// RustCrypto `StreamCipher` implementation backed by the
        /// embassy-crypto-driver unitrait.
        ///
        /// Uses AES-CTR mode with a 128-bit big-endian counter (NIST SP 800-38A).
        /// Encryption and decryption are the same operation.
        pub struct $name {
            ctx: <$drv as $trait>::Context,
        }

        impl core::fmt::Debug for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.debug_struct(stringify!($name)).finish_non_exhaustive()
            }
        }

        impl KeySizeUser for $name {
            type KeySize = $key_size;
        }

        impl IvSizeUser for $name {
            type IvSize = U16;
        }

        impl KeyIvInit for $name {
            #[inline]
            fn new(key: &Key<Self>, iv: &cipher::Iv<Self>) -> Self {
                Self {
                    ctx: <$drv>::init(
                        key.as_slice().try_into().unwrap(),
                        iv.as_slice().try_into().unwrap(),
                    ),
                }
            }
        }

        impl StreamCipher for $name {
            #[inline]
            fn check_remaining(&self, _data_len: usize) -> Result<(), StreamCipherError> {
                // AES-CTR with a 128-bit counter has 2^128 blocks = 2^132 bytes
                // of keystream before repetition. For any practical embedded
                // buffer this is effectively infinite.
                Ok(())
            }

            #[inline]
            fn unchecked_apply_keystream_inout(&mut self, buf: InOutBuf<'_, '_, u8>) {
                <$drv>::apply_keystream(&mut self.ctx, unwrap_inout(buf));
            }

            #[inline]
            fn unchecked_write_keystream(&mut self, buf: &mut [u8]) {
                buf.fill(0);
                <$drv>::apply_keystream(&mut self.ctx, buf.into());
            }
        }
    };
}

impl_ctr!(
    Aes128Ctr,
    embassy_crypto_driver::Aes128CtrImpl,
    embassy_crypto_driver::Aes128Ctr,
    U16
);

impl_ctr!(
    Aes256Ctr,
    embassy_crypto_driver::Aes256CtrImpl,
    embassy_crypto_driver::Aes256Ctr,
    U32
);

// ===========================================================================
// CMAC macro
// ===========================================================================

macro_rules! impl_cmac {
    (
        $name:ident,
        $drv:path,
        $trait:path,
        $key_size:ty
    ) => {
        /// RustCrypto `Mac` implementation backed by the embassy-crypto-driver unitrait.
        #[derive(Clone)]
        pub struct $name {
            ctx: <$drv as $trait>::Context,
        }

        impl core::fmt::Debug for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.debug_struct(stringify!($name)).finish_non_exhaustive()
            }
        }

        impl OutputSizeUser for $name {
            type OutputSize = U16;
        }

        impl KeySizeUser for $name {
            type KeySize = $key_size;
        }

        impl KeyInit for $name {
            #[inline]
            fn new(key: &Key<Self>) -> Self {
                Self {
                    ctx: <$drv>::init(key.as_slice().try_into().unwrap()),
                }
            }
        }

        impl Update for $name {
            #[inline]
            fn update(&mut self, data: &[u8]) {
                <$drv>::update(&mut self.ctx, data);
            }
        }

        impl FixedOutput for $name {
            #[inline]
            fn finalize_into(self, out: &mut Output<Self>) {
                <$drv>::finalize(self.ctx, out.as_mut_slice().try_into().unwrap());
            }
        }

        impl Reset for $name {
            #[inline]
            fn reset(&mut self) {
                <$drv>::reset(&mut self.ctx);
            }
        }

        impl FixedOutputReset for $name {
            #[inline]
            fn finalize_into_reset(&mut self, out: &mut Output<Self>) {
                self.clone().finalize_into(out);
                self.reset();
            }
        }

        impl MacMarker for $name {}
    };
}

// ===========================================================================
// CMAC implementations
// ===========================================================================

impl_cmac!(
    Aes128Cmac,
    embassy_crypto_driver::Aes128CmacImpl,
    embassy_crypto_driver::Aes128Cmac,
    U16
);

impl_cmac!(
    Aes256Cmac,
    embassy_crypto_driver::Aes256CmacImpl,
    embassy_crypto_driver::Aes256Cmac,
    U32
);
