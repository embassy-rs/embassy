#![no_std]
//! RustCrypto trait implementations backed by `embassy-crypto-driver` unitraits.
///
/// This crate wraps the hardware-agnostic unitraits from `embassy-crypto-driver`
/// with the standard RustCrypto traits, so existing RustCrypto code can use
/// embassy-registered crypto drivers without modification.
///
/// # Supported Operations
///
/// ## Digests
/// - `Md5`, `Sha1`, `Sha224`, `Sha256`, `Sha384`, `Sha512`, `Sha512_224`, `Sha512_256`
///
/// ## HMAC
/// - `HmacSha1`, `HmacSha224`, `HmacSha256`, `HmacSha384`, `HmacSha512`, `HmacSha512_224`, `HmacSha512_256`
///
/// ## Block Ciphers
/// - `Aes128Ecb`, `Aes256Ecb` — ECB mode
/// - `Aes128Cbc`, `Aes256Cbc` — CBC mode
///
/// ## AEAD
/// - `Aes128Gcm`, `Aes256Gcm` — GCM mode
/// - `Aes128Ccm<TagSize, NonceSize>`, `Aes256Ccm<TagSize, NonceSize>` — CCM mode
///
/// ## Elliptic Curve (P256)
/// - `p256::SecretKey`, `p256::PublicKey`, `p256::SharedSecret` — ECDH primitives
/// - `p256::ecdsa::SigningKey`, `p256::ecdsa::VerifyingKey`, `p256::ecdsa::Signature` — ECDSA primitives
///
/// # Digest Usage
/// ```rust,ignore
/// use embassy_crypto::Sha256;
/// use digest::Digest;
///
/// let mut hasher = Sha256::new();
/// hasher.update(b"hello world");
/// let result = hasher.finalize();
/// ```
///
/// # HMAC Usage
/// ```rust,ignore
/// use embassy_crypto::HmacSha256;
/// use digest::Mac;
///
/// let mut mac = HmacSha256::new_from_slice(b"my key").unwrap();
/// mac.update(b"hello world");
/// let result = mac.finalize();
/// ```
///
/// # Block Cipher Usage
/// ```rust,ignore
/// use embassy_crypto::Aes128Cbc;
/// use cipher::{BlockEncryptMut, KeyIvInit};
///
/// let mut cipher = Aes128Cbc::new_from_slices(b"my secret key!!!", b"my iv!!!").unwrap();
/// let mut block = [0u8; 16];
/// cipher.encrypt_block_mut((&mut block).into());
/// ```
///
/// # AEAD Usage
/// ```rust,ignore
/// use embassy_crypto::Aes128Gcm;
/// use aead::{Aead, KeyInit, Nonce};
///
/// let cipher = Aes128Gcm::new_from_slice(b"my secret key!!!").unwrap();
/// let nonce = Nonce::from_slice(b"unique nonce");
/// let ciphertext = cipher.encrypt(nonce, b"plaintext message".as_ref()).unwrap();
/// ```
///
/// # P256 ECDH Usage
/// ```rust,ignore
/// use embassy_crypto::p256::{SecretKey, PublicKey};
///
/// let (secret_key, public_key) = SecretKey::generate().unwrap();
///
/// let peer_public_key = PublicKey::from_bytes(&[0u8; 65]); // received from peer
/// let shared_secret = secret_key.diffie_hellman(&peer_public_key).unwrap();
/// ```
///
/// # P256 ECDSA Usage
/// ```rust,ignore
/// use embassy_crypto::p256::ecdsa::{SigningKey, VerifyingKey, Signature};
/// use signature::{Signer, Verifier};
///
/// let signing_key = SigningKey::from_bytes(&[0u8; 32]);
/// let signature: Signature = signing_key.sign(b"message");
///
/// let verifying_key = signing_key.verifying_key().unwrap();
/// verifying_key.verify(b"message", &signature).unwrap();
/// ```
///
/// # Linkage
/// At link time exactly one crate in the dependency tree must register a driver
/// using the `embassy_crypto_*_impl!` macros from `embassy-crypto-driver`.
/// If zero or multiple drivers are registered, linking will fail.
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
            fn encrypt_block(&self, block: InOut<'_, '_, cipher::Block<Self>>) {
                let out: &mut cipher::Block<Self> = block.into_out_with_copied_in();
                let arr: &mut [u8; 16] = out.as_mut_slice().try_into().unwrap();
                let blocks = core::slice::from_mut(arr);
                $enc(&self.ctx, blocks);
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
            fn decrypt_block(&self, block: InOut<'_, '_, cipher::Block<Self>>) {
                let out: &mut cipher::Block<Self> = block.into_out_with_copied_in();
                let arr: &mut [u8; 16] = out.as_mut_slice().try_into().unwrap();
                let blocks = core::slice::from_mut(arr);
                $dec(&self.ctx, blocks);
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
            fn encrypt_block(&mut self, block: InOut<'_, '_, cipher::Block<Self>>) {
                let out: &mut cipher::Block<Self> = block.into_out_with_copied_in();
                let arr: &mut [u8; 16] = out.as_mut_slice().try_into().unwrap();
                let blocks = core::slice::from_mut(arr);
                $enc(&mut self.ctx, blocks);
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
                let ptr = blocks.as_mut_ptr() as *mut [u8; 16];
                let blocks_arr = unsafe { core::slice::from_raw_parts_mut(ptr, blocks.len()) };
                $enc(&mut self.ctx, blocks_arr);
            }
        }

        impl BlockModeDecBackend for $name {
            #[inline]
            fn decrypt_block(&mut self, block: InOut<'_, '_, cipher::Block<Self>>) {
                let out: &mut cipher::Block<Self> = block.into_out_with_copied_in();
                let arr: &mut [u8; 16] = out.as_mut_slice().try_into().unwrap();
                let blocks = core::slice::from_mut(arr);
                $dec(&mut self.ctx, blocks);
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
                let ptr = blocks.as_mut_ptr() as *mut [u8; 16];
                let blocks_arr = unsafe { core::slice::from_raw_parts_mut(ptr, blocks.len()) };
                $dec(&mut self.ctx, blocks_arr);
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

// ===========================================================================
// P256 Elliptic Curve
// ===========================================================================

/// P-256 (secp256r1) elliptic-curve types.
///
/// Provides ECDH key exchange and ECDSA signing/verification primitives
/// backed by the `embassy-crypto-driver` unitraits. These types mirror
/// the API shape of RustCrypto's `p256` crate but delegate all operations
/// to the registered hardware (or software) driver.
///
/// # TLS and Bluetooth LE
/// - `SecretKey::diffie_hellman` is used for TLS 1.2/1.3 key exchange and
///   Bluetooth LE Secure Connections pairing.
/// - `ecdsa::SigningKey::sign` / `ecdsa::VerifyingKey::verify`
///   are used for TLS certificate authentication and Bluetooth LE signing.
pub mod p256 {
    use core::fmt;

    /// A P-256 secret key (32-byte scalar).
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct SecretKey([u8; 32]);

    impl SecretKey {
        /// Create a secret key from a 32-byte scalar.
        pub fn from_bytes(bytes: &[u8; 32]) -> Self {
            Self(*bytes)
        }

        /// Return the secret key bytes.
        pub fn as_bytes(&self) -> &[u8; 32] {
            &self.0
        }

        /// Generate a new random secret key and its corresponding public key.
        pub fn generate() -> Result<(Self, PublicKey), embassy_crypto_driver::CryptoError> {
            let mut sk = [0u8; 32];
            let mut pk = [0u8; 65];
            embassy_crypto_driver::p256ecdh_generate_keypair(&mut sk, &mut pk)?;
            Ok((Self(sk), PublicKey(pk)))
        }

        /// Derive the public key from this secret key.
        pub fn public_key(&self) -> Result<PublicKey, embassy_crypto_driver::CryptoError> {
            let mut pk = [0u8; 65];
            embassy_crypto_driver::p256ecdh_derive_public_key(&self.0, &mut pk)?;
            Ok(PublicKey(pk))
        }

        /// Compute the ECDH shared secret with a peer's public key.
        pub fn diffie_hellman(
            &self,
            peer_public_key: &PublicKey,
        ) -> Result<SharedSecret, embassy_crypto_driver::CryptoError> {
            let mut shared = [0u8; 32];
            embassy_crypto_driver::p256ecdh_shared_secret(&self.0, peer_public_key.as_bytes(), &mut shared)?;
            Ok(SharedSecret(shared))
        }
    }

    impl fmt::Debug for SecretKey {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("SecretKey").finish_non_exhaustive()
        }
    }

    /// A P-256 public key (65-byte uncompressed point: 0x04 || x || y).
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct PublicKey([u8; 65]);

    impl PublicKey {
        /// Create a public key from a 65-byte uncompressed point.
        pub fn from_bytes(bytes: &[u8; 65]) -> Self {
            Self(*bytes)
        }

        /// Return the public key bytes.
        pub fn as_bytes(&self) -> &[u8; 65] {
            &self.0
        }
    }

    impl fmt::Debug for PublicKey {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("PublicKey").finish_non_exhaustive()
        }
    }

    /// A P-256 ECDH shared secret.
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct SharedSecret([u8; 32]);

    impl SharedSecret {
        /// Return the shared secret bytes.
        pub fn as_bytes(&self) -> &[u8; 32] {
            &self.0
        }
    }

    impl fmt::Debug for SharedSecret {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("SharedSecret").finish_non_exhaustive()
        }
    }

    /// P-256 ECDSA types.
    pub mod ecdsa {
        use core::fmt;

        use digest::Digest;

        use super::*;

        /// A raw P-256 ECDSA signature (64 bytes: r || s, big-endian).
        #[derive(Clone, Copy, PartialEq, Eq)]
        pub struct Signature([u8; 64]);

        impl Signature {
            /// Create a signature from its raw bytes.
            pub fn from_bytes(bytes: &[u8; 64]) -> Self {
                Self(*bytes)
            }

            /// Return the signature as a 64-byte array.
            pub fn to_bytes(&self) -> [u8; 64] {
                self.0
            }
        }

        impl fmt::Debug for Signature {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_struct("Signature").finish_non_exhaustive()
            }
        }

        impl AsRef<[u8]> for Signature {
            fn as_ref(&self) -> &[u8] {
                &self.0
            }
        }

        /// P-256 ECDSA signing key.
        #[derive(Clone)]
        pub struct SigningKey(SecretKey);

        impl SigningKey {
            /// Create a signing key from a 32-byte secret key.
            pub fn from_bytes(secret_key: &[u8; 32]) -> Self {
                Self(SecretKey(*secret_key))
            }

            /// Create a signing key from a `SecretKey`.
            pub fn from_secret_key(secret_key: &SecretKey) -> Self {
                Self(*secret_key)
            }

            /// Return the underlying secret key.
            pub fn secret_key(&self) -> &SecretKey {
                &self.0
            }

            /// Derive the verifying key from this signing key.
            pub fn verifying_key(&self) -> Result<VerifyingKey, embassy_crypto_driver::CryptoError> {
                self.0.public_key().map(|key| VerifyingKey::from_public_key(&key))
            }
        }

        impl fmt::Debug for SigningKey {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_struct("SigningKey").finish_non_exhaustive()
            }
        }

        impl signature::Signer<Signature> for SigningKey {
            fn try_sign(&self, msg: &[u8]) -> Result<Signature, signature::Error> {
                let mut hasher = crate::Sha256::new();
                digest::Update::update(&mut hasher, msg);
                let out = digest::FixedOutput::finalize_fixed(hasher);
                let mut digest = [0u8; 32];
                digest.copy_from_slice(out.as_slice());

                let mut sig = [0u8; 64];
                embassy_crypto_driver::p256ecdsa_sign(self.0.as_bytes(), &digest, &mut sig)
                    .map_err(|_| signature::Error::new())?;
                Ok(Signature(sig))
            }
        }

        impl<D> signature::DigestSigner<D, Signature> for SigningKey
        where
            D: Default + digest::Update + digest::FixedOutput,
        {
            fn try_sign_digest<F: Fn(&mut D) -> Result<(), signature::Error>>(
                &self,
                f: F,
            ) -> Result<Signature, signature::Error> {
                let mut digest = D::default();
                f(&mut digest)?;
                let out = digest.finalize_fixed();
                let mut digest_bytes = [0u8; 32];
                digest_bytes.copy_from_slice(out.as_slice());

                let mut sig = [0u8; 64];
                embassy_crypto_driver::p256ecdsa_sign(self.0.as_bytes(), &digest_bytes, &mut sig)
                    .map_err(|_| signature::Error::new())?;
                Ok(Signature(sig))
            }
        }

        /// P-256 ECDSA verifying key.
        #[derive(Clone, Copy)]
        pub struct VerifyingKey(PublicKey);

        impl VerifyingKey {
            /// Create a verifying key from a public key.
            pub fn from_public_key(public_key: &PublicKey) -> Self {
                Self(*public_key)
            }

            /// Create a verifying key from raw uncompressed point bytes.
            pub fn from_bytes(bytes: &[u8; 65]) -> Self {
                Self(PublicKey(*bytes))
            }

            /// Return the underlying public key.
            pub fn public_key(&self) -> &PublicKey {
                &self.0
            }
        }

        impl fmt::Debug for VerifyingKey {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_struct("VerifyingKey").finish_non_exhaustive()
            }
        }

        impl signature::Verifier<Signature> for VerifyingKey {
            fn verify(&self, msg: &[u8], signature: &Signature) -> Result<(), signature::Error> {
                let mut hasher = crate::Sha256::new();
                digest::Update::update(&mut hasher, msg);
                let out = digest::FixedOutput::finalize_fixed(hasher);
                let mut digest = [0u8; 32];
                digest.copy_from_slice(out.as_slice());

                embassy_crypto_driver::p256ecdsa_verify(
                    self.0.as_bytes(),
                    &digest,
                    signature.as_ref().try_into().unwrap(),
                )
                .map_err(|_| signature::Error::new())
            }
        }

        impl<D> signature::DigestVerifier<D, Signature> for VerifyingKey
        where
            D: Default + digest::Update + digest::FixedOutput,
        {
            fn verify_digest<F: Fn(&mut D) -> Result<(), signature::Error>>(
                &self,
                f: F,
                signature: &Signature,
            ) -> Result<(), signature::Error> {
                let mut digest = D::default();
                f(&mut digest)?;
                let out = digest.finalize_fixed();
                let mut digest_bytes = [0u8; 32];
                digest_bytes.copy_from_slice(out.as_slice());

                embassy_crypto_driver::p256ecdsa_verify(
                    self.0.as_bytes(),
                    &digest_bytes,
                    signature.as_ref().try_into().unwrap(),
                )
                .map_err(|_| signature::Error::new())
            }
        }
    }
}
