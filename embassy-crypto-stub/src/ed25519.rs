use embassy_crypto::Error;
use embassy_crypto::driver::{Ed25519PublicKey, Ed25519SecretKey, Ed25519Signature};

struct Driver;

impl embassy_crypto::driver::Ed25519 for Driver {
    fn public_key(_: &Ed25519SecretKey) -> Result<Ed25519PublicKey, Error> {
        unreachable!()
    }

    fn sign(_: &Ed25519SecretKey, _: &[u8]) -> Result<Ed25519Signature, Error> {
        unreachable!()
    }

    fn verify(_: &Ed25519PublicKey, _: &[u8], _: &Ed25519Signature) -> Result<(), Error> {
        unreachable!()
    }
}

embassy_crypto::ed25519_impl!(Driver);
