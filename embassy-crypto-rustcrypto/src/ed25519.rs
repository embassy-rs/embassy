use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use embassy_crypto::Error;
use embassy_crypto::driver::{Ed25519PublicKey, Ed25519SecretKey, Ed25519Signature};

struct Driver;

impl embassy_crypto::driver::Ed25519 for Driver {
    fn public_key(k: &Ed25519SecretKey) -> Result<Ed25519PublicKey, Error> {
        let sk = SigningKey::from_bytes(&k.0);
        Ok(Ed25519PublicKey(sk.verifying_key().to_bytes()))
    }

    fn sign(k: &Ed25519SecretKey, msg: &[u8]) -> Result<Ed25519Signature, Error> {
        let sk = SigningKey::from_bytes(&k.0);
        Ok(Ed25519Signature(sk.sign(msg).to_bytes()))
    }

    fn verify(a: &Ed25519PublicKey, msg: &[u8], sig: &Ed25519Signature) -> Result<(), Error> {
        let vk = VerifyingKey::from_bytes(&a.0).map_err(|_| Error::InvalidKey)?;
        // `Signature::from_bytes` does not validate; `verify` decodes `R` and
        // rejects `S >= L`, so malleable signatures fail.
        vk.verify(msg, &Signature::from_bytes(&sig.0))
            .map_err(|_| Error::InvalidSignature)
    }
}

embassy_crypto::ed25519_impl!(Driver);
