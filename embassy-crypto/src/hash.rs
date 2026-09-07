//! Hash functions and HMAC.

use crate::driver;

macro_rules! impl_digest {
    ($(#[$meta:meta])* $name:ident, $drv:ident, $ctx:ident, $size:literal) => {
        $(#[$meta])*
        ///
        /// Served by the driver registered for it; see [`driver`].
        ///
        /// # Example
        ///
        /// ```ignore
        #[doc = concat!("let digest = ", stringify!($name), "::digest(b\"hello world\");")]
        ///
        /// // Or incrementally:
        #[doc = concat!("let mut hasher = ", stringify!($name), "::new();")]
        /// hasher.update(b"hello ");
        /// hasher.update(b"world");
        /// assert_eq!(hasher.finalize(), digest);
        /// ```
        #[derive(Clone)]
        pub struct $name {
            ctx: driver::$ctx,
        }

        impl $name {
            /// Size of the digest, in bytes.
            pub const OUTPUT_SIZE: usize = $size;

            /// Start a new hash computation.
            pub fn new() -> Self {
                Self {
                    ctx: driver::$drv::init(),
                }
            }

            /// Absorb `data`.
            pub fn update(&mut self, data: &[u8]) {
                driver::$drv::update(&mut self.ctx, data);
            }

            /// Finish the computation and return the digest.
            pub fn finalize(self) -> [u8; $size] {
                let mut out = [0u8; $size];
                driver::$drv::finalize(self.ctx, &mut out);
                out
            }

            /// Compute the digest of `data` in one call.
            pub fn digest(data: &[u8]) -> [u8; $size] {
                let mut h = Self::new();
                h.update(data);
                h.finalize()
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl core::fmt::Debug for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.debug_struct(stringify!($name)).finish_non_exhaustive()
            }
        }
    };
}

macro_rules! impl_hmac {
    ($(#[$meta:meta])* $name:ident, $drv:ident, $ctx:ident, $size:literal) => {
        $(#[$meta])*
        ///
        /// Keys of any length are accepted. Served by the driver registered for
        /// it; see [`driver`].
        ///
        /// # Example
        ///
        /// ```ignore
        #[doc = concat!("let tag = ", stringify!($name), "::mac(b\"my key\", b\"hello world\");")]
        ///
        /// // Verification, in constant time:
        #[doc = concat!("let mut mac = ", stringify!($name), "::new(b\"my key\");")]
        /// mac.update(b"hello world");
        /// mac.verify(&tag).unwrap();
        /// ```
        #[derive(Clone)]
        pub struct $name {
            ctx: driver::$ctx,
        }

        impl $name {
            /// Size of the tag, in bytes.
            pub const OUTPUT_SIZE: usize = $size;

            /// Start a new MAC computation with `key`.
            pub fn new(key: &[u8]) -> Self {
                Self {
                    ctx: driver::$drv::init(key),
                }
            }

            /// Absorb `data`.
            pub fn update(&mut self, data: &[u8]) {
                driver::$drv::update(&mut self.ctx, data);
            }

            /// Finish the computation and return the tag.
            pub fn finalize(self) -> [u8; $size] {
                let mut out = [0u8; $size];
                driver::$drv::finalize(self.ctx, &mut out);
                out
            }

            /// Finish the computation and compare the tag with `tag` in constant time.
            ///
            /// A truncated `tag` (shorter than the full tag) is compared against
            /// the corresponding prefix; an empty one never verifies.
            pub fn verify(self, tag: &[u8]) -> Result<(), crate::Error> {
                let out = self.finalize();
                if tag.is_empty() || tag.len() > out.len() || !crate::ct::eq(&out[..tag.len()], tag) {
                    return Err(crate::Error::InvalidSignature);
                }
                Ok(())
            }

            /// Compute the tag of `data` under `key` in one call.
            pub fn mac(key: &[u8], data: &[u8]) -> [u8; $size] {
                let mut m = Self::new(key);
                m.update(data);
                m.finalize()
            }
        }

        impl core::fmt::Debug for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.debug_struct(stringify!($name)).finish_non_exhaustive()
            }
        }
    };
}

impl_digest!(
    /// MD5.
    ///
    /// Broken as a cryptographic hash; provided for legacy protocols only.
    Md5, Md5Impl, Md5ImplContext, 16
);
impl_digest!(
    /// SHA-1.
    ///
    /// Collision-broken; provided for legacy protocols (HMAC-SHA-1 remains sound).
    Sha1, Sha1Impl, Sha1ImplContext, 20
);
impl_digest!(
    /// SHA-224.
    Sha224, Sha224Impl, Sha224ImplContext, 28
);
impl_digest!(
    /// SHA-256.
    Sha256, Sha256Impl, Sha256ImplContext, 32
);
impl_digest!(
    /// SHA-384.
    Sha384, Sha384Impl, Sha384ImplContext, 48
);
impl_digest!(
    /// SHA-512/224.
    Sha512_224, Sha512_224Impl, Sha512_224ImplContext, 28
);
impl_digest!(
    /// SHA-512/256.
    Sha512_256, Sha512_256Impl, Sha512_256ImplContext, 32
);
impl_digest!(
    /// SHA-512.
    Sha512, Sha512Impl, Sha512ImplContext, 64
);

impl_hmac!(
    /// HMAC-SHA-1.
    HmacSha1, HmacSha1Impl, HmacSha1ImplContext, 20
);
impl_hmac!(
    /// HMAC-SHA-224.
    HmacSha224, HmacSha224Impl, HmacSha224ImplContext, 28
);
impl_hmac!(
    /// HMAC-SHA-256.
    HmacSha256, HmacSha256Impl, HmacSha256ImplContext, 32
);
impl_hmac!(
    /// HMAC-SHA-384.
    HmacSha384, HmacSha384Impl, HmacSha384ImplContext, 48
);
impl_hmac!(
    /// HMAC-SHA-512/224.
    HmacSha512_224, HmacSha512_224Impl, HmacSha512_224ImplContext, 28
);
impl_hmac!(
    /// HMAC-SHA-512/256.
    HmacSha512_256, HmacSha512_256Impl, HmacSha512_256ImplContext, 32
);
impl_hmac!(
    /// HMAC-SHA-512.
    HmacSha512, HmacSha512Impl, HmacSha512ImplContext, 64
);
