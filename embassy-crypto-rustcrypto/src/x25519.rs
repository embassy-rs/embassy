use embassy_crypto::Error;
use embassy_crypto::driver::{X25519PublicKey, X25519SecretKey};
use x25519_dalek::{PublicKey, StaticSecret};

struct Driver;

impl embassy_crypto::driver::X25519 for Driver {
    fn public_key(k: &X25519SecretKey) -> Result<X25519PublicKey, Error> {
        let secret = StaticSecret::from(k.0);
        Ok(X25519PublicKey(PublicKey::from(&secret).to_bytes()))
    }

    fn shared_secret(k: &X25519SecretKey, peer: &X25519PublicKey) -> Result<[u8; 32], Error> {
        let secret = StaticSecret::from(k.0);
        Ok(secret.diffie_hellman(&PublicKey::from(peer.0)).to_bytes())
    }
}

embassy_crypto::x25519_impl!(Driver);
