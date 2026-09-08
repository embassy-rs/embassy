#![no_std]
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

#[cfg(any(
    feature = "embassy-crypto-md5",
    feature = "embassy-crypto-sha1",
    feature = "embassy-crypto-sha224",
    feature = "embassy-crypto-sha256",
    feature = "embassy-crypto-sha384",
    feature = "embassy-crypto-sha512",
    feature = "embassy-crypto-sha512-224",
    feature = "embassy-crypto-sha512-256",
    feature = "embassy-crypto-hmac-sha1",
    feature = "embassy-crypto-hmac-sha224",
    feature = "embassy-crypto-hmac-sha256",
    feature = "embassy-crypto-hmac-sha384",
    feature = "embassy-crypto-hmac-sha512",
    feature = "embassy-crypto-hmac-sha512-224",
    feature = "embassy-crypto-hmac-sha512-256",
))]
mod hash;

#[cfg(any(
    feature = "embassy-crypto-aes128-ecb",
    feature = "embassy-crypto-aes128-cbc",
    feature = "embassy-crypto-aes128-ctr",
    feature = "embassy-crypto-aes128-gcm",
    feature = "embassy-crypto-aes128-ccm",
    feature = "embassy-crypto-aes128-cmac",
    feature = "embassy-crypto-aes256-ecb",
    feature = "embassy-crypto-aes256-cbc",
    feature = "embassy-crypto-aes256-ctr",
    feature = "embassy-crypto-aes256-gcm",
    feature = "embassy-crypto-aes256-ccm",
    feature = "embassy-crypto-aes256-cmac",
))]
mod aes;

#[cfg(any(feature = "embassy-crypto-aes128-ccm", feature = "embassy-crypto-aes256-ccm"))]
mod ccm;

#[cfg(any(
    feature = "embassy-crypto-p256-arith",
    feature = "embassy-crypto-p256-ecdh",
    feature = "embassy-crypto-p256-ecdsa",
    feature = "embassy-crypto-p384-arith",
    feature = "embassy-crypto-p384-ecdh",
    feature = "embassy-crypto-p384-ecdsa",
))]
mod ec;

#[cfg(feature = "embassy-crypto-x25519")]
mod x25519;

#[cfg(feature = "embassy-crypto-ed25519")]
mod ed25519;
