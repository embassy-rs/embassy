#![no_std]
#![doc = "RustCrypto `digest` and `Mac` trait implementations backed by `embassy-crypto-driver` unitraits.\n\n\
This crate wraps the hardware-agnostic hash and HMAC unitraits from\n\
`embassy-crypto-driver` with the standard RustCrypto `digest` traits,\n\
so existing RustCrypto code can use embassy-registered crypto drivers\n\
without modification.\n\n\
# Digest Usage\n\
```rust,ignore\n\
use embassy_crypto::Sha256;\n\
use digest::Digest;\n\
\n\
let mut hasher = Sha256::new();\n\
hasher.update(b\"hello world\");\n\
let result = hasher.finalize();\n\
```\n\n\
# HMAC Usage\n\
```rust,ignore\n\
use embassy_crypto::HmacSha256;\n\
use digest::Mac;\n\
\n\
let mut mac = HmacSha256::new_from_slice(b\"my key\").unwrap();\n\
mac.update(b\"hello world\");\n\
let result = mac.finalize();\n\
```\n\n\
# Linkage\n\
At link time exactly one crate in the dependency tree must register a driver\n\
using the `embassy_crypto_*_impl!` and `embassy_crypto_hmac_*_impl!` macros\n\
from `embassy-crypto-driver`. If zero or multiple drivers are registered,\n\
linking will fail."]

use aead::common::array::ArraySize;
use aead::inout::InOutBuf;
use aead::{AeadCore, AeadInOut, TagPosition};
use cipher::{
    BlockCipherDecBackend, BlockCipherDecClosure, BlockCipherDecrypt, BlockCipherEncBackend, BlockCipherEncClosure,
    BlockCipherEncrypt, BlockModeDecBackend, BlockModeDecClosure, BlockModeDecrypt, BlockModeEncBackend,
    BlockModeEncClosure, BlockModeEncrypt, BlockSizeUser, InOut, IvSizeUser, KeyIvInit, ParBlocksSizeUser,
};
use crypto_common::KeySizeUser;
pub use digest;
use digest::{
    FixedOutput, FixedOutputReset, HashMarker, InvalidLength, Key, KeyInit, MacMarker, Output, OutputSizeUser, Reset,
    Update,
};
use generic_array::typenum::{U1, U12, U16, U20, U28, U32, U48, U64, U128};

// ===========================================================================
// Digest macro
// ===========================================================================

macro_rules! impl_digest {
    (
        $name:ident,
        $ctx:ty,
        $init:path,
        $clone:path,
        $update:path,
        $finalize:path,
        $size:ty
    ) => {
        /// RustCrypto `Digest` implementation backed by the embassy-crypto-driver unitrait.
        pub struct $name {
            ctx: $ctx,
        }

        impl Clone for $name {
            #[inline]
            fn clone(&self) -> Self {
                Self {
                    ctx: $clone(&self.ctx),
                }
            }
        }

        impl Default for $name {
            #[inline]
            fn default() -> Self {
                Self { ctx: $init() }
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
                $update(&mut self.ctx, data);
            }
        }

        impl FixedOutput for $name {
            #[inline]
            fn finalize_into(self, out: &mut Output<Self>) {
                $finalize(self.ctx, out.as_mut_slice());
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
    };
}

// ===========================================================================
// HMAC macro
// ===========================================================================

macro_rules! impl_hmac {
    (
        $name:ident,
        $ctx:ty,
        $init:path,
        $clone:path,
        $update:path,
        $finalize:path,
        $reset:path,
        $key_size:ty,
        $out_size:ty
    ) => {
        /// RustCrypto `Mac` implementation backed by the embassy-crypto-driver HMAC unitrait.
        pub struct $name {
            ctx: $ctx,
        }

        impl Clone for $name {
            #[inline]
            fn clone(&self) -> Self {
                Self {
                    ctx: $clone(&self.ctx),
                }
            }
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
                    ctx: $init(key.as_slice()),
                }
            }

            #[inline]
            fn new_from_slice(key: &[u8]) -> Result<Self, InvalidLength> {
                Ok(Self { ctx: $init(key) })
            }
        }

        impl Update for $name {
            #[inline]
            fn update(&mut self, data: &[u8]) {
                $update(&mut self.ctx, data);
            }
        }

        impl FixedOutput for $name {
            #[inline]
            fn finalize_into(self, out: &mut Output<Self>) {
                $finalize(self.ctx, out.as_mut_slice());
            }
        }

        impl Reset for $name {
            #[inline]
            fn reset(&mut self) {
                $reset(&mut self.ctx);
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
// Digests
// ===========================================================================

impl_digest!(
    Md5,
    embassy_crypto_driver::Md5Context,
    embassy_crypto_driver::md5_init,
    embassy_crypto_driver::md5_clone,
    embassy_crypto_driver::md5_update,
    embassy_crypto_driver::md5_finalize,
    U16
);

impl_digest!(
    Sha1,
    embassy_crypto_driver::Sha1Context,
    embassy_crypto_driver::sha1_init,
    embassy_crypto_driver::sha1_clone,
    embassy_crypto_driver::sha1_update,
    embassy_crypto_driver::sha1_finalize,
    U20
);

impl_digest!(
    Sha224,
    embassy_crypto_driver::Sha224Context,
    embassy_crypto_driver::sha224_init,
    embassy_crypto_driver::sha224_clone,
    embassy_crypto_driver::sha224_update,
    embassy_crypto_driver::sha224_finalize,
    U28
);

impl_digest!(
    Sha256,
    embassy_crypto_driver::Sha256Context,
    embassy_crypto_driver::sha256_init,
    embassy_crypto_driver::sha256_clone,
    embassy_crypto_driver::sha256_update,
    embassy_crypto_driver::sha256_finalize,
    U32
);

impl_digest!(
    Sha384,
    embassy_crypto_driver::Sha384Context,
    embassy_crypto_driver::sha384_init,
    embassy_crypto_driver::sha384_clone,
    embassy_crypto_driver::sha384_update,
    embassy_crypto_driver::sha384_finalize,
    U48
);

impl_digest!(
    Sha512_224,
    embassy_crypto_driver::Sha512_224Context,
    embassy_crypto_driver::sha512_224_init,
    embassy_crypto_driver::sha512_224_clone,
    embassy_crypto_driver::sha512_224_update,
    embassy_crypto_driver::sha512_224_finalize,
    U28
);

impl_digest!(
    Sha512_256,
    embassy_crypto_driver::Sha512_256Context,
    embassy_crypto_driver::sha512_256_init,
    embassy_crypto_driver::sha512_256_clone,
    embassy_crypto_driver::sha512_256_update,
    embassy_crypto_driver::sha512_256_finalize,
    U32
);

impl_digest!(
    Sha512,
    embassy_crypto_driver::Sha512Context,
    embassy_crypto_driver::sha512_init,
    embassy_crypto_driver::sha512_clone,
    embassy_crypto_driver::sha512_update,
    embassy_crypto_driver::sha512_finalize,
    U64
);

// ===========================================================================
// HMACs
// ===========================================================================

impl_hmac!(
    HmacSha1,
    embassy_crypto_driver::HmacSha1Context,
    embassy_crypto_driver::hmac_sha1_init,
    embassy_crypto_driver::hmac_sha1_clone,
    embassy_crypto_driver::hmac_sha1_update,
    embassy_crypto_driver::hmac_sha1_finalize,
    embassy_crypto_driver::hmac_sha1_reset,
    U64,
    U20
);

impl_hmac!(
    HmacSha224,
    embassy_crypto_driver::HmacSha224Context,
    embassy_crypto_driver::hmac_sha224_init,
    embassy_crypto_driver::hmac_sha224_clone,
    embassy_crypto_driver::hmac_sha224_update,
    embassy_crypto_driver::hmac_sha224_finalize,
    embassy_crypto_driver::hmac_sha224_reset,
    U64,
    U28
);

impl_hmac!(
    HmacSha256,
    embassy_crypto_driver::HmacSha256Context,
    embassy_crypto_driver::hmac_sha256_init,
    embassy_crypto_driver::hmac_sha256_clone,
    embassy_crypto_driver::hmac_sha256_update,
    embassy_crypto_driver::hmac_sha256_finalize,
    embassy_crypto_driver::hmac_sha256_reset,
    U64,
    U32
);

impl_hmac!(
    HmacSha384,
    embassy_crypto_driver::HmacSha384Context,
    embassy_crypto_driver::hmac_sha384_init,
    embassy_crypto_driver::hmac_sha384_clone,
    embassy_crypto_driver::hmac_sha384_update,
    embassy_crypto_driver::hmac_sha384_finalize,
    embassy_crypto_driver::hmac_sha384_reset,
    U128,
    U48
);

impl_hmac!(
    HmacSha512_224,
    embassy_crypto_driver::HmacSha512_224Context,
    embassy_crypto_driver::hmac_sha512_224_init,
    embassy_crypto_driver::hmac_sha512_224_clone,
    embassy_crypto_driver::hmac_sha512_224_update,
    embassy_crypto_driver::hmac_sha512_224_finalize,
    embassy_crypto_driver::hmac_sha512_224_reset,
    U128,
    U28
);

impl_hmac!(
    HmacSha512_256,
    embassy_crypto_driver::HmacSha512_256Context,
    embassy_crypto_driver::hmac_sha512_256_init,
    embassy_crypto_driver::hmac_sha512_256_clone,
    embassy_crypto_driver::hmac_sha512_256_update,
    embassy_crypto_driver::hmac_sha512_256_finalize,
    embassy_crypto_driver::hmac_sha512_256_reset,
    U128,
    U32
);

impl_hmac!(
    HmacSha512,
    embassy_crypto_driver::HmacSha512Context,
    embassy_crypto_driver::hmac_sha512_init,
    embassy_crypto_driver::hmac_sha512_clone,
    embassy_crypto_driver::hmac_sha512_update,
    embassy_crypto_driver::hmac_sha512_finalize,
    embassy_crypto_driver::hmac_sha512_reset,
    U128,
    U64
);

// ===========================================================================
// ECB block-cipher macro
// ===========================================================================

macro_rules! impl_ecb {
    (
        $name:ident,
        $ctx:ty,
        $init:path,
        $clone:path,
        $enc:path,
        $dec:path,
        $key_size:ty
    ) => {
        /// RustCrypto `BlockCipherEncrypt`/`BlockCipherDecrypt` implementation backed by the embassy-crypto-driver unitrait.
        pub struct $name {
            ctx: $ctx,
        }

        impl Clone for $name {
            #[inline]
            fn clone(&self) -> Self {
                Self {
                    ctx: $clone(&self.ctx),
                }
            }
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
                    ctx: $init(key.as_slice().try_into().unwrap()),
                }
            }
        }

        impl BlockCipherEncBackend for $name {
            #[inline]
            fn encrypt_block(&self, mut block: InOut<'_, '_, cipher::Block<Self>>) {
                let out: &mut cipher::Block<Self> = block.get_out();
                $enc(&self.ctx, out.as_mut_slice().try_into().unwrap());
            }
        }

        impl BlockCipherEncrypt for $name {
            #[inline]
            fn encrypt_with_backend(&self, f: impl BlockCipherEncClosure<BlockSize = Self::BlockSize>) {
                f.call(self);
            }
        }

        impl BlockCipherDecBackend for $name {
            #[inline]
            fn decrypt_block(&self, mut block: InOut<'_, '_, cipher::Block<Self>>) {
                let out: &mut cipher::Block<Self> = block.get_out();
                $dec(&self.ctx, out.as_mut_slice().try_into().unwrap());
            }
        }

        impl BlockCipherDecrypt for $name {
            #[inline]
            fn decrypt_with_backend(&self, f: impl BlockCipherDecClosure<BlockSize = Self::BlockSize>) {
                f.call(self);
            }
        }
    };
}

// ===========================================================================
// CBC block-cipher macro
// ===========================================================================

macro_rules! impl_cbc {
    (
        $name:ident,
        $ctx:ty,
        $init:path,
        $clone:path,
        $enc:path,
        $dec:path,
        $key_size:ty
    ) => {
        /// RustCrypto `BlockModeEncrypt`/`BlockModeDecrypt` implementation backed by the embassy-crypto-driver unitrait.
        pub struct $name {
            ctx: $ctx,
        }

        impl Clone for $name {
            #[inline]
            fn clone(&self) -> Self {
                Self {
                    ctx: $clone(&self.ctx),
                }
            }
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
                    ctx: $init(
                        key.as_slice().try_into().unwrap(),
                        iv.as_slice().try_into().unwrap(),
                    ),
                }
            }
        }

        impl BlockModeEncBackend for $name {
            #[inline]
            fn encrypt_block(&mut self, mut block: InOut<'_, '_, cipher::Block<Self>>) {
                let out: &mut cipher::Block<Self> = block.get_out();
                $enc(&mut self.ctx, out.as_mut_slice().try_into().unwrap());
            }
        }

        impl BlockModeEncrypt for $name {
            #[inline]
            fn encrypt_with_backend(&mut self, f: impl BlockModeEncClosure<BlockSize = Self::BlockSize>) {
                f.call(self);
            }
        }

        impl BlockModeDecBackend for $name {
            #[inline]
            fn decrypt_block(&mut self, mut block: InOut<'_, '_, cipher::Block<Self>>) {
                let out: &mut cipher::Block<Self> = block.get_out();
                $dec(&mut self.ctx, out.as_mut_slice().try_into().unwrap());
            }
        }

        impl BlockModeDecrypt for $name {
            #[inline]
            fn decrypt_with_backend(&mut self, f: impl BlockModeDecClosure<BlockSize = Self::BlockSize>) {
                f.call(self);
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
        $ctx:ty,
        $init:path,
        $clone:path,
        $enc:path,
        $dec:path,
        $key_size:ty
    ) => {
        /// RustCrypto `AeadInPlace` implementation backed by the embassy-crypto-driver unitrait.
        pub struct $name {
            ctx: $ctx,
        }

        impl Clone for $name {
            #[inline]
            fn clone(&self) -> Self {
                Self {
                    ctx: $clone(&self.ctx),
                }
            }
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
                    ctx: $init(key.as_slice().try_into().unwrap()),
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
                $enc(
                    &self.ctx,
                    nonce.as_slice(),
                    associated_data,
                    buffer.into_out_with_copied_in(),
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
                $dec(
                    &self.ctx,
                    nonce.as_slice(),
                    associated_data,
                    buffer.into_out_with_copied_in(),
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
    Aes128Ecb,
    embassy_crypto_driver::Aes128EcbContext,
    embassy_crypto_driver::aes128ecb_init,
    embassy_crypto_driver::aes128ecb_clone,
    embassy_crypto_driver::aes128ecb_encrypt_block,
    embassy_crypto_driver::aes128ecb_decrypt_block,
    U16
);

impl_ecb!(
    Aes256Ecb,
    embassy_crypto_driver::Aes256EcbContext,
    embassy_crypto_driver::aes256ecb_init,
    embassy_crypto_driver::aes256ecb_clone,
    embassy_crypto_driver::aes256ecb_encrypt_block,
    embassy_crypto_driver::aes256ecb_decrypt_block,
    U32
);

// ===========================================================================
// CBC implementations
// ===========================================================================

impl_cbc!(
    Aes128Cbc,
    embassy_crypto_driver::Aes128CbcContext,
    embassy_crypto_driver::aes128cbc_init,
    embassy_crypto_driver::aes128cbc_clone,
    embassy_crypto_driver::aes128cbc_encrypt_block,
    embassy_crypto_driver::aes128cbc_decrypt_block,
    U16
);

impl_cbc!(
    Aes256Cbc,
    embassy_crypto_driver::Aes256CbcContext,
    embassy_crypto_driver::aes256cbc_init,
    embassy_crypto_driver::aes256cbc_clone,
    embassy_crypto_driver::aes256cbc_encrypt_block,
    embassy_crypto_driver::aes256cbc_decrypt_block,
    U32
);

// ===========================================================================
// GCM implementations
// ===========================================================================

impl_gcm!(
    Aes128Gcm,
    embassy_crypto_driver::Aes128GcmContext,
    embassy_crypto_driver::aes128gcm_init,
    embassy_crypto_driver::aes128gcm_clone,
    embassy_crypto_driver::aes128gcm_encrypt,
    embassy_crypto_driver::aes128gcm_decrypt,
    U16
);

impl_gcm!(
    Aes256Gcm,
    embassy_crypto_driver::Aes256GcmContext,
    embassy_crypto_driver::aes256gcm_init,
    embassy_crypto_driver::aes256gcm_clone,
    embassy_crypto_driver::aes256gcm_encrypt,
    embassy_crypto_driver::aes256gcm_decrypt,
    U32
);

// ===========================================================================
// CCM implementations
// ===========================================================================

/// RustCrypto `AeadInPlace` implementation for AES-128 CCM.
///
/// Generic over `TagSize` (4, 8, or 16) and `NonceSize` (7–13).
pub struct Aes128Ccm<TagSize, NonceSize> {
    ctx: embassy_crypto_driver::Aes128CcmContext,
    _phantom: core::marker::PhantomData<(TagSize, NonceSize)>,
}

impl<TagSize, NonceSize> Clone for Aes128Ccm<TagSize, NonceSize> {
    fn clone(&self) -> Self {
        Self {
            ctx: embassy_crypto_driver::aes128ccm_clone(&self.ctx),
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
            ctx: embassy_crypto_driver::aes128ccm_init(key.as_slice().try_into().unwrap()),
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
        embassy_crypto_driver::aes128ccm_encrypt(
            &self.ctx,
            nonce.as_slice(),
            associated_data,
            buffer.into_out_with_copied_in(),
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
        embassy_crypto_driver::aes128ccm_decrypt(
            &self.ctx,
            nonce.as_slice(),
            associated_data,
            buffer.into_out_with_copied_in(),
            tag.as_slice(),
        )
        .map_err(|_| aead::Error)
    }
}

/// RustCrypto `AeadInPlace` implementation for AES-256 CCM.
///
/// Generic over `TagSize` (4, 8, or 16) and `NonceSize` (7–13).
pub struct Aes256Ccm<TagSize, NonceSize> {
    ctx: embassy_crypto_driver::Aes256CcmContext,
    _phantom: core::marker::PhantomData<(TagSize, NonceSize)>,
}

impl<TagSize, NonceSize> Clone for Aes256Ccm<TagSize, NonceSize> {
    fn clone(&self) -> Self {
        Self {
            ctx: embassy_crypto_driver::aes256ccm_clone(&self.ctx),
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
            ctx: embassy_crypto_driver::aes256ccm_init(key.as_slice().try_into().unwrap()),
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
        embassy_crypto_driver::aes256ccm_encrypt(
            &self.ctx,
            nonce.as_slice(),
            associated_data,
            buffer.into_out_with_copied_in(),
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
        embassy_crypto_driver::aes256ccm_decrypt(
            &self.ctx,
            nonce.as_slice(),
            associated_data,
            buffer.into_out_with_copied_in(),
            tag.as_slice(),
        )
        .map_err(|_| aead::Error)
    }
}
