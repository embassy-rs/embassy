//! Hash and HMAC Operations

use cipher::BlockSizeUser;
use crypto_common::{AlgorithmName, KeySizeUser};
pub use digest;
use digest::{
    FixedOutput, FixedOutputReset, HashMarker, InvalidLength, Key, KeyInit, MacMarker, Output, OutputSizeUser, Reset,
    Update,
};
use generic_array::typenum::{U16, U20, U28, U32, U48, U64, U128};

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
