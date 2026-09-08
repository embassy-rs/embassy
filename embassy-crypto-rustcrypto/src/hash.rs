macro_rules! digest_driver {
    ($feature:literal, $driver:ident, $trait:ident, $ty:ty, $size:literal, $register:ident) => {
        #[cfg(feature = $feature)]
        mod $driver {
            use digest::Digest;

            struct Driver;

            impl embassy_crypto::driver::$trait for Driver {
                type Context = $ty;

                fn init() -> Self::Context {
                    <$ty>::new()
                }

                fn update(ctx: &mut Self::Context, data: &[u8]) {
                    ctx.update(data);
                }

                fn finalize(ctx: Self::Context, out: &mut [u8; $size]) {
                    out.copy_from_slice(&ctx.finalize());
                }
            }

            embassy_crypto::$register!(Driver);
        }
    };
}

macro_rules! hmac_driver {
    ($feature:literal, $driver:ident, $trait:ident, $hash:ty, $size:literal, $register:ident) => {
        #[cfg(feature = $feature)]
        mod $driver {
            use digest::{KeyInit, Mac};

            struct Driver;

            impl embassy_crypto::driver::$trait for Driver {
                type Context = hmac::Hmac<$hash>;

                fn init(key: &[u8]) -> Self::Context {
                    // `Hmac` accepts keys of any length.
                    hmac::Hmac::<$hash>::new_from_slice(key).unwrap()
                }

                fn update(ctx: &mut Self::Context, data: &[u8]) {
                    ctx.update(data);
                }

                fn finalize(ctx: Self::Context, out: &mut [u8; $size]) {
                    out.copy_from_slice(&ctx.finalize().into_bytes());
                }
            }

            embassy_crypto::$register!(Driver);
        }
    };
}

digest_driver!("embassy-crypto-md5", md5_driver, Md5, md5::Md5, 16, md5_impl);
digest_driver!("embassy-crypto-sha1", sha1_driver, Sha1, sha1::Sha1, 20, sha1_impl);
digest_driver!(
    "embassy-crypto-sha224",
    sha224_driver,
    Sha224,
    sha2::Sha224,
    28,
    sha224_impl
);
digest_driver!(
    "embassy-crypto-sha256",
    sha256_driver,
    Sha256,
    sha2::Sha256,
    32,
    sha256_impl
);
digest_driver!(
    "embassy-crypto-sha384",
    sha384_driver,
    Sha384,
    sha2::Sha384,
    48,
    sha384_impl
);
digest_driver!(
    "embassy-crypto-sha512",
    sha512_driver,
    Sha512,
    sha2::Sha512,
    64,
    sha512_impl
);
digest_driver!(
    "embassy-crypto-sha512-224",
    sha512_224_driver,
    Sha512_224,
    sha2::Sha512_224,
    28,
    sha512_224_impl
);
digest_driver!(
    "embassy-crypto-sha512-256",
    sha512_256_driver,
    Sha512_256,
    sha2::Sha512_256,
    32,
    sha512_256_impl
);

hmac_driver!(
    "embassy-crypto-hmac-sha1",
    hmac_sha1_driver,
    HmacSha1,
    sha1::Sha1,
    20,
    hmac_sha1_impl
);
hmac_driver!(
    "embassy-crypto-hmac-sha224",
    hmac_sha224_driver,
    HmacSha224,
    sha2::Sha224,
    28,
    hmac_sha224_impl
);
hmac_driver!(
    "embassy-crypto-hmac-sha256",
    hmac_sha256_driver,
    HmacSha256,
    sha2::Sha256,
    32,
    hmac_sha256_impl
);
hmac_driver!(
    "embassy-crypto-hmac-sha384",
    hmac_sha384_driver,
    HmacSha384,
    sha2::Sha384,
    48,
    hmac_sha384_impl
);
hmac_driver!(
    "embassy-crypto-hmac-sha512",
    hmac_sha512_driver,
    HmacSha512,
    sha2::Sha512,
    64,
    hmac_sha512_impl
);
hmac_driver!(
    "embassy-crypto-hmac-sha512-224",
    hmac_sha512_224_driver,
    HmacSha512_224,
    sha2::Sha512_224,
    28,
    hmac_sha512_224_impl
);
hmac_driver!(
    "embassy-crypto-hmac-sha512-256",
    hmac_sha512_256_driver,
    HmacSha512_256,
    sha2::Sha512_256,
    32,
    hmac_sha512_256_impl
);
