use embassy_crypto::Error;
use embassy_crypto::driver::{X25519PublicKey, X25519SecretKey};

struct Driver;

impl embassy_crypto::driver::X25519 for Driver {
    fn public_key(_: &X25519SecretKey) -> Result<X25519PublicKey, Error> {
        unreachable!()
    }

    fn shared_secret(_: &X25519SecretKey, _: &X25519PublicKey) -> Result<[u8; 32], Error> {
        unreachable!()
    }
}

embassy_crypto::x25519_impl!(Driver);
