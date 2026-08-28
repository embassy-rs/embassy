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

use crypto_common::KeySizeUser;
pub use digest;
use digest::generic_array::typenum::{U16, U20, U28, U32, U48, U64, U128};
use digest::{
    FixedOutput, FixedOutputReset, HashMarker, InvalidLength, Key, KeyInit, MacMarker, Output, OutputSizeUser, Reset,
    Update,
};

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
