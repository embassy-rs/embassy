//! The shared `embassy-crypto-test` suites, run against the software drivers.

#![cfg(feature = "all")]

// A crate that is never named is not linked, so the drivers must be pulled in explicitly.
use embassy_crypto_rustcrypto as _;

macro_rules! suites {
    ($($name:ident),* $(,)?) => {$(
        #[test]
        fn $name() {
            let stats = embassy_crypto_test::$name().unwrap();
            assert!(stats.passed > 0);
        }
    )*};
}

suites! {
    md5, sha1, sha224, sha256, sha384, sha512, sha512_224, sha512_256,
    hmac_sha1, hmac_sha224, hmac_sha256, hmac_sha384, hmac_sha512, hmac_sha512_224, hmac_sha512_256,
    aes128_ecb, aes256_ecb, aes128_cbc, aes256_cbc, aes128_ctr, aes256_ctr,
    aes128_gcm, aes256_gcm, aes128_ccm, aes256_ccm, aes128_cmac, aes256_cmac,
    p256_arith, p256_ecdh, p256_ecdsa, p384_arith, p384_ecdh, p384_ecdsa,
    x25519_dh, x25519_keygen,
    ed25519_verify, ed25519_sign,
}
