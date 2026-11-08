macro_rules! digest_driver {
    ($feature:literal, $driver:ident, $trait:ident, $size:literal, $register:ident) => {
        #[cfg(feature = $feature)]
        mod $driver {
            struct Driver;

            impl embassy_crypto::driver::$trait for Driver {
                type Context = ();

                fn init() -> Self::Context {
                    unreachable!()
                }

                fn update(_: &mut Self::Context, _: &[u8]) {
                    unreachable!()
                }

                fn finalize(_: Self::Context, _: &mut [u8; $size]) {
                    unreachable!()
                }
            }

            embassy_crypto::$register!(Driver);
        }
    };
}

macro_rules! hmac_driver {
    ($feature:literal, $driver:ident, $trait:ident, $size:literal, $register:ident) => {
        #[cfg(feature = $feature)]
        mod $driver {
            struct Driver;

            impl embassy_crypto::driver::$trait for Driver {
                type Context = ();

                fn init(_: &[u8]) -> Self::Context {
                    unreachable!()
                }

                fn update(_: &mut Self::Context, _: &[u8]) {
                    unreachable!()
                }

                fn finalize(_: Self::Context, _: &mut [u8; $size]) {
                    unreachable!()
                }
            }

            embassy_crypto::$register!(Driver);
        }
    };
}

digest_driver!("embassy-crypto-md5", md5_driver, Md5, 16, md5_impl);
digest_driver!("embassy-crypto-sha1", sha1_driver, Sha1, 20, sha1_impl);
digest_driver!("embassy-crypto-sha224", sha224_driver, Sha224, 28, sha224_impl);
digest_driver!("embassy-crypto-sha256", sha256_driver, Sha256, 32, sha256_impl);
digest_driver!("embassy-crypto-sha384", sha384_driver, Sha384, 48, sha384_impl);
digest_driver!("embassy-crypto-sha512", sha512_driver, Sha512, 64, sha512_impl);
digest_driver!(
    "embassy-crypto-sha512-224",
    sha512_224_driver,
    Sha512_224,
    28,
    sha512_224_impl
);
digest_driver!(
    "embassy-crypto-sha512-256",
    sha512_256_driver,
    Sha512_256,
    32,
    sha512_256_impl
);

hmac_driver!(
    "embassy-crypto-hmac-sha1",
    hmac_sha1_driver,
    HmacSha1,
    20,
    hmac_sha1_impl
);
hmac_driver!(
    "embassy-crypto-hmac-sha224",
    hmac_sha224_driver,
    HmacSha224,
    28,
    hmac_sha224_impl
);
hmac_driver!(
    "embassy-crypto-hmac-sha256",
    hmac_sha256_driver,
    HmacSha256,
    32,
    hmac_sha256_impl
);
hmac_driver!(
    "embassy-crypto-hmac-sha384",
    hmac_sha384_driver,
    HmacSha384,
    48,
    hmac_sha384_impl
);
hmac_driver!(
    "embassy-crypto-hmac-sha512",
    hmac_sha512_driver,
    HmacSha512,
    64,
    hmac_sha512_impl
);
hmac_driver!(
    "embassy-crypto-hmac-sha512-224",
    hmac_sha512_224_driver,
    HmacSha512_224,
    28,
    hmac_sha512_224_impl
);
hmac_driver!(
    "embassy-crypto-hmac-sha512-256",
    hmac_sha512_256_driver,
    HmacSha512_256,
    32,
    hmac_sha512_256_impl
);
